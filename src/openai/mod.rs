use crate::{ Serialize, Deserialize };
use crate::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String
}
impl Message {
    pub fn to_string( &self ) -> String {
        format!("{}: {}", self.role, self.content)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CompletionRequest {
    model: String,
    messages: Vec<Message>
}
#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<CompletionChoice>
}
#[derive(Debug, Deserialize)]
struct CompletionChoice {
    message: Message
}

pub async fn turbo_request( messages: Vec<Message> ) -> Result<Message, ()> {
    let completion_request = CompletionRequest {
        model: String::from("gpt-3.5-turbo"),
        messages
    };
    
    let request: String = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()))
        .set("Content-Type", "application/json")
        .send_string(&serde_json::to_string(&completion_request).unwrap()).unwrap()
        .into_string().unwrap();

    let deserialized_completion_response: CompletionResponse = serde_json::from_str(&request).unwrap();

    Ok(deserialized_completion_response.choices[0].message.clone())
}


#[derive(Debug, Serialize)]
struct InstructionRequest {
    model: String,
    prompt: String,
    temperature: f32,
    max_tokens: usize
}
#[derive(Debug, Deserialize)]
struct InstructionResponse {
    choices: Vec<InstructionChoice>
}
#[derive(Debug, Deserialize)]
struct InstructionChoice {
    text: String
}

pub async fn instruction_request( prompt: String ) -> Result<String, ()> {
    let instruct_request = serde_json::to_string(&InstructionRequest {
        model: String::from("gpt-3.5-turbo-instruct-0914"),
        prompt,
        temperature: 1.,
        max_tokens: 256
    }).unwrap();
    
    let request: String = ureq::post("https://api.openai.com/v1/completions")
        .set("Authorization", &format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()))
        .set("Content-Type", "application/json")
        .send_string(&instruct_request).unwrap()
        .into_string().unwrap();

    let deserialized_instruction_response: InstructionResponse = serde_json::from_str(&request).unwrap();

    Ok(deserialized_instruction_response.choices[0].text.clone())
}


#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>
}
#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f64>
}

pub async fn embedding_request( input: String ) -> Result<Vec<f64>, ()> {
    let embed_request = serde_json::to_string(&EmbeddingRequest {
        model: String::from("text-embedding-ada-002"),
        input
    }).unwrap();
    
    let request: String = ureq::post("https://api.openai.com/v1/embeddings")
        .set("Authorization", &format!("Bearer {}", env::var("OPENAI_API_KEY").unwrap()))
        .set("Content-Type", "application/json")
        .send_string(&embed_request).unwrap()
        .into_string().unwrap();

    let deserialized_embedding_response: EmbeddingResponse = serde_json::from_str(&request).unwrap();

    Ok(deserialized_embedding_response.data[0].embedding.clone())
}