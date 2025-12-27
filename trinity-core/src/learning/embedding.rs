//! Embedding Model for Semantic Search
//!
//! Provides real semantic embeddings via fastembed (all-MiniLM-L6-v2) when
//! the `semantic` feature is enabled. Falls back to hash-based embeddings otherwise.

use anyhow::Result;
use std::sync::Arc;

use super::EMBEDDING_DIM;

// ============================================================================
// Semantic Embedder (feature-gated)
// ============================================================================

#[cfg(feature = "semantic")]
use fastembed::{EmbeddingModel as FastEmbedModel, InitOptions, TextEmbedding};

/// Semantic embedding model
///
/// When `semantic` feature is enabled, uses all-MiniLM-L6-v2 via fastembed.
/// Otherwise falls back to hash-based pseudo-embeddings.
pub struct SemanticEmbedder {
    #[cfg(feature = "semantic")]
    model: TextEmbedding,
    #[cfg(not(feature = "semantic"))]
    _phantom: std::marker::PhantomData<()>,
}

impl SemanticEmbedder {
    /// Create a new semantic embedder
    #[cfg(feature = "semantic")]
    pub fn new() -> Result<Self> {
        tracing::info!("Initializing semantic embeddings (all-MiniLM-L6-v2)");
        let mut options = InitOptions::default();
        options.model_name = FastEmbedModel::AllMiniLML6V2;
        options.show_download_progress = true;
        let model = TextEmbedding::try_new(options)?;
        Ok(Self { model })
    }

    #[cfg(not(feature = "semantic"))]
    pub fn new() -> Result<Self> {
        tracing::info!("Using hash-based embedding fallback (semantic feature disabled)");
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    /// Generate embedding for a single text
    #[cfg(feature = "semantic")]
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings
            .into_iter()
            .next()
            .unwrap_or_else(|| hash_based_embedding(text)))
    }

    #[cfg(not(feature = "semantic"))]
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        Ok(hash_based_embedding(text))
    }

    /// Generate embeddings for multiple texts (batch)
    #[cfg(feature = "semantic")]
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let text_vec: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        Ok(self.model.embed(text_vec, None)?)
    }

    #[cfg(not(feature = "semantic"))]
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    /// Check if using real semantic embeddings
    pub fn is_semantic(&self) -> bool {
        cfg!(feature = "semantic")
    }
}

impl Default for SemanticEmbedder {
    fn default() -> Self {
        Self::new().expect("Failed to initialize embedder")
    }
}

// ============================================================================
// Legacy API (backwards compatibility)
// ============================================================================

/// Legacy embedding model (alias for SemanticEmbedder)
pub type EmbeddingModel = SemanticEmbedder;

impl EmbeddingModel {
    /// Load an embedding model (legacy API)
    pub fn load<P: AsRef<std::path::Path>>(_model_dir: P) -> Result<Self> {
        Self::new()
    }
}

/// Shared embedding model instance
pub type SharedEmbeddingModel = Arc<SemanticEmbedder>;

// ============================================================================
// Hash-based Fallback
// ============================================================================

/// Hash-based pseudo-embedding for fallback or testing
///
/// Note: This produces deterministic embeddings based on text hash,
/// but does NOT provide true semantic similarity. "programming" won't
/// match "code" unless they share text similarity.
pub fn hash_based_embedding(text: &str) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    let mut embedding = Vec::with_capacity(EMBEDDING_DIM);
    let mut seed = hash;

    for _ in 0..EMBEDDING_DIM {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let value = ((seed >> 33) as f32) / (u32::MAX as f32) - 0.5;
        embedding.push(value);
    }

    // Normalize
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    for x in &mut embedding {
        *x /= magnitude;
    }

    embedding
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_based_embedding() {
        let emb1 = hash_based_embedding("hello world");
        let emb2 = hash_based_embedding("hello world");
        let emb3 = hash_based_embedding("goodbye world");

        assert_eq!(emb1.len(), EMBEDDING_DIM);
        assert_eq!(emb1, emb2); // Same text = same embedding
        assert_ne!(emb1, emb3); // Different text = different embedding

        // Check normalized
        let magnitude: f32 = emb1.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_semantic_embedder_creation() {
        let embedder = SemanticEmbedder::new().unwrap();
        let result = embedder.embed("test text").unwrap();
        assert_eq!(result.len(), EMBEDDING_DIM);
    }
}
