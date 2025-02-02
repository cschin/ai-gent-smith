use candle_transformers::models::jina_bert::{BertModel, Config};

use anyhow::Error as E;
use candle_core::{DType, Device, Module, Tensor};
use candle_nn::VarBuilder;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgRange;
use tokenizers::{Encoding, Tokenizer};
use tokio::sync::OnceCell;
use tron_app::TRON_APP;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Cursor, Write};
use std::ops::Bound;

use pgvector::Vector;

pub struct EmbeddingService {
    model: BertModel,
    normalize_embeddings: bool,
}

pub struct TextChunkingService {
    tokenizer: Tokenizer,
    chunk_size: usize,
    chunk_overlap: usize,
    tokenize_max_tokens: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EmbeddingChunk {
    pub text: String,
    pub span: (usize, usize),
    pub token_ids: Option<Vec<u32>>,
    pub embedding_vec: Option<Vec<f32>>,
}

impl TextChunkingService {
    pub fn new(
        tokenizer: Option<String>,
        chunk_size: usize,
        chunk_overlap: usize,
        tokenize_max_tokens: usize,
    ) -> Self {
        let make_tokenizer = || -> anyhow::Result<tokenizers::Tokenizer> {
            use hf_hub::{api::sync::Api, Repo, RepoType};
            let tokenizer = match tokenizer {
                Some(file) => std::path::PathBuf::from(file),
                None => Api::new()?
                    .repo(Repo::new(
                        "sentence-transformers/all-MiniLM-L6-v2".to_string(),
                        RepoType::Model,
                    ))
                    .get("tokenizer.json")?,
            };

            let mut tokenizer = tokenizers::Tokenizer::from_file(tokenizer).map_err(E::msg)?;

            if let Some(pp) = tokenizer.get_truncation_mut() {
                pp.max_length = tokenize_max_tokens;
            } else {
                let pp = tokenizers::TruncationParams {
                    max_length: tokenize_max_tokens,
                    ..Default::default()
                };
                let _ = tokenizer.with_truncation(Some(pp));
            }

            Ok(tokenizer)
        };

        let tokenizer = make_tokenizer().expect("fail to build the tokenizer");

        Self {
            tokenizer,
            chunk_size,
            chunk_overlap,
            tokenize_max_tokens,
        }
    }

    pub fn get_token_ids(&self, s: &str) -> anyhow::Result<Encoding> {
        self.tokenizer.encode(s, false).map_err(E::msg)
    }

