use std::fs::{ File };
use std::io::{ Write, Seek };
use crate::{ Serialize, Deserialize };
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
    async fn respond( &mut self ) {
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
                            content: format!("You believe you may need additional information to respond. Here is a related memory: {}", most_similar.conversation)
                        });
                }
            }
        }

        // Finally, prompt the model
        let response = openai::turbo_request( messages ).await.unwrap().content;

        print::monikai(&response);

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