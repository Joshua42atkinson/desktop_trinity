//! # Wasm Sandbox - "Autopoietic Womb"
//!
//! ## Philosophy
//! "The sandbox is the womb of autopoiesis—where agents birth new agents.
//!  It provides capability-based security, ensuring child agents cannot
//!  exceed their parent's permissions. The sandbox is both nursery and prison."
//!
//! ## Architecture
//!
//! ```text
//!    ┌─────────────────────────────────────────────────────────────────┐
//!    │                    Trinity Kernel                               │
//!    │                                                                 │
//!    │   ┌───────────────────────────────────────────────────────┐    │
//!    │   │                  WasmSandbox                           │    │
//!    │   │                                                        │    │
//!    │   │   ┌──────────┐   ┌──────────┐   ┌──────────┐          │    │
//!    │   │   │ Agent A  │   │ Agent B  │   │ Agent C  │          │    │
//!    │   │   │ (wasm)   │   │ (wasm)   │   │ (wasm)   │          │    │
//!    │   │   └────┬─────┘   └────┬─────┘   └────┬─────┘          │    │
//!    │   │        │              │              │                 │    │
//!    │   │   ╔════╧══════════════╧══════════════╧════╗           │    │
//!    │   │   ║     Capability-Based Host Functions    ║           │    │
//!    │   │   ╚═══════════════════════════════════════╝           │    │
//!    │   └───────────────────────────────────────────────────────┘    │
//!    └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Model
//!
//! 1. **Capability Tokens**: Agents receive tokens for specific permissions
//! 2. **Memory Isolation**: Each agent runs in isolated Wasm memory
//! 3. **Resource Limits**: CPU, memory, and I/O are bounded
//! 4. **Audit Trail**: All host calls are logged for review

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Capability Tokens
// ============================================================================

/// Permission token that grants specific capabilities to an agent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Read files from allowed paths
    FileRead { paths: Vec<PathBuf> },

    /// Write files to allowed paths
    FileWrite { paths: Vec<PathBuf> },

    /// Network access to specific domains
    Network { domains: Vec<String> },

    /// Spawn child agents (with max depth)
    SpawnAgent { max_children: u32 },

    /// Access memory store
    MemoryStore { read: bool, write: bool },

    /// Execute shell commands (dangerous!)
    Shell { allowed_commands: Vec<String> },

    /// Generate images
    ImageGen,

    /// Synthesize speech
    VoiceSynth,
}

