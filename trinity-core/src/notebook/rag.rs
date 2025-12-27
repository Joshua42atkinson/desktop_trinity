//! RAG Engine for TrinityNotebook
//!
//! Retrieval-Augmented Generation with grounded citations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::brain::orchestrator::BrainOrchestrator;
use crate::learning::{
    hash_based_embedding, MemoryFragment, SemanticEmbedder, VectorStore,
};

/// A query to the notebook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookQuery {
    /// The question to answer
    pub question: String,
    /// Maximum number of sources to retrieve
    pub max_sources: usize,
    /// Minimum relevance threshold (0.0 - 1.0)
    pub relevance_threshold: f32,
}

impl Default for NotebookQuery {
    fn default() -> Self {
        Self {
            question: String::new(),
            max_sources: 5,
            relevance_threshold: 0.3,
        }
    }
}

/// A citation linking back to source material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Memory fragment ID
    pub fragment_id: Uuid,
    /// Document or conversation source ID
    pub source_id: Uuid,
    /// Chunk index if from a document
    pub chunk_index: Option<usize>,
    /// The relevant text snippet
    pub text_snippet: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
}

/// Response from the RAG engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResponse {
    /// The generated answer (or context summary if no LLM)
    pub answer: String,
    /// Citations supporting the answer
    pub citations: Vec<Citation>,
    /// Overall confidence in the answer (0.0 - 1.0)
    pub confidence: f32,
    /// Whether an LLM was used for generation
    pub llm_generated: bool,
}

/// RAG query engine
pub struct RagEngine {
    vector_store: Arc<VectorStore>,
    /// Optional LLM orchestrator for synthesis
    orchestrator: Option<Arc<BrainOrchestrator>>,
    /// Optional semantic embedder for queries
    embedder: Option<Arc<SemanticEmbedder>>,
}

impl RagEngine {
    /// Create a new RAG engine
    pub fn new(vector_store: Arc<VectorStore>) -> Self {
        Self {
            vector_store,
            orchestrator: None,
            embedder: None,
        }
    }

    /// Add an LLM orchestrator for answer synthesis
    pub fn with_orchestrator(mut self, orchestrator: Arc<BrainOrchestrator>) -> Self {
        self.orchestrator = Some(orchestrator);
        self
    }

    /// Add a semantic embedder for query embedding
    pub fn with_embedder(mut self, embedder: Arc<SemanticEmbedder>) -> Self {
        self.embedder = Some(embedder);
        self
    }

    /// Query the knowledge base with RAG
    pub async fn query(&self, question: &str) -> Result<RagResponse> {
        let query_params = NotebookQuery {
            question: question.to_string(),
            ..Default::default()
        };

        self.query_with_params(&query_params).await
    }

    /// Query with custom parameters
    pub async fn query_with_params(&self, query: &NotebookQuery) -> Result<RagResponse> {
        // Generate query embedding
        let query_embedding = self.generate_query_embedding(&query.question)?;

        // Retrieve relevant fragments
        let fragments = self
            .vector_store
            .search(&query_embedding, query.max_sources)
            .await?;

        // Filter by relevance threshold
        let relevant_fragments: Vec<_> = fragments
            .into_iter()
            .filter(|f| f.relevance >= query.relevance_threshold)
            .collect();

        // Build citations
        let citations: Vec<Citation> = relevant_fragments
            .iter()
            .map(|f| self.fragment_to_citation(f))
            .collect();

        // Calculate overall confidence
        let confidence = if citations.is_empty() {
            0.0
        } else {
            citations.iter().map(|c| c.relevance).sum::<f32>() / citations.len() as f32
        };

        // Generate answer - use LLM if orchestrator available, else raw context
        let (answer, llm_generated) = if let Some(ref orch) = self.orchestrator {
            match self
                .generate_llm_answer(orch, &query.question, &relevant_fragments)
                .await
            {
                Ok(llm_answer) => (llm_answer, true),
                Err(e) => {
                    tracing::warn!("LLM synthesis failed, falling back to context: {}", e);
                    (
                        self.build_context_summary(&query.question, &relevant_fragments),
                        false,
                    )
                }
            }
        } else {
            (
                self.build_context_summary(&query.question, &relevant_fragments),
                false,
            )
        };

        Ok(RagResponse {
            answer,
            citations,
            confidence,
            llm_generated,
        })
    }

