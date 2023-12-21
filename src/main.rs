use std::fs::{ OpenOptions, File };
use std::io::{ Read };
use std::env;
use serde::{ Serialize, Deserialize };

mod openai;
mod memory;
mod monikai;
mod linalg;

#[tokio::main]
async fn main() {
    println!("Initializing Monikai");

    let mut character_file_handle: File = OpenOptions::new()
        .read(true)
        .write(true)
        .open("data/monikai.json")
        .expect("Unable to get handle on './data/monikai.json'!");

    let mut character_json_string = String::new();
    character_file_handle.read_to_string(&mut character_json_string)
        .expect("Unable to read './data/monikai.json'!");

    let mut monikai: monikai::Monikai = serde_json::from_str(&character_json_string)
        .expect("Unable to parse!");

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
            "wipe" => {
                monikai = monikai::Monikai { 
                    description: monikai.description.clone(), 
                    memories: Vec::new(), 
                    current_conversation: Vec::new() 
                };
                println!("[Wiped]");
            },
            "save" => {
                monikai.save_to_file(&mut character_file_handle);
                println!("[Saved]");
            },
            "end" => {
                monikai.end_conversation().await;
                println!("[Ended Conversation]");
            },
            "log" => {
                let mut monikai_no_embeddings = monikai.clone();

                for memory in monikai_no_embeddings.memories.iter_mut() {
                    memory.embedding = Vec::new();
                }

                println!("{}", serde_json::to_string_pretty(&monikai_no_embeddings).unwrap());
            },
            "get" => {
                println!("> Please enter a key phrase to search by");
                let mut keyword = String::new();
                stdin.read_line(&mut keyword).unwrap();

                let key_phrase_embedding = openai::embedding_request(&keyword).await.unwrap();

                let mut memories_sorted: Vec<memory::Memory> = monikai.memories
                    .clone();
                    
                memories_sorted.sort_by(|a, b| {
                        let a_sim = linalg::cosine_similarity(&key_phrase_embedding, &a.embedding);
                        let b_sim = linalg::cosine_similarity(&key_phrase_embedding, &b.embedding);

                        a_sim.partial_cmp(&b_sim).unwrap()
                    });

                println!("Most similar: {}", memories_sorted.last().unwrap().conversation);
            }
            _ => {
                monikai.send_message(buffer.clone()).await;
            }
        }
    
        buffer.clear();
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn initialize() -> Result<(), ()>{
        let character_json_str = std::fs::read_to_string("data/monikai.json")
            .map_err(|_| ())?;

        let mut monikai: monikai::Monikai = serde_json::from_str(&character_json_str)
            .map_err(|_| ())?;
        
        Ok(())
    }

    #[tokio::test]
    async fn build_memory() -> Result<(), ()>{
        let conversation = "MC: Hello!\nMonika: Hi!\nMC:Do you have any good book recommendations?\nMonika: Dune - Frank Herbert!!";

        let memory = memory::Memory::new(conversation.to_string()).await;
        
        println!("{:?}", memory);
        
        Ok(())
    }
}