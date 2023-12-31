use std::fs::{ File };
use std::io::{ Write, Seek };
use std::time::{ Duration, SystemTime, UNIX_EPOCH };
use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    routing::get,
    response::{ Html},
    Router,
};
use tower_http::services::ServeDir;
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::time::sleep;

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
    pub current_conversation: Vec<openai::Message>,
    pub last_spoken_to: u64
}
impl Monikai {
    async fn respond( &mut self ) -> String {
        // Set the last spoken to timestamp to now
        self.last_spoken_to = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
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
        let manual_memory_check_prompt = format!("
            In the following conversation, you are 'assistant' (or Monikai).
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
        let automatic_memory_check_prompt = format!("
            In the following conversation, you are the 'assistant' (or Monikai).
            Generate an incredibly short phrase to check your memory embeddings for similar things to the current conversation.

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
                \"needs_memory_check\": true,
            ", user_profile, messages.iter().last().unwrap().content);

        // Prompt davinci-003 to generate keyphrases
        let manual_memory_check_unparsed = openai::instruction_request(manual_memory_check_prompt).await.unwrap();
        let automatic_memory_check_unparsed = openai::instruction_request(automatic_memory_check_prompt).await.unwrap();

        // If the input parses, try to grab context.
        if let Ok(memory_check) = serde_json::from_str::<MemoryDiveConformation>(format!("{{\"needs_memory_check\": true, {}", automatic_memory_check_unparsed).as_str()) {
            let key_phrase_embedding = openai::embedding_request(&memory_check.memory_check_phrase).await.unwrap();

            self.memories
                .sort_by(|a, b| {
                    let a_sim = linalg::cosine_similarity(&key_phrase_embedding, &a.embedding);
                    let b_sim = linalg::cosine_similarity(&key_phrase_embedding, &b.embedding);

                    a_sim.partial_cmp(&b_sim).unwrap()
                });

            if let Some(most_similar) = self.memories.last_mut() {
                most_similar.times_read += 1usize;

                messages.insert(
                    2, 
                    openai::Message { 
                        role: String::from("system"), 
                        content: format!("You believe you may need additional information to respond. Here is a related memory from {} ago: {}", most_similar.readable_time_since(), most_similar.conversation)
                    });
                
                print::debug("Grabbed automatic memory");
            }
        }
        if let Ok(memory_check) = serde_json::from_str::<MemoryDiveConformation>(format!("{{{}", manual_memory_check_unparsed).as_str()) {
            if memory_check.needs_memory_check {
                let key_phrase_embedding = openai::embedding_request(&memory_check.memory_check_phrase).await.unwrap();

                self.memories
                    .sort_by(|a, b| {
                        let a_sim = linalg::cosine_similarity(&key_phrase_embedding, &a.embedding);
                        let b_sim = linalg::cosine_similarity(&key_phrase_embedding, &b.embedding);

                        a_sim.partial_cmp(&b_sim).unwrap()
                    });

                if let Some(most_similar) = self.memories.last_mut() {
                    most_similar.times_read += 1usize;

                    messages.insert(
                        2, 
                        openai::Message { 
                            role: String::from("system"), 
                            content: format!("You believe you may need additional information to respond. Here is a related memory from {} ago: {}", most_similar.readable_time_since(), most_similar.conversation)
                        });
                    
                    print::debug("Grabbed manually checked memory");
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
 Probably the least convoluted method of communication.

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
                    current_conversation: Vec::new(),
                    last_spoken_to: 0u64
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

                if let Some(most_similar) = memories_sorted.last() {
                    print::debug(&format!("Most similar: {}", most_similar.conversation));
                } else {
                    print::debug("Your Monikai has no memories! Go make some :3");
                }
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

                {{\"message\":\"{}\",\"emotion\":
            ", description, conversation, response, response)).await.unwrap();

            let response_with_emotion = format!(r#"{{"message": "{}","emotion":{}"#, response, emotion);

            sender
                .send(axum::extract::ws::Message::Text(response_with_emotion))
                .await.unwrap();
        }
    }
}
/**
 Automatically removes memories based on the ratio of time to access.

 Based on the Ebbinghaus Forgetting Curve (with slightly more aggressive constants)
 and Trace Decay Theory of Forgetting.


 During testing, I found that this created the most 'human-like' interactions, and that 
 limits had to be set, as the 'forgetting curve' isn't all inclusive, many people don't
 ever forget some memories, regardless of time. In Monikai's case, I quantified this as 50+
 recalls, a completely arbitrary number.
**/
pub async fn monikai_memory_agent( monikai: Arc<Mutex<Monikai>> ) {
    loop {
        let memories_clone = monikai.lock().await.memories.clone();
        monikai.lock().await.memories = memories_clone
            .iter()
            .filter_map(|memory| {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
        
                let days_since = current_time.saturating_sub( memory.timestamp ) / 86400;

                if days_since > 2 && memory.times_read < 50 {
                    let forget_score: f64 = 1.15f64.powf(days_since as f64 - 6.99f64) / 1.15f64.powi(memory.times_read as i32) - 1f64;
                    if forget_score < 1f64 {
                        print::debug(&format!("Pruned memory: {}...", memory.interaction_summary.get(0..if memory.interaction_summary.len() < 35 { memory.interaction_summary.len() } else { 35 } ).unwrap()));

                        return None;
                    }
                }

                Some(memory.clone())
            })
            .collect::<Vec<memory::Memory>>();
        

        sleep(Duration::from_secs(15)).await;
    }
}
/*
 Automatically saves the Monikai every 5 seconds.

 Saves the conversation as a memory after 5 minutes.
*/
pub async fn monikai_autosave( monikai: Arc<Mutex<Monikai>> ) {
    loop {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let minutes_since = current_time.saturating_sub( monikai.lock().await.last_spoken_to ) / 60;

        let conversation_length = monikai.lock().await.current_conversation.len();
        if minutes_since > 5 && conversation_length > 0 {
            monikai.lock().await.end_conversation().await;

            print::debug("Ended conversation")
        }

        if let Ok(mut file_handle) = OpenOptions::new()
            .read(true)
            .write(true)
            .open("data/monikai.json")
        {
            monikai.lock().await.save_to_file( &mut file_handle );
        } else {
            print::debug("File busy!");
        }

        sleep(Duration::from_secs(5)).await;
    }
}