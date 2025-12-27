//! Brain Trait - Core Thinking Interface for Trinity AI OS
//!
//! Defines the abstract interface for LLM inference backends.
//! Supports both blocking and streaming generation modes.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;
use tokio::sync::mpsc;

// ============================================================================
// Model Information
// ============================================================================

/// Information about the currently loaded model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name/identifier
    pub name: String,
    /// Path to the model file
    pub path: String,
    /// Model size in bytes
    pub size_bytes: u64,
    /// Quantization type (e.g., "Q4_K_M", "Q8_0", "F16")
    pub quantization: String,
    /// Context window size in tokens
    pub context_size: u32,
    /// Whether the model is fully loaded
    pub loaded: bool,
}

impl ModelInfo {
    /// Get model size in GB
    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

impl std::fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}, {:.1} GB, {} ctx)",
            self.name,
            self.quantization,
            self.size_gb(),
            self.context_size
        )
    }
}

// ============================================================================
// Generation Configuration
// ============================================================================

/// Configuration for text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature for sampling (0.0 = deterministic, 1.0+ = creative)
    pub temperature: f32,
    /// Top-p nucleus sampling
    pub top_p: f32,
    /// Top-k sampling (0 = disabled)
    pub top_k: u32,
    /// Repetition penalty
    pub repetition_penalty: f32,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2048,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repetition_penalty: 1.1,
            stop_sequences: vec![],
        }
    }
}

// ============================================================================
// Streaming Token
// ============================================================================

/// A single token from streaming generation
#[derive(Debug, Clone)]
pub struct StreamToken {
    /// The text content of this token
    pub text: String,
    /// Whether this is the final token
    pub is_final: bool,
    /// Token index in the generation
    pub index: usize,
}

// ============================================================================
// Brain Trait
// ============================================================================

/// The "Brain" trait defines the core thinking capabilities of the agent.
/// It abstracts the underlying inference engine (Native/ROCm vs Web/WebGPU).
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Brain: Any + Sync + Send {
    /// Generate a response to a given prompt (blocking, returns full response).
    async fn think(&self, prompt: &str) -> Result<String>;

    /// Generate with custom configuration
    async fn think_with_config(&self, prompt: &str, config: &GenerationConfig) -> Result<String> {
        // Default implementation ignores config
        let _ = config;
        self.think(prompt).await
    }

    /// Generate a streaming response (tokens sent via channel).
    /// Returns the full response after completion.
    async fn think_stream(
        &self,
        prompt: &str,
        token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String> {
        // Default implementation: just send the full response as one "token"
        let response = self.think(prompt).await?;
        let _ = token_tx
            .send(StreamToken {
                text: response.clone(),
                is_final: true,
                index: 0,
            })
            .await;
        Ok(response)
    }

    /// Load or switch the active model.
    async fn load_model(&self, model_path: &str) -> Result<()>;

    /// Get information about the currently loaded model.
    fn model_info(&self) -> Option<ModelInfo>;

    /// Check if a model is loaded and ready.
    fn is_ready(&self) -> bool {
        self.model_info().map(|m| m.loaded).unwrap_or(false)
    }

    /// Get the name of this brain implementation
    fn name(&self) -> &'static str {
        "Unknown"
    }
}

// ============================================================================
// Platform Implementations
// ============================================================================

#[cfg(feature = "desktop")]
pub mod desktop;

#[cfg(feature = "desktop")]
pub mod model_manager;

#[cfg(feature = "desktop")]
pub mod brain_resource;

#[cfg(feature = "desktop")]
pub mod tiered;

#[cfg(feature = "desktop")]
pub mod orchestrator;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web;

// ============================================================================
// Factory Functions
// ============================================================================

/// Create the platform-specific brain
pub async fn create_brain() -> Result<Arc<dyn Brain>> {
    #[cfg(feature = "desktop")]
    {
        Ok(Arc::new(desktop::DesktopBrain::new()))
    }

    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    {
        return Ok(Arc::new(web::WebBrain));
    }

    #[cfg(not(any(feature = "desktop", feature = "web")))]
    {
        anyhow::bail!("No brain feature enabled (desktop or web)")
    }
}

/// Create a brain from a specific model path
#[cfg(feature = "desktop")]
pub async fn create_brain_with_model(model_path: &str) -> Result<Arc<dyn Brain>> {
    let brain = desktop::DesktopBrain::new();
    brain.load_model(model_path).await?;
    Ok(Arc::new(brain))
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

    pub fn with_delay(delay_ms: u64) -> Self {
        Self { delay_ms }
    }
}

impl Default for MockBrain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Brain for MockBrain {
    async fn think(&self, prompt: &str) -> Result<String> {
        // Simulate thinking time
        tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
        Ok(format!(
            "Mock response to: {}...",
            &prompt.chars().take(50).collect::<String>()
        ))
    }

    async fn load_model(&self, model_path: &str) -> Result<()> {
        tracing::info!("MockBrain: Pretending to load {}", model_path);
        Ok(())
    }

    fn model_info(&self) -> Option<ModelInfo> {
        Some(ModelInfo {
            name: "MockModel".to_string(),
            path: "/mock/model.gguf".to_string(),
            size_bytes: 1024 * 1024 * 1024, // 1GB
            quantization: "Mock".to_string(),
            context_size: 4096,
            loaded: true,
        })
    }

    fn name(&self) -> &'static str {
        "MockBrain"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_brain() {
        let brain = MockBrain::with_delay(10);
        let response = brain.think("Hello").await.unwrap();
        assert!(response.contains("Mock response"));
    }

    #[test]
    fn test_model_info_display() {
        let info = ModelInfo {
            name: "Qwen-235B".to_string(),
            path: "/path/to/model.gguf".to_string(),
            size_bytes: 105 * 1024 * 1024 * 1024,
            quantization: "Q3_K_L".to_string(),
            context_size: 8192,
            loaded: true,
        };
        let display = format!("{}", info);
        assert!(display.contains("Qwen-235B"));
        assert!(display.contains("Q3_K_L"));
    }
}
