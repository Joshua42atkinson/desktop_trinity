#![allow(unused)]
use common::{Archetype, QuizSubmission};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

pub struct ArchetypeCalculationResult {
    pub primary_archetype: Archetype,
    pub stats: HashMap<String, i32>,
}

pub async fn calculate_archetype(
    pool: &PgPool,
    submission: &QuizSubmission,
) -> Result<ArchetypeCalculationResult, String> {
    let choice_ids: Vec<i32> = submission.answers.values().cloned().collect();

    let points_rows = sqlx::query(
        "SELECT archetype_id, points FROM dilemma_choice_archetype_points WHERE dilemma_choice_id = ANY($1)"
    )
    .bind(&choice_ids)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut archetype_scores: HashMap<i32, i32> = HashMap::new();
    for row in points_rows {
        let archetype_id: i32 = row.get("archetype_id");
        let points: i32 = row.get("points");
        *archetype_scores.entry(archetype_id).or_insert(0) += points;
    }

    let primary_archetype_id = archetype_scores
        .into_iter()
        .max_by_key(|&(_, score)| score)
        .map(|(id, _)| id)
        .ok_or("No archetype points found for submission")?;

    let archetype_row = sqlx::query("SELECT id, name, description FROM archetypes WHERE id = $1")
        .bind(primary_archetype_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    let primary_archetype = Archetype {
        id: archetype_row.get("id"),
        name: archetype_row.get("name"),
        description: archetype_row.get("description"),
    };

    let stat_buff_rows = sqlx::query(
        "SELECT s.name, asb.buff_value FROM archetype_stat_buffs asb
         JOIN stats s ON asb.stat_id = s.id
         WHERE asb.archetype_id = $1",
    )
    .bind(primary_archetype_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let stats = stat_buff_rows
        .into_iter()
        .map(|row| {
            let stat_name: String = row.get("name");
            let buff_value: i32 = row.get("buff_value");
            (stat_name, buff_value)
        })
        .collect();

    Ok(ArchetypeCalculationResult {
        primary_archetype,
        stats,
    })
}