    pub fn text_to_chunks(&self, text: &str) -> Vec<EmbeddingChunk> {
        let text = text.chars().collect::<Vec<char>>();
        let mut segment_start = 0_usize;
        let mut segment_end = self.tokenize_max_tokens * 2; // assume average token length is greater than 2
        let mut chunks = Vec::new();

        let mut process_segment =
            |segment_start: usize, segment: &str, encoding: Encoding| -> usize {
                let mut chunk_bgn = 0_usize;
                let mut chunk_end = self.chunk_size;
                let token_ids = encoding.get_ids();

                let offsets = encoding
                    .get_offsets()
                    .iter()
                    .filter(|&o| o.0 != 0 || o.1 != 0)
                    .cloned()
                    .collect::<Vec<(usize, usize)>>();

                let next_segment_bgn;
                loop {
                    if chunk_bgn == 0 || chunk_end < offsets.len() {
                        let text_bgn = offsets[chunk_bgn].0;
                        if chunk_end > offsets.len() {
                            chunk_end = offsets.len();
                        };
                        let text_end = offsets[chunk_end - 1].1;
                        let mut tids = token_ids[chunk_bgn..chunk_end].to_vec();
                        if tids.len() < self.chunk_size {
                            tids.extend(vec![0; self.chunk_size - tids.len()]);
                        };
                        //assert!(tids.len() == self.chunk_size);
                        let chunk = EmbeddingChunk {
                            text: segment[text_bgn..text_end].to_string(),
                            span: (segment_start + text_bgn, segment_start + text_end),
                            token_ids: Some(tids),
                            ..Default::default()
                        };
                        chunks.push(chunk);
                        if chunk_end > self.chunk_overlap {
                            chunk_bgn = chunk_end - self.chunk_overlap;
                            chunk_end = chunk_bgn + self.chunk_size;
                        } else {
                            next_segment_bgn = segment_start + segment.chars().count();
                            break;
                        };
                    } else {
                        if chunk_bgn < offsets.len() {
                            let text_bgn = offsets[chunk_bgn].0;
                            next_segment_bgn = segment_start + segment[..text_bgn].chars().count();
                        } else {
                            next_segment_bgn = segment_start + segment.chars().count();
                        }
                        break;
                    };
                }
                next_segment_bgn
            };

        loop {
            if segment_end < text.len() {
                let segment = &text[segment_start..segment_end].iter().collect::<String>();
                let result = self.get_token_ids(segment);
                let encoding = result.unwrap();
                segment_start = process_segment(segment_start, segment, encoding);
                segment_end = segment_start + self.tokenize_max_tokens * 2; // assume average token length is greater than 2
            } else {
                let segment = &text[segment_start..].iter().collect::<String>();
                let result = self.get_token_ids(segment);
                let encoding = result.unwrap();
                segment_start = process_segment(segment_start, segment, encoding);
                break;
            }
        }

        if segment_start < text.len() {
            let segment = &text[segment_start..].iter().collect::<String>();
            let result = self.get_token_ids(segment);
            let encoding = result.unwrap();
            let offsets = encoding
                .get_offsets()
                .iter()
                .filter(|&o| o.0 != 0 || o.1 != 0)
                .cloned()
                .collect::<Vec<(usize, usize)>>();
            let chunk_bgn = 0_usize;
            let chunk_end = offsets.len();
            let token_ids = encoding.get_ids();
            let mut tids = token_ids[chunk_bgn..chunk_end].to_vec();
            if tids.len() < self.chunk_size {
                tids.extend(vec![0; self.chunk_size - tids.len()]);
            };
            let text_bgn = offsets[chunk_bgn].0;
            let text_end = offsets[chunk_end - 1].1;
            assert!(tids.len() == self.chunk_size);
            let chunk = EmbeddingChunk {
                text: segment[text_bgn..text_end].to_string(),
                span: (segment_start + text_bgn, segment_start + text_end),
                token_ids: Some(tids),
                ..Default::default()
            };

            chunks.push(chunk);
        };

        chunks
    }
}

impl EmbeddingService {
    pub fn new(model: Option<String>) -> Self {
        let make_model = || -> anyhow::Result<BertModel> {
            use hf_hub::{api::sync::Api, Repo, RepoType};
            let model = match model {
                Some(model_file) => std::path::PathBuf::from(model_file),
                None => Api::new()?
                    .repo(Repo::new(
                        "jinaai/jina-embeddings-v2-base-en".to_string(),
                        RepoType::Model,
                    ))
                    .get("model.safetensors")?,
            };

            //let device = Device::new_cuda(0)?;
            //let device = Device::new_metal(0).unwrap();
            let device = Device::Cpu;
            let config = Config::v2_base();

            let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model], DType::F32, &device)? };
            let model = BertModel::new(vb, &config)?;
            Ok(model)
        };

        let model = make_model().expect("fail to build the model");

        let normalize_embeddings = false;

        Self {
            model,
            normalize_embeddings,
        }
    }

    pub fn get_embedding_for_chunks(&self, chunks: &mut [EmbeddingChunk]) -> anyhow::Result<()> {
        let device = &self.model.device;

        let embeddings = chunks
            .iter()
            .map(|c| {
                let tokens = c
                    .token_ids
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|&t| *t != 0)
                    .cloned()
                    .collect::<Vec<_>>();
                //assert!(tokens.len() == 256);
                //tracing::info!(target: "tron_app", "tokens {:?}", tokens);
                let token_ids = vec![Tensor::new(tokens.as_slice(), device).unwrap()];
                let token_ids = Tensor::stack(&token_ids, 0).unwrap();
                let embeddings = self.model.forward(&token_ids).unwrap();
                let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3().unwrap();
                let embeddings = (embeddings.sum(1).unwrap() / (n_tokens as f64)).unwrap();
                let embeddings = if self.normalize_embeddings {
                    normalize_l2(&embeddings).unwrap()
                } else {
                    embeddings
                };
                let embeddings = embeddings.to_vec2::<f32>().unwrap();
                embeddings.first().unwrap().clone()
            })
            .collect::<Vec<_>>();
        //let token_ids = Tensor::stack(&token_ids, 0)?;
        //let embeddings = self.model.forward(&token_ids)?;

        (0..chunks.len()).for_each(|i| {
            let v = embeddings.get(i).unwrap().to_vec();
            // assert!(v.len() == 256);
            // assert!(chunks[i].embedding_vec.as_ref().unwrap().len() == 256);
            chunks[i].embedding_vec = Some(v)
        });
        Ok(())
    }
}

