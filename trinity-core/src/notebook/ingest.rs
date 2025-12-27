//! Source Ingestion for TrinityNotebook
//!
//! Handles ingesting documents from various sources (text, file).
//! URL ingestion deferred to avoid dependency conflicts.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use super::chunker::DocumentChunker;
use crate::learning::{MemorySource, SemanticEmbedder, VectorStore};

/// Type of content being ingested
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    Text,
    File,
    Markdown,
}

/// Result of ingesting a source
#[derive(Debug, Clone)]
pub struct IngestResult {
    /// ID of the ingested source
    pub source_id: Uuid,
    /// Number of chunks created
    pub chunk_count: usize,
    /// Total characters processed
    pub total_chars: usize,
}

/// Source ingestion engine
pub struct SourceIngester {
    vector_store: Arc<VectorStore>,
    chunker: DocumentChunker,
    embedder: Arc<SemanticEmbedder>,
}

impl SourceIngester {
    /// Create a new source ingester with shared embedder
    pub fn new(vector_store: Arc<VectorStore>, embedder: Arc<SemanticEmbedder>) -> Self {
        Self {
            vector_store,
            chunker: DocumentChunker::new(),
            embedder,
        }
    }

    /// Create with default embedder (for backwards compatibility)
    pub fn with_default_embedder(vector_store: Arc<VectorStore>) -> Result<Self> {
        let embedder = Arc::new(SemanticEmbedder::new()?);
        Ok(Self::new(vector_store, embedder))
    }

    /// Ingest plain text content
    pub async fn ingest_text(&self, name: &str, content: &str) -> Result<IngestResult> {
        let source_id = Uuid::new_v4();
        let chunks = self.chunker.chunk(source_id, content);
        let total_chars = content.len();

        for chunk in &chunks {
            // Generate embedding using shared embedder
            let embedding = self.embedder.embed(&chunk.content)?;

            let source = MemorySource::Document {
                doc_id: source_id,
                chunk_index: chunk.index,
            };

            self.vector_store
                .store(
                    Uuid::new_v4(),
                    &chunk.content,
                    &source,
                    &embedding,
                    chrono::Utc::now(),
                )
                .await
                .context("Failed to store chunk in vector store")?;
        }

        tracing::info!(
            "Ingested text source '{}' ({} chunks, {} chars, semantic={})",
            name,
            chunks.len(),
            total_chars,
            self.embedder.is_semantic()
        );

        Ok(IngestResult {
            source_id,
            chunk_count: chunks.len(),
            total_chars,
        })
    }

    /// Ingest content from a file
    pub async fn ingest_file(&self, path: &Path) -> Result<IngestResult> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        self.ingest_text(name, &content).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_serialization() {
        let ct = ContentType::Text;
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"Text\"");
    }
}
