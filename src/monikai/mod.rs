use std::fs::{ File };
use std::io::{ Write, Seek };
use crate::{ Serialize, Deserialize };
use crate::memory;
use crate::openai; 

#[derive(Debug, Serialize, Deserialize)]
pub struct Monikai {
    pub description: String,
    pub memories: Vec<memory::Memory>,
    pub current_conversation: Vec<openai::Message>
}
impl Monikai {
    async fn respond( &mut self ) {
        let mut messages = self.current_conversation.clone();
        messages.insert(
            0, 
            openai::Message { 
                role: String::from("system"), 
                content: self.description.clone()
            });
        

        let response = openai::turbo_request( messages ).await.unwrap().content;

        println!("{}", response);

        self.current_conversation.push( openai::Message { role: String::from("assistant"), content: response } );

    }
    pub async fn send_message( &mut self, message: String ) {
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