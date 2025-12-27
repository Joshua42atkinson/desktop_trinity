//! Chat Engine for Trinity
//!
//! Coordinates memory retrieval, prompt construction, LLM inference, and memory storage.
//! This is the high-level interface for conversational AI.

use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use crate::brain::orchestrator::{BrainOrchestrator, OrchRequest};
use crate::learning::UnifiedMemory;

/// Configuration for the chat engine
#[derive(Debug, Clone)]
pub struct ChatConfig {
    /// System prompt to set the AI's persona
    pub system_prompt: String,
    /// Whether to use memory augmentation
    pub use_memory: bool,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            system_prompt: "You are Trinity, an advanced AI operating system.".to_string(),
            use_memory: true,
        }
    }
}

/// The main chat engine
pub struct ChatEngine {
    memory: Arc<UnifiedMemory>,
    orchestrator: Arc<BrainOrchestrator>,
    config: ChatConfig,
}

impl ChatEngine {
    /// Create a new chat engine
    pub fn new(
        memory: Arc<UnifiedMemory>,
        orchestrator: Arc<BrainOrchestrator>,
        config: ChatConfig,
    ) -> Self {
        Self {
            memory,
            orchestrator,
            config,
        }
    }

    /// Process a user message and generate a response
    pub async fn chat(&self, session_id: Uuid, message: &str) -> Result<String> {
        // 1. Ensure session is active
        if self.memory.current_session().await != Some(session_id) {
            self.memory.resume_session(session_id).await;
        }

        // 2. Build context from memory (if enabled)
        let context = if self.config.use_memory {
            self.memory.build_context(message).await?
        } else {
            String::new()
        };

        // 3. Construct prompt
        let full_prompt = self.construct_prompt(&context, message);

        // 4. Generate response via orchestrator
        // Note: Orchestrator internally routes to the appropriate model/tier
        tracing::debug!("Thinking with prompt length: {}", full_prompt.len());
        let request = OrchRequest::new(full_prompt);
        let response_obj = self.orchestrator.process(request).await?;
        let response_text = response_obj.response;

        // 5. Store conversation turn in memory
        if self.config.use_memory {
            self.memory.store_turn(message, &response_text).await?;
        }

        Ok(response_text)
    }

    /// Construct the full prompt string
    fn construct_prompt(&self, context: &str, user_message: &str) -> String {
        let mut prompt = String::new();

        // System Prompt
        prompt.push_str(&self.config.system_prompt);
        prompt.push_str("\n\n");

        // Context (if any)
        if !context.is_empty() {
            prompt.push_str(context);
            prompt.push_str("\n"); // Context already includes a separator
        }

        // User Message
        prompt.push_str("User: ");
        prompt.push_str(user_message);
        prompt.push_str("\nAssistant:");

        prompt
    }

    /// Start a new session
    pub async fn start_session(&self) -> Uuid {
        self.memory.start_session().await
    }
}
