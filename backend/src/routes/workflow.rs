use crate::handlers::workflow::get_workflow_state;
use crate::AppState;
use axum::{routing::get, Router};

pub fn workflow_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/workflow/state", get(get_workflow_state))
        .with_state(state.clone())
}
