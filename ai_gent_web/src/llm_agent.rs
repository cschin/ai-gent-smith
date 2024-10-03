use axum::async_trait;

use ai_gent_lib::fsm::FSMBuilder;

use ai_gent_lib::llm_agent::{FSMAgentConfigBuilder, LLMAgent, LLMClient};

// use futures::StreamExt;
use ai_gent_lib::llm_service::{openai_service, openai_stream_service, LLMStreamOut};

pub struct OAI_LLMClient {}


#[async_trait]
impl LLMClient for OAI_LLMClient {
    async fn generate(&self, prompt: &str, msgs: &[(String, String)]) -> String {
        // r#"{"message": "Test response", "tool": null, "tool_input": null, "next_state": null}"#
        //     .to_string()
        openai_service(prompt, msgs).await
    }

    async fn generate_stream(&self, prompt: &str, msgs: &[(String, String)]) -> LLMStreamOut {
        openai_stream_service(prompt, msgs).await
    }
}
