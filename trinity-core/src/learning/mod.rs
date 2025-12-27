//! Trinity Learning Memory Layer
//!
//! This module provides the persistent memory layer for Trinity's learning capabilities.
//! Unlike the VRAM `UnifiedMemoryManager`, this handles semantic/episodic memory.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Trinity Learning Layer                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │   WorkingMemory (Short-term)                                │
//! │        │                                                     │
//! │        ▼  consolidation                                      │
//! │   ┌─────────────────────────────────────────┐              │
//! │   │           TrinityMemory Trait           │              │
//! │   └───────────────┬─────────────────────────┘              │
//! │                   │                                          │
//! │     ┌─────────────┴─────────────┐                          │
//! │     ▼                           ▼                           │
//! │ ┌─────────────┐          ┌─────────────┐                   │
//! │ │ PostgreSQL  │          │   LanceDB   │                   │
//! │ │ (Relational)│          │  (Vector)   │                   │
//! │ └─────────────┘          └─────────────┘                   │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod consolidation;
mod embedding;
mod memory_system;
mod relational;

mod vector;

pub use consolidation::{ConsolidationReport, MemoryConsolidator};
pub use embedding::{hash_based_embedding, EmbeddingModel, SemanticEmbedder, SharedEmbeddingModel};
pub use memory_system::{MemoryConfig, UnifiedMemory};
pub mod scanner;
pub use relational::RelationalStore;
pub use vector::{EmbeddingVector, VectorStore};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Dimension of embedding vectors (all-MiniLM-L6-v2)
pub const EMBEDDING_DIM: usize = 384;

/// A fragment of memory retrieved from storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFragment {
    /// Unique identifier
    pub id: Uuid,
    /// The text content of this memory
    pub content: String,
    /// Source of this memory (conversation, document, etc.)
    pub source: MemorySource,
    /// Relevance score from vector search (0.0 - 1.0)
    pub relevance: f32,
    /// Timestamp when this memory was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Source of a memory fragment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemorySource {
    /// From a conversation turn
    Conversation { session_id: Uuid },
    /// From an ingested document
    Document { doc_id: Uuid, chunk_index: usize },
    /// Synthesized insight from consolidation
    Insight { derived_from: Vec<Uuid> },
}

/// Unified memory interface for Trinity
///
/// This trait abstracts over the dual-database architecture,
/// providing a simple interface for storing and retrieving memories.
#[allow(async_fn_in_trait)]
pub trait TrinityMemory: Send + Sync {
    /// Store a text fragment with its embedding
    async fn store(&self, content: &str, source: MemorySource, embedding: &[f32]) -> Result<Uuid>;

    /// Recall memories similar to the query embedding
    async fn recall(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<MemoryFragment>>;

    /// Get memory statistics
    async fn stats(&self) -> Result<MemoryStats>;
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    /// Total number of memory fragments
    pub total_fragments: usize,
    /// Number of conversation memories
    pub conversation_count: usize,
    /// Number of document memories
    pub document_count: usize,
    /// Number of synthesized insights
    pub insight_count: usize,
    /// Total size in bytes (approximate)
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_source_serialization() {
        let source = MemorySource::Conversation {
            session_id: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&source).unwrap();
        let deserialized: MemorySource = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, MemorySource::Conversation { .. }));
    }
}
