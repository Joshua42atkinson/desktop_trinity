//! # RPC Memory Pool - "Context Keeper"
//!
//! ## Philosophy
//! "The satellite node serves as extended memory—treating the Dell laptop not as a
//!  parallel processor, but as intelligent, active swap memory. KV caches flow to
//!  the keeper, freeing local VRAM for active model weights and graphics."
//!
//! ## Architecture
//!
//! ```text
//!    ┌─────────────────────────────────────┐
//!    │       GMKtek Evo2 (Primary)         │
//!    │  128GB Unified Memory               │
//!    │  ├── Model Weights (active)         │
//!    │  ├── Graphics Frame Buffers         │
//!    │  └── Hot KV Cache                   │
//!    └──────────────┬──────────────────────┘
//!                   │ Tailscale (100.x.y.z)
//!                   │ TCP_NODELAY enabled
//!                   ▼
//!    ┌─────────────────────────────────────┐
//!    │       Dell Laptop (Satellite)       │
//!    │  32GB RAM                           │
//!    │  ├── Cold KV Cache                  │
//!    │  ├── Expert Heads (MoE offload)     │
//!    │  └── llama-server --rpc-server-host │
//!    └─────────────────────────────────────┘
//! ```
//!
//! ## Tailscale Optimization
//!
//! - Bind RPC server to Tailscale IP (100.x.y.z) to avoid relay routing
//! - Enable TCP_NODELAY to disable Nagle's algorithm
//! - Use Pipeline Parallelism, not Tensor Parallelism (latency-tolerant)

use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// RPC Node Configuration
// ============================================================================

/// Configuration for a remote RPC node
#[derive(Debug, Clone)]
pub struct RpcNodeConfig {
    /// Tailscale IP address (preferably 100.x.y.z)
    pub address: String,

    /// RPC server port (default: 9001)
    pub port: u16,

    /// Node name for logging
    pub name: String,

    /// Available RAM in MB
    pub available_ram_mb: u64,

    /// Enable TCP_NODELAY
    pub tcp_nodelay: bool,

    /// Connection timeout in ms
    pub connect_timeout_ms: u64,
}

impl Default for RpcNodeConfig {
    fn default() -> Self {
        Self {
            address: "100.84.217.60".into(), // quadratical (Dell laptop)
            port: 9001,
            name: "quadratical".into(),
            available_ram_mb: 32 * 1024, // 32GB
            tcp_nodelay: true,
            connect_timeout_ms: 5000,
        }
    }
}

impl RpcNodeConfig {
    /// Create config for the Dell laptop via Tailscale
    pub fn dell_laptop() -> Self {
        Self::default()
    }

    /// Get socket address
    pub fn socket_addr(&self) -> Result<SocketAddr> {
        let addr = format!("{}:{}", self.address, self.port);
        addr.parse().map_err(Into::into)
    }
}

// ============================================================================
// Remote Handle (KV Cache Reference)
// ============================================================================

/// Handle to data stored on a remote node
#[derive(Debug, Clone)]
pub struct RemoteHandle {
    /// Unique identifier for this allocation
    pub id: Uuid,

    /// Node hosting this data
    pub node_name: String,

    /// Size in bytes
    pub size_bytes: u64,

    /// Content type
    pub content_type: RemoteContentType,
}

/// Types of content that can be offloaded
#[derive(Debug, Clone)]
pub enum RemoteContentType {
    /// Key-Value cache from attention layers
    KvCache { layer_start: u32, layer_end: u32 },

    /// Expert weights from MoE models
    ExpertWeights { expert_ids: Vec<u32> },

    /// Embeddings cache
    Embeddings { token_count: u64 },
}

// ============================================================================
// RPC Node (Connection State)
// ============================================================================

/// Active connection to a remote RPC node
#[derive(Debug)]
struct RpcNode {
    config: RpcNodeConfig,
    connected: bool,
    allocations: HashMap<Uuid, u64>, // id → size
    used_bytes: u64,
}

impl RpcNode {
    fn new(config: RpcNodeConfig) -> Self {
        Self {
            config,
            connected: false,
            allocations: HashMap::new(),
            used_bytes: 0,
        }
    }

    fn available_bytes(&self) -> u64 {
        let total = self.config.available_ram_mb * 1024 * 1024;
        total.saturating_sub(self.used_bytes)
    }
}

// ============================================================================
// RPC Memory Pool
// ============================================================================

/// Distributed memory pool using remote RPC nodes
///
/// Manages KV cache and expert weight offloading over Tailscale.
/// The Dell laptop acts as "intelligent swap memory" for the primary node.
pub struct RpcMemoryPool {
    nodes: Arc<RwLock<Vec<RpcNode>>>,
}

impl RpcMemoryPool {
    /// Create a new RPC memory pool
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a remote node to the pool
    pub async fn add_node(&self, config: RpcNodeConfig) -> Result<()> {
        let mut nodes = self.nodes.write().await;

        tracing::info!(
            "🌐 Adding RPC node: {} at {}:{}",
            config.name,
            config.address,
            config.port
        );

        nodes.push(RpcNode::new(config));
        Ok(())
    }

