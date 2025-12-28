// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Advanced Memory System with Vector Indexing
//!
//! ## Philosophy
//! "Memory is the foundation of intelligence. Without efficient recall,
//!  the agent cannot learn from experience or maintain context."
//!
//! ## Architecture
//! - **SQLite**: Persistence layer for metadata and content
//! - **In-Memory Index**: Brute-force cosine for small datasets (<10k)
//! - **Partitioned Storage**: Separate sections for code, conversations, documents
//! - **Caching**: LRU cache for hot embeddings
//!
//! ## Future: Lance DB Integration
//! When datasets exceed 100k vectors, migrate to Lance DB for:
//! - ANN (Approximate Nearest Neighbor) search
//! - Disk-based vector storage
//! - IVF-PQ index compression

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

// ============================================================================
// Memory Entry Types
// ============================================================================

/// Memory fragment stored in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub embedding: Vec<f32>,
    pub source_type: MemorySource,
    pub metadata: MemoryMetadata,
    pub created_at: i64,
}

/// Source type for memories (enables filtered recall)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemorySource {
    /// Conversation history
    Conversation,
    /// Code snippets and implementations
    Code,
    /// Documentation and notes
    Document,
    /// Task execution history
    Task,
    /// System events and logs
    System,
    /// User preferences and facts
    UserContext,
}

impl MemorySource {
    fn as_str(&self) -> &'static str {
        match self {
            MemorySource::Conversation => "conversation",
            MemorySource::Code => "code",
            MemorySource::Document => "document",
            MemorySource::Task => "task",
            MemorySource::System => "system",
            MemorySource::UserContext => "user_context",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "conversation" => MemorySource::Conversation,
            "code" => MemorySource::Code,
            "document" => MemorySource::Document,
            "task" => MemorySource::Task,
            "system" => MemorySource::System,
            "user_context" => MemorySource::UserContext,
            _ => MemorySource::System,
        }
    }
}

/// Additional metadata for memories
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// File path if code-related
    pub file_path: Option<String>,
    /// Language if code
    pub language: Option<String>,
    /// Summary/title
    pub summary: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Importance score (0.0-1.0)
    pub importance: f32,
    /// Access count (for LRU)
    pub access_count: u32,
}

/// Result of a memory recall
#[derive(Debug, Clone)]
pub struct RecallResult {
    pub entry: MemoryEntry,
    pub similarity: f32,
    pub rank: usize,
}

// ============================================================================
// Advanced Memory Store
// ============================================================================

/// Configuration for the memory store
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Path to the SQLite database
    pub db_path: String,
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Maximum entries before suggesting pruning
    pub max_entries: usize,
    /// Enable in-memory caching
    pub enable_cache: bool,
    /// Cache size (number of embeddings)
    pub cache_size: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            db_path: "trinity_memory.db".to_string(),
            embedding_dim: 384,
            max_entries: 100_000,
            enable_cache: true,
            cache_size: 10_000,
        }
    }
}

/// Advanced unified memory store with vector indexing
pub struct AdvancedMemory {
    /// SQLite connection for persistence
    conn: Arc<Mutex<Connection>>,
    /// Embedding dimension
    embedding_dim: usize,
    /// In-memory embedding cache (id -> embedding)
    embedding_cache: Arc<RwLock<HashMap<Uuid, Vec<f32>>>>,
    /// Configuration
    config: MemoryConfig,
}

impl AdvancedMemory {
    /// Create or open a memory store with advanced features
    pub fn open(config: MemoryConfig) -> Result<Self> {
        let path = Path::new(&config.db_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        // Initialize tables with enhanced schema
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                embedding BLOB NOT NULL,
                source_type TEXT NOT NULL,
                metadata TEXT,
                created_at INTEGER NOT NULL,
                access_count INTEGER DEFAULT 0
            )
            "#,
            [],
        )?;

