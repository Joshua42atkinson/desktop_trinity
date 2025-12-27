//! Trinity Memory Service
//!
//! Standalone REST API for distributed memory storage.
//! Runs on the laptop and provides memory services to the desktop.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use uuid::Uuid;

mod memory;
use memory::MemoryStore;

// ============================================================================
// API Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreRequest {
    pub content: String,
    pub source: Option<String>,
    pub session_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreResponse {
    pub id: Uuid,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct RecallQuery {
    pub query: String,
    pub limit: Option<usize>,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryFragment {
    pub id: Uuid,
    pub content: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub similarity: f32,
}

#[derive(Debug, Serialize)]
pub struct RecallResponse {
    pub memories: Vec<MemoryFragment>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_memories: usize,
    pub storage_bytes: u64,
    pub sessions: usize,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

// ============================================================================
// App State
// ============================================================================

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<MemoryStore>,
    pub start_time: std::time::Instant,
}

// ============================================================================
// Handlers
// ============================================================================

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

async fn store_memory(
    State(state): State<AppState>,
    Json(req): Json<StoreRequest>,
) -> Result<Json<StoreResponse>, (StatusCode, String)> {
    let id = state
        .store
        .store(&req.content, req.source.as_deref(), req.session_id, req.metadata)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(StoreResponse { id, success: true }))
}

async fn recall_memories(
    State(state): State<AppState>,
    Query(params): Query<RecallQuery>,
) -> Result<Json<RecallResponse>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(10);
    
    let memories = state
        .store
        .recall(&params.query, limit, params.session_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RecallResponse {
        total: memories.len(),
        memories,
    }))
}

async fn get_stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, (StatusCode, String)> {
    let stats = state
        .store
        .stats()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(stats))
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    info!("🧠 Trinity Memory Service starting...");

    // Setup storage directory
    let data_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".trinity_memory");
    
    std::fs::create_dir_all(&data_dir)?;
    info!("📁 Data directory: {}", data_dir.display());

    // Initialize memory store
    let store = MemoryStore::new(&data_dir)?;
    info!("✅ Memory store initialized");

    let state = AppState {
        store: Arc::new(store),
        start_time: std::time::Instant::now(),
    };

    // Build router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/store", post(store_memory))
        .route("/recall", get(recall_memories))
        .route("/stats", get(get_stats))
        .layer(cors)
        .with_state(state);

    // Bind to all interfaces for Tailscale access
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("🌐 Listening on http://{}", addr);
    info!("📡 Remote access via Tailscale: http://100.84.217.60:8080");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
