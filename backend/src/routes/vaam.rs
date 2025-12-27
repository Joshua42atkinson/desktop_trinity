use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;

use crate::handlers::vaam::{get_context_inventory, log_word_usage};

pub fn vaam_routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/vaam/context/:tag", get(get_context_inventory))
        .route("/api/vaam/log", post(log_word_usage))
        .with_state(app_state.clone())
}
