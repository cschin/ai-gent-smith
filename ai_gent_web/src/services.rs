use axum::{http::Method, response::IntoResponse, Json};
use candle_core::WithDType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::embedding_service::{self, DocumentChunk, EmbeddingChunk, EMBEDDING_SERVICE, TEXT_CHUNKING_SERVICE};

#[derive(Serialize)]
struct EmbeddingServiceResponse {
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
            let response = EmbeddingServiceResponse {
                message: "succeed".into(),
                data: Some(chunks)
            };
            Json(response)
        }
        Err(e) => {
            let response = EmbeddingServiceResponse {
                message: format!("Error parsing input: {}", e),
                data: None
            };
            Json(response)
        }
    }
}


#[derive(Serialize, Deserialize)]
struct IuputForQuery {
    query: String,
    asset_id: i32,
    top_k: Option<usize>,
    threshold: Option<f32>
}

#[derive(Serialize)]
struct QueryServiceResponse {
    message: String,
    data: Option<Vec<ChunkPoint>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentChunkJson {
    pub text: String,
    pub span: (usize, usize),
    pub filename: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct ChunkPoint {
    pub similarity: f32,
    pub chunk: DocumentChunkJson,
}

pub async fn query_for_chunks(
    _method: Method,
    Json(payload): Json<Value>,
) -> impl IntoResponse {

    let input: Result<IuputForQuery, _> = serde_json::from_value(payload);


    match input {
        Ok(input) => {

            let query = &input.query;
            let top_k = input.top_k.unwrap_or(8);
            let threshold = input.threshold.unwrap_or(0.65);
            let chunk_points = embedding_service::search_asset(query, input.asset_id, top_k, threshold).await
            .into_iter()
            .map(|c| {
                ChunkPoint {
                    similarity: 1.0 - c.d.to_f64() as f32,
                    chunk: DocumentChunkJson {
                        text: c.chunk.text,
                        span: c.chunk.span,
                        filename: c.chunk.filename,
                        title: c.chunk.title
                    }
                }
            }).collect::<Vec<_>>();
            let response = QueryServiceResponse {
                message: "succeed".into(),
                data: Some(chunk_points)
            };
            Json(response)
        }
        Err(e) => {
            let response = QueryServiceResponse {
                message: format!("Error parsing input: {}", e),
                data: None
            };
            Json(response)
        }
    }
}