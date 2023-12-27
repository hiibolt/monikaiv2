use crate::{Serialize, Deserialize};
use crate::openai;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Memory {
    pub embedding: Embedding,
    pub user_profile: UserProfile,
    pub interaction_summary: InteractionSummary,
    pub conversation: Conversation,
    pub timestamp: u64,
    pub times_read: usize
}
pub type Embedding = Vec<f64>;
pub type UserProfile = String;
pub type InteractionSummary = String;
pub type Conversation = String;
impl Memory {
    pub async fn new( conversation: String ) -> Self {
        // Creates the time first, since the later parts can cause notable delay.
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
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
                    conversation,
                    timestamp,
                    times_read: 0usize
                }
            }
            Err(_) => todo!()
        }
    }
    async fn generate_embedding( input: &String ) -> Result<Embedding, ()> {
        Ok(openai::embedding_request(input).await.unwrap())
    }
    async fn generate_user_profile( input: &String ) -> Result<UserProfile, ()> {
        let prompt = format!("
            In the following conversation, you are Monikai.
            Create a USER PROFILE detailing what you've learned about the MC from the following conversation.

            Example:
            I have learned that the MC enjoys ..., ..., and .... In the future, I should talk about ... more.

            CONVERSATION:
            {}
        
            USER PROFILE:

        ", input);

        Ok(openai::instruction_request(prompt).await.unwrap())
    }
    async fn generate_interaction_summary( input: &String ) -> Result<InteractionSummary, ()> {
        let prompt = format!("
            In the following conversation, you are Monikai.
            Create an INTERACTION SUMMARY detailing what you've learned about the MC from the following conversation.

            Example:
            We talked about ..., ..., and ....

            CONVERSATION:
            {}
        
            INTERACTION SUMMARY:

        ", input);

        Ok(openai::instruction_request(prompt).await.unwrap())
    }
    pub fn readable_time_since( &self ) -> String {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let seconds_since = current_time.saturating_sub( self.timestamp );

        if seconds_since > 86400 {
            return format!("{} day/s", seconds_since / 86400);
        } else if seconds_since > 3600 {
            return format!("{} hour/s", seconds_since / 3600);
        } else if seconds_since > 60 {
            return format!("{} minute/s", seconds_since / 60);
        }
        format!("{} seconds", seconds_since)
    }
}