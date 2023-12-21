use std::{
    fs::{ OpenOptions, File },
    env,
    io::Read,
    sync::{ Arc }
};
use tokio::sync::Mutex;
use serde::{ Serialize, Deserialize };
use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    routing::get,
    response::{ Html},
    Router,
};
use tower_http::services::ServeDir;
use futures::{sink::SinkExt, stream::StreamExt};

mod openai;
mod memory;
mod monikai;
mod linalg;
mod print;

async fn monikai_repl( monikai: Arc<Mutex<monikai::Monikai>> ) {
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

                *monikai.lock().await = monikai::Monikai { 
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
async fn monikai_backend( monikai: Arc<Mutex<monikai::Monikai>>) {
    let app = Router::new()
        .route("/", get(|| async { Html(std::include_str!("../public/index.html")) }))
        .route("/ws", get(
            |
                ws: WebSocketUpgrade,
                axum::extract::State(state): axum::extract::State<Arc<Mutex<monikai::Monikai>>>,
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
async fn monikai_websocket(stream: WebSocket, monikai: Arc<Mutex<monikai::Monikai>>) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        if let axum::extract::ws::Message::Text(msg) = message {
            println!("(remote) {}", msg);

            let response = monikai.lock().await.send_message(msg.clone()).await;

            let response_with_emotion = format!(r#"{{"message": "{}", "emotion": "NEUTRAL"}}"#, response);

            sender
                .send(axum::extract::ws::Message::Text(response_with_emotion))
                .await.unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    print::info("Initializing Monikai");
    // Open the data file
    let mut character_file_handle: File = OpenOptions::new()
        .read(true)
        .write(true)
        .open("data/monikai.json")
        .expect("Unable to get handle on './data/monikai.json'!");

    // Read from said data file
    let mut character_json_string = String::new();
    character_file_handle.read_to_string(&mut character_json_string)
        .expect("Unable to read './data/monikai.json'!");

    // Close the file handle
    drop(character_file_handle);
    print::info("Done!");

    // Build a thread and asynchronus reference to the character
    let monikai = Arc::new(
        Mutex::new(
            serde_json::from_str::<monikai::Monikai>(&character_json_string)
                .expect("Unable to parse!")
        ));

    // Start the repl and frontend
    tokio::spawn(monikai_repl( monikai.clone() ));
    tokio::spawn(monikai_backend( monikai.clone() ));

    // I may need to look into doing this a different way.
    // However, I'll probably end up using the main fn for timing tasks and plugins.
    loop {}
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn initialize() -> Result<(), ()>{
        let character_json_str = std::fs::read_to_string("data/monikai.json")
            .map_err(|_| ())?;

        let monikai: monikai::Monikai = serde_json::from_str(&character_json_str)
            .map_err(|_| ())?;
        
        println!("{:?}", monikai);

        Ok(())
    }

    #[tokio::test]
    async fn build_memory() -> Result<(), ()> {
        let conversation = "MC: Hello!\nMonika: Hi!\nMC:Do you have any good book recommendations?\nMonika: Dune - Frank Herbert!!";

        let memory = memory::Memory::new(conversation.to_string()).await;
        
        println!("{:?}", memory);
        
        Ok(())
    }

    #[tokio::test]
    async fn build_monikai_repl() -> Result<(), ()> {
        // Open the data file
        let mut character_file_handle: File = OpenOptions::new()
            .read(true)
            .write(true)
            .open("data/monikai.json")
            .expect("Unable to get handle on './data/monikai.json'!");

        // Read from said data file
        let mut character_json_string = String::new();
        character_file_handle.read_to_string(&mut character_json_string)
            .expect("Unable to read './data/monikai.json'!");

        // Close the file handle
        drop(character_file_handle);
        print::info("Done!");

        // Build a thread and asynchronus reference to the character
        let monikai = Arc::new(
            Mutex::new(
                serde_json::from_str::<monikai::Monikai>(&character_json_string)
                    .expect("Unable to parse!")
            ));

        // Start the repl and frontend
        tokio::spawn(monikai_repl( monikai.clone() ));

        Ok(())
    }
    #[tokio::test]
    async fn build_monikai_backend() -> Result<(), ()> {
        // Open the data file
        let mut character_file_handle: File = OpenOptions::new()
            .read(true)
            .write(true)
            .open("data/monikai.json")
            .expect("Unable to get handle on './data/monikai.json'!");
    
        // Read from said data file
        let mut character_json_string = String::new();
        character_file_handle.read_to_string(&mut character_json_string)
            .expect("Unable to read './data/monikai.json'!");
    
        // Close the file handle
        drop(character_file_handle);
        print::info("Done!");
    
        // Build a thread and asynchronus reference to the character
        let monikai = Arc::new(
            Mutex::new(
                serde_json::from_str::<monikai::Monikai>(&character_json_string)
                    .expect("Unable to parse!")
            ));
    
        // Start the repl and backend
        tokio::spawn(monikai_backend( monikai.clone() ));

        Ok(())
    }
}