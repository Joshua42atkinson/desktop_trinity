// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Trinity Configuration
//!
//! Configuration management for Trinity Genesis.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration for Trinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrinityConfig {
    /// Node type (Brain or Body)
    pub node_type: NodeType,
    /// Model configuration
    pub model: ModelConfig,
    /// Memory configuration
    pub memory: MemoryConfig,
    /// Network configuration
    pub network: NetworkConfig,
}

/// Type of node in the Trinity network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// Desktop inference node (Brain)
    Brain,
    /// Laptop UI node (Body)
    Body,
    /// Combined node (single machine)
    Combined,
}

/// Model loading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Path to the primary model file
    pub model_path: PathBuf,
    /// Number of GPU layers to offload
    pub n_gpu_layers: u32,
    /// Context window size
    pub context_size: usize,
    /// Batch size for inference
    pub batch_size: usize,
    /// Number of threads for CPU
    pub n_threads: usize,
}

/// Memory system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Path to vector store
    pub vector_store_path: PathBuf,
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Maximum memories to return in recall
    pub max_recall: usize,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// RPC listen address
    pub listen_addr: String,
    /// Remote Brain address (for Body nodes)
    pub brain_addr: Option<String>,
    /// Remote Body address (for Brain nodes)
    pub body_addr: Option<String>,
}

impl Default for TrinityConfig {
    fn default() -> Self {
        Self {
            node_type: NodeType::Combined,
            model: ModelConfig {
                model_path: PathBuf::from("models/qwen-235b-q3.gguf"),
                n_gpu_layers: 999, // Offload all
                context_size: 32768,
                batch_size: 512,
                n_threads: 8,
            },
            memory: MemoryConfig {
                vector_store_path: PathBuf::from(".trinity/vectors"),
                embedding_dim: 384,
                max_recall: 10,
            },
            network: NetworkConfig {
                listen_addr: "0.0.0.0:9000".to_string(),
                brain_addr: Some("100.115.247.4:9000".to_string()), // Desktop via Tailscale
                body_addr: Some("100.84.217.60:9000".to_string()),  // Laptop via Tailscale
            },
        }
    }
}

impl TrinityConfig {
    /// Load configuration for a specific profile
    pub fn load_profile(profile: &str) -> Self {
        let (model_path, ctx_size) = match profile {
            "planner" => (
                "/home/joshua/antigravity/models/Llama-4-Scout-17B-16E-Instruct-Q4_K_M-00001-of-00002.gguf",
                32768
            ),
            "fast" => (
                "/home/joshua/antigravity/models/GLM-4.6V-Flash-GGUF/GLM-4.6V-Flash-Q4_K_M.gguf",
                8192
            ),
             "code_assistant" => (
                "/home/joshua/antigravity/models/Devstral-Small-2-24B-Instruct-2512-GGUF/Devstral-Small-2-24B-Instruct-2512-Q4_K_M.gguf",
                32768
            ),
            "nemotron" => (
                "/home/joshua/.lmstudio/models/lmstudio-community/NVIDIA-Nemotron-3-Nano-30B-A3B-GGUF/NVIDIA-Nemotron-3-Nano-30B-A3B-Q4_K_M.gguf",
                32768
            ),
            "rust_coder" | _ => (
                "/home/joshua/.lmstudio/models/Fortytwo-Network/Strand-Rust-Coder-14B-v1-GGUF/Fortytwo_Strand-Rust-Coder-14B-v1-Q4_K_M.gguf",
                32768
            ),
        };
        println!(
            "   DEBUG [config.rs] profile: {}, path: {}",
            profile, model_path
        );

        Self {
            node_type: NodeType::Brain,
            model: ModelConfig {
                model_path: PathBuf::from(model_path),
                n_gpu_layers: 999, // Strix Halo: Always full offload
                context_size: ctx_size,
                batch_size: 2048,
                n_threads: 16,
            },
            memory: MemoryConfig {
                vector_store_path: PathBuf::from(format!(
                    "{}/.trinity/vectors",
                    std::env::var("HOME").unwrap_or("/home/joshua".to_string())
                )),
                embedding_dim: 384,
                max_recall: 20,
            },
            network: NetworkConfig {
                listen_addr: "0.0.0.0:9000".to_string(),
                brain_addr: None,
                body_addr: None,
            },
        }
    }

    /// Load default Strix Halo configuration
    pub fn strix_halo_brain() -> Self {
        Self::load_profile("rust_coder")
    }

    /// Get the active configuration
    pub fn active() -> Self {
        let profile = std::env::var("TRINITY_PROFILE").unwrap_or_else(|_| "rust_coder".to_string());
        Self::load_profile(&profile)
    }

    /// Load config for laptop (Body node)
    pub fn laptop_body() -> Self {
        Self {
            node_type: NodeType::Body,
            model: ModelConfig {
                model_path: PathBuf::new(), // No local model
                n_gpu_layers: 0,
                context_size: 0,
                batch_size: 0,
                n_threads: 0,
            },
            memory: MemoryConfig {
                vector_store_path: PathBuf::from("/home/joshua/.trinity/vectors"),
                embedding_dim: 384,
                max_recall: 10,
            },
            network: NetworkConfig {
                listen_addr: "0.0.0.0:9001".to_string(),
                brain_addr: Some("100.115.247.4:9000".to_string()),
                body_addr: None,
            },
        }
    }
}