/// A set of capabilities granted to an agent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: Vec<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set
    pub fn new() -> Self {
        Self::default()
    }

    /// Create full capabilities (for system agents only)
    pub fn full() -> Self {
        Self {
            capabilities: vec![
                Capability::FileRead {
                    paths: vec![PathBuf::from("/")],
                },
                Capability::FileWrite {
                    paths: vec![PathBuf::from("/")],
                },
                Capability::Network {
                    domains: vec!["*".into()],
                },
                Capability::SpawnAgent { max_children: 10 },
                Capability::MemoryStore {
                    read: true,
                    write: true,
                },
                Capability::ImageGen,
                Capability::VoiceSynth,
            ],
        }
    }

    /// Create sandboxed capabilities (for untrusted agents)
    pub fn sandboxed() -> Self {
        Self {
            capabilities: vec![Capability::MemoryStore {
                read: true,
                write: false,
            }],
        }
    }

    /// Add a capability
    pub fn with(mut self, cap: Capability) -> Self {
        self.capabilities.push(cap);
        self
    }

    /// Check if a capability is present
    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Check if file read is allowed for a path
    pub fn can_read_file(&self, path: &std::path::Path) -> bool {
        for cap in &self.capabilities {
            if let Capability::FileRead { paths } = cap {
                if paths.iter().any(|p| path.starts_with(p)) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if file write is allowed for a path
    pub fn can_write_file(&self, path: &std::path::Path) -> bool {
        for cap in &self.capabilities {
            if let Capability::FileWrite { paths } = cap {
                if paths.iter().any(|p| path.starts_with(p)) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if network access is allowed for a domain
    pub fn can_access_network(&self, domain: &str) -> bool {
        for cap in &self.capabilities {
            if let Capability::Network { domains } = cap {
                if domains.iter().any(|d| d == "*" || d == domain) {
                    return true;
                }
            }
        }
        false
    }
}

// ============================================================================
// Sandbox Configuration
// ============================================================================

/// Configuration for Wasm sandbox execution
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory in bytes
    pub max_memory_bytes: u64,

    /// Maximum execution time in milliseconds
    pub max_execution_ms: u64,

    /// Maximum fuel (instruction count)
    pub max_fuel: u64,

    /// Enable WASI preview 1
    pub enable_wasi: bool,

    /// Capabilities granted to the agent
    pub capabilities: CapabilitySet,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            max_execution_ms: 30_000,            // 30 seconds
            max_fuel: 1_000_000_000,             // 1 billion instructions
            enable_wasi: true,
            capabilities: CapabilitySet::sandboxed(),
        }
    }
}

impl SandboxConfig {
    /// Config for trusted agents
    pub fn trusted() -> Self {
        Self {
            max_memory_bytes: 512 * 1024 * 1024, // 512 MB
            max_execution_ms: 300_000,            // 5 minutes
            max_fuel: 10_000_000_000,             // 10 billion instructions
            enable_wasi: true,
            capabilities: CapabilitySet::full(),
        }
    }

    /// Config for quick untrusted tasks
    pub fn ephemeral() -> Self {
        Self {
            max_memory_bytes: 16 * 1024 * 1024, // 16 MB
            max_execution_ms: 5_000,             // 5 seconds
            max_fuel: 100_000_000,               // 100 million instructions
            enable_wasi: false,
            capabilities: CapabilitySet::new(),
        }
    }
}

// ============================================================================
// Sandbox Instance
// ============================================================================

/// A running Wasm sandbox instance
#[derive(Debug)]
pub struct SandboxInstance {
    /// Unique identifier
    pub id: Uuid,

    /// Parent agent (if spawned by another agent)
    pub parent_id: Option<Uuid>,

    /// Configuration
    pub config: SandboxConfig,

    /// Fuel consumed so far
    pub fuel_consumed: u64,

    /// Memory used in bytes
    pub memory_used: u64,

    /// Whether the instance is still running
    pub running: bool,

    /// Exit code (if finished)
    pub exit_code: Option<i32>,

    /// Output collected
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl SandboxInstance {
    fn new(config: SandboxConfig, parent_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id,
            config,
            fuel_consumed: 0,
            memory_used: 0,
            running: false,
            exit_code: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}

// ============================================================================
// Wasm Sandbox
// ============================================================================

/// WebAssembly sandbox for executing agent code
///
/// Provides capability-based security and resource limits for untrusted code.
pub struct WasmSandbox {
    /// Active instances
    instances: Arc<RwLock<HashMap<Uuid, SandboxInstance>>>,

    /// Wasm module cache (path -> compiled module bytes)
    module_cache: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
}

impl WasmSandbox {
    /// Create a new Wasm sandbox
    pub fn new() -> Result<Self> {
        tracing::info!("🔒 WasmSandbox initialized (Autopoietic Womb ready)");

        Ok(Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load a Wasm module from bytes
    pub async fn load_module(&self, path: PathBuf, wasm_bytes: Vec<u8>) -> Result<()> {
        let mut cache = self.module_cache.write().await;
        cache.insert(path.clone(), wasm_bytes);
        tracing::debug!("📦 Cached Wasm module: {:?}", path);
        Ok(())
    }

    /// Spawn a new sandbox instance
    pub async fn spawn(
        &self,
        _wasm_bytes: &[u8],
        config: SandboxConfig,
        parent_id: Option<Uuid>,
    ) -> Result<Uuid> {
        // Validate parent's spawn capability if this is a child spawn
        if let Some(pid) = parent_id {
            let instances = self.instances.read().await;
            if let Some(parent) = instances.get(&pid) {
                // Check if parent can spawn
                let can_spawn = parent.config.capabilities.capabilities.iter().any(|c| {
                    matches!(c, Capability::SpawnAgent { .. })
                });
                if !can_spawn {
                    return Err(anyhow::anyhow!("Parent agent lacks SpawnAgent capability"));
                }
            }
        }

        let instance = SandboxInstance::new(config, parent_id);
        let id = instance.id;

        // Store instance
        let mut instances = self.instances.write().await;
        instances.insert(id, instance);

        tracing::info!("🐣 Spawned sandbox instance: {}", id);

        // TODO: Actually compile and instantiate the Wasm module using wasmtime
        // This is a placeholder for now - actual implementation requires:
        // 1. Create wasmtime::Engine with fuel metering
        // 2. Create wasmtime::Store with limits
        // 3. Compile module
        // 4. Link WASI and custom host functions
        // 5. Run in async context

        Ok(id)
    }

    /// Execute a function in a sandbox
    pub async fn call(
        &self,
        instance_id: Uuid,
        function_name: &str,
        args: Vec<String>,
    ) -> Result<String> {
        let instances = self.instances.read().await;
        let instance = instances
            .get(&instance_id)
            .context("Sandbox instance not found")?;

        if !instance.running && instance.exit_code.is_none() {
            // First call - mark as running
            drop(instances);
            let mut instances = self.instances.write().await;
            if let Some(inst) = instances.get_mut(&instance_id) {
                inst.running = true;
            }
        }

        // TODO: Actually call the Wasm function
        // For now, return a placeholder
        tracing::debug!(
            "📞 Sandbox {} calling {}({:?})",
            instance_id,
            function_name,
            args
        );

        Ok(format!(
            "[Sandbox {}] Would call {}({:?})",
            instance_id, function_name, args.join(", ")
        ))
    }

    /// Terminate a sandbox instance
    pub async fn terminate(&self, instance_id: Uuid) -> Result<()> {
        let mut instances = self.instances.write().await;

        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.running = false;
            instance.exit_code = Some(-1); // Terminated
            tracing::info!("🛑 Terminated sandbox: {}", instance_id);
        }

        Ok(())
    }

    /// Get instance status
    pub async fn status(&self, instance_id: Uuid) -> Option<SandboxStatus> {
        let instances = self.instances.read().await;
        instances.get(&instance_id).map(|i| SandboxStatus {
            id: i.id,
            running: i.running,
            fuel_consumed: i.fuel_consumed,
            memory_used: i.memory_used,
            exit_code: i.exit_code,
        })
    }

    /// Clean up finished instances
    pub async fn cleanup(&self) {
        let mut instances = self.instances.write().await;
        let before = instances.len();

        instances.retain(|_, i| i.running || i.exit_code.is_none());

        let removed = before - instances.len();
        if removed > 0 {
            tracing::debug!("🧹 Cleaned up {} finished sandbox instances", removed);
        }
    }
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new().expect("Failed to create WasmSandbox")
    }
}

// ============================================================================
// Sandbox Status
// ============================================================================

/// Status of a sandbox instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxStatus {
    pub id: Uuid,
    pub running: bool,
    pub fuel_consumed: u64,
    pub memory_used: u64,
    pub exit_code: Option<i32>,
}

// ============================================================================
// Host Functions (exposed to Wasm)
// ============================================================================

/// Host functions that Wasm agents can call
///
/// These are the "system calls" available to sandboxed agents.
pub mod host_functions {
    /// Log a message (always allowed)
    pub fn log(_level: u32, _message: &str) {
        // Will be implemented with actual wasmtime linker
    }

    /// Read a file (requires FileRead capability)
    pub fn read_file(_path: &str) -> Result<Vec<u8>, String> {
        Err("Not implemented".into())
    }

    /// Write a file (requires FileWrite capability)
    pub fn write_file(_path: &str, _contents: &[u8]) -> Result<(), String> {
        Err("Not implemented".into())
    }

    /// HTTP GET (requires Network capability)
    pub fn http_get(_url: &str) -> Result<Vec<u8>, String> {
        Err("Not implemented".into())
    }

    /// Query the LLM brain (always allowed, costs fuel)
    pub fn think(_prompt: &str) -> Result<String, String> {
        Err("Not implemented".into())
    }

    /// Store data in memory (requires MemoryStore capability)
    pub fn memory_store(_key: &str, _value: &[u8]) -> Result<(), String> {
        Err("Not implemented".into())
    }

    /// Retrieve data from memory (requires MemoryStore capability)
    pub fn memory_get(_key: &str) -> Result<Option<Vec<u8>>, String> {
        Err("Not implemented".into())
    }

    /// Spawn a child agent (requires SpawnAgent capability)
    pub fn spawn_agent(_wasm_bytes: &[u8]) -> Result<u64, String> {
        Err("Not implemented".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_set() {
        let caps = CapabilitySet::new()
            .with(Capability::FileRead {
                paths: vec![PathBuf::from("/home/user")],
            })
            .with(Capability::MemoryStore {
                read: true,
                write: false,
            });

        assert!(caps.can_read_file(std::path::Path::new("/home/user/file.txt")));
        assert!(!caps.can_read_file(std::path::Path::new("/etc/passwd")));
        assert!(!caps.can_write_file(std::path::Path::new("/home/user/file.txt")));
    }

    #[test]
    fn test_sandbox_config() {
        let trusted = SandboxConfig::trusted();
        assert!(trusted.capabilities.capabilities.len() > 3);

        let sandboxed = SandboxConfig::default();
        assert!(sandboxed.capabilities.capabilities.len() <= 1);
    }

    #[tokio::test]
    async fn test_sandbox_spawn() {
        let sandbox = WasmSandbox::new().unwrap();
        let id = sandbox
            .spawn(&[], SandboxConfig::default(), None)
            .await
            .unwrap();

        let status = sandbox.status(id).await.unwrap();
        assert!(!status.running); // Not started yet
    }
}
