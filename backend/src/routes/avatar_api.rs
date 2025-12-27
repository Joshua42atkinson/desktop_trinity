//! Avatar API - Exposes Trinity's current operational state
//!
//! Reads from the AutonomousRuntime to reflect actual system state.

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use std::sync::{Arc, Mutex};

use crate::agent::autonomous::AutonomousRuntime;
use crate::AppState;

#[derive(Serialize)]
pub struct AvatarStatus {
    pub state: String,
    pub message: String,
    pub pending_tasks: usize,
    pub completed_tasks: usize,
    pub uptime_secs: Option<u64>,
}

/// Shared runtime for avatar status queries
pub type SharedRuntime = Arc<Mutex<AutonomousRuntime>>;

async fn get_avatar_status(
    State(_state): State<AppState>,
    runtime: Option<axum::Extension<SharedRuntime>>,
) -> Json<AvatarStatus> {
    // Try to read from runtime if available
    let (avatar_state, message, pending, completed, uptime) = if let Some(ext) = runtime {
        if let Ok(guard) = ext.lock() {
            let status = guard.queue_status();
            let state_str = if status.pending > 0 {
                "Coding"
            } else if status.is_running {
                "Thinking"
            } else {
                "Idle"
            };
            let msg = if status.pending > 0 {
                format!("Processing {} tasks", status.pending)
            } else if status.is_running {
                "Monitoring systems...".to_string()
            } else {
                "Awaiting instructions.".to_string()
            };
            (
                state_str.to_string(),
                msg,
                status.pending,
                status.completed,
                status.uptime.map(|d| d.as_secs()),
            )
        } else {
            (
                "Idle".to_string(),
                "Runtime locked.".to_string(),
                0,
                0,
                None,
            )
        }
    } else {
        // No runtime extension available - return default idle state
        (
            "Idle".to_string(),
            "Systems nominal.".to_string(),
            0,
            0,
            None,
        )
    };

    Json(AvatarStatus {
        state: avatar_state,
        message,
        pending_tasks: pending,
        completed_tasks: completed,
        uptime_secs: uptime,
    })
}

pub fn avatar_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/game/avatar", get(get_avatar_status))
        .with_state(state.clone())
}