pub fn normalize_l2(v: &Tensor) -> candle_core::Result<Tensor> {
    v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)
}

#[derive(Deserialize, Debug, Clone)]
pub struct DocumentChunk {
    pub text: String,
    pub span: (usize, usize),
    pub token_ids: Option<Vec<u32>>,
    pub two_d_embedding: Option<(f32, f32)>,
    pub embedding_vec: Option<Vec<f32>>,
    pub filename: String,
    pub title: String,
}

pub struct DocumentChunks {
    pub chunks: Vec<DocumentChunk>,
}

pub static EMBEDDING_SERVICE: OnceCell<EmbeddingService> = OnceCell::const_new();

pub static DOCUMENT_CHUNKS: OnceCell<DocumentChunks> = OnceCell::const_new();

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl DocumentChunk {
    pub fn get_fid(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.filename.hash(&mut hasher);
        hasher.finish()
    }
}

impl DocumentChunks {
    pub fn global() -> &'static DocumentChunks {
        DOCUMENT_CHUNKS
            .get()
            .expect("document chunks are not initialized")
    }

    pub fn from_gz_file(filename: String) -> Option<DocumentChunks> {
        let mut chunks = Vec::new();
        let file = BufReader::new(File::open(filename).unwrap());
        let decoder = GzDecoder::new(file);
        let reader = BufReader::new(decoder);

        tracing::info!(target: "tron_app", "loading embeding data file");
        // Read the file line by line
        let mut count = 0;
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(chunk)  = serde_json::from_str::<DocumentChunk>(&line) {
                chunks.push(chunk);
                count += 1;
                } else {
                    return None
                }
            } else {
                return None
            }
        }
        tracing::info!(target: TRON_APP, "{} records loaded", count);

        Some(DocumentChunks { chunks })
    }

    pub fn from_gz_data(data: &[u8]) -> Option<DocumentChunks> {
        let cursor = Cursor::new(data);
        let mut chunks = Vec::new();
        let decoder = GzDecoder::new(cursor);
        let reader = BufReader::new(decoder);

        tracing::info!(target: "tron_app", "loading embeding from upload data");
        // Read the file line by line
        let mut count = 0;
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(chunk)  = serde_json::from_str::<DocumentChunk>(&line) {
                chunks.push(chunk);
                count += 1;
                } else {
                    return None
                }
            } else {
                return None
            }
        }
        tracing::info!(target: TRON_APP, "{} records loaded", count);

        Some (DocumentChunks { chunks })
    }

    pub fn from_data(data: &[u8]) -> Option<DocumentChunks> {
        let cursor = Cursor::new(data);
        let reader = BufReader::new(cursor);
        let mut chunks = Vec::new();
        
        tracing::info!(target: "tron_app", "loading embeding from upload data");
        // Read the file line by line
        let mut count = 0;
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(chunk)  = serde_json::from_str::<DocumentChunk>(&line) {
                chunks.push(chunk);
                count += 1;
                } else {
                    return None
                }
            } else {
                return None
            }
        }
        tracing::info!(target: TRON_APP, "{} records loaded", count);

        Some (DocumentChunks { chunks })
    }
}

fn bound_to_usize(bound: Bound<i32>) -> Option<usize> {
    match bound {
        Bound::Included(value) => Some(value as usize), // Convert Included value
        Bound::Excluded(value) => Some(value as usize), // Convert Excluded value
        Bound::Unbounded => None,                       // Unbounded cannot be converted to usize
    }
}

pub async fn initialize_embedding_model() {
    let _result = EMBEDDING_SERVICE
        .get_or_init(|| async {
            println!("load embedding model");
            let es = EmbeddingService::new(None);
            println!("finish loading embedding model");
            es
        })
        .await;
}

