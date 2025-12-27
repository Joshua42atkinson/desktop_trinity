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
    /// Load config for Strix Halo desktop (Brain node)
    pub fn strix_halo_brain() -> Self {
        Self {
            node_type: NodeType::Brain,
            model: ModelConfig {
                model_path: PathBuf::from("/home/joshua/antigravity/models/qwen-235b-q3.gguf"),
                n_gpu_layers: 999,
                context_size: 32768,
                batch_size: 512,
                n_threads: 16,
            },
            memory: MemoryConfig {
                vector_store_path: PathBuf::from("/home/joshua/.trinity/vectors"),
                embedding_dim: 384,
                max_recall: 20,
            },
            network: NetworkConfig {
                listen_addr: "0.0.0.0:9000".to_string(),
                brain_addr: None,
                body_addr: Some("100.84.217.60:9000".to_string()),
            },
        }
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