        // Indexes for efficient filtering
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_source ON memories(source_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at DESC)",
            [],
        )?;

        // Full-text search on content (SQLite FTS5)
        let _ = conn.execute(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                content,
                content='memories',
                content_rowid='rowid'
            )
            "#,
            [],
        );

        let embedding_dim = config.embedding_dim;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            embedding_dim,
            embedding_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Store a memory with its embedding
    pub fn store(
        &self,
        content: &str,
        embedding: &[f32],
        source: MemorySource,
        metadata: Option<MemoryMetadata>,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let created_at = chrono::Utc::now().timestamp();

        // Serialize embedding to bytes
        let embedding_bytes: Vec<u8> = embedding
            .iter()
            .flat_map(|f| f.to_le_bytes().to_vec())
            .collect();

        // Serialize metadata
        let metadata_json = serde_json::to_string(&metadata.unwrap_or_default())?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO memories (id, content, embedding, source_type, metadata, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id.to_string(),
                content,
                embedding_bytes,
                source.as_str(),
                metadata_json,
                created_at
            ],
        )?;

        // Cache the embedding
        if self.config.enable_cache {
            let mut cache = self.embedding_cache.write().unwrap();
            if cache.len() < self.config.cache_size {
                cache.insert(id, embedding.to_vec());
            }
        }

        Ok(id)
    }

    /// Recall memories similar to the query embedding
    pub fn recall(
        &self,
        query_embedding: &[f32],
        limit: usize,
        filter: Option<MemorySource>,
    ) -> Result<Vec<RecallResult>> {
        let conn = self.conn.lock().unwrap();

        let query = match filter {
            Some(source) => format!(
                "SELECT id, content, embedding, source_type, metadata, created_at FROM memories WHERE source_type = '{}'",
                source.as_str()
            ),
            None => "SELECT id, content, embedding, source_type, metadata, created_at FROM memories".to_string(),
        };

        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let embedding_bytes: Vec<u8> = row.get(2)?;
            let source_str: String = row.get(3)?;
            let metadata_json: String = row.get(4).unwrap_or_default();
            let created_at: i64 = row.get(5)?;

            // Deserialize embedding
            let embedding: Vec<f32> = embedding_bytes
                .chunks(4)
                .map(|chunk| {
                    let arr: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                    f32::from_le_bytes(arr)
                })
                .collect();

            // Deserialize metadata
            let metadata: MemoryMetadata = serde_json::from_str(&metadata_json).unwrap_or_default();

            Ok(MemoryEntry {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                content,
                embedding,
                source_type: MemorySource::from_str(&source_str),
                metadata,
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

        // Build results with rank
        Ok(scored
            .into_iter()
            .take(limit)
            .enumerate()
            .map(|(rank, (similarity, entry))| RecallResult {
                entry,
                similarity,
                rank,
            })
            .collect())
    }

    /// Recall with hybrid search (vector + keyword)
    pub fn hybrid_recall(
        &self,
        query_embedding: &[f32],
        keywords: &[&str],
        limit: usize,
    ) -> Result<Vec<RecallResult>> {
        // First, get keyword matches
        let keyword_filter = if keywords.is_empty() {
            None
        } else {
            let pattern = keywords.join("%");
            Some(format!("%{}%", pattern))
        };

        let conn = self.conn.lock().unwrap();

        let query = match &keyword_filter {
            Some(pattern) => format!(
                "SELECT id, content, embedding, source_type, metadata, created_at FROM memories WHERE content LIKE '{}'",
                pattern
            ),
            None => "SELECT id, content, embedding, source_type, metadata, created_at FROM memories".to_string(),
        };

        let mut stmt = conn.prepare(&query)?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let embedding_bytes: Vec<u8> = row.get(2)?;
            let source_str: String = row.get(3)?;
            let metadata_json: String = row.get(4).unwrap_or_default();
            let created_at: i64 = row.get(5)?;

            let embedding: Vec<f32> = embedding_bytes
                .chunks(4)
                .map(|chunk| {
                    let arr: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                    f32::from_le_bytes(arr)
                })
                .collect();

            let metadata: MemoryMetadata = serde_json::from_str(&metadata_json).unwrap_or_default();

            Ok(MemoryEntry {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                content,
                embedding,
                source_type: MemorySource::from_str(&source_str),
                metadata,
                created_at,
            })
        })?;

        let mut scored: Vec<(f32, MemoryEntry)> = Vec::new();

        for row in rows {
            let entry = row?;
            let vector_score = cosine_similarity(query_embedding, &entry.embedding);

            // Boost score if keywords match
            let keyword_boost = if keyword_filter.is_some() { 0.2 } else { 0.0 };
            let final_score = vector_score + keyword_boost;

            scored.push((final_score, entry));
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .take(limit)
            .enumerate()
            .map(|(rank, (similarity, entry))| RecallResult {
                entry,
                similarity,
                rank,
            })
            .collect())
    }

    /// Get memories by source type
    pub fn get_by_source(&self, source: MemorySource, limit: usize) -> Result<Vec<MemoryEntry>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, content, embedding, source_type, metadata, created_at FROM memories WHERE source_type = ?1 ORDER BY created_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![source.as_str(), limit as i64], |row| {
            let id_str: String = row.get(0)?;
            let content: String = row.get(1)?;
            let embedding_bytes: Vec<u8> = row.get(2)?;
            let source_str: String = row.get(3)?;
            let metadata_json: String = row.get(4).unwrap_or_default();
            let created_at: i64 = row.get(5)?;

            let embedding: Vec<f32> = embedding_bytes
                .chunks(4)
                .map(|chunk| {
                    let arr: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                    f32::from_le_bytes(arr)
                })
                .collect();

            let metadata: MemoryMetadata = serde_json::from_str(&metadata_json).unwrap_or_default();

            Ok(MemoryEntry {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                content,
                embedding,
                source_type: MemorySource::from_str(&source_str),
                metadata,
                created_at,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    /// Update access count (for LRU tracking)
    pub fn touch(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE memories SET access_count = access_count + 1 WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    /// Delete old memories (pruning)
    pub fn prune_old(&self, older_than_days: u32, max_keep: usize) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp() - (older_than_days as i64 * 24 * 60 * 60);

        let conn = self.conn.lock().unwrap();

        // Count current entries
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?;

        if count as usize <= max_keep {
            return Ok(0);
        }

        // Delete oldest entries beyond max_keep, prioritizing low access count
        let deleted = conn.execute(
            "DELETE FROM memories WHERE id IN (
                SELECT id FROM memories 
                WHERE created_at < ?1 
                ORDER BY access_count ASC, created_at ASC 
                LIMIT ?2
            )",
            params![cutoff, count as i64 - max_keep as i64],
        )?;

        // Clear cache for deleted items (evict oldest if over size limit)
        if self.config.enable_cache {
            let mut cache = self.embedding_cache.write().unwrap();
            // Simple eviction: clear cache after prune to ensure consistency
            // Future: maintain a list of deleted IDs for selective removal
            if cache.len() > self.config.cache_size {
                cache.clear();
            }
        }

        Ok(deleted)
    }

    /// Get total count of memories
    pub fn count(&self) -> usize {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
            .unwrap_or(0);
        count as usize
    }

    /// Get count by source type
    pub fn count_by_source(&self, source: MemorySource) -> usize {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM memories WHERE source_type = ?1",
                params![source.as_str()],
                |row| row.get(0),
            )
            .unwrap_or(0);
        count as usize
    }

    /// Get embedding dimension
    pub fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }

    /// Clear all memories
    pub fn clear(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM memories", [])?;

        if self.config.enable_cache {
            let mut cache = self.embedding_cache.write().unwrap();
            cache.clear();
        }

        Ok(())
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        let conn = self.conn.lock().unwrap();

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
            .unwrap_or(0);

        let by_source: HashMap<String, usize> = {
            let mut stmt = conn
                .prepare("SELECT source_type, COUNT(*) FROM memories GROUP BY source_type")
                .ok();

            if let Some(ref mut s) = stmt {
                s.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
                })
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
                .unwrap_or_default()
            } else {
                HashMap::new()
            }
        };

        let cache_size = self.embedding_cache.read().unwrap().len();

        MemoryStats {
            total_entries: total as usize,
            entries_by_source: by_source,
            cache_size,
            cache_enabled: self.config.enable_cache,
            max_entries: self.config.max_entries,
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub entries_by_source: HashMap<String, usize>,
    pub cache_size: usize,
    pub cache_enabled: bool,
    pub max_entries: usize,
}

// ============================================================================
// Helper Functions
// ============================================================================

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
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
    }

    #[test]
    fn test_memory_source_roundtrip() {
        let sources = [
            MemorySource::Conversation,
            MemorySource::Code,
            MemorySource::Document,
            MemorySource::Task,
        ];

        for source in sources {
            assert_eq!(MemorySource::from_str(source.as_str()), source);
        }
    }

    #[test]
    fn test_memory_store() {
        let config = MemoryConfig {
            db_path: "/tmp/test_memory.db".to_string(),
            ..Default::default()
        };

        let store = AdvancedMemory::open(config).unwrap();
        store.clear().unwrap();

        let embedding = vec![0.1; 384];
        let id = store
            .store("Test content", &embedding, MemorySource::Code, None)
            .unwrap();

        assert_ne!(id, Uuid::nil());
        assert_eq!(store.count(), 1);
    }
}
