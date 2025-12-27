use crate::{AppState, Result};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AiRequest {
    pub prompt: String,
}

#[derive(Serialize)]
pub struct AiResponse {
    pub result: String,
}

pub async fn handle_ai_inference(
    State(_app_state): State<AppState>,
    Json(payload): Json<AiRequest>,
) -> Result<Json<AiResponse>> {
    // MANDATORY PATTERN: Use spawn_blocking for heavy compute
    // This prevents blocking the Tokio runtime worker threads
    let result = tokio::task::spawn_blocking(move || {
        // Simulate heavy blocking work (e.g., AI inference)
        std::thread::sleep(std::time::Duration::from_secs(2));
        format!("Processed: {}", payload.prompt)
    })
    .await
    .unwrap();

    Ok(Json(AiResponse { result }))
}
