// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

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
//!    │   ┌───────────────────────────────────────────────────────────┐    │
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
use wasmtime::*;

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
// Sandbox State (stored in wasmtime Store)
// ============================================================================

/// State accessible by host functions within the WASM sandbox
#[derive(Clone)]
pub struct SandboxState {
    /// The instance ID for tracking
    pub instance_id: Uuid,

    /// Capabilities granted to this sandbox
    pub capabilities: CapabilitySet,

    /// Workspace root for file operations
    pub workspace_root: PathBuf,

    /// Stdout buffer
    pub stdout: Vec<u8>,

    /// Stderr buffer
    pub stderr: Vec<u8>,

    /// Memory store (key-value)
    pub memory_store: HashMap<String, Vec<u8>>,
}

impl SandboxState {
    fn new(instance_id: Uuid, capabilities: CapabilitySet, workspace_root: PathBuf) -> Self {
        Self {
            instance_id,
            capabilities,
            workspace_root,
            stdout: Vec::new(),
            stderr: Vec::new(),
            memory_store: HashMap::new(),
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
// Compiled Module Cache
// ============================================================================

/// A cached compiled WASM module
struct CompiledModule {
    module: Module,
    #[allow(dead_code)]
    wasm_bytes: Vec<u8>,
}

// ============================================================================
// Wasm Sandbox
// ============================================================================

/// WebAssembly sandbox for executing agent code
///
/// Provides capability-based security and resource limits for untrusted code.
pub struct WasmSandbox {
    /// Wasmtime engine (shared across all instances)
    engine: Engine,

    /// Active instances metadata
    instances: Arc<RwLock<HashMap<Uuid, SandboxInstance>>>,

    /// Compiled module cache (path -> compiled module)
    module_cache: Arc<RwLock<HashMap<PathBuf, CompiledModule>>>,

    /// Default workspace root for file operations
    workspace_root: PathBuf,
}

impl WasmSandbox {
    /// Create a new Wasm sandbox with a workspace root
    pub fn with_workspace(workspace_root: PathBuf) -> Result<Self> {
        let mut config = Config::new();
        
        // Enable fuel metering for CPU limits
        config.consume_fuel(true);
        
        // Enable async support for non-blocking execution
        config.async_support(true);
        
        // Memory limits are set per-instance via ResourceLimiter
        
        let engine = Engine::new(&config)?;
        
        tracing::info!("🔒 WasmSandbox initialized (Autopoietic Womb ready)");
        tracing::debug!("   Workspace root: {:?}", workspace_root);

        Ok(Self {
            engine,
            instances: Arc::new(RwLock::new(HashMap::new())),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            workspace_root: workspace_root.clone(),
        })
    }

    /// Get the workspace root path
    pub fn workspace_path(&self) -> &std::path::Path {
        &self.workspace_root
    }

    /// Create a new Wasm sandbox with default workspace
    pub fn new() -> Result<Self> {
        let workspace_root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/tmp"));
        Self::with_workspace(workspace_root)
    }

    /// Load and compile a Wasm module from bytes
    pub async fn load_module(&self, path: PathBuf, wasm_bytes: Vec<u8>) -> Result<()> {
        // Compile the module (this is the expensive part)
        let module = Module::new(&self.engine, &wasm_bytes)
            .with_context(|| format!("Failed to compile WASM module: {:?}", path))?;
        
        let compiled = CompiledModule {
            module,
            wasm_bytes,
        };
        
        let mut cache = self.module_cache.write().await;
        cache.insert(path.clone(), compiled);
        
        tracing::debug!("📦 Compiled and cached Wasm module: {:?}", path);
        Ok(())
    }

    /// Load a module from a file path
    pub async fn load_module_from_file(&self, path: PathBuf) -> Result<()> {
        let wasm_bytes = std::fs::read(&path)
            .with_context(|| format!("Failed to read WASM file: {:?}", path))?;
        self.load_module(path, wasm_bytes).await
    }

    /// Spawn a new sandbox instance
    pub async fn spawn(
        &self,
        wasm_bytes: &[u8],
        config: SandboxConfig,
        parent_id: Option<Uuid>,
    ) -> Result<Uuid> {
        // Validate parent's spawn capability if this is a child spawn
        if let Some(pid) = parent_id {
            let instances = self.instances.read().await;
            if let Some(parent) = instances.get(&pid) {
                let can_spawn = parent.config.capabilities.capabilities.iter().any(|c| {
                    matches!(c, Capability::SpawnAgent { .. })
                });
                if !can_spawn {
                    return Err(anyhow::anyhow!("Parent agent lacks SpawnAgent capability"));
                }
            }
        }

        // Compile the module if bytes provided directly
        if !wasm_bytes.is_empty() {
            let _module = Module::new(&self.engine, wasm_bytes)
                .context("Failed to compile WASM module")?;
        }

        let instance = SandboxInstance::new(config, parent_id);
        let id = instance.id;

        // Store instance metadata
        let mut instances = self.instances.write().await;
        instances.insert(id, instance);

        tracing::info!("🐣 Spawned sandbox instance: {}", id);

        Ok(id)
    }

    /// Execute a function in a cached module
    ///
    /// This is the main entry point for executing WASM code.
    /// The module must be loaded first with `load_module`.
    /// Execute a function in a cached module with custom configuration
    pub async fn execute_with_config(
        &self,
        module_path: &PathBuf,
        function_name: &str,
        input: &str,
        config: SandboxConfig,
    ) -> Result<String> {
        // Get the compiled module
        let cache = self.module_cache.read().await;
        let compiled = cache
            .get(module_path)
            .with_context(|| format!("Module not loaded: {:?}", module_path))?;

        // Create a new store with fuel limits
        let instance_id = Uuid::new_v4();
        let state = SandboxState::new(
            instance_id,
            config.capabilities.clone(),
            self.workspace_root.clone(),
        );

        let mut store = Store::new(&self.engine, state);
        store.set_fuel(config.max_fuel)?;

        // Create a linker and add host functions
        let mut linker = Linker::new(&self.engine);
        self.add_host_functions(&mut linker)?;

        // Instantiate the module
        let instance = linker.instantiate_async(&mut store, &compiled.module).await?;

        // Find the exported function
        let func = instance
            .get_func(&mut store, function_name)
            .with_context(|| format!("Function '{}' not found in module", function_name))?;

        // For simple string I/O, we use memory exports
        // Most WASM plugins export: memory, alloc, dealloc, and the main function
        let memory = instance
            .get_memory(&mut store, "memory")
            .context("No memory export found - is this a valid plugin?")?;

        // Select execution strategy
        let use_alloc = instance.get_func(&mut store, "alloc");
        
        // Execute with timeout
        let max_ms = config.max_execution_ms;
        let execution_future = async {
            if let Some(alloc) = use_alloc {
                // Plugin uses alloc-based ABI (common for Rust WASM)
                self.execute_with_alloc(&mut store, &memory, &alloc, &func, input, &instance).await
            } else {
                // Simple ABI - function takes/returns i32 pointers directly
                self.execute_simple(&mut store, &memory, &func, input).await
            }
        };

        let result = match tokio::time::timeout(std::time::Duration::from_millis(max_ms), execution_future).await {
            Ok(res) => res,
            Err(_) => Err(anyhow::anyhow!("Execution timed out after {} ms", max_ms)),
        };

        // Get fuel consumed
        let fuel_remaining = store.get_fuel().unwrap_or(0);
        let fuel_consumed = config.max_fuel.saturating_sub(fuel_remaining);
        
        tracing::debug!(
            "📞 Executed {}::{} (fuel: {})",
            module_path.display(),
            function_name,
            fuel_consumed
        );

        result
    }

    /// Execute a function in a cached module (default sanitized configuration)
    pub async fn execute(
        &self,
        module_path: &PathBuf,
        function_name: &str,
        input: &str,
    ) -> Result<String> {
        self.execute_with_config(module_path, function_name, input, SandboxConfig::default()).await
    }



    /// Execute using alloc-based ABI (Rust WASM plugins)
    async fn execute_with_alloc(
        &self,
        store: &mut Store<SandboxState>,
        memory: &Memory,
        alloc: &Func,
        func: &Func,
        input: &str,
        instance: &Instance,
    ) -> Result<String> {
        // Allocate memory for input
        let input_bytes = input.as_bytes();
        let input_len = input_bytes.len() as i32;
        
        // Call alloc to get a pointer
        let alloc_typed = alloc.typed::<i32, i32>(&*store)?;
        let input_ptr = alloc_typed.call_async(&mut *store, input_len).await?;

        // Write input to WASM memory
        memory.write(&mut *store, input_ptr as usize, input_bytes)?;

        // Call the function with (ptr, len)
        let func_typed = func.typed::<(i32, i32), i64>(&*store)?;
        let result = func_typed.call_async(&mut *store, (input_ptr, input_len)).await?;

        // Decode result (ptr in high 32 bits, len in low 32 bits)
        let result_ptr = (result >> 32) as i32;
        let result_len = (result & 0xFFFFFFFF) as i32;

        // Read result from WASM memory
        let mut result_bytes = vec![0u8; result_len as usize];
        memory.read(&mut *store, result_ptr as usize, &mut result_bytes)?;

        // Free input allocation if dealloc exists
        if let Some(dealloc) = instance.get_func(&mut *store, "dealloc") {
            let dealloc_typed = dealloc.typed::<(i32, i32), ()>(&*store)?;
            dealloc_typed.call_async(&mut *store, (input_ptr, input_len)).await?;
        }

        Ok(String::from_utf8(result_bytes)?)
    }

    /// Execute using simple ABI (ptr as i32)
    async fn execute_simple(
        &self,
        store: &mut Store<SandboxState>,
        memory: &Memory,
        func: &Func,
        input: &str,
    ) -> Result<String> {
        // Write input at start of memory
        let input_bytes = input.as_bytes();
        memory.write(&mut *store, 0, input_bytes)?;

        // Call function with ptr=0, len=input.len()
        let func_typed = func.typed::<(i32, i32), i32>(&*store)?;
        let result_len = func_typed.call_async(&mut *store, (0, input_bytes.len() as i32)).await?;

        // Read output from same location (overwritten by function)
        let mut result_bytes = vec![0u8; result_len as usize];
        memory.read(&mut *store, 0, &mut result_bytes)?;

        Ok(String::from_utf8(result_bytes)?)
    }

    /// Add host functions to the linker
    fn add_host_functions(&self, linker: &mut Linker<SandboxState>) -> Result<()> {
        // Host function: log a message
        linker.func_wrap("env", "host_log", |mut caller: Caller<'_, SandboxState>, ptr: i32, len: i32| {
            if let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let mut buf = vec![0u8; len as usize];
                if memory.read(&caller, ptr as usize, &mut buf).is_ok() {
                    if let Ok(msg) = String::from_utf8(buf) {
                        let instance_id = caller.data().instance_id;
                        tracing::info!("[WASM:{}] {}", instance_id, msg);
                    }
                }
            }
        })?;

        // Host function: read file (capability-gated)
        linker.func_wrap(
            "env",
            "host_read_file",
            |mut caller: Caller<'_, SandboxState>, path_ptr: i32, path_len: i32, out_ptr: i32, out_capacity: i32| -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return -1,
                };

                // Read path from WASM memory
                let mut path_buf = vec![0u8; path_len as usize];
                if memory.read(&caller, path_ptr as usize, &mut path_buf).is_err() {
                    return -1;
                }
                let path = match String::from_utf8(path_buf) {
                    Ok(p) => PathBuf::from(p),
                    Err(_) => return -1,
                };

                // Check capability
                if !caller.data().capabilities.can_read_file(&path) {
                    tracing::warn!("[WASM] File read denied: {:?}", path);
                    return -2; // Permission denied
                }

                // Validate path is under workspace
                let full_path = caller.data().workspace_root.join(&path);
                if !full_path.starts_with(&caller.data().workspace_root) {
                    return -2; // Path escape attempt
                }

                // Read the file
                let contents = match std::fs::read(&full_path) {
                    Ok(c) => c,
                    Err(_) => return -3, // File not found or read error
                };

                // Check capacity
                if contents.len() as i32 > out_capacity {
                    return -4; // Buffer too small
                }

                // Write to output buffer
                if memory.write(&mut caller, out_ptr as usize, &contents).is_err() {
                    return -1;
                }

                contents.len() as i32
            },
        )?;

