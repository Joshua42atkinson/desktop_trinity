//! Trinity Memory API Routes
//!
//! Endpoints for semantic search over Trinity's long-term memory.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use trinity_core::learning::MemorySource;

/// Query parameters for memory recall
#[derive(Debug, Deserialize)]
pub struct RecallQuery {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    5
}

/// Request to store a memory fragment
#[derive(Debug, Deserialize)]
pub struct StoreMemoryRequest {
    pub content: String,
    pub source: String,
}

/// A memory fragment returned from recall
#[derive(Debug, Serialize)]
pub struct MemoryResponse {
    pub id: Uuid,
    pub content: String,
    pub source: String,
    pub relevance: f32,
    pub created_at: String,
}

/// Memory system statistics
#[derive(Debug, Serialize)]
pub struct MemoryStats {
    pub total_fragments: usize,
    pub conversations_stored: usize,
    pub facts_learned: usize,
}

/// Generate a simple deterministic embedding for a query
/// TODO: Replace with real embedding model via trinity-core
fn generate_query_embedding(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    const EMBEDDING_DIM: usize = 384;

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    let mut embedding = Vec::with_capacity(EMBEDDING_DIM);
    let mut seed = hash;

    for _ in 0..EMBEDDING_DIM {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let value = ((seed >> 33) as f32) / (u32::MAX as f32) - 0.5;
        embedding.push(value);
    }

    // Normalize
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    for x in &mut embedding {
        *x /= magnitude;
    }

    embedding
}

/// Semantic search over memory
async fn recall_memories(
    State(state): State<AppState>,
    Query(params): Query<RecallQuery>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    tracing::info!(
        "Memory: Recalling for query '{}' (limit: {})",
        params.query,
        params.limit
    );

    let query_embedding = generate_query_embedding(&params.query);

    match state
        .memory
        .vector_store()
        .search(&query_embedding, params.limit)
        .await
    {
        Ok(fragments) => {
            let results: Vec<MemoryResponse> = fragments
                .iter()
                .map(|f| {
                    let source_str = match &f.source {
                        trinity_core::learning::MemorySource::Conversation { session_id } => {
                            format!("conversation:{}", session_id)
                        }
                        trinity_core::learning::MemorySource::Document {
                            doc_id,
                            chunk_index,
                        } => {
                            format!("document:{}:{}", doc_id, chunk_index)
                        }
                        trinity_core::learning::MemorySource::Insight { derived_from } => {
                            format!("insight:{}", derived_from.len())
                        }
                    };

                    MemoryResponse {
                        id: f.id,
                        content: f.content.clone(),
                        source: source_str,
                        relevance: f.relevance,
                        created_at: f.created_at.to_rfc3339(),
                    }
                })
                .collect();

            Ok(Json(results))
        }
        Err(e) => {
            tracing::error!("Memory recall failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Store a memory fragment explicitly
async fn store_memory(
    State(state): State<AppState>,
    Json(request): Json<StoreMemoryRequest>,
) -> Result<Json<MemoryResponse>, StatusCode> {
    tracing::info!("Memory: Storing fragment from '{}'", request.source);

    let id = Uuid::new_v4();
    let embedding = generate_query_embedding(&request.content);
    let created_at = chrono::Utc::now();

    // Parse source type from string
    let source = if request.source.starts_with("conversation:") {
        MemorySource::Conversation {
            session_id: Uuid::new_v4(),
        }
    } else {
        MemorySource::Document {
            doc_id: Uuid::new_v4(),
            chunk_index: 0,
        }
    };

    match state
        .memory
        .vector_store()
        .store(id, &request.content, &source, &embedding, created_at)
        .await
    {
        Ok(()) => Ok(Json(MemoryResponse {
            id,
            content: request.content,
            source: request.source,
            relevance: 1.0,
            created_at: created_at.to_rfc3339(),
        })),
        Err(e) => {
            tracing::error!("Memory store failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get memory system statistics
async fn get_stats(State(state): State<AppState>) -> Result<Json<MemoryStats>, StatusCode> {
    match state.memory.vector_store().count().await {
        Ok(count) => {
            Ok(Json(MemoryStats {
                total_fragments: count,
                conversations_stored: 0, // TODO: Track by source type
                facts_learned: 0,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to get memory stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Trigger memory consolidation (admin endpoint)
async fn trigger_consolidation(State(_state): State<AppState>) -> StatusCode {
    tracing::info!("Memory: Triggering consolidation cycle");
    // TODO: Integrate MemoryConsolidator when PostgreSQL is connected
    StatusCode::ACCEPTED
}

/// Seed example memories for testing
async fn seed_memories(
    State(state): State<AppState>,
) -> Result<Json<Vec<MemoryResponse>>, StatusCode> {
    tracing::info!("Memory: Seeding example memories");

    let examples = vec![
        ("Trinity can help with Rust programming, including async/await patterns and error handling.", "system:capabilities"),
        ("The AMD Strix Halo APU has 128GB unified memory, perfect for running large LLMs locally.", "knowledge:hardware"),
        ("Video game development involves game design, programming, art assets, and audio production.", "knowledge:gamedev"),
        ("Machine learning models can be quantized to reduce memory usage while maintaining accuracy.", "knowledge:ml"),
        ("The frontend uses Leptos, a Rust web framework with fine-grained reactivity.", "knowledge:stack"),
        ("Yesterday we discussed optimizing inference performance on the local GPU.", "conversation:recent"),
        ("User prefers dark mode interfaces with minimal distractions.", "preference:ui"),
        ("Code reviews should focus on correctness, readability, and performance in that order.", "insight:coding"),
    ];

    let mut responses = Vec::new();

    for (content, source) in examples {
        let id = Uuid::new_v4();
        let embedding = generate_query_embedding(content);
        let created_at = chrono::Utc::now();

        let mem_source = if source.starts_with("conversation:") {
            MemorySource::Conversation {
                session_id: Uuid::new_v4(),
            }
        } else if source.starts_with("insight:") {
            MemorySource::Insight {
                derived_from: vec![Uuid::new_v4()],
            }
        } else {
            MemorySource::Document {
                doc_id: Uuid::new_v4(),
                chunk_index: 0,
            }
        };

        if let Ok(()) = state
            .memory
            .vector_store()
            .store(id, content, &mem_source, &embedding, created_at)
            .await
        {
            responses.push(MemoryResponse {
                id,
                content: content.to_string(),
                source: source.to_string(),
                relevance: 1.0,
                created_at: created_at.to_rfc3339(),
            });
        }
    }

    tracing::info!("Seeded {} example memories", responses.len());
    Ok(Json(responses))
}

/// Create memory routes
pub fn memory_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/memory/recall", get(recall_memories))
        .route("/api/memory/store", post(store_memory))
        .route("/api/memory/stats", get(get_stats))
        .route("/api/memory/consolidate", post(trigger_consolidation))
        .route("/api/memory/seed", post(seed_memories))
        .with_state(state.clone())
}
