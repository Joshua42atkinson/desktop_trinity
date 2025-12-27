//! # Agent Compiler - "Autopoietic Forge"
//!
//! ## Philosophy
//! "The compiler is the forge where agent DNA (Rust code) is transmuted into
//!  executable form (Wasm). It enables mother agents to birth child agents
//!  with precisely defined capabilities and behaviors."
//!
//! ## Compilation Pipeline
//!
//! ```text
//!    ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//!    │ Agent Spec  │ ──► │  Rust Code  │ ──► │   .wasm     │
//!    │   (JSON)    │     │  (Template) │     │   Module    │
//!    └─────────────┘     └─────────────┘     └─────────────┘
//!          │                   │                   │
//!          ▼                   ▼                   ▼
//!    • Name/Role         • Generated         • Validated
//!    • Capabilities      • Type-checked      • Sandboxed
//!    • System Prompt     • Compiled          • Capability-bound
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! let compiler = AgentCompiler::new()?;
//! let spec = AgentSpec::new("DataAnalyst")
//!     .with_capability(Capability::FileRead { paths: vec!["./data".into()] })
//!     .with_system_prompt("You analyze data files...");
//!
//! let wasm = compiler.compile(&spec).await?;
//! let sandbox = WasmSandbox::new()?;
//! let agent_id = sandbox.spawn(&wasm, spec.capabilities())?;
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::wasm_sandbox::{Capability, CapabilitySet};

// ============================================================================
// Agent Specification
// ============================================================================

/// Specification for a new agent to be compiled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    /// Unique identifier (assigned at creation)
    pub id: Uuid,

    /// Agent name
    pub name: String,

    /// Agent role/specialization
    pub role: AgentRole,

    /// System prompt defining behavior
    pub system_prompt: String,

    /// Capabilities to grant
    pub capabilities: CapabilitySet,

    /// Maximum memory in MB
    pub max_memory_mb: u32,

    /// Maximum execution time in seconds
    pub max_runtime_secs: u32,

    /// Parent agent ID (if spawned by another agent)
    pub parent_id: Option<Uuid>,

    /// Custom code to inject (advanced)
    pub custom_code: Option<String>,
}

/// Agent role (determines base behavior)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    /// General assistant
    Assistant,
    /// Code generator
    Coder,
    /// Data analyst
    Analyst,
    /// Research gatherer
    Researcher,
    /// Task executor
    Worker,
    /// Custom role
    Custom,
}

impl AgentSpec {
    /// Create a new agent specification
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            role: AgentRole::Assistant,
            system_prompt: String::new(),
            capabilities: CapabilitySet::sandboxed(),
            max_memory_mb: 64,
            max_runtime_secs: 60,
            parent_id: None,
            custom_code: None,
        }
    }

    /// Set the role
    pub fn with_role(mut self, role: AgentRole) -> Self {
        self.role = role;
        self
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Add a capability
    pub fn with_capability(mut self, cap: Capability) -> Self {
        self.capabilities = self.capabilities.with(cap);
        self
    }

    /// Set capabilities
    pub fn with_capabilities(mut self, caps: CapabilitySet) -> Self {
        self.capabilities = caps;
        self
    }

    /// Set memory limit
    pub fn with_max_memory(mut self, mb: u32) -> Self {
        self.max_memory_mb = mb;
        self
    }

    /// Set runtime limit
    pub fn with_max_runtime(mut self, secs: u32) -> Self {
        self.max_runtime_secs = secs;
        self
    }

    /// Set parent agent
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Add custom code
    pub fn with_custom_code(mut self, code: impl Into<String>) -> Self {
        self.custom_code = Some(code.into());
        self
    }
}

// ============================================================================
// Compilation Result
// ============================================================================

/// Result of compiling an agent
#[derive(Debug, Clone)]
pub struct CompiledAgent {
    /// Agent specification
    pub spec: AgentSpec,

    /// Compiled Wasm bytes
    pub wasm_bytes: Vec<u8>,

    /// Source code (for debugging)
    pub source_code: String,

    /// Compilation metadata
    pub metadata: CompilationMetadata,
}

/// Metadata about the compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationMetadata {
    /// Compiler version
    pub compiler_version: String,

    /// Compilation timestamp
    pub compiled_at: chrono::DateTime<chrono::Utc>,

    /// Source code hash
    pub source_hash: String,

    /// Wasm module hash
    pub wasm_hash: String,

    /// Size in bytes
    pub wasm_size: usize,
}

// ============================================================================
// Agent Compiler
// ============================================================================

/// Compiler that generates Wasm agents from specifications
pub struct AgentCompiler {
    /// Cache directory for compiled modules
    cache_dir: PathBuf,

    /// Template for agent code
    agent_template: String,
}

impl AgentCompiler {
    /// Create a new agent compiler
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .context("Could not find cache directory")?
            .join("trinity")
            .join("agents");

        std::fs::create_dir_all(&cache_dir)?;

        tracing::info!("🔧 AgentCompiler initialized (cache: {:?})", cache_dir);

