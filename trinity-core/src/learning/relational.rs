//! SQLite Relational Store for Trinity
//!
//! Handles structured data: conversation metadata, document records, learned facts.
//! Uses embedded SQLite via rusqlite - no external database server required.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::{MemoryFragment, MemorySource, MemoryStats};

/// SQLite-backed relational store for structured memory data
pub struct RelationalStore {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl RelationalStore {
    /// Create a new relational store at the given path
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open SQLite database: {:?}", db_path))?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };

        store.run_migrations()?;
        Ok(store)
    }

    /// Create with default path (~/.trinity/memory.db)
    pub fn default_path() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let db_path = PathBuf::from(home).join(".trinity").join("memory.db");
        Self::new(db_path)
    }

    /// Run database migrations
    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS memory_fragments (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                metadata TEXT DEFAULT '{}'
            );

            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                started_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_activity TEXT NOT NULL DEFAULT (datetime('now')),
                turn_count INTEGER NOT NULL DEFAULT 0,
                metadata TEXT DEFAULT '{}'
            );

            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                content_type TEXT NOT NULL,
                chunk_count INTEGER NOT NULL DEFAULT 0,
                ingested_at TEXT NOT NULL DEFAULT (datetime('now')),
                metadata TEXT DEFAULT '{}'
            );

            CREATE INDEX IF NOT EXISTS idx_memory_source_type ON memory_fragments(source_type);
            CREATE INDEX IF NOT EXISTS idx_memory_created_at ON memory_fragments(created_at);
            "#,
        )
        .context("Failed to run migrations")?;

        tracing::info!("SQLite migrations complete: {:?}", self.db_path);
        Ok(())
    }

    /// Store a memory fragment record
    pub fn store_fragment(
        &self,
        id: Uuid,
        content: &str,
        source: &MemorySource,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let (source_type, source_id) = match source {
            MemorySource::Conversation { session_id } => {
                ("conversation", Some(session_id.to_string()))
            }
            MemorySource::Document {
                doc_id,
                chunk_index,
            } => ("document", Some(format!("{}:{}", doc_id, chunk_index))),
            MemorySource::Insight { derived_from } => {
                let ids: Vec<String> = derived_from.iter().map(|u| u.to_string()).collect();
                ("insight", Some(ids.join(",")))
            }
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT OR REPLACE INTO memory_fragments (id, content, source_type, source_id, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                id.to_string(),
                content,
                source_type,
                source_id,
                metadata.unwrap_or(serde_json::json!({})).to_string()
            ],
        )?;

        Ok(())
    }

    /// Record a new document ingestion
    pub fn record_document(
        &self,
        id: Uuid,
        name: &str,
        content_type: &str,
        chunk_count: usize,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO documents (id, name, content_type, chunk_count)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![id.to_string(), name, content_type, chunk_count as i32],
        )?;

        Ok(())
    }

    /// Update or create a conversation record
    pub fn update_conversation(&self, session_id: Uuid, turn_count: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO conversations (id, turn_count, last_activity)
            VALUES (?1, ?2, datetime('now'))
            ON CONFLICT(id) DO UPDATE SET
                turn_count = ?2,
                last_activity = datetime('now')
            "#,
            params![session_id.to_string(), turn_count],
        )?;

        Ok(())
    }

    /// Get memory statistics
    pub fn stats(&self) -> Result<MemoryStats> {
        let conn = self.conn.lock().unwrap();

        let total: i64 = conn.query_row("SELECT COUNT(*) FROM memory_fragments", [], |row| {
            row.get(0)
        })?;

        let conversations: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_fragments WHERE source_type = 'conversation'",
            [],
            |row| row.get(0),
        )?;

        let documents: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_fragments WHERE source_type = 'document'",
            [],
            |row| row.get(0),
        )?;

        let insights: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memory_fragments WHERE source_type = 'insight'",
            [],
            |row| row.get(0),
        )?;

        let size_bytes: i64 = conn.query_row(
            "SELECT COALESCE(SUM(LENGTH(content)), 0) FROM memory_fragments",
            [],
            |row| row.get(0),
        )?;

        Ok(MemoryStats {
            total_fragments: total as usize,
            conversation_count: conversations as usize,
            document_count: documents as usize,
            insight_count: insights as usize,
            size_bytes: size_bytes as u64,
        })
    }

    /// Get recent memory fragments for consolidation
    pub fn recent_fragments(&self, limit: usize) -> Result<Vec<MemoryFragment>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, content, source_type, source_id, created_at
            FROM memory_fragments
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map([limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        let mut fragments = Vec::new();
        for row_result in rows {
            let (id_str, content, source_type, source_id, created_at_str) = row_result?;

            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            let source = match source_type.as_str() {
                "conversation" => MemorySource::Conversation {
                    session_id: source_id.and_then(|s| s.parse().ok()).unwrap_or_default(),
                },
                "document" => {
                    let parts: Vec<&str> = source_id.as_deref().unwrap_or("").split(':').collect();
                    MemorySource::Document {
                        doc_id: parts
                            .first()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or_default(),
                        chunk_index: parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                    }
                }
                _ => MemorySource::Insight {
                    derived_from: source_id
                        .map(|s| s.split(',').filter_map(|id| id.parse().ok()).collect())
                        .unwrap_or_default(),
                },
            };

            fragments.push(MemoryFragment {
                id,
                content,
                source,
                relevance: 1.0,
                created_at,
            });
        }

        Ok(fragments)
    }

    /// Get the database path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_relational_store_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = RelationalStore::new(db_path.clone());
        assert!(store.is_ok());
        assert!(db_path.exists());
    }

    #[test]
    fn test_store_and_retrieve_fragment() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = RelationalStore::new(db_path).unwrap();

        let id = Uuid::new_v4();
        let source = MemorySource::Conversation {
            session_id: Uuid::new_v4(),
        };

        store
            .store_fragment(id, "Test content", &source, None)
            .unwrap();

        let stats = store.stats().unwrap();
        assert_eq!(stats.total_fragments, 1);
        assert_eq!(stats.conversation_count, 1);
    }

    #[test]
    fn test_recent_fragments() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = RelationalStore::new(db_path).unwrap();

        let session_id = Uuid::new_v4();
        for i in 0..5 {
            let id = Uuid::new_v4();
            let source = MemorySource::Conversation { session_id };
            store
                .store_fragment(id, &format!("Content {}", i), &source, None)
                .unwrap();
        }

        let recent = store.recent_fragments(3).unwrap();
        assert_eq!(recent.len(), 3);
    }
}
