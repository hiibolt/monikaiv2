use std::fs::{ File };
use std::io::{ Write, Seek };
use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    routing::get,
    response::{ Html},
    Router,
};
use tower_http::services::ServeDir;
use futures::{sink::SinkExt, stream::StreamExt};

use crate::{ Serialize, Deserialize };
use crate::{ Mutex, Arc };
use crate::OpenOptions;
use crate::memory;
use crate::openai; 
use crate::linalg;
use crate::print;

#[derive(Debug, Deserialize)]
struct MemoryDiveConformation {
    needs_memory_check: bool,
    memory_check_phrase: String
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Monikai {
    pub description: String,
    pub memories: Vec<memory::Memory>,
    pub current_conversation: Vec<openai::Message>
}
impl Monikai {
    async fn respond( &mut self ) -> String {
        // First, compile the conversation and user profile
        let mut messages = self.current_conversation.clone();
        let user_profile = self.memories.iter()
            .map(|memory| memory.user_profile.clone() )
            .collect::<Vec<String>>()
            .join("\n");

        // Next, insert the nessecary context about who the Monikai is
        messages.insert(
            0, 
            openai::Message { 
                role: String::from("system"), 
                content: self.description.clone()
            });

        // Insert the user profile context
        messages.insert(
            1, 
            openai::Message { 
                role: String::from("system"), 
                content: format!("The following is information about MC you have gathered from previous conversations. {}", user_profile)
            });
        
        // Build the prompt to check if more context is needed to respond
        let memory_check_prompt = format!("
            In the following conversation, you are Monikai.
            Decide and return JSON on whether based on the user profile and current conversation if you have the context to answer.
            If not, create a phrase to rack your memory for. For instance, if the user wants to know about a car recommendation:

            Example:
            {{
                \"needs_memory_check\": true,
                \"memory_check_phrase\": \"car recommendations\"
            }}

            USER PROFILE:
            {}

            RECENT CONVERSATION:
            {}

            Response:
            {{
            ", user_profile, messages.iter().last().unwrap().content);

        // Prompt davinci-003 to decide
        let memory_check_unparsed = openai::instruction_request(memory_check_prompt).await.unwrap();

        // If the input parses, try to grab context.
        if let Ok(memory_check) = serde_json::from_str::<MemoryDiveConformation>(format!("{{{}", memory_check_unparsed).as_str()) {
            if memory_check.needs_memory_check {
                print::debug("Need to perform memory check!");
                let key_phrase_embedding = openai::embedding_request(&memory_check.memory_check_phrase).await.unwrap();

                let mut memories_sorted: Vec<memory::Memory> = self.memories
                    .clone();
                    
                memories_sorted.sort_by(|a, b| {
                        let a_sim = linalg::cosine_similarity(&key_phrase_embedding, &a.embedding);
                        let b_sim = linalg::cosine_similarity(&key_phrase_embedding, &b.embedding);

                        a_sim.partial_cmp(&b_sim).unwrap()
                    });

                if let Some(most_similar) = memories_sorted.last() {
                    print::debug(&format!("Most similar: {}", most_similar.conversation));

                    messages.insert(
                        2, 
                        openai::Message { 
                            role: String::from("system"), 
                            content: format!("You believe you may need additional information to respond. Here is a related memory from {} ago: {}", most_similar.readable_time_since(), most_similar.conversation)
                        });
                }
            }
        }

        // Finally, prompt the model
        let response = openai::turbo_request( messages ).await.unwrap().content;

        print::monikai(&response);

        self.current_conversation.push( openai::Message { role: String::from("assistant"), content: response.clone() } );

        response
    }
    pub async fn send_message( &mut self, message: String ) -> String {
        self.current_conversation.push( openai::Message { role: String::from("user"), content: message } );

        self.respond().await
    }
    pub async fn end_conversation( &mut self ) {
        let conversation_as_string: String = self.current_conversation
            .iter()
            .map(|message| message.to_string() )
            .collect::<Vec<String>>()
            .join("\n");
        
        let new_memory = memory::Memory::new( conversation_as_string ).await;

        self.memories.push(new_memory);
        self.current_conversation = Vec::new();
    }
    pub fn save_to_file( &self, file_handle: &mut File ) {
        let self_as_string: String = serde_json::to_string_pretty(&self).unwrap();

        file_handle.set_len(0).unwrap();
        file_handle.rewind().unwrap();
        file_handle.write_all(self_as_string.as_bytes()).expect("Failed to write!");
    }
}

/* 
 A Read-Eval-Print Loop (REPL) for Monikai.
 Probably the least convuleted method of communication.

 Saying something that's not a command forwards said message to the Monikai.

 Commands:
  'wipe': Clear the Monikai's memories and recent conversation, preserves the description.
  'save': Writes the Monikai in memory to 'monikai.json'.
  'end': Manually marks the current conversation as completed and encodes it as a memory.
  'log': Prints the Monikai in memory to stdout.
  'get': Takes another line as input, and prints the memory most similar in cosine.
*/
pub async fn monikai_repl( monikai: Arc<Mutex<Monikai>> ) {
    let mut character_file_handle: File = OpenOptions::new()
        .read(true)
        .write(true)
        .open("data/monikai.json")
        .expect("Unable to get handle on './data/monikai.json'!");

    let stdin = std::io::stdin();
    let mut buffer = String::new();

    loop {
        stdin.read_line(&mut buffer).unwrap();

        // Remove the trailing '\n' character
        buffer = buffer
            .split("\n")
            .nth(0)
            .unwrap()
            .to_string();

        // Check for any commandsx
        match buffer.as_str() {
            "clear" => panic!("This isn't a terminal, you know..."),
            "wipe" => {
                let description = monikai.lock().await.description.clone();

                *monikai.lock().await = Monikai { 
                    description, 
                    memories: Vec::new(), 
                    current_conversation: Vec::new() 
                };

                print::info("Wiped");
            },
            "save" => {
                monikai.lock().await.save_to_file(&mut character_file_handle);
                print::info("Saved");
            },
            "end" => {
                monikai.lock().await.end_conversation().await;
                print::info("Ended Conversation");
            },
            "log" => {
                print::info("Logging");

                let mut monikai_no_embeddings = monikai.lock().await.clone();

                for memory in monikai_no_embeddings.memories.iter_mut() {
                    memory.embedding = Vec::new();
                }

                print::debug(&serde_json::to_string_pretty(&monikai_no_embeddings).unwrap());
            },
            "get" => {
                print::info("Please enter a key phrase to search by");
                let mut keyword = String::new();
                stdin.read_line(&mut keyword).unwrap();

                let key_phrase_embedding = openai::embedding_request(&keyword).await.unwrap();

                let mut memories_sorted: Vec<memory::Memory> = monikai.lock().await.memories
                    .clone();
                    
                memories_sorted.sort_by(|a, b| {
                        let a_sim = linalg::cosine_similarity(&key_phrase_embedding, &a.embedding);
                        let b_sim = linalg::cosine_similarity(&key_phrase_embedding, &b.embedding);

                        a_sim.partial_cmp(&b_sim).unwrap()
                    });

                print::debug(&format!("Most similar: {}", memories_sorted.last().unwrap().conversation));
            }
            _ => {
                monikai.lock().await.send_message(buffer.clone()).await;
            }
        }
    
        buffer.clear();
    }
}
/*
 Backend for Monikai with an extra layer for emotion generation.
 There is an example client in ../../public.

 For instance, given a response and context, the Monikai determines its visible emotion.
*/
pub async fn monikai_backend( monikai: Arc<Mutex<Monikai>>) {
    let app = Router::new()
        .route("/", get(|| async { Html(std::include_str!("../../public/index.html")) }))
        .route("/ws", get(
            |
                ws: WebSocketUpgrade,
                axum::extract::State(state): axum::extract::State<Arc<Mutex<Monikai>>>,
            | async {
                println!("Connection!");
                ws.on_upgrade(|socket| monikai_websocket(socket, state))
            }
        ))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(monikai);
        
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
// Helper function to preserve readability for the above backend.
async fn monikai_websocket(stream: WebSocket, monikai: Arc<Mutex<Monikai>>) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        if let axum::extract::ws::Message::Text(msg) = message {
            println!("(remote) {}", msg);

            let response = monikai.lock().await.send_message(msg.clone()).await;
            let conversation = monikai.lock().await.current_conversation
                .iter()
                .map(|message| message.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            let description = monikai.lock().await.description.clone();

            let emotion = openai::instruction_request(format!("
                {}

                Based on the conversation, create an meotion (NEUTRAL | SAD | CRYING | LAUGHING | CONCERNED) that pairs well with your.

                Example:
                {{
                    \"message\": \"blah blah blah\",
                    \"emotion\": \"NEUTRAL | SAD | CRYING | LAUGHING | CONCERNED\"
                }}

                CONVERSATION:
                {}

                NEXT RESPONSE:
                {}

                {{
                    \"message\": \"{}\",
            ", description, conversation, response, response)).await.unwrap();

            let response_with_emotion = format!(r#"{{"message": "{}",{}"#, response, emotion);

            sender
                .send(axum::extract::ws::Message::Text(response_with_emotion))
                .await.unwrap();
        }
    }
}