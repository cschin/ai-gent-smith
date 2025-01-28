// use std::sync::Arc;

use std::pin::Pin;

use genai::chat::{ChatMessage, ChatRequest, ChatStreamEvent, StreamChunk};
use genai::Client;

use futures::{Stream, StreamExt};

pub type LLMStreamOut = Pin<Box<dyn Stream<Item = Option<String>> + Send>>;

pub async fn openai_stream_service(prompt: &str, msgs: &[(String, String)]) -> LLMStreamOut {
    let mut messages: Vec<ChatMessage> = vec![ChatMessage::system(prompt.to_string())];

    msgs.iter().for_each(|(role, msg)| match role.as_str() {
        "user" => {
            messages.push(ChatMessage::user(msg.clone()));
        }
        "assistant" => {
            messages.push(ChatMessage::assistant(msg.clone()));
        }
        _ => {}
    });

    let chat_req = ChatRequest::new(messages);

    let client = Client::default();

    let llm_stream = client
        .exec_chat_stream("gpt-4o", chat_req.clone(), None)
        .await
        .unwrap()
        .stream;

    let llm_output = StreamExt::then(llm_stream, |result| async {
        match result {
            Ok(response) => {
                match response {
                    ChatStreamEvent::Start => Some("".to_string()),
                    ChatStreamEvent::Chunk(StreamChunk { content }) => Some(content.to_string()),
                    ChatStreamEvent::End(_end_event) => None,
                }
                // if let Some(choice) = response.choices.first() {
                //     choice.delta.content.clone()
                // } else {
                //     None
                // }
            }
            Err(_err) => {
                tracing::info!(target: "log", "LLM stream error");
                None
            }
        }
    });

    Box::pin(llm_output)
}

pub async fn openai_service(prompt: &str, msgs: &[(String, String)]) -> String {
    let mut messages: Vec<ChatMessage> = vec![ChatMessage::system(prompt.to_string())];

    msgs.iter().for_each(|(role, msg)| match role.as_str() {
        "user" => {
            messages.push(ChatMessage::user(msg.clone()));
        }
        "assistant" => {
            messages.push(ChatMessage::assistant(msg.clone()));
        }
        _ => {}
    });

    let chat_req = ChatRequest::new(messages);

    let client = Client::default();



    let llm_output = client
        .exec_chat("gpt-4o", chat_req.clone(), None)
        .await
        .unwrap()
        .content_text_into_string()
        .unwrap();

    llm_output
}
