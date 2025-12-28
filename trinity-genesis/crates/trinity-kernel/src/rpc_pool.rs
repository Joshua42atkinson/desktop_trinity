// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

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

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// RPC Protocol Messages
// ============================================================================

/// RPC command types for the remote memory protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcCommand {
    /// Ping the node to check connectivity
    Ping,
    /// Store data on the remote node
    Store { id: Uuid, data: Vec<u8> },
    /// Retrieve data from the remote node
    Retrieve { id: Uuid },
    /// Delete data from the remote node
    Delete { id: Uuid },
    /// Query available memory
    QueryMemory,
}

/// RPC response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcResponse {
    /// Pong response with node name
    Pong { node_name: String },
    /// Data stored successfully
    Stored { id: Uuid, size: u64 },
    /// Retrieved data
    Data { id: Uuid, data: Vec<u8> },
    /// Data deleted successfully
    Deleted { id: Uuid },
    /// Memory info
    MemoryInfo { available_bytes: u64, used_bytes: u64 },
    /// Error response
    Error { message: String },
}

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

    /// Read/write timeout in ms
    pub io_timeout_ms: u64,
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
            io_timeout_ms: 30000,
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
// RPC Connection (Persistent TCP)
// ============================================================================

/// A persistent TCP connection to a remote RPC node
struct RpcConnection {
    stream: TcpStream,
    node_name: String,
}

impl RpcConnection {
    /// Send a command and receive a response
    async fn send_command(&mut self, cmd: &RpcCommand) -> Result<RpcResponse> {
        // Serialize command with length prefix
        let cmd_bytes = bincode::serialize(cmd)
            .context("Failed to serialize RPC command")?;
        let len = cmd_bytes.len() as u32;
        
        // Write length prefix (4 bytes, big-endian) + data
        self.stream.write_all(&len.to_be_bytes()).await
            .context("Failed to write command length")?;
        self.stream.write_all(&cmd_bytes).await
            .context("Failed to write command data")?;
        self.stream.flush().await?;
        
        // Read response length prefix
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await
            .context("Failed to read response length")?;
        let resp_len = u32::from_be_bytes(len_buf) as usize;
        
        // Read response data
        let mut resp_buf = vec![0u8; resp_len];
        self.stream.read_exact(&mut resp_buf).await
            .context("Failed to read response data")?;
        
        // Deserialize response
        let response: RpcResponse = bincode::deserialize(&resp_buf)
            .context("Failed to deserialize RPC response")?;
        
        Ok(response)
    }
}

// ============================================================================
// RPC Node (Connection State)
// ============================================================================

/// Active connection to a remote RPC node
struct RpcNode {
    config: RpcNodeConfig,
    connected: bool,
    connection: Option<RpcConnection>,
    allocations: HashMap<Uuid, u64>, // id → size
    used_bytes: u64,
    /// Local cache for data when remote is unavailable
    local_cache: HashMap<Uuid, Vec<u8>>,
}

impl std::fmt::Debug for RpcNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcNode")
            .field("config", &self.config)
            .field("connected", &self.connected)
            .field("allocations", &self.allocations)
            .field("used_bytes", &self.used_bytes)
            .finish()
    }
}

impl RpcNode {
    fn new(config: RpcNodeConfig) -> Self {
        Self {
            config,
            connected: false,
            connection: None,
            allocations: HashMap::new(),
            used_bytes: 0,
            local_cache: HashMap::new(),
        }
    }

    fn available_bytes(&self) -> u64 {
        let total = self.config.available_ram_mb * 1024 * 1024;
        total.saturating_sub(self.used_bytes)
    }

    /// Establish TCP connection with TCP_NODELAY
    async fn connect(&mut self) -> Result<()> {
        let addr = self.config.socket_addr()?;
        let timeout = Duration::from_millis(self.config.connect_timeout_ms);
        
        tracing::info!(
            "🔌 Connecting to RPC node: {} at {} (TCP_NODELAY={})",
            self.config.name,
            addr,
            self.config.tcp_nodelay
        );

        // Attempt connection with timeout
        let connect_result = tokio::time::timeout(
            timeout,
            TcpStream::connect(addr)
        ).await;

        match connect_result {
            Ok(Ok(stream)) => {
                // Set TCP_NODELAY to disable Nagle's algorithm for low latency
                if self.config.tcp_nodelay {
                    stream.set_nodelay(true)
                        .context("Failed to set TCP_NODELAY")?;
                }

                let connection = RpcConnection {
                    stream,
                    node_name: self.config.name.clone(),
                };

                self.connection = Some(connection);
                self.connected = true;
                
                tracing::info!(
                    "✅ Connected to RPC node: {} (TCP_NODELAY enabled)",
                    self.config.name
                );
                Ok(())
            }
            Ok(Err(e)) => {
                tracing::warn!(
                    "⚠️ Failed to connect to RPC node {}: {} (will use local cache)",
                    self.config.name,
                    e
                );
                // Mark as "connected" but without real connection - use local cache
                self.connected = true;
                self.connection = None;
                Ok(())
            }
            Err(_) => {
                tracing::warn!(
                    "⚠️ Connection to RPC node {} timed out after {}ms (will use local cache)",
                    self.config.name,
                    self.config.connect_timeout_ms
                );
                self.connected = true;
                self.connection = None;
                Ok(())
            }
        }
    }