    /// Convert a memory fragment to a citation
    fn fragment_to_citation(&self, fragment: &MemoryFragment) -> Citation {
        let (source_id, chunk_index) = match &fragment.source {
            crate::learning::MemorySource::Document {
                doc_id,
                chunk_index,
            } => (*doc_id, Some(*chunk_index)),
            crate::learning::MemorySource::Conversation { session_id } => (*session_id, None),
            crate::learning::MemorySource::Insight { derived_from } => {
                (derived_from.first().copied().unwrap_or_default(), None)
            }
        };

        // Truncate snippet for display
        let text_snippet = if fragment.content.len() > 200 {
            format!("{}...", &fragment.content[..200])
        } else {
            fragment.content.clone()
        };

        Citation {
            fragment_id: fragment.id,
            source_id,
            chunk_index,
            text_snippet,
            relevance: fragment.relevance,
        }
    }

    /// Generate an LLM-synthesized answer from context
    async fn generate_llm_answer(
        &self,
        orchestrator: &BrainOrchestrator,
        question: &str,
        fragments: &[MemoryFragment],
    ) -> Result<String> {
        use crate::brain::Brain;

        if fragments.is_empty() {
            return Ok(format!(
                "I don't have enough information in my sources to answer: \"{}\"",
                question
            ));
        }

        // Build context block
        let mut context = String::new();
        for (i, fragment) in fragments.iter().enumerate() {
            context.push_str(&format!("[Source {}]\n{}\n\n", i + 1, fragment.content));
        }

        // Build RAG prompt
        let prompt = format!(
            "You are a helpful assistant. Answer the user's question using ONLY the provided sources. \
             If the sources don't contain enough information, say so.\n\n\
             ## Sources\n{}\n\
             ## Question\n{}\n\n\
             ## Answer",
            context, question
        );

        orchestrator.think(&prompt).await
    }

    /// Build a context summary from retrieved fragments (fallback when no LLM)
    fn build_context_summary(&self, question: &str, fragments: &[MemoryFragment]) -> String {
        if fragments.is_empty() {
            return format!(
                "I don't have enough information in my sources to answer: \"{}\"",
                question
            );
        }

        let mut summary = format!("Based on {} relevant sources:\n\n", fragments.len());

        for (i, fragment) in fragments.iter().enumerate() {
            let snippet = if fragment.content.len() > 300 {
                format!("{}...", &fragment.content[..300])
            } else {
                fragment.content.clone()
            };
            summary.push_str(&format!(
                "[{}] (relevance: {:.0}%)\n{}\n\n",
                i + 1,
                fragment.relevance * 100.0,
                snippet
            ));
        }

        summary.push_str(
            "\n---\nNote: No LLM orchestrator configured. \
             Above is the raw context from your sources.",
        );

        summary
    }

    /// Generate embedding for query text
    ///
    /// Uses semantic embedder if available, falls back to hash-based embedding.
    fn generate_query_embedding(&self, text: &str) -> Result<Vec<f32>> {
        if let Some(ref embedder) = self.embedder {
            embedder.embed(text)
        } else {
            Ok(hash_based_embedding(text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notebook_query_default() {
        let query = NotebookQuery::default();
        assert_eq!(query.max_sources, 5);
        assert_eq!(query.relevance_threshold, 0.3);
    }

    #[test]
    fn test_rag_response_serialization() {
        let response = RagResponse {
            answer: "Test answer".to_string(),
            citations: vec![],
            confidence: 0.8,
            llm_generated: false,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Test answer"));
    }
}
