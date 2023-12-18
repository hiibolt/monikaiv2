use std::fs::{ OpenOptions, File };
use std::io::{ Read, Write, Seek };
use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
struct Memory {
    embedding: Embedding,
    user_profile: UserProfile,
    interaction_summary: InteractionSummary,
    conversation: Conversation
}
type Embedding = Vec<f64>;
type UserProfile = String;
type InteractionSummary = String;
type Conversation = String;
impl Memory {
    async fn new( conversation: String ) -> Self {
        /*
         Generates the embeddings, user profile, and summary asynchronously.
         Finally, returns the generated Memory object.
        */
        match tokio::try_join!(
            Self::generate_embedding(&conversation),
            Self::generate_user_profile(&conversation),
            Self::generate_interaction_summary(&conversation)
        ) {
            Ok((embedding, user_profile, interaction_summary)) => {
                return Self {
                    embedding,
                    user_profile, 
                    interaction_summary,
                    conversation
                }
            }
            Err(_) => todo!()
        }
    }
    async fn generate_embedding( input: &String ) -> Result<Embedding, ()> {
        todo!();
    }
    async fn generate_user_profile( input: &String ) -> Result<UserProfile, ()> {
        todo!();
    }
    async fn generate_interaction_summary( input: &String ) -> Result<InteractionSummary, ()> {
        todo!();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    sender: String,
    content: String
}
impl Message {
    fn to_string( &self ) -> String {
        format!("{}: {}", self.sender, self.content)
    }
}

#[derive(Debug)]
struct Response {
    content: String,
    end_conversation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Monikai {
    memories: Vec<Memory>,
    current_conversation: Vec<Message>
}
impl Monikai {
    async fn respond( &self ) -> Response {
        todo!();
    }
    async fn send_message( &mut self, message: String ) {
        self.current_conversation.push( Message { sender: String::from("mc"), content: message } );

        /*
        let response = self.respond().await;
        self.current_conversation.push( Message { sender: String::from("monikai"), content: response.content } );

        if response.end_conversation {
            self.end_conversation();
        }*/
    }
    async fn end_conversation( &mut self ) {
        let conversation_as_string: String = self.current_conversation
            .iter()
            .map(|message| message.to_string() )
            .collect::<Vec<String>>()
            .join("\n");
        
        let new_memory = Memory::new( conversation_as_string ).await;

        self.memories.push(new_memory);
        self.current_conversation = Vec::new();
    }
    fn save_to_file( &self, file_handle: &mut File ) {
        let self_as_string: String = serde_json::to_string(&self).unwrap();

        file_handle.set_len(0).unwrap();
        file_handle.rewind().unwrap();
        file_handle.write_all(self_as_string.as_bytes()).expect("Failed to write!");
    }
}

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

    let mut monikai: Monikai = serde_json::from_str(&character_json_string)
        .expect("Unable to parse!");

    monikai.send_message(String::from("hiiii :3")).await;

    println!("{:?}", monikai);
}




#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn initialize() -> Result<(), ()>{
        let character_json_str = std::fs::read_to_string("data/monikai.json")
            .map_err(|_| ())?;

        let mut monikai: Monikai = serde_json::from_str(&character_json_str)
            .map_err(|_| ())?;
        
        Ok(())
    }
}