use crate::handlers::research::{get_research_log, get_virtue_topology};
use crate::AppState;
use axum::{routing::get, Router};

pub fn research_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/research/log", get(get_research_log))
        .route("/api/research/virtues", get(get_virtue_topology))
        .with_state(state.clone())
}
