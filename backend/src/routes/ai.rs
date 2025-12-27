use crate::handlers::ai::handle_ai_inference;
use crate::AppState;
use axum::{routing::post, Router};

pub fn ai_routes(state: &AppState) -> Router<AppState> {
    Router::new().route("/api/ai/inference", post(handle_ai_inference))
}
