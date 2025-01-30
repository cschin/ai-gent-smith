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
    async fn generate(&self, prompt: &str, msgs: &[(String, String)]) -> String {
        // r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
        //     .to_string()
        genai_service(prompt, msgs, &self.model, &self.api_key).await
    }

    async fn generate_stream(&self, prompt: &str, msgs: &[(String, String)]) -> LLMStreamOut {
        genai_stream_service(prompt, msgs, &self.model, &self.api_key).await
    }
}
