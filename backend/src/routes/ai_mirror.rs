use crate::handlers::ai_mirror::{handle_create_session, handle_get_history, handle_send_message};
use crate::AppState;
use axum::{routing::post, Router};

pub fn ai_mirror_routes() -> Router<AppState> {
    Router::new()
        .route("/create-session", post(handle_create_session))
        .route("/send-message", post(handle_send_message))
        .route("/get-history", post(handle_get_history))
}
