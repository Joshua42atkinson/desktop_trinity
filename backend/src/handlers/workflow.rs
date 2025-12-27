use crate::agent::workflow::SharedWorkflowState;
use crate::AppState;
use axum::{extract::State, response::IntoResponse, Json};

pub async fn get_workflow_state(State(app_state): State<AppState>) -> impl IntoResponse {
    if let Ok(state) = app_state.shared_workflow_state.read() {
        Json(state.clone())
    } else {
        Json(SharedWorkflowState::default())
    }
}