use ordered_float::OrderedFloat;
use std::collections::BinaryHeap;

use crate::DB_POOL;

#[derive(Debug, Clone)]
pub struct TwoDPoint {
    pub d: OrderedFloat<f64>,
    pub point: (f64, f64),
    pub chunk: DocumentChunk,
}

impl Ord for TwoDPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.d.cmp(&self.d)
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for TwoDPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TwoDPoint {
    fn eq(&self, other: &Self) -> bool {
        self.d == other.d
    }
}

impl Eq for TwoDPoint {}

use sqlx::Row;

pub async fn vector_query_and_sort_points(
    asset_id: i32,
    ref_vec: &[f32],
    top_k: Option<i32>,
) -> Vec<TwoDPoint> {
    //tracing::info!(target:"tron_app", "ref_vec:{:?}", ref_vec);
    let mut all_points = Vec::new();

    let v0 = Vector::from(ref_vec.to_vec());
    let db_pool = DB_POOL.clone();

    let results = if let Some(top_k) = top_k {
        sqlx::query(
            r#"SELECT filename, title, text, span, embedding_vector, 
                   COALESCE(two_d_embedding, '[0.0, 0.0]'::vector) AS two_d_embedding, 
                   1.0 - (embedding_vector <=> $1) AS similarity
                   FROM text_embedding
                   WHERE asset_id = $2
                   ORDER BY similarity DESC LIMIT $3;"#)
            .bind(v0)
            .bind(asset_id)
            .bind(top_k)
            .fetch_all(&db_pool)
            .await
    } else {
        sqlx::query(
        r#"SELECT filename, title, text, span, embedding_vector, 
                       COALESCE(two_d_embedding, '[0.0, 0.0]'::vector) AS two_d_embedding, 
                       1.0 - (embedding_vector <=> $1) AS similarity
               FROM text_embedding
               WHERE asset_id = $2
               ORDER BY similarity DESC;"#)
        .bind(v0)
        .bind(asset_id)
        .fetch_all(&db_pool)
        .await
    };

    if let Ok(rows) = results {
        for r in rows {
            let p = pgrow_to_point(r);
            all_points.push(p);
        }
    }

    all_points.sort();
    all_points.reverse();
    all_points
}

pub async fn get_all_points(asset_id: i32) -> Vec<TwoDPoint> {
    //tracing::info!(target:"tron_app", "ref_vec:{:?}", ref_vec);
    let mut all_points = Vec::new();

    let db_pool = DB_POOL.clone();

    let results = sqlx::query(
        r#"SELECT filename, title, text, span, embedding_vector, 
                       COALESCE(two_d_embedding, '[0.0, 0.0]'::vector) AS two_d_embedding, 
                       CAST(0.0 AS FLOAT8) AS similarity 
               FROM text_embedding
               WHERE asset_id = $1;"#)
        .bind(asset_id)
    .fetch_all(&db_pool)
    .await;

    if let Ok(rows) = results {
        for r in rows {
            let p = pgrow_to_point(r);
            all_points.push(p);
        }
    }

    all_points.sort();
    all_points.reverse();
    all_points
}

fn pgrow_to_point(r: sqlx::postgres::PgRow) -> TwoDPoint {
    let span = r.get::<PgRange<i32>, &str>("span");
    let span = (
        bound_to_usize(span.start).unwrap(),
        bound_to_usize(span.end).unwrap(),
    );
    let embedding_vec = r.get::<Vector, &str>("embedding_vector").to_vec();
    let two_d_embedding = r.get::<Vector, &str>("two_d_embedding").to_vec();
    let chunk = DocumentChunk {
        embedding_vec: Some(embedding_vec),
        filename: r.get::<String, &str>("filename"),
        span,
        token_ids: None,
        two_d_embedding: Some((two_d_embedding[0], two_d_embedding[1])),
        text: r.get::<String, &str>("text"),
        title: r.get::<String, &str>("title"),
    };
    let d = OrderedFloat::from(1.0 - r.get::<f64, &str>("similarity"));
    let point = chunk.two_d_embedding.unwrap();
    let point = (point.0 as f64, point.1 as f64);
    TwoDPoint { d, chunk, point }
}
