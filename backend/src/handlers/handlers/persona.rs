use axum::{extract::State, Json};
use common::{Dilemma, Archetype, DilemmaChoice};
use std::collections::HashMap;
use crate::{AppState, Result};

pub async fn get_dilemmas(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<Dilemma>>> {
    let dilemma_rows = sqlx::query("SELECT id, title, dilemma_text FROM dilemmas")
        .fetch_all(&app_state.pool)
        .await?;

    let choice_rows = sqlx::query("SELECT id, dilemma_id, choice_text FROM dilemma_choices")
        .fetch_all(&app_state.pool)
        .await?;

    let mut choices_map: HashMap<i32, Vec<DilemmaChoice>> = HashMap::new();
    for row in choice_rows {
        let choice = DilemmaChoice {
            id: row.get("id"),
            dilemma_id: row.get("dilemma_id"),
            choice_text: row.get("choice_text"),
        };
        choices_map.entry(choice.dilemma_id).or_default().push(choice);
    }

    let dilemmas = dilemma_rows.into_iter().map(|row| {
        let dilemma_id: i32 = row.get("id");
        Dilemma {
            id: dilemma_id,
            title: row.get("title"),
            dilemma_text: row.get("dilemma_text"),
            choices: choices_map.remove(&dilemma_id).unwrap_or_default(),
        }
    }).collect();

    Ok(Json(dilemmas))
}

pub async fn get_archetypes(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<Archetype>>> {
    let rows = sqlx::query("SELECT id, name, description FROM archetypes")
        .fetch_all(&app_state.pool)
        .await?;

    let archetypes = rows.into_iter().map(|row| {
        Archetype {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
        }
    }).collect();

    Ok(Json(archetypes))
}

use common::{QuizSubmission, PlayerCharacter};
use crate::domain::persona_logic::calculate_archetype;
use tokio::sync::oneshot;
use crate::AppError;
use sqlx::Row;

pub async fn submit_quiz(
    State(app_state): State<AppState>,
    Json(submission): Json<QuizSubmission>,
) -> Result<Json<PlayerCharacter>> {
    let result = calculate_archetype(&app_state.pool, &submission).await?;

    let (one_tx, one_rx) = oneshot::channel();
    let command = format!("set_archetype {} {}", result.primary_archetype.id, serde_json::to_string(&result.stats).unwrap());
    app_state.tx.send((command, one_tx)).await.map_err(|_| AppError::InternalServerError)?;

    let updated_player = one_rx.await.map_err(|_| AppError::InternalServerError)?;

    Ok(Json(updated_player))
}
