use async_trait::async_trait;
use llm_agent::LlmClient;
use llm_service::{genai_service, genai_stream_service, LLMStreamOut};

pub mod fsm;
pub mod llm_service;
pub mod llm_agent;
pub mod fsm_chat_state;


pub struct GenaiLlmclient {
    pub model: String,
    pub api_key: String,
}


#[async_trait]
impl LlmClient for GenaiLlmclient {
    async fn generate(&self, prompt: &str, msgs: &[(String, String)], temperature: Option<f32>) -> Result<String, anyhow::Error> {
        let t = temperature.unwrap_or(0.5); 
        genai_service(prompt, msgs, &self.model, &self.api_key, t).await
    }

    async fn generate_stream(&self, prompt: &str, msgs: &[(String, String)], temperature: Option<f32>) -> LLMStreamOut {
        let t = temperature.unwrap_or(0.5); 
        genai_stream_service(prompt, msgs, &self.model, &self.api_key, t).await
    }
}
