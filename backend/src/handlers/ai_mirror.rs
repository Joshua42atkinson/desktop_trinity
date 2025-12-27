#![allow(unused)]
use crate::ai::{SessionContext, SocraticEngine};
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub user_id: i64,
    pub message: String,
    pub archetype: Option<String>,
    pub focus_area: Option<String>,
}

#[derive(Serialize)]
pub struct SendMessageResponse {
    pub ai_response: String,
    pub session_id: Uuid,
}

/// Handle a message from the user and return AI's Socratic response
pub async fn handle_send_message(
    State(app_state): State<AppState>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, (StatusCode, String)> {
    log::info!(
        "Received message from user {} in session {}",
        payload.user_id,
        payload.session_id
    );

    // Build session context
    let context = SessionContext {
        session_id: payload.session_id,
        user_id: payload.user_id,
        archetype: payload.archetype,
        focus_area: payload.focus_area,
    };

    // Get Socratic engine from app state
    // Note: In production, this would be stored in app state
    // For now, we'll need to create it on demand
    // TODO: Move engine initialization to app state

    // For Phase 1, return a hardcoded response until full engine is initialized
    let response_text =
        "I hear what you're saying. Can you tell me more about what that means to you?".to_string();

    log::debug!("Generated response for session {}", payload.session_id);

    Ok(Json(SendMessageResponse {
        ai_response: response_text,
        session_id: payload.session_id,
    }))
}

/// Create a new conversation session
#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
}

pub async fn handle_create_session(
    State(_app_state): State<AppState>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, String)> {
    let session_id = Uuid::new_v4();
    log::info!("Created new conversation session: {}", session_id);

    Ok(Json(CreateSessionResponse { session_id }))
}

/// Get conversation history for a session
#[derive(Deserialize)]
pub struct GetHistoryRequest {
    pub session_id: Uuid,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct ConversationTurn {
    pub speaker: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct GetHistoryResponse {
    pub turns: Vec<ConversationTurn>,
}

pub async fn handle_get_history(
    State(_app_state): State<AppState>,
    Json(payload): Json<GetHistoryRequest>,
) -> Result<Json<GetHistoryResponse>, (StatusCode, String)> {
    log::info!("Fetching history for session {}", payload.session_id);

    // TODO: Implement actual history retrieval from ConversationMemory
    // For Phase 1, return empty history

    Ok(Json(GetHistoryResponse { turns: vec![] }))
}
