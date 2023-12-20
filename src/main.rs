use std::fs::{ OpenOptions, File };
use std::io::{ Read };
use std::env;
use serde::{ Serialize, Deserialize };

mod openai;
mod memory;
mod monikai;

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
                println!("{:?}", monikai);
            },
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