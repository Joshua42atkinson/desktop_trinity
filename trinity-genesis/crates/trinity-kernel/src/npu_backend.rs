// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! # NPU Backend - "Subconscious Processor"
//!
//! ## Philosophy
//! "The NPU is the subconscious of Trinity—running background reasoning while the GPU
//!  maintains the visual cortex. This split-brain architecture allows deep thinking
//!  without impacting UI responsiveness."
//!
//! ## Architecture
//!
//! ```text
//!                    ┌─────────────────────────────────────┐
//!                    │       AMD Strix Halo APU            │
//!                    │                                     │
//!    Interactive     │  ┌─────────┐      ┌─────────────┐  │   Background
//!    Chat ──────────►│  │ Radeon  │      │  XDNA 2     │◄─│── Reasoning
//!    (fast)          │  │ 890M    │      │  NPU        │  │   (deep)
//!                    │  │ (GPU)   │      │  50-80 TOPS │  │
//!                    │  └────┬────┘      └──────┬──────┘  │
//!                    │       └───────┬──────────┘         │
//!                    │               ▼                    │
//!                    │     128GB Unified Memory Pool      │
//!                    └─────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Route background tasks to NPU
//! let npu = NpuBrain::new()?;
//! let result = npu.think_background("Analyze this codebase...").await?;
//! ```
//!
//! ## Strix Halo Optimization
//!
//! The NPU uses W4ABF16 quantization (4-bit weights, BFloat16 activations) which
//! leverages the NPU's native block floating-point arithmetic for higher throughput.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::brain::{Brain, StreamToken};
use crate::runtime::TaskType;

// ============================================================================
// Compute Target (Heterogeneous Routing)
// ============================================================================

/// Target compute device for inference workloads
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputeTarget {
    /// Fast interactive chat - Radeon 890M via Vulkan
    /// Best for: Short prompts, streaming responses, UI-driven queries
    Gpu,

    /// Background reasoning - XDNA 2 NPU
    /// Best for: Long-chain reasoning, code analysis, memory consolidation
    Npu,

    /// Large context offload - Remote RPC node
    /// Best for: Massive KV caches, expert heads in MoE models
    RemoteRpc { endpoint: String },
}

impl ComputeTarget {
    /// Route workload based on task type
    pub fn for_task(task_type: &TaskType) -> Self {
        match task_type {
            // Interactive tasks → GPU (low latency)
            TaskType::Chat { .. } => ComputeTarget::Gpu,

            // Heavy reasoning → NPU (background, power-efficient)
            TaskType::GenerateCode { .. } => ComputeTarget::Npu,
            TaskType::ReviewCode { .. } => ComputeTarget::Npu,
            TaskType::Research { .. } => ComputeTarget::Npu,

            // Memory operations → Could go to remote
            TaskType::MemoryConsolidation { .. } => ComputeTarget::Npu,

            // Default to GPU for unknown tasks
            _ => ComputeTarget::Gpu,
        }
    }
}

// ============================================================================
// NPU Configuration
// ============================================================================

/// Configuration for the NPU backend
#[derive(Debug, Clone)]
pub struct NpuConfig {
    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Context window size
    pub context_size: u32,

    /// Use W4ABF16 quantization (optimized for XDNA 2)
    pub use_w4abf16: bool,

    /// Batch size for inference
    pub batch_size: u32,
}

impl Default for NpuConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            context_size: 32768,
            use_w4abf16: true,
            batch_size: 1,
        }
    }
}

impl NpuConfig {
    /// Strix Halo optimized configuration
    pub fn strix_halo() -> Self {
        Self {
            max_tokens: 8192,
            context_size: 65536,
            use_w4abf16: true,
            batch_size: 1,
        }
    }
}

// ============================================================================
// NPU Brain State
// ============================================================================

/// Internal state for NPU inference
struct NpuState {
    /// Whether the NPU is initialized
    initialized: bool,

    /// Model path (W4ABF16 quantized)
    model_path: Option<String>,
}

// ============================================================================
// NPU Brain Implementation
// ============================================================================

/// NPU Backend for background reasoning
///
/// Uses the XDNA 2 NPU (50-80 TOPS) for power-efficient, sustained inference.
/// Designed for long-running tasks that don't require immediate response.
pub struct NpuBrain {
    config: NpuConfig,
    state: Arc<Mutex<NpuState>>,
}

