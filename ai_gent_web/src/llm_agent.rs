use async_trait::async_trait;

use ai_gent_lib::fsm::FSMBuilder;

use ai_gent_lib::llm_agent::{FSMAgentConfigBuilder, LLMAgent, LLMClient};

// use futures::StreamExt;
use ai_gent_lib::llm_service::{genai_service, genai_stream_service, LLMStreamOut};

pub struct GenaiLlmclient {
    pub model: String,
    pub api_key: String,
}


#[async_trait]
impl LLMClient for GenaiLlmclient {
    async fn generate(&self, prompt: &str, msgs: &[(String, String)], temperature: Option<f32>) -> String {
        let t = temperature.unwrap_or(0.5); 
        genai_service(prompt, msgs, &self.model, &self.api_key, t).await
    }

    async fn generate_stream(&self, prompt: &str, msgs: &[(String, String)], temperature: Option<f32>) -> LLMStreamOut {
        let t = temperature.unwrap_or(0.5); 
        genai_stream_service(prompt, msgs, &self.model, &self.api_key, t).await
    }
}
