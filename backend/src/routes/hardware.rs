use crate::ai::hardware::detect_hardware;
use crate::AppState;
use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use trinity_core::brain::tiered::BrainTier;

#[derive(Serialize)]
pub struct ModelStatusResponse {
    pub loaded_models: Vec<String>,
    pub total_vram_gb: f32,
    pub used_vram_gb: f32,
    pub active_tier: Option<String>,
}

pub async fn get_hardware_info() -> impl IntoResponse {
    let info = detect_hardware();
    Json(info)
}

pub async fn get_model_status(State(state): State<AppState>) -> impl IntoResponse {
    let summary = state.orchestrator.memory_summary().await;
    let tiers = state.orchestrator.loaded_tiers().await;

    // We can't access active_tier directly on orchestrator as it's not exposed publically easily
    // But we can infer or just return what we have.
    // Actually TieredBrainManager has active_tier() but Orchestrator wraps it in a private field `manager`.
    // Let's just return memory summary for now.

    let model_names: Vec<String> = tiers.iter().map(|t| format!("{:?}", t)).collect();

    Json(ModelStatusResponse {
        loaded_models: model_names,
        total_vram_gb: summary.total_vram_gb,
        used_vram_gb: summary.used_gb,
        active_tier: None, // Placeholder
    })
}

pub fn hardware_routes() -> Router<AppState> {
    Router::new()
        .route("/api/hardware", get(get_hardware_info))
        .route("/api/models/status", get(get_model_status))
}