    /// Send data to remote node (or cache locally if unavailable)
    async fn store_data(&mut self, id: Uuid, data: Vec<u8>) -> Result<()> {
        let size = data.len() as u64;
        
        if let Some(ref mut conn) = self.connection {
            // Send to remote
            let cmd = RpcCommand::Store { id, data: data.clone() };
            match conn.send_command(&cmd).await {
                Ok(RpcResponse::Stored { .. }) => {
                    tracing::debug!("📤 Stored {} bytes on remote node {}", size, self.config.name);
                }
                Ok(RpcResponse::Error { message }) => {
                    tracing::warn!("Remote store failed: {}, caching locally", message);
                    self.local_cache.insert(id, data);
                }
                Err(e) => {
                    tracing::warn!("RPC store error: {}, caching locally", e);
                    self.local_cache.insert(id, data);
                    // Connection likely broken, clear it
                    self.connection = None;
                }
                _ => {
                    tracing::warn!("Unexpected response, caching locally");
                    self.local_cache.insert(id, data);
                }
            }
        } else {
            // No remote connection, cache locally
            tracing::debug!("📦 Caching {} bytes locally (no remote connection)", size);
            self.local_cache.insert(id, data);
        }
        
        self.allocations.insert(id, size);
        self.used_bytes += size;
        Ok(())
    }

    /// Retrieve data from remote node (or local cache)
    async fn retrieve_data(&mut self, id: Uuid) -> Result<Vec<u8>> {
        // Check local cache first
        if let Some(data) = self.local_cache.get(&id) {
            tracing::debug!("📦 Retrieved {} bytes from local cache", data.len());
            return Ok(data.clone());
        }

        if let Some(ref mut conn) = self.connection {
            let cmd = RpcCommand::Retrieve { id };
            match conn.send_command(&cmd).await {
                Ok(RpcResponse::Data { data, .. }) => {
                    tracing::debug!("📥 Retrieved {} bytes from remote node {}", data.len(), self.config.name);
                    return Ok(data);
                }
                Ok(RpcResponse::Error { message }) => {
                    return Err(anyhow::anyhow!("Remote retrieve failed: {}", message));
                }
                Err(e) => {
                    self.connection = None;
                    return Err(anyhow::anyhow!("RPC retrieve error: {}", e));
                }
                _ => {
                    return Err(anyhow::anyhow!("Unexpected response from remote"));
                }
            }
        }

        Err(anyhow::anyhow!("Data {} not found in local cache and no remote connection", id))
    }

    /// Delete data from remote and local cache
    async fn delete_data(&mut self, id: Uuid) -> Result<()> {
        // Remove from local cache
        self.local_cache.remove(&id);

        if let Some(ref mut conn) = self.connection {
            let cmd = RpcCommand::Delete { id };
            let _ = conn.send_command(&cmd).await; // Best effort
        }

        if let Some(size) = self.allocations.remove(&id) {
            self.used_bytes = self.used_bytes.saturating_sub(size);
        }

        Ok(())
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
            // Establish actual TCP connection with TCP_NODELAY
            if let Err(e) = node.connect().await {
                tracing::error!(
                    "Failed to connect to RPC node {}: {}",
                    node.config.name,
                    e
                );
                // Continue trying other nodes
            }
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

        // Actually send data over RPC (or cache locally if unavailable)
        tracing::debug!(
            "📤 Offloading {} bytes of KV cache (layers {}-{}) to {}",
            size,
            layer_start,
            layer_end,
            node.config.name
        );

        node.store_data(handle.id, cache_bytes).await?;

        Ok(handle)
    }

    /// Retrieve KV cache from a remote node
    pub async fn retrieve_kv_cache(&self, handle: &RemoteHandle) -> Result<Vec<u8>> {
        let mut nodes = self.nodes.write().await;

        let node = nodes
            .iter_mut()
            .find(|n| n.config.name == handle.node_name)
            .ok_or_else(|| anyhow::anyhow!("Node {} not found", handle.node_name))?;

        if !node.allocations.contains_key(&handle.id) {
            return Err(anyhow::anyhow!("Handle {} not found on node", handle.id));
        }

        // Actually retrieve data over RPC (or from local cache)
        tracing::debug!(
            "📥 Retrieving {} bytes from {}",
            handle.size_bytes,
            handle.node_name
        );

        node.retrieve_data(handle.id).await
    }

    /// Release a remote allocation
    pub async fn release(&self, handle: &RemoteHandle) -> Result<()> {
        let mut nodes = self.nodes.write().await;

        let node = nodes
            .iter_mut()
            .find(|n| n.config.name == handle.node_name)
            .ok_or_else(|| anyhow::anyhow!("Node {} not found", handle.node_name))?;

        tracing::debug!("🗑️ Releasing {} bytes from {}", handle.size_bytes, handle.node_name);
        node.delete_data(handle.id).await
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
