// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Brain Trait - LLM Inference Interface
//!
//! Defines the abstract interface for LLM inference backends.

use anyhow::Result;
use async_trait::async_trait;

use tokio::sync::mpsc;

// ============================================================================
// Streaming Token
// ============================================================================

/// A single token from streaming generation
#[derive(Debug, Clone)]
pub struct StreamToken {
    /// The token text
    pub text: String,
    /// Token index in sequence
    pub index: usize,
    /// Whether this is the final token
    pub is_final: bool,
}

// ============================================================================
// Grammar Specification
// ============================================================================

/// Grammar specification for constrained output generation.
/// When specified, the LLM can only generate tokens that conform to the grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrammarSpec {
    /// No grammar constraint - free-form output
    None,
    /// Constrain output to valid Rust syntax
    Rust,
    /// Constrain output to valid JSON
    Json,
    /// Constrain output to valid Markdown
    Markdown,
}

impl GrammarSpec {
    /// Get the grammar file path relative to assets directory
    pub fn grammar_path(&self) -> Option<&'static str> {
        match self {
            GrammarSpec::None => None,
            GrammarSpec::Rust => Some("grammars/rust.gbnf"),
            GrammarSpec::Json => Some("grammars/json.gbnf"),
            GrammarSpec::Markdown => Some("grammars/markdown.gbnf"),
        }
    }
}


/// The Brain trait defines the core thinking capabilities.
///
/// It abstracts the underlying inference engine, supporting:
/// - Desktop: llama-cpp-2 with ROCm/HIPBLAS
/// - Web: Candle with WebGPU
#[async_trait]
pub trait Brain: Send + Sync {
    /// Generate text from a prompt (blocking)
    async fn think(&self, prompt: &str) -> Result<String>;

    /// Generate text with grammar constraints.
    /// When grammar is not `GrammarSpec::None`, output is constrained to valid syntax.
    /// This prevents hallucinated code and ensures parseable output.
    async fn think_with_grammar(&self, prompt: &str, grammar: GrammarSpec) -> Result<String> {
        // Default implementation ignores grammar (for backends that don't support it)
        let _ = grammar;
        self.think(prompt).await
    }

    /// Generate text with streaming (tokens sent via channel)
    async fn think_stream(
        &self,
        prompt: &str,
        token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String>;

    /// Generate embeddings for a given text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Check if a model is loaded and ready
    fn is_ready(&self) -> bool;

    /// Get the name of this brain implementation
    fn name(&self) -> &'static str;

    /// Count tokens in text (for pre-flight task validation)
    /// Default implementation estimates ~4 chars per token
    fn count_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    /// Get the batch limit (max tokens per decode call)
    /// Default is 1536 (2048 - 512 reserved for system prompt)
    fn get_batch_limit(&self) -> u32 {
        1536
    }
}


// ============================================================================
// Mock Brain (for testing)
// ============================================================================

/// Mock brain for testing without actual inference
pub struct MockBrain {
    delay_ms: u64,
}

impl MockBrain {
    pub fn new() -> Self {
        Self { delay_ms: 100 }
    }

    #[allow(dead_code)]
    pub fn with_delay(delay_ms: u64) -> Self {
        Self { delay_ms }
    }
}

impl Default for MockBrain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Brain for MockBrain {
    async fn think(&self, prompt: &str) -> Result<String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        Ok(format!(
            "[MockBrain] Received prompt with {} chars. This is a simulated response.",
            prompt.len()
        ))
    }

    async fn think_stream(
        &self,
        prompt: &str,
        token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String> {
        let response = format!(
            "[MockBrain] Streaming response for {} char prompt.",
            prompt.len()
        );
        let words: Vec<&str> = response.split_whitespace().collect();

        for (i, word) in words.iter().enumerate() {
            let _ = token_tx
                .send(StreamToken {
                    text: format!("{} ", word),
                    index: i,
                    is_final: i == words.len() - 1,
                })
                .await;
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        Ok(response)
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        // Return a mock embedding (size 384 for standard small transformers)
        Ok(vec![0.0; 384])
    }


    fn is_ready(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "MockBrain"
    }
}

// ============================================================================
// Desktop Brain (llama-cpp-2)
// ============================================================================

#[cfg(feature = "desktop")]
#[path = "brain_desktop.rs"]
pub mod desktop;

#[cfg(feature = "desktop")]
pub use desktop::DesktopBrain;
