//! Sled-backed Vector Store for Trinity
//!
//! Provides semantic search over memory fragments using embeddings.
//! Uses sled for persistence with in-memory vector similarity search.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{MemoryFragment, MemorySource, EMBEDDING_DIM};

/// Embedding vector type alias
pub type EmbeddingVector = Vec<f32>;

/// Stored vector entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorEntry {
    id: Uuid,
    content: String,
    source: MemorySource,
    embedding: Vec<f32>,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Sled-backed vector store for semantic memory search
pub struct VectorStore {
    db: Arc<sled::Db>,
    vectors: Arc<RwLock<Vec<VectorEntry>>>,
}

impl VectorStore {
    /// Create a new vector store at the given path
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("trinity_vectors");
        let db = sled::open(&db_path).context("Failed to open sled database")?;

        // Load existing vectors from disk
        let mut vectors = Vec::new();
        for result in db.iter() {
            let (_, value) = result?;
            if let Ok(entry) = serde_json::from_slice::<VectorEntry>(&value) {
                vectors.push(entry);
            }
        }

        tracing::info!(
            "VectorStore: Loaded {} vectors from {}",
            vectors.len(),
            db_path.display()
        );

        Ok(Self {
            db: Arc::new(db),
            vectors: Arc::new(RwLock::new(vectors)),
        })
    }

    /// Store a memory fragment with its embedding
    pub async fn store(
        &self,
        id: Uuid,
        content: &str,
        source: &MemorySource,
        embedding: &[f32],
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        if embedding.len() != EMBEDDING_DIM {
            anyhow::bail!(
                "Embedding dimension mismatch: expected {}, got {}",
                EMBEDDING_DIM,
                embedding.len()
            );
        }

        let entry = VectorEntry {
            id,
            content: content.to_string(),
            source: source.clone(),
            embedding: embedding.to_vec(),
            created_at,
        };

        // Persist to sled
        let key = id.as_bytes().to_vec();
        let value = serde_json::to_vec(&entry)?;
        self.db.insert(key, value)?;
        self.db.flush_async().await?;

        // Add to in-memory index
        let mut vectors = self.vectors.write().await;
        vectors.push(entry);

        tracing::debug!("Stored memory fragment {} in vector store", id);
        Ok(())
    }

    /// Search for similar memories using vector similarity (cosine distance)
    pub async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<MemoryFragment>> {
        if query_embedding.len() != EMBEDDING_DIM {
            anyhow::bail!(
                "Query embedding dimension mismatch: expected {}, got {}",
                EMBEDDING_DIM,
                query_embedding.len()
            );
        }

        let vectors = self.vectors.read().await;

        // Calculate cosine similarity for all vectors
        let mut scored: Vec<(f32, &VectorEntry)> = vectors
            .iter()
            .map(|entry| {
                let similarity = cosine_similarity(query_embedding, &entry.embedding);
                (similarity, entry)
            })
            .collect();

        // Sort by similarity (descending)
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N and convert to MemoryFragment
        let fragments: Vec<MemoryFragment> = scored
            .into_iter()
            .take(limit)
            .map(|(similarity, entry)| MemoryFragment {
                id: entry.id,
                content: entry.content.clone(),
                source: entry.source.clone(),
                relevance: similarity,
                created_at: entry.created_at,
            })
            .collect();

        Ok(fragments)
    }

    /// Get the number of stored memories
    pub async fn count(&self) -> Result<usize> {
        let vectors = self.vectors.read().await;
        Ok(vectors.len())
    }

    /// Delete a memory by ID
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let key = id.as_bytes().to_vec();
        let removed = self.db.remove(key)?.is_some();

        if removed {
            let mut vectors = self.vectors.write().await;
            vectors.retain(|e| e.id != id);
            self.db.flush_async().await?;
        }

        Ok(removed)
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_vector_store_creation() {
        let dir = tempdir().unwrap();
        let store = VectorStore::new(dir.path().to_path_buf()).await;
        assert!(store.is_ok());
    }

    #[tokio::test]
    async fn test_store_and_search() {
        let dir = tempdir().unwrap();
        let store = VectorStore::new(dir.path().to_path_buf()).await.unwrap();

        // Create a simple embedding
        let embedding: Vec<f32> = (0..EMBEDDING_DIM)
            .map(|i| (i as f32) / EMBEDDING_DIM as f32)
            .collect();

        let source = MemorySource::Conversation {
            session_id: Uuid::new_v4(),
        };

        store
            .store(
                Uuid::new_v4(),
                "Test memory content",
                &source,
                &embedding,
                chrono::Utc::now(),
            )
            .await
            .unwrap();

        assert_eq!(store.count().await.unwrap(), 1);

        // Search with same embedding should return high similarity
        let results = store.search(&embedding, 5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].relevance > 0.99);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 0.001);
    }
}
