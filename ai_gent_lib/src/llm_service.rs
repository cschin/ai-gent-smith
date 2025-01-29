// use std::sync::Arc;

use std::pin::Pin;

use genai::chat::{ChatMessage, ChatRequest, ChatStreamEvent, StreamChunk};
use genai::resolver::{AuthData, AuthResolver};
use genai::Client;

use futures::{Stream, StreamExt};

pub type LLMStreamOut = Pin<Box<dyn Stream<Item = Option<String>> + Send>>;

pub async fn genai_stream_service(
    prompt: &str,
    msgs: &[(String, String)],
    model: &str,
    api_key: &str,
) -> LLMStreamOut {
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

    let api_key = api_key.to_string();
    let auth_resolver =
        AuthResolver::from_resolver_fn(|_| Ok(Some(AuthData::from_single(api_key))));

    let client = Client::builder().with_auth_resolver(auth_resolver).build();

    let llm_stream = client
        .exec_chat_stream(model, chat_req.clone(), None)
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

pub async fn genai_service(
    prompt: &str,
    msgs: &[(String, String)],
    model: &str,
    api_key: &str,
) -> String {

    tracing::info!(target: "tron_app", "in genai_service, prompt: {}", prompt );
    tracing::info!(target: "tron_app", "in genai_service, msg: {:?}", msgs );
    tracing::info!(target: "tron_app", "in genai_service, model: {}", model );


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

    let api_key = api_key.to_string();
    let auth_resolver =
        AuthResolver::from_resolver_fn(|_| Ok(Some(AuthData::from_single(api_key))));

    let client = Client::builder().with_auth_resolver(auth_resolver).build();

    let llm_output = client
        .exec_chat(model, chat_req.clone(), None)
        .await;
    tracing::info!(target: "tron_app", "in genai_service, llm_output: {:?}", llm_output );

    llm_output.unwrap().content_text_into_string().unwrap()
}
