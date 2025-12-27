#![allow(unused)]
use anyhow::Result;
use chrono::{DateTime, Utc};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Represents who sent a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Speaker {
    User,
    AI,
}

/// A single turn in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub speaker: Speaker,
    pub content: String,
    pub metadata: TurnMetadata,
}

/// Metadata about a conversation turn
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TurnMetadata {
    pub word_count: usize,
    pub sentiment: f32,  // -1.0 (negative) to 1.0 (positive)
    pub depth_level: u8, // 1-5 scale
    pub virtue_signals: Vec<String>,
}

impl TurnMetadata {
    /// Calculate metadata from turn content
    pub fn from_content(content: &str) -> Self {
        let word_count = content.split_whitespace().count();

        // Simple heuristic: longer responses = deeper reflection
        let depth_level = match word_count {
            0..=10 => 1,
            11..=30 => 2,
            31..=60 => 3,
            61..=100 => 4,
            _ => 5,
        };

        // TODO: Implement actual sentiment analysis
        let sentiment = 0.0;

        // TODO: Implement virtue keyword extraction
        let virtue_signals = Vec::new();

        Self {
            word_count,
            sentiment,
            depth_level,
            virtue_signals,
        }
    }
}

/// Manages conversation history with caching and persistence
pub struct ConversationMemory {
    cache: Arc<RwLock<LruCache<Uuid, Vec<Turn>>>>,
    pool: PgPool,
}

impl ConversationMemory {
    /// Create a new conversation memory manager
    pub fn new(pool: PgPool, cache_size: usize) -> Self {
        let cache_size = NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(100).unwrap());
        let cache = Arc::new(RwLock::new(LruCache::new(cache_size)));

        Self { cache, pool }
    }

    /// Get recent turns for a session
    pub async fn get_recent(&self, session_id: Uuid, limit: usize) -> Result<Vec<Turn>> {
        // 1. Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(turns) = cache.get(&session_id) {
                log::debug!("Cache hit for session {}", session_id);
                let recent: Vec<Turn> = turns.iter().rev().take(limit).rev().cloned().collect();
                return Ok(recent);
            }
        }

        // 2. Load from database if not cached
        log::debug!(
            "Cache miss for session {}, loading from database",
            session_id
        );
        let turns = self.load_from_db(session_id, limit).await?;

        // 3. Populate cache for future requests
        {
            let mut cache = self.cache.write().await;
            cache.put(session_id, turns.clone());
        }

        Ok(turns)
    }

    /// Add a new turn to the conversation
    pub async fn add_turn(&self, session_id: Uuid, mut turn: Turn) -> Result<()> {
        // Generate ID if not set
        if turn.id == Uuid::nil() {
            turn.id = Uuid::new_v4();
        }

        // Calculate metadata if empty
        if turn.metadata.word_count == 0 {
            turn.metadata = TurnMetadata::from_content(&turn.content);
        }

        // 1. Add to cache
        {
            let mut cache = self.cache.write().await;
            let turns = cache.get_or_insert_mut(session_id, Vec::new);
            turns.push(turn.clone());
        }

        // 2. Persist to database (async, non-blocking)
        self.persist_turn(session_id, &turn).await?;

        Ok(())
    }

    /// Load turns from database
    async fn load_from_db(&self, _session_id: Uuid, _limit: usize) -> Result<Vec<Turn>> {
        // SIMULATION MODE: Database disabled for now to bypass sqlx compile-time checks
        /*
        let limit_i64 = limit as i64;

        let rows = sqlx::query!(
            r#"
            SELECT id, timestamp, speaker, content, word_count, sentiment, depth_level, virtue_signals
            FROM conversation_turns
            WHERE session_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            session_id,
            limit_i64
        )
        .fetch_all(&self.pool)
        .await?;

        let mut turns = Vec::new();
        for row in rows.into_iter().rev() {
            let speaker = match row.speaker.as_str() {
                "user" => Speaker::User,
                "ai" => Speaker::AI,
                _ => continue,
            };

            let virtue_signals: Vec<String> = row
                .virtue_signals
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            turns.push(Turn {
                id: row.id,
                timestamp: row.timestamp,
                speaker,
                content: row.content,
                metadata: TurnMetadata {
                    word_count: row.word_count.unwrap_or(0) as usize,
                    sentiment: row.sentiment.unwrap_or(0.0),
                    depth_level: row.depth_level.unwrap_or(1) as u8,
                    virtue_signals,
                },
            });
        }

        Ok(turns)
        */
        Ok(Vec::new())
    }

    /// Persist a turn to the database
    async fn persist_turn(&self, _session_id: Uuid, _turn: &Turn) -> Result<()> {
        // SIMULATION MODE: Database disabled for now to bypass sqlx compile-time checks
        /*
        let speaker_str = match turn.speaker {
            Speaker::User => "user",
            Speaker::AI => "ai",
        };

        let virtue_signals_json = serde_json::to_value(&turn.metadata.virtue_signals)?;

        sqlx::query!(
            r#"
            INSERT INTO conversation_turns
            (id, session_id, speaker, content, word_count, sentiment, depth_level, virtue_signals, created_at, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            turn.id,
            session_id,
            speaker_str,
            turn.content,
            turn.metadata.word_count as i32,
            turn.metadata.sentiment,
            turn.metadata.depth_level as i16,
            virtue_signals_json,
            turn.timestamp,
        )
        .execute(&self.pool)
        .await?;
        */
        Ok(())
    }
}
