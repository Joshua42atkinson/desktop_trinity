// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Unified Memory System (Rusqlite Backed)
//!
//! Vector + relational memory storage for Trinity Genesis.
//! Uses SQLite for persistence, storing embeddings as BLOBs.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Memory fragment stored in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub embedding: Vec<f32>,
    pub source_type: String,
    pub created_at: i64,
}

/// Unified memory interface
pub struct UnifiedMemory {
    conn: Arc<Mutex<Connection>>,
    embedding_dim: usize,
}

impl UnifiedMemory {
    /// Create or open a memory store at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        // Initialize tables
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                embedding BLOB NOT NULL,
                source_type TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
            [],
        )?;

        // Index on source type for filtering
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_source ON memories(source_type)",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            embedding_dim: 384,
        })
    }

    /// Store a memory with its embedding
    pub fn store(&self, content: &str, embedding: &[f32], source: &str) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let created_at = chrono::Utc::now().timestamp();

        // Serialize embedding to bytes (f32 is 4 bytes)
        let embedding_bytes: Vec<u8> = embedding
            .iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect();

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO memories (id, content, embedding, source_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id.to_string(),
                content,
                embedding_bytes,
                source,
                created_at
            ],
        )?;

        Ok(id)
    }

    /// Recall memories similar to the query embedding
    ///
    /// Note: This performs a full table scan and in-memory cosine similarity.
    /// Provide enough RAM or limit the row count if dataset grows > 100k.
    pub fn recall(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<MemoryEntry>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt =
            conn.prepare("SELECT id, content, embedding, source_type, created_at FROM memories")?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let embedding_bytes: Vec<u8> = row.get(2)?;
            let source_type: String = row.get(3)?;
            let created_at: i64 = row.get(4)?;

            // Desertialize embedding
            let embedding: Vec<f32> = embedding_bytes
                .chunks(4)
                .map(|chunk| {
                    let arr: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                    f32::from_le_bytes(arr)
                })
                .collect();

            Ok(MemoryEntry {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                content,
                embedding,
                source_type,
                created_at,
            })
        })?;

        let mut scored: Vec<(f32, MemoryEntry)> = Vec::new();

        for row in rows {
            let entry = row?;
            let score = cosine_similarity(query_embedding, &entry.embedding);
            scored.push((score, entry));
        }

        // Sort by score descending
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored.into_iter().take(limit).map(|(_, e)| e).collect())
    }

    /// Get total count of memories
    pub fn count(&self) -> usize {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
            .unwrap_or(0);
        count as usize
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }

    /// Clear all memories (useful for tests/reset)
    pub fn clear(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM memories", [])?;
        Ok(())
    }
}

/// Compute cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 0.001);
    }
}
