use crate::AppState;
use axum::{
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[allow(dead_code)]
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
}

pub fn chat_routes() -> Router<AppState> {
    Router::new().route("/", post(handle_chat))
}

async fn handle_chat(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    // Deserialize session_id or create a new one
    let session_id = payload
        .session_id
        .and_then(|s| uuid::Uuid::parse_str(&s).ok())
        .unwrap_or_else(uuid::Uuid::new_v4); // Note: ideally we'd persist this new ID back to client

    // Use ChatEngine
    match state.chat_engine.chat(session_id, &payload.message).await {
        Ok(response) => {
            // Note: In a real app we should return the session_id if new
            Json(ChatResponse { response })
        }
        Err(e) => Json(ChatResponse {
            response: format!("Error processing request: {}", e),
        }),
    }
}
