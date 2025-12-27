#![allow(unused)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct ReflectionEntry {
    pub id: Uuid,
    pub user_id: i64,
    pub challenge_name: String,
    pub reflection_text: String,
    pub created_at: DateTime<Utc>,
}

pub const CREATE_REFLECTION_ENTRIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS reflection_entries (
    id UUID PRIMARY KEY,
    user_id BIGINT NOT NULL,
    challenge_name TEXT NOT NULL,
    reflection_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
"#;
