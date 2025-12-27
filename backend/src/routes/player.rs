use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;

use crate::handlers::player::{
    get_profile_data,
    get_player_character,
    get_journal_data,
    handle_submit_command,
    get_player_profile,
};

pub fn player_routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/profile_data", get(get_profile_data))
        .route("/api/player_character", get(get_player_character))
        .route("/api/journal_data", get(get_journal_data))
        .route("/api/submit_command", post(handle_submit_command))
        .route("/api/player_profile", get(get_player_profile))
        .with_state(app_state.clone())
}
