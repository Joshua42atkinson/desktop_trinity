//! Unified Memory System for Trinity
//!
//! Combines VectorStore (semantic search) and RelationalStore (structured data)
//! into a single, easy-to-use memory interface for conversations and learning.

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    hash_based_embedding, MemoryFragment, MemorySource, MemoryStats, RelationalStore,
    SemanticEmbedder, VectorStore,
};

/// Configuration for the unified memory system
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Path to the data directory (default: ~/.trinity)
    pub data_dir: PathBuf,
    /// Number of memories to retrieve for context
    pub context_limit: usize,
    /// Minimum relevance score for memory retrieval (0.0-1.0)
    pub relevance_threshold: f32,
    /// Whether to use semantic embeddings (requires `semantic` feature)
    pub use_semantic: bool,
    /// URL for remote memory service (if distributed)
    pub remote_url: Option<String>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Self {
            data_dir: PathBuf::from(home).join(".trinity"),
            context_limit: 5,
            relevance_threshold: 0.5,
            use_semantic: cfg!(feature = "semantic"),
            remote_url: std::env::var("TRINITY_MEMORY_URL").ok(),
        }
    }
}

/// Unified memory system for Trinity
///
/// Provides a simple interface for:
/// - Storing conversation turns
/// - Semantic recall of relevant memories
/// - Building context windows for LLM prompts
pub struct UnifiedMemory {
    vector_store: Arc<VectorStore>,
    relational_store: Arc<RelationalStore>,
    embedder: Arc<RwLock<Option<SemanticEmbedder>>>,
    config: MemoryConfig,
    current_session: RwLock<Option<Uuid>>,
    turn_count: RwLock<i32>,
    http_client: reqwest::Client,
}

