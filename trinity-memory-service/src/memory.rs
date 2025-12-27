//! Memory Store - Vector database backed by sled
//!
//! Stores memories with hash-based embeddings for similarity search.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;
use uuid::Uuid;

use crate::{MemoryFragment, StatsResponse};

// ============================================================================
// Internal Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct StoredMemory {
    id: Uuid,
    content: String,
    source: String,
    embedding: Vec<f32>,
    session_id: Option<Uuid>,
    metadata: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
}

// ============================================================================
// Memory Store
// ============================================================================

pub struct MemoryStore {
    db: Db,
    embedding_dim: usize,
}

impl MemoryStore {
    /// Create a new memory store
    pub fn new(data_dir: &Path) -> Result<Self> {
        let db_path = data_dir.join("memories.sled");
        let db = sled::open(&db_path)?;

        tracing::info!("📦 Opened sled database at {}", db_path.display());

        Ok(Self {
            db,
            embedding_dim: 128,
        })
    }

    /// Store a memory
    pub fn store(
        &self,
        content: &str,
        source: Option<&str>,
        session_id: Option<Uuid>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let embedding = self.hash_embed(content);

        let memory = StoredMemory {
            id,
            content: content.to_string(),
            source: source.unwrap_or("user").to_string(),
            embedding,
            session_id,
            metadata,
            created_at: Utc::now(),
        };

        let key = id.as_bytes().to_vec();
        let value = serde_json::to_vec(&memory)?;

        self.db.insert(key, value)?;
        self.db.flush()?;

        tracing::debug!("Stored memory {}", id);
        Ok(id)
    }

    /// Recall memories similar to query
    pub fn recall(
        &self,
        query: &str,
        limit: usize,
        session_filter: Option<Uuid>,
    ) -> Result<Vec<MemoryFragment>> {
        let query_embedding = self.hash_embed(query);

        let mut scored: Vec<(f32, StoredMemory)> = Vec::new();

        for result in self.db.iter() {
            let (_, value) = result?;
            let memory: StoredMemory = serde_json::from_slice(&value)?;

            // Filter by session if specified
            if let Some(sid) = session_filter {
                if memory.session_id != Some(sid) {
                    continue;
                }
            }

            let similarity = self.cosine_similarity(&query_embedding, &memory.embedding);
            scored.push((similarity, memory));
        }

        // Sort by similarity descending
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N
        let results: Vec<MemoryFragment> = scored
            .into_iter()
            .take(limit)
            .map(|(sim, mem)| MemoryFragment {
                id: mem.id,
                content: mem.content,
                source: mem.source,
                timestamp: mem.created_at,
                similarity: sim,
            })
            .collect();

        Ok(results)
    }

    /// Get memory statistics
    pub fn stats(&self) -> Result<StatsResponse> {
        let total = self.db.len();
        let size = self.db.size_on_disk().unwrap_or(0);

        // Count unique sessions
        let mut sessions = std::collections::HashSet::new();
        for result in self.db.iter() {
            let (_, value) = result?;
            let memory: StoredMemory = serde_json::from_slice(&value)?;
            if let Some(sid) = memory.session_id {
                sessions.insert(sid);
            }
        }

        Ok(StatsResponse {
            total_memories: total,
            storage_bytes: size,
            sessions: sessions.len(),
        })
    }

    // ========================================================================
    // Embedding Functions
    // ========================================================================

    /// Simple hash-based embedding (can upgrade to real model later)
    fn hash_embed(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let words: Vec<&str> = text.split_whitespace().collect();
        let mut embedding = vec![0.0f32; self.embedding_dim];

        for (i, word) in words.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            word.to_lowercase().hash(&mut hasher);
            let hash = hasher.finish();

            // Distribute hash into embedding dimensions
            for j in 0..8 {
                let idx = ((hash >> (j * 8)) as usize + i) % self.embedding_dim;
                let val = ((hash >> (j * 4)) & 0xFF) as f32 / 255.0;
                embedding[idx] += val;
            }
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        embedding
    }

    /// Cosine similarity between two embeddings
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}
