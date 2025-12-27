//! TrinityNotebook - Rust NotebookLM
//!
//! A RAG-powered knowledge assistant that grounds responses in user-provided sources.
//!
//! # Features
//!
//! - Source ingestion (text, files)
//! - Document chunking with overlap
//! - Semantic search via sled-backed vector store
//! - Grounded responses with citations

mod chunker;
mod ingest;
mod rag;

pub use chunker::{Chunk, ChunkingStrategy, DocumentChunker};
pub use ingest::{ContentType, IngestResult, SourceIngester};
pub use rag::{Citation, NotebookQuery, RagEngine, RagResponse};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

use crate::learning::{SemanticEmbedder, VectorStore};

/// A source document in the notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Unique identifier
    pub id: Uuid,
    /// Human-readable name
    pub name: String,
    /// Type of content
    pub content_type: ContentType,
    /// Number of chunks created from this source
    pub chunk_count: usize,
    /// When this source was ingested
    pub ingested_at: chrono::DateTime<chrono::Utc>,
}

/// The main TrinityNotebook interface
pub struct TrinityNotebook {
    #[allow(dead_code)] // Used internally by rag_engine
    vector_store: Arc<VectorStore>,
    ingester: SourceIngester,
    rag_engine: RagEngine,
    sources: Vec<Source>,
}

impl TrinityNotebook {
    /// Create a new TrinityNotebook with its own VectorStore
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let vector_store = Arc::new(VectorStore::new(data_dir.clone()).await?);
        Self::with_vector_store(vector_store)
    }

    /// Create a TrinityNotebook with an existing VectorStore (avoids sled lock conflict)
    pub fn with_vector_store(vector_store: Arc<VectorStore>) -> Result<Self> {
        // Create shared embedder for both ingester and query
        let embedder = Arc::new(SemanticEmbedder::new()?);
        let ingester = SourceIngester::new(vector_store.clone(), embedder.clone());
        let rag_engine = RagEngine::new(vector_store.clone()).with_embedder(embedder);

        Ok(Self {
            vector_store,
            ingester,
            rag_engine,
            sources: Vec::new(),
        })
    }

    /// Add a text source to the notebook
    pub async fn add_text_source(&mut self, name: &str, content: &str) -> Result<Source> {
        let result = self.ingester.ingest_text(name, content).await?;

        let source = Source {
            id: result.source_id,
            name: name.to_string(),
            content_type: ContentType::Text,
            chunk_count: result.chunk_count,
            ingested_at: chrono::Utc::now(),
        };

        self.sources.push(source.clone());
        Ok(source)
    }

    /// Add a file source to the notebook
    pub async fn add_file_source(&mut self, path: &Path) -> Result<Source> {
        let result = self.ingester.ingest_file(path).await?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let source = Source {
            id: result.source_id,
            name,
            content_type: ContentType::File,
            chunk_count: result.chunk_count,
            ingested_at: chrono::Utc::now(),
        };

        self.sources.push(source.clone());
        Ok(source)
    }

    /// Query the notebook with RAG
    pub async fn query(&self, question: &str) -> Result<RagResponse> {
        self.rag_engine.query(question).await
    }

    /// List all sources in the notebook
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }

    /// Remove a source from the notebook
    pub fn remove_source(&mut self, source_id: Uuid) -> Option<Source> {
        if let Some(pos) = self.sources.iter().position(|s| s.id == source_id) {
            Some(self.sources.remove(pos))
        } else {
            None
        }
    }

    /// Get notebook statistics
    pub fn stats(&self) -> NotebookStats {
        NotebookStats {
            source_count: self.sources.len(),
            total_chunks: self.sources.iter().map(|s| s.chunk_count).sum(),
        }
    }
}

/// Notebook usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookStats {
    pub source_count: usize,
    pub total_chunks: usize,
}