impl UnifiedMemory {
    /// Create a new unified memory system
    pub async fn new(config: MemoryConfig) -> Result<Self> {
        // Ensure data directory exists
        std::fs::create_dir_all(&config.data_dir)?;

        // Initialize stores
        let vector_store = VectorStore::new(config.data_dir.clone()).await?;
        let relational_store = RelationalStore::new(config.data_dir.join("memory.db"))?;

        // Initialize embedder (lazy, may be slow on first use)
        let embedder = if config.use_semantic {
            match SemanticEmbedder::new() {
                Ok(e) => Some(e),
                Err(err) => {
                    tracing::warn!("Failed to initialize semantic embedder: {}", err);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            vector_store: Arc::new(vector_store),
            relational_store: Arc::new(relational_store),
            embedder: Arc::new(RwLock::new(embedder)),
            config,
            current_session: RwLock::new(None),
            turn_count: RwLock::new(0),
            http_client: reqwest::Client::new(),
        })
    }

    /// Create with default configuration
    pub async fn default_config() -> Result<Self> {
        Self::new(MemoryConfig::default()).await
    }

    /// Start a new conversation session
    pub async fn start_session(&self) -> Uuid {
        let session_id = Uuid::new_v4();
        *self.current_session.write().await = Some(session_id);
        *self.turn_count.write().await = 0;
        tracing::info!("Started memory session: {}", session_id);
        session_id
    }

    /// Resume an existing session
    pub async fn resume_session(&self, session_id: Uuid) {
        *self.current_session.write().await = Some(session_id);
        tracing::info!("Resumed memory session: {}", session_id);
    }

    /// Get the current session ID
    pub async fn current_session(&self) -> Option<Uuid> {
        *self.current_session.read().await
    }

    /// Store a conversation turn (user message + assistant response)
    pub async fn store_turn(&self, user_message: &str, assistant_response: &str) -> Result<()> {
        let session_id = self
            .current_session
            .read()
            .await
            .ok_or_else(|| anyhow::anyhow!("No active session - call start_session() first"))?;

        // Hand off to remote service if configured
        if let Some(ref url) = self.config.remote_url {
            let endpoint = format!("{}/store", url);
            let payload = serde_json::json!({
                "content": format!("User: {}\nAssistant: {}", user_message, assistant_response),
                "session_id": session_id,
                "source": "desktop_turn",
                "metadata": {
                    "user_msg_len": user_message.len(),
                    "response_len": assistant_response.len()
                }
            });

            match self.http_client.post(&endpoint).json(&payload).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        tracing::debug!("Stored turn remotely via {}", url);
                        // Still update local turn count for session tracking
                        let mut count = self.turn_count.write().await;
                        *count += 1;
                        return Ok(());
                    } else {
                        tracing::warn!("Remote memory store failed: {}", res.status());
                        // Fallback to local
                    }
                }
                Err(e) => {
                    tracing::warn!("Remote memory connection failed: {}", e);
                    // Fallback to local
                }
            }
        }

        // Increment turn count
        let turn_num = {
            let mut count = self.turn_count.write().await;
            *count += 1;
            *count
        };

        // Update session in relational store
        self.relational_store
            .update_conversation(session_id, turn_num)?;

        // Store user message
        let user_id = Uuid::new_v4();
        let user_content = format!("User: {}", user_message);
        let user_embedding = self.embed(&user_content).await?;
        let user_source = MemorySource::Conversation { session_id };

        self.vector_store
            .store(
                user_id,
                &user_content,
                &user_source,
                &user_embedding,
                chrono::Utc::now(),
            )
            .await?;
        self.relational_store
            .store_fragment(user_id, &user_content, &user_source, None)?;

        // Store assistant response
        let assistant_id = Uuid::new_v4();
        let assistant_content = format!("Assistant: {}", assistant_response);
        let assistant_embedding = self.embed(&assistant_content).await?;

        self.vector_store
            .store(
                assistant_id,
                &assistant_content,
                &user_source,
                &assistant_embedding,
                chrono::Utc::now(),
            )
            .await?;
        self.relational_store.store_fragment(
            assistant_id,
            &assistant_content,
            &user_source,
            None,
        )?;

        tracing::debug!("Stored turn {} in session {}", turn_num, session_id);
        Ok(())
    }

    /// Store a generic memory fragment
    pub async fn store_fragment(&self, content: &str, source: MemorySource) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let embedding = self.embed(content).await?;

        self.vector_store
            .store(id, content, &source, &embedding, chrono::Utc::now())
            .await?;

        self.relational_store
            .store_fragment(id, content, &source, None)?;

        tracing::debug!("Stored fragment {} from {:?}", id, source);
        Ok(id)
    }

    /// Recall relevant memories for a given query
    pub async fn recall(&self, query: &str, limit: Option<usize>) -> Result<Vec<MemoryFragment>> {
        let limit = limit.unwrap_or(self.config.context_limit);

        // Try remote recall first
        if let Some(ref url) = self.config.remote_url {
            let endpoint = format!("{}/recall", url);
            let params = [("query", query), ("limit", &limit.to_string())];

            match self.http_client.get(&endpoint).query(&params).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        #[derive(serde::Deserialize)]
                        struct RemoteRecall {
                            memories: Vec<MemoryFragment>,
                        }
                        if let Ok(wrapper) = res.json::<RemoteRecall>().await {
                            tracing::debug!(
                                "Recalled {} memories from remote",
                                wrapper.memories.len()
                            );
                            // TODO: Merge with local? For now, return remote if successful
                            return Ok(wrapper.memories);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Remote memory recall failed: {}", e);
                }
            }
        }

        let query_embedding = self.embed(query).await?;
        let memories = self.vector_store.search(&query_embedding, limit).await?;

        // Filter by relevance threshold
        let filtered: Vec<_> = memories
            .into_iter()
            .filter(|m| m.relevance >= self.config.relevance_threshold)
            .collect();

        tracing::debug!("Recalled {} memories for query", filtered.len());
        Ok(filtered)
    }

    /// Build a context string from recalled memories
    pub async fn build_context(&self, query: &str) -> Result<String> {
        let memories = self.recall(query, None).await?;

        if memories.is_empty() {
            return Ok(String::new());
        }

        let mut context = String::from("Relevant context from previous conversations:\n");
        for (i, memory) in memories.iter().enumerate() {
            context.push_str(&format!("\n{}. {}", i + 1, memory.content));
        }
        context.push_str("\n\n---\n");

        Ok(context)
    }

    /// Get memory statistics
    pub async fn stats(&self) -> Result<MemoryStats> {
        self.relational_store.stats()
    }

    /// Generate embedding for text
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embedder = self.embedder.read().await;

        if let Some(ref e) = *embedder {
            e.embed(text)
        } else {
            // Fallback to hash-based embedding
            Ok(hash_based_embedding(text))
        }
    }

    /// Check if using semantic embeddings
    pub async fn is_semantic(&self) -> bool {
        self.embedder.read().await.is_some()
    }

    /// Get the vector store (for advanced usage)
    pub fn vector_store(&self) -> Arc<VectorStore> {
        self.vector_store.clone()
    }

    /// Get the relational store (for advanced usage)
    pub fn relational_store(&self) -> Arc<RelationalStore> {
        self.relational_store.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_unified_memory_creation() {
        let dir = tempdir().unwrap();
        let config = MemoryConfig {
            data_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let memory = UnifiedMemory::new(config).await;
        assert!(memory.is_ok());
    }

    #[tokio::test]
    async fn test_session_management() {
        let dir = tempdir().unwrap();
        let config = MemoryConfig {
            data_dir: dir.path().to_path_buf(),
            ..Default::default()
        };
        let memory = UnifiedMemory::new(config).await.unwrap();

        // No session initially
        assert!(memory.current_session().await.is_none());

        // Start a session
        let session_id = memory.start_session().await;
        assert!(memory.current_session().await.is_some());
        assert_eq!(memory.current_session().await.unwrap(), session_id);
    }

    #[tokio::test]
    async fn test_store_and_recall() {
        let dir = tempdir().unwrap();
        let config = MemoryConfig {
            data_dir: dir.path().to_path_buf(),
            relevance_threshold: 0.0, // Accept all for testing
            ..Default::default()
        };
        let memory = UnifiedMemory::new(config).await.unwrap();

        // Start session and store a turn
        memory.start_session().await;
        memory
            .store_turn("Hello, how are you?", "I'm doing well, thank you!")
            .await
            .unwrap();

        // Check stats
        let stats = memory.stats().await.unwrap();
        assert_eq!(stats.total_fragments, 2); // User + Assistant
        assert_eq!(stats.conversation_count, 2);

        // Recall memories
        let memories = memory.recall("hello", Some(5)).await.unwrap();
        assert!(!memories.is_empty());
    }
}
