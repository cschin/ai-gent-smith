use axum::{http::Method, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::embedding_service::{EMBEDDING_SERVICE, TEXT_CHUNKING_SERVICE, EmbeddingChunk};

#[derive(Serialize)]
struct Response {
    message: String,
    data: Option<Vec<EmbeddingChunk>>,
}

#[derive(Serialize, Deserialize)]
struct TextIuputForEmbeddingService {
    text: String,
}

pub async fn text_to_embedding(
    _method: Method,
    Json(payload): Json<Value>,
) -> impl IntoResponse {

    let input: Result<TextIuputForEmbeddingService, _> = serde_json::from_value(payload);

    match input {
        Ok(text_input) => {
            // Process the text_input
            // For now, we'll just return a success message
            let mut chunks: Vec<EmbeddingChunk> = TEXT_CHUNKING_SERVICE.get().unwrap().text_to_chunks(&text_input.text);
            let _ = EMBEDDING_SERVICE.get().unwrap().get_embedding_for_chunks(&mut chunks);
            let response = Response {
                message: "succeed".into(),
                data: Some(chunks)
            };
            Json(response)
        }
        Err(e) => {
            let response = Response {
                message: format!("Error parsing input: {}", e),
                data: None
            };
            Json(response)
        }
    }
}