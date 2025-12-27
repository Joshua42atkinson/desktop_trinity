use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    domain::vaam::{VaamService, VocabWord, WordUsageRequest},
    Result, AppState,
};

/// GET /api/vaam/context/:tag
/// Returns the "Lexical Inventory" for a specific scene.
pub async fn get_context_inventory(
    State(app_state): State<AppState>,
    Path(tag): Path<String>,
) -> Result<Json<Vec<VocabWord>>> {
    let words = VaamService::get_words_for_context(&app_state.pool, &tag).await?;
    Ok(Json(words))
}

/// POST /api/vaam/log
/// The game client calls this when a player chooses a dialogue option.
/// Returns: true if the player just achieved mastery, false otherwise.
pub async fn log_word_usage(
    State(app_state): State<AppState>,
    Json(payload): Json<WordUsageRequest>,
) -> Result<Json<bool>> {
    let mastered_just_now = VaamService::log_usage(&app_state.pool, payload).await?;
    Ok(Json(mastered_just_now))
}