        Ok(Self {
            cache_dir,
            agent_template: Self::default_template(),
        })
    }

    /// Default agent code template
    fn default_template() -> String {
        r##"
//! Auto-generated Trinity Agent
//! Name: {{AGENT_NAME}}
//! Role: {{AGENT_ROLE}}

#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

// Host function imports (provided by sandbox)
#[link(wasm_import_module = "trinity")]
extern "C" {
    fn log(level: u32, msg_ptr: *const u8, msg_len: u32);
    fn think(prompt_ptr: *const u8, prompt_len: u32, out_ptr: *mut u8, out_len: *mut u32) -> i32;
    fn memory_get(key_ptr: *const u8, key_len: u32, out_ptr: *mut u8, out_len: *mut u32) -> i32;
    fn memory_store(key_ptr: *const u8, key_len: u32, val_ptr: *const u8, val_len: u32) -> i32;
}

const SYSTEM_PROMPT: &str = "{{SYSTEM_PROMPT}}";

#[no_mangle]
pub extern "C" fn run(input_ptr: *const u8, input_len: u32) -> i32 {
    // Agent implementation
    // Will be customized based on role and capabilities
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
"##
        .to_string()
    }

    /// Compile an agent specification into Wasm
    pub async fn compile(&self, spec: &AgentSpec) -> Result<CompiledAgent> {
        tracing::info!("🔨 Compiling agent: {} ({})", spec.name, spec.id);

        // Generate source code from template
        let source_code = self.generate_source(spec)?;

        // For now, we don't actually compile to Wasm - that requires rustc + wasm target
        // Instead, return a placeholder that indicates compilation would happen
        let wasm_bytes = self.placeholder_wasm(spec);

        let source_hash = format!("{:x}", md5_hash(source_code.as_bytes()));
        let wasm_hash = format!("{:x}", md5_hash(&wasm_bytes));

        let metadata = CompilationMetadata {
            compiler_version: "0.1.0-placeholder".into(),
            compiled_at: chrono::Utc::now(),
            source_hash,
            wasm_hash,
            wasm_size: wasm_bytes.len(),
        };

        tracing::debug!(
            "📦 Agent {} compiled: {} bytes",
            spec.name,
            wasm_bytes.len()
        );

        Ok(CompiledAgent {
            spec: spec.clone(),
            wasm_bytes,
            source_code,
            metadata,
        })
    }

    /// Generate Rust source from spec
    fn generate_source(&self, spec: &AgentSpec) -> Result<String> {
        let source = self
            .agent_template
            .replace("{{AGENT_NAME}}", &spec.name)
            .replace("{{AGENT_ROLE}}", &format!("{:?}", spec.role))
            .replace("{{SYSTEM_PROMPT}}", &spec.system_prompt);

        Ok(source)
    }

    /// Generate placeholder Wasm (for testing)
    fn placeholder_wasm(&self, spec: &AgentSpec) -> Vec<u8> {
        // Minimal valid Wasm module (magic number + version + empty)
        let mut wasm = vec![
            0x00, 0x61, 0x73, 0x6D, // \0asm
            0x01, 0x00, 0x00, 0x00, // version 1
        ];

        // Add agent ID as custom section for identification
        let id_bytes = spec.id.as_bytes();
        wasm.extend_from_slice(&[
            0x00,                 // custom section
            (id_bytes.len() + 6) as u8, // section size
            0x05,                 // name length
            b'a', b'g', b'e', b'n', b't', // "agent"
        ]);
        wasm.extend_from_slice(id_bytes);

        wasm
    }

    /// Cache a compiled agent
    pub async fn cache(&self, agent: &CompiledAgent) -> Result<PathBuf> {
        let filename = format!("{}.wasm", agent.spec.id);
        let path = self.cache_dir.join(filename);
        tokio::fs::write(&path, &agent.wasm_bytes).await?;
        tracing::debug!("💾 Cached agent at {:?}", path);
        Ok(path)
    }

    /// Load a cached agent
    pub async fn load_cached(&self, id: Uuid) -> Result<Option<Vec<u8>>> {
        let filename = format!("{}.wasm", id);
        let path = self.cache_dir.join(filename);
        if path.exists() {
            let bytes = tokio::fs::read(&path).await?;
            Ok(Some(bytes))
        } else {
            Ok(None)
        }
    }
}

/// Simple hash function (placeholder for proper hashing)
fn md5_hash(data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_spec() {
        let spec = AgentSpec::new("TestAgent")
            .with_role(AgentRole::Coder)
            .with_system_prompt("You write code.");

        assert_eq!(spec.name, "TestAgent");
        assert_eq!(spec.role, AgentRole::Coder);
        assert!(!spec.system_prompt.is_empty());
    }

    #[tokio::test]
    async fn test_compile() {
        let compiler = AgentCompiler::new().unwrap();
        let spec = AgentSpec::new("CompileTest").with_role(AgentRole::Worker);

        let result = compiler.compile(&spec).await.unwrap();

        assert!(!result.wasm_bytes.is_empty());
        assert!(!result.source_code.is_empty());
        assert!(result.wasm_bytes.starts_with(&[0x00, 0x61, 0x73, 0x6D]));
    }
}
