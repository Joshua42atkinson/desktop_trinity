//! TrinityNotebook API Routes
//!
//! Endpoints for source ingestion and RAG-powered queries.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

/// Request to add a text source
#[derive(Debug, Deserialize)]
pub struct AddSourceRequest {
    pub name: String,
    pub content: String,
}

/// Request to query the notebook
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

/// Response for a source
#[derive(Debug, Serialize)]
pub struct SourceResponse {
    pub id: Uuid,
    pub name: String,
    pub chunk_count: usize,
    pub ingested_at: String,
}

/// Response for RAG query
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub answer: String,
    pub citations: Vec<CitationResponse>,
}

/// Citation in query response
#[derive(Debug, Serialize)]
pub struct CitationResponse {
    pub source_id: Uuid,
    pub text_snippet: String,
    pub relevance: f32,
}

/// Notebook statistics
#[derive(Debug, Serialize)]
pub struct NotebookStats {
    pub source_count: usize,
    pub total_chunks: usize,
}

/// Add a text source to the notebook
async fn add_source(
    State(state): State<AppState>,
    Json(request): Json<AddSourceRequest>,
) -> Result<Json<SourceResponse>, StatusCode> {
    let mut notebook = state.notebook.write().await;

    match notebook
        .add_text_source(&request.name, &request.content)
        .await
    {
        Ok(source) => {
            tracing::info!(
                "Notebook: Added source '{}' ({} chunks)",
                source.name,
                source.chunk_count
            );
            Ok(Json(SourceResponse {
                id: source.id,
                name: source.name,
                chunk_count: source.chunk_count,
                ingested_at: source.ingested_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to add source: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List all sources in the notebook
async fn list_sources(
    State(state): State<AppState>,
) -> Result<Json<Vec<SourceResponse>>, StatusCode> {
    let notebook = state.notebook.read().await;
    let sources: Vec<SourceResponse> = notebook
        .sources()
        .iter()
        .map(|s| SourceResponse {
            id: s.id,
            name: s.name.clone(),
            chunk_count: s.chunk_count,
            ingested_at: s.ingested_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(sources))
}

/// Query the notebook with RAG
async fn query_notebook(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    tracing::info!("Notebook: Query '{}'", request.query);

    let notebook = state.notebook.read().await;

    match notebook.query(&request.query).await {
        Ok(rag_response) => {
            let citations: Vec<CitationResponse> = rag_response
                .citations
                .iter()
                .map(|c| CitationResponse {
                    source_id: c.source_id,
                    text_snippet: c.text_snippet.clone(),
                    relevance: c.relevance,
                })
                .collect();

            Ok(Json(QueryResponse {
                answer: rag_response.answer,
                citations,
            }))
        }
        Err(e) => {
            tracing::error!("Notebook query failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a source from the notebook
async fn delete_source(State(state): State<AppState>, Path(source_id): Path<Uuid>) -> StatusCode {
    let mut notebook = state.notebook.write().await;

    if notebook.remove_source(source_id).is_some() {
        tracing::info!("Notebook: Deleted source {}", source_id);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Get notebook statistics
async fn get_stats(State(state): State<AppState>) -> Result<Json<NotebookStats>, StatusCode> {
    let notebook = state.notebook.read().await;
    let stats = notebook.stats();

    Ok(Json(NotebookStats {
        source_count: stats.source_count,
        total_chunks: stats.total_chunks,
    }))
}

/// Create notebook routes
pub fn notebook_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/notebook/sources", post(add_source))
        .route("/api/notebook/sources", get(list_sources))
        .route("/api/notebook/sources/{source_id}", delete(delete_source))
        .route("/api/notebook/query", post(query_notebook))
        .route("/api/notebook/stats", get(get_stats))
        .with_state(state.clone())
}
