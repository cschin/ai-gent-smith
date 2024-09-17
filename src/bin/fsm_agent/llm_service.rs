// use std::sync::Arc;

use std::pin::Pin;

#[allow(unused_imports)]
use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use futures::{Stream, StreamExt};

pub type LLMStreamOut = Pin<Box<dyn Stream<Item = Option<String>> + Send>>;

pub async fn openai_stream_service(prompt: &str, query: &str) -> LLMStreamOut {
    let mut messages: Vec<ChatCompletionRequestMessage> =
        vec![ChatCompletionRequestSystemMessageArgs::default()
            .content(prompt)
            .build()
            .expect("error")
            .into()];

    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(query)
            .build()
            .expect("error")
            .into(),
    );

    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(2048u16)
        //.model("gpt-4")
        .model("gpt-4o")
        .messages(messages)
        .build()
        .expect("error");

    let llm_stream = client
        .chat()
        .create_stream(request)
        .await
        .expect("create stream fail for LLM API call");

    let llm_output = StreamExt::then(llm_stream, |result| async {
        match result {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    choice.delta.content.clone()
                } else {
                    None
                }
            }
            Err(_err) => {
                tracing::info!(target: "log", "LLM stream error");
                None
            }
        }
    });

    Box::pin(llm_output)
}
