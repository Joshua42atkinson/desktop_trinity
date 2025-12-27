#![allow(unused)]
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A word available to be "discovered" or "used" in the game.
#[derive(Debug, Serialize, FromRow)]
pub struct VocabWord {
    pub id: i32,
    pub word: String,
    pub definition: String,
    pub context_tag: Option<String>, // e.g., "throne_room", "market"
    pub complexity_tier: Option<i32>,
}

/// The payload sent by the frontend when a player makes a choice.
#[derive(Debug, Deserialize)]
pub struct WordUsageRequest {
    pub player_id: i32, // In prod, this comes from Auth Token
    pub word_id: i32,
    pub context_used: String,
}

use crate::Result;
use sqlx::PgPool; // Our custom error alias

pub struct VaamService;

impl VaamService {
    /// Fetch words relevant to the current game scene.
    /// e.g., If player enters "Throne Room", fetch "Implore", "Beseech", "Sovereignty".
    pub async fn get_words_for_context(
        _pool: &PgPool,
        _context_tag: &str,
    ) -> Result<Vec<VocabWord>> {
        // SIMULATION MODE: Database disabled
        /*
        let words = sqlx::query_as!(
            VocabWord,
            "SELECT id, word, definition, context_tag, complexity_tier
             FROM vocabulary_words
             WHERE context_tag = $1",
            context_tag
        )
        .fetch_all(pool)
        .await?;

        Ok(words)
        */
        Ok(Vec::new())
    }

    /// The Core Loop: Log usage -> Check Threshold -> Grant Mastery.
    pub async fn log_usage(_pool: &PgPool, _req: WordUsageRequest) -> Result<bool> {
        // SIMULATION MODE: Database disabled
        /*
        // 1. Log the usage event
        // Note: We use a transaction to ensure data integrity
        let mut tx = pool.begin().await?;

        // Insert the raw log (Privacy: We store WHEN and WHERE, but minimizing PII)
        sqlx::query!(
            "INSERT INTO word_usage_logs (player_id, word_id, context_used) VALUES ($1, $2, $3)",
            req.player_id, req.word_id, req.context_used
        )
        .execute(&mut *tx)
        .await?;

        // 2. Update the aggregate mastery record
        // We increment usage count. If it hits 3, we flip 'is_mastered' to true.
        // This is the 'Rule of Three' from Learning Science.
        let record = sqlx::query!(
            "INSERT INTO player_mastery (player_id, word_id, times_used, is_mastered)
             VALUES ($1, $2, 1, false)
             ON CONFLICT (player_id, word_id)
             DO UPDATE SET
                times_used = player_mastery.times_used + 1,
                is_mastered = (player_mastery.times_used + 1) >= 3,
                last_used_at = NOW()
             RETURNING is_mastered",
            req.player_id, req.word_id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(record.is_mastered)
        */
        Ok(false)
    }
}