    /// Connect to all configured nodes
    pub async fn connect_all(&self) -> Result<()> {
        let mut nodes = self.nodes.write().await;

        for node in nodes.iter_mut() {
            // TODO: Actual TCP connection with TCP_NODELAY
            tracing::info!(
                "🔌 Connecting to RPC node: {} (TCP_NODELAY={})",
                node.config.name,
                node.config.tcp_nodelay
            );

            // Simulate connection
            node.connected = true;
        }

        Ok(())
    }

    /// Offload KV cache to a remote node
    ///
    /// Returns a handle that can be used to retrieve the cache later.
    pub async fn offload_kv_cache(
        &self,
        cache_bytes: Vec<u8>,
        layer_start: u32,
        layer_end: u32,
    ) -> Result<RemoteHandle> {
        let mut nodes = self.nodes.write().await;
        let size = cache_bytes.len() as u64;

        // Find a node with enough space
        let node = nodes
            .iter_mut()
            .find(|n| n.connected && n.available_bytes() >= size)
            .ok_or_else(|| anyhow::anyhow!("No remote node with {} bytes available", size))?;

        let handle = RemoteHandle {
            id: Uuid::new_v4(),
            node_name: node.config.name.clone(),
            size_bytes: size,
            content_type: RemoteContentType::KvCache {
                layer_start,
                layer_end,
            },
        };

        // TODO: Actually send data over RPC
        tracing::debug!(
            "📤 Offloading {} bytes of KV cache (layers {}-{}) to {}",
            size,
            layer_start,
            layer_end,
            node.config.name
        );

        node.allocations.insert(handle.id, size);
        node.used_bytes += size;

        Ok(handle)
    }

    /// Retrieve KV cache from a remote node
    pub async fn retrieve_kv_cache(&self, handle: &RemoteHandle) -> Result<Vec<u8>> {
        let nodes = self.nodes.read().await;

        let node = nodes
            .iter()
            .find(|n| n.config.name == handle.node_name)
            .ok_or_else(|| anyhow::anyhow!("Node {} not found", handle.node_name))?;

        if !node.allocations.contains_key(&handle.id) {
            return Err(anyhow::anyhow!("Handle {} not found on node", handle.id));
        }

        // TODO: Actually retrieve data over RPC
        tracing::debug!(
            "📥 Retrieving {} bytes from {}",
            handle.size_bytes,
            handle.node_name
        );

        // Placeholder: return empty buffer
        Ok(vec![0u8; handle.size_bytes as usize])
    }

    /// Release a remote allocation
    pub async fn release(&self, handle: &RemoteHandle) -> Result<()> {
        let mut nodes = self.nodes.write().await;

        let node = nodes
            .iter_mut()
            .find(|n| n.config.name == handle.node_name)
            .ok_or_else(|| anyhow::anyhow!("Node {} not found", handle.node_name))?;

        if let Some(size) = node.allocations.remove(&handle.id) {
            node.used_bytes = node.used_bytes.saturating_sub(size);
            tracing::debug!("🗑️ Released {} bytes from {}", size, handle.node_name);
        }

        Ok(())
    }

    /// Get total available memory across all nodes
    pub async fn total_available_bytes(&self) -> u64 {
        let nodes = self.nodes.read().await;
        nodes
            .iter()
            .filter(|n| n.connected)
            .map(|n| n.available_bytes())
            .sum()
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> RpcPoolStats {
        let nodes = self.nodes.read().await;

        RpcPoolStats {
            node_count: nodes.len(),
            connected_count: nodes.iter().filter(|n| n.connected).count(),
            total_capacity_mb: nodes.iter().map(|n| n.config.available_ram_mb).sum(),
            used_mb: nodes.iter().map(|n| n.used_bytes / (1024 * 1024)).sum(),
            allocation_count: nodes.iter().map(|n| n.allocations.len()).sum(),
        }
    }
}

impl Default for RpcMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Pool Statistics
// ============================================================================

/// Statistics for the RPC memory pool
#[derive(Debug, Clone, Default)]
pub struct RpcPoolStats {
    /// Total number of configured nodes
    pub node_count: usize,

    /// Number of connected nodes
    pub connected_count: usize,

    /// Total capacity in MB
    pub total_capacity_mb: u64,

    /// Used memory in MB
    pub used_mb: u64,

    /// Number of active allocations
    pub allocation_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_pool_creation() {
        let pool = RpcMemoryPool::new();
        pool.add_node(RpcNodeConfig::dell_laptop()).await.unwrap();

        let stats = pool.get_stats().await;
        assert_eq!(stats.node_count, 1);
    }

    #[tokio::test]
    async fn test_kv_cache_offload() {
        let pool = RpcMemoryPool::new();
        pool.add_node(RpcNodeConfig::dell_laptop()).await.unwrap();
        pool.connect_all().await.unwrap();

        let cache = vec![0u8; 1024 * 1024]; // 1MB
        let handle = pool.offload_kv_cache(cache, 0, 32).await.unwrap();

        assert_eq!(handle.node_name, "quadratical");
        assert_eq!(handle.size_bytes, 1024 * 1024);
    }
}
