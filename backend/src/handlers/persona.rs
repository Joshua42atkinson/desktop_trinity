use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use common::{Archetype, Dilemma, DilemmaChoice};
use sqlx::Row;
use std::collections::HashMap;

pub async fn get_dilemmas(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<Dilemma>>, (StatusCode, String)> {
    let pool = match app_state.pool {
        Some(ref p) => p,
        None => return Ok(Json(Vec::new())),
    };

    let dilemma_rows = match sqlx::query("SELECT id, title, dilemma_text FROM dilemmas")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch dilemmas: {}", e),
            ));
        }
    };

    let choice_rows = match sqlx::query("SELECT id, dilemma_id, choice_text FROM dilemma_choices")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch dilemma choices: {}", e),
            ));
        }
    };

    let mut choices_map: HashMap<i32, Vec<DilemmaChoice>> = HashMap::new();
    for row in choice_rows {
        let choice = DilemmaChoice {
            id: row.get("id"),
            dilemma_id: row.get("dilemma_id"),
            choice_text: row.get("choice_text"),
        };
        choices_map
            .entry(choice.dilemma_id)
            .or_default()
            .push(choice);
    }

    let dilemmas = dilemma_rows
        .into_iter()
        .map(|row| {
            let dilemma_id: i32 = row.get("id");
            Dilemma {
                id: dilemma_id,
                title: row.get("title"),
                dilemma_text: row.get("dilemma_text"),
                choices: choices_map.remove(&dilemma_id).unwrap_or_default(),
            }
        })
        .collect();

    Ok(Json(dilemmas))
}

pub async fn get_archetypes(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<Archetype>>, (StatusCode, String)> {
    let pool = match app_state.pool {
        Some(ref p) => p,
        None => return Ok(Json(Vec::new())),
    };

    let rows = match sqlx::query("SELECT id, name, description FROM archetypes")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch archetypes: {}", e),
            ));
        }
    };

    let archetypes = rows
        .into_iter()
        .map(|row| Archetype {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
        })
        .collect();

    Ok(Json(archetypes))
}

use crate::domain::player::get_simulated_character;
use common::{PlayerCharacter, QuizSubmission};

pub async fn submit_quiz(
    State(_app_state): State<AppState>,
    Json(_submission): Json<QuizSubmission>,
) -> Result<Json<PlayerCharacter>, (StatusCode, String)> {
    // Mock implementation while AsyncWorld/tx is disabled
    Ok(Json(get_simulated_character()))
}