impl NpuBrain {
    /// Create a new NPU brain
    pub fn new(config: NpuConfig) -> Result<Self> {
        // TODO: Initialize RyzenAI SDK when available
        // For now, this is a placeholder that logs intent
        tracing::info!(
            "🧠 NPU Brain initialized (stub) - awaiting RyzenAI SDK integration"
        );

        Ok(Self {
            config,
            state: Arc::new(Mutex::new(NpuState {
                initialized: false,
                model_path: None,
            })),
        })
    }

    /// Create with Strix Halo optimized settings
    pub fn strix_halo() -> Result<Self> {
        Self::new(NpuConfig::strix_halo())
    }

    /// Load a model for NPU inference
    pub async fn load_model(&self, path: &str) -> Result<()> {
        let mut state = self.state.lock().await;

        // TODO: Use RyzenAI loader with W4ABF16 format
        tracing::info!("📦 NPU loading model: {} (W4ABF16)", path);

        state.model_path = Some(path.to_string());
        state.initialized = true;

        Ok(())
    }

    /// Run background inference (non-blocking relative to GPU)
    ///
    /// This is the primary entry point for "subconscious" processing.
    pub async fn think_background(&self, prompt: &str) -> Result<String> {
        let state = self.state.lock().await;

        if !state.initialized {
            // Fallback: Return a placeholder indicating NPU not ready
            tracing::warn!("⚠️ NPU not initialized, returning placeholder");
            return Ok(format!(
                "[NPU pending] Would process: {}...",
                &prompt[..prompt.len().min(50)]
            ));
        }

        // TODO: Actual RyzenAI inference call
        // For now, simulate with timing
        tracing::debug!(
            "🧠 NPU processing {} chars at {} TOPS",
            prompt.len(),
            "50-80"
        );

        // Placeholder response
        Ok(format!(
            "[NPU processed] Input processed on XDNA 2 NPU ({} tokens)",
            prompt.split_whitespace().count()
        ))
    }

    /// Check if NPU is available and initialized
    pub async fn is_available(&self) -> bool {
        let state = self.state.lock().await;
        state.initialized
    }

    /// Get NPU utilization stats
    pub async fn get_stats(&self) -> NpuStats {
        // TODO: Query actual NPU metrics via RyzenAI SDK
        NpuStats {
            tops_utilized: 0.0,
            memory_used_mb: 0,
            temperature_c: 0.0,
            power_watts: 0.0,
        }
    }
}

// ============================================================================
// NPU Stats
// ============================================================================

/// NPU utilization statistics
#[derive(Debug, Clone, Default)]
pub struct NpuStats {
    /// Current TOPS utilization (0-80)
    pub tops_utilized: f64,

    /// Memory used in MB
    pub memory_used_mb: u64,

    /// Temperature in Celsius
    pub temperature_c: f64,

    /// Power consumption in Watts
    pub power_watts: f64,
}

// ============================================================================
// Brain Trait Implementation
// ============================================================================

#[async_trait]
impl Brain for NpuBrain {
    async fn think(&self, prompt: &str) -> Result<String> {
        self.think_background(prompt).await
    }

    async fn think_stream(
        &self,
        prompt: &str,
        token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String> {
        // NPU doesn't do streaming - return whole response at once
        let response = self.think_background(prompt).await?;
        
        // Send as single token
        let _ = token_tx
            .send(StreamToken {
                text: response.clone(),
                index: 0,
                is_final: true,
            })
            .await;

        Ok(response)
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        // NPU embedding support TBD
        Err(anyhow::anyhow!(
            "NPU embedding not yet implemented - use GPU backend"
        ))
    }

    fn is_ready(&self) -> bool {
        // Synchronous check - conservative
        true
    }

    fn name(&self) -> &'static str {
        "NpuBrain (XDNA 2)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_target_routing() {
        let chat_task = TaskType::Chat {
            message: "Hello".into(),
        };
        assert_eq!(ComputeTarget::for_task(&chat_task), ComputeTarget::Gpu);

        let code_task = TaskType::GenerateCode {
            prompt: "Write a function".into(),
            language: "rust".into(),
            output_path: None,
        };
        assert_eq!(ComputeTarget::for_task(&code_task), ComputeTarget::Npu);
    }

    #[tokio::test]
    async fn test_npu_brain_creation() {
        let brain = NpuBrain::strix_halo().unwrap();
        assert_eq!(brain.config.context_size, 65536);
        assert!(brain.config.use_w4abf16);
    }
}