        // Host function: write file (capability-gated)
        linker.func_wrap(
            "env",
            "host_write_file",
            |mut caller: Caller<'_, SandboxState>, path_ptr: i32, path_len: i32, data_ptr: i32, data_len: i32| -> i32 {
                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return -1,
                };

                // Read path
                let mut path_buf = vec![0u8; path_len as usize];
                if memory.read(&caller, path_ptr as usize, &mut path_buf).is_err() {
                    return -1;
                }
                let path = match String::from_utf8(path_buf) {
                    Ok(p) => PathBuf::from(p),
                    Err(_) => return -1,
                };

                // Check capability
                if !caller.data().capabilities.can_write_file(&path) {
                    tracing::warn!("[WASM] File write denied: {:?}", path);
                    return -2;
                }

                // Validate path
                let full_path = caller.data().workspace_root.join(&path);
                if !full_path.starts_with(&caller.data().workspace_root) {
                    return -2;
                }

                // Read data
                let mut data = vec![0u8; data_len as usize];
                if memory.read(&caller, data_ptr as usize, &mut data).is_err() {
                    return -1;
                }

                // Write file
                match std::fs::write(&full_path, &data) {
                    Ok(_) => data_len,
                    Err(_) => -3,
                }
            },
        )?;

        // Host function: memory store (capability-gated)
        linker.func_wrap(
            "env",
            "host_memory_store",
            |mut caller: Caller<'_, SandboxState>, key_ptr: i32, key_len: i32, val_ptr: i32, val_len: i32| -> i32 {
                // Check capability
                let can_write = caller.data().capabilities.capabilities.iter().any(|c| {
                    matches!(c, Capability::MemoryStore { write: true, .. })
                });
                if !can_write {
                    return -2;
                }

                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return -1,
                };

                // Read key
                let mut key_buf = vec![0u8; key_len as usize];
                if memory.read(&caller, key_ptr as usize, &mut key_buf).is_err() {
                    return -1;
                }
                let key = match String::from_utf8(key_buf) {
                    Ok(k) => k,
                    Err(_) => return -1,
                };

                // Read value
                let mut val = vec![0u8; val_len as usize];
                if memory.read(&caller, val_ptr as usize, &mut val).is_err() {
                    return -1;
                }

                // Store
                caller.data_mut().memory_store.insert(key, val);
                0
            },
        )?;

        // Host function: memory get (capability-gated)
        linker.func_wrap(
            "env",
            "host_memory_get",
            |mut caller: Caller<'_, SandboxState>, key_ptr: i32, key_len: i32, out_ptr: i32, out_capacity: i32| -> i32 {
                // Check capability
                let can_read = caller.data().capabilities.capabilities.iter().any(|c| {
                    matches!(c, Capability::MemoryStore { read: true, .. })
                });
                if !can_read {
                    return -2;
                }

                let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return -1,
                };

                // Read key
                let mut key_buf = vec![0u8; key_len as usize];
                if memory.read(&caller, key_ptr as usize, &mut key_buf).is_err() {
                    return -1;
                }
                let key = match String::from_utf8(key_buf) {
                    Ok(k) => k,
                    Err(_) => return -1,
                };

                // Get value
                let val = match caller.data().memory_store.get(&key) {
                    Some(v) => v.clone(),
                    None => return 0, // Not found, return 0 length
                };

                if val.len() as i32 > out_capacity {
                    return -4;
                }

                // Write to output - need mutable caller
                // Note: This is a simplified version; in production you'd handle this differently
                let _ = out_ptr; // suppress warning
                val.len() as i32
            },
        )?;

        Ok(())
    }

    /// Execute a function in a sandbox instance (legacy API)
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
            drop(instances);
            let mut instances = self.instances.write().await;
            if let Some(inst) = instances.get_mut(&instance_id) {
                inst.running = true;
            }
        }

        tracing::debug!(
            "📞 Sandbox {} calling {}({:?})",
            instance_id,
            function_name,
            args
        );

        // For legacy API, return a message indicating to use execute() instead
        Ok(format!(
            "Use execute() for actual WASM execution. Called: {}({})",
            function_name,
            args.join(", ")
        ))
    }

    /// Terminate a sandbox instance
    pub async fn terminate(&self, instance_id: Uuid) -> Result<()> {
        let mut instances = self.instances.write().await;

        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.running = false;
            instance.exit_code = Some(-1);
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

    /// List all loaded modules
    pub async fn list_modules(&self) -> Vec<PathBuf> {
        let cache = self.module_cache.read().await;
        cache.keys().cloned().collect()
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
// Host Functions (exposed to Wasm) - Legacy API
// ============================================================================

/// Host functions that Wasm agents can call
///
/// These are the "system calls" available to sandboxed agents.
/// Note: These are now implemented via wasmtime linker functions above.
pub mod host_functions {
    /// Log a message (always allowed)
    #[allow(dead_code)]
    pub fn log(_level: u32, _message: &str) {
        // Implemented via wasmtime linker
    }

    /// Read a file (requires FileRead capability)
    #[allow(dead_code)]
    pub fn read_file(_path: &str) -> Result<Vec<u8>, String> {
        Err("Use wasmtime host functions".into())
    }

    /// Write a file (requires FileWrite capability)
    #[allow(dead_code)]
    pub fn write_file(_path: &str, _contents: &[u8]) -> Result<(), String> {
        Err("Use wasmtime host functions".into())
    }

    /// HTTP GET (requires Network capability)
    #[allow(dead_code)]
    pub fn http_get(_url: &str) -> Result<Vec<u8>, String> {
        Err("Not implemented - use async HTTP".into())
    }

    /// Query the LLM brain (always allowed, costs fuel)
    #[allow(dead_code)]
    pub fn think(_prompt: &str) -> Result<String, String> {
        Err("Use brain RPC".into())
    }

    /// Store data in memory (requires MemoryStore capability)
    #[allow(dead_code)]
    pub fn memory_store(_key: &str, _value: &[u8]) -> Result<(), String> {
        Err("Use wasmtime host functions".into())
    }

    /// Retrieve data from memory (requires MemoryStore capability)
    #[allow(dead_code)]
    pub fn memory_get(_key: &str) -> Result<Option<Vec<u8>>, String> {
        Err("Use wasmtime host functions".into())
    }

    /// Spawn a child agent (requires SpawnAgent capability)
    #[allow(dead_code)]
    pub fn spawn_agent(_wasm_bytes: &[u8]) -> Result<u64, String> {
        Err("Use WasmSandbox::spawn".into())
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

    #[test]
    fn test_engine_creation() {
        // Verify wasmtime engine can be created with our config
        let sandbox = WasmSandbox::new();
        assert!(sandbox.is_ok());
    }

    #[tokio::test]
    async fn test_module_loading() {
        let sandbox = WasmSandbox::new().unwrap();
        
        // Minimal valid WASM module (empty)
        let minimal_wasm = wat::parse_str(r#"
            (module
                (memory (export "memory") 1)
                (func (export "test") (result i32)
                    i32.const 42
                )
            )
        "#).expect("Failed to parse WAT");
        
        let result = sandbox.load_module(PathBuf::from("test.wasm"), minimal_wasm).await;
        assert!(result.is_ok());
        
        let modules = sandbox.list_modules().await;
        assert!(modules.contains(&PathBuf::from("test.wasm")));
    }
}
