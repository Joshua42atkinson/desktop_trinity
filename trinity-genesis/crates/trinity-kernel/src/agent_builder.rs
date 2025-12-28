// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! # Agent Builder - Ergonomic Agent Construction
//!
//! ## Philosophy
//! "Each agent is a personality with capabilities. The Builder pattern allows
//!  dynamic construction of agents at runtime—enabling Mother Agents to spawn
//!  Child Agents with precisely defined traits and constraints."
//!
//! ## Usage
//!
//! ```rust,ignore
//! let coder = AgentBuilder::new("Jessica")
//!     .specialization(AgentSpecialization::Coder)
//!     .with_brain(BrainTier::Worker)
//!     .with_tool(Tool::FileSystem)
//!     .with_tool(Tool::CodeExecution)
//!     .system_prompt("You are a meticulous Rust developer...")
//!     .build()?;
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::orchestrator::AgentSpecialization;

// ============================================================================
// Brain Tier (Compute Allocation)
// ============================================================================

/// Brain tier determines which compute resources the agent uses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BrainTier {
    /// High-intelligence model (e.g., Llama 4 Scout) - for planning
    Planner,

    /// High-speed/context model (e.g., GLM-4) - for execution
    #[default]
    Worker,

    /// NPU-accelerated background processing
    Background,

    /// Remote RPC node for large context
    Remote,
}

// ============================================================================
// Tool Definition
// ============================================================================

/// Tools available to agents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tool {
    /// File system read/write
    FileSystem,

    /// Code execution (sandboxed)
    CodeExecution,

    /// Web browsing
    WebBrowse,

    /// Memory store access
    MemoryStore,

    /// Spawn sub-agents
    AgentSpawn,

    /// Shell command execution
    Shell,

    /// Image generation
    ImageGen,

    /// Voice synthesis
    VoiceSynth,

    /// Custom tool with name
    Custom(String),
}

// ============================================================================
// Agent Capabilities
// ============================================================================

/// Capability restrictions for an agent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Allowed file paths for reading
    pub file_read_paths: Vec<String>,

    /// Allowed file paths for writing
    pub file_write_paths: Vec<String>,

    /// Network access allowed
    pub network_access: bool,

    /// Can spawn sub-agents
    pub can_spawn_agents: bool,

    /// Maximum memory in MB
    pub max_memory_mb: u32,

    /// Maximum execution time in seconds
    pub max_execution_secs: u32,
}

impl AgentCapabilities {
    /// Full access (for trusted system agents)
    pub fn full() -> Self {
        Self {
            file_read_paths: vec!["/".into()],
            file_write_paths: vec!["/".into()],
            network_access: true,
            can_spawn_agents: true,
            max_memory_mb: 8192,
            max_execution_secs: 3600,
        }
    }

    /// Restricted sandbox (for generated agents)
    pub fn sandbox() -> Self {
        Self {
            file_read_paths: vec![],
            file_write_paths: vec![],
            network_access: false,
            can_spawn_agents: false,
            max_memory_mb: 512,
            max_execution_secs: 60,
        }
    }

    /// Coder profile (file access, no network)
    pub fn coder(workspace: impl Into<String>) -> Self {
        let ws = workspace.into();
        Self {
            file_read_paths: vec![ws.clone()],
            file_write_paths: vec![ws],
            network_access: false,
            can_spawn_agents: false,
            max_memory_mb: 2048,
            max_execution_secs: 300,
        }
    }
}

// ============================================================================
// Agent Definition (Built Result)
// ============================================================================

/// A fully-defined agent ready for instantiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Unique identifier
    pub id: Uuid,

    /// Agent name/persona
    pub name: String,

    /// Specialization
    pub specialization: AgentSpecialization,

    /// Compute tier
    pub brain_tier: BrainTier,

    /// Available tools
    pub tools: HashSet<Tool>,

    /// System prompt
    pub system_prompt: String,

    /// Capability restrictions
    pub capabilities: AgentCapabilities,

    /// Parent agent (if spawned by another agent)
    pub parent_id: Option<Uuid>,

    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl AgentDefinition {
    /// Check if agent has a specific tool
    pub fn has_tool(&self, tool: &Tool) -> bool {
        self.tools.contains(tool)
    }

    /// Check if agent can read a path
    pub fn can_read(&self, path: &str) -> bool {
        self.capabilities
            .file_read_paths
            .iter()
            .any(|p| path.starts_with(p))
    }

    /// Check if agent can write to a path
    pub fn can_write(&self, path: &str) -> bool {
        self.capabilities
            .file_write_paths
            .iter()
            .any(|p| path.starts_with(p))
    }
}

// ============================================================================
// Agent Builder
// ============================================================================

/// Builder for constructing agent definitions
pub struct AgentBuilder {
    name: String,
    specialization: AgentSpecialization,
    brain_tier: BrainTier,
    tools: HashSet<Tool>,
    system_prompt: String,
    capabilities: AgentCapabilities,
    parent_id: Option<Uuid>,
    metadata: std::collections::HashMap<String, String>,
}

impl AgentBuilder {
    /// Create a new agent builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            specialization: AgentSpecialization::Coder,
            brain_tier: BrainTier::Worker,
            tools: HashSet::new(),
            system_prompt: String::new(),
            capabilities: AgentCapabilities::default(),
            parent_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set specialization
    pub fn specialization(mut self, spec: AgentSpecialization) -> Self {
        self.specialization = spec;
        self
    }

    /// Set brain tier
    pub fn with_brain(mut self, tier: BrainTier) -> Self {
        self.brain_tier = tier;
        self
    }

    /// Add a tool
    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tools.insert(tool);
        self
    }

    /// Add multiple tools
    pub fn with_tools(mut self, tools: impl IntoIterator<Item = Tool>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Set system prompt
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Set capabilities
    pub fn capabilities(mut self, caps: AgentCapabilities) -> Self {
        self.capabilities = caps;
        self
    }

    /// Set parent agent (for spawned agents)
    pub fn parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the agent definition
    pub fn build(self) -> Result<AgentDefinition> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Agent name cannot be empty"));
        }

        Ok(AgentDefinition {
            id: Uuid::new_v4(),
            name: self.name,
            specialization: self.specialization,
            brain_tier: self.brain_tier,
            tools: self.tools,
            system_prompt: self.system_prompt,
            capabilities: self.capabilities,
            parent_id: self.parent_id,
            metadata: self.metadata,
        })
    }
}

// ============================================================================
// Predefined Agent Templates
// ============================================================================

impl AgentBuilder {
    /// Create Joshua (Planner) template
    pub fn joshua() -> Self {
        Self::new("Joshua")
            .specialization(AgentSpecialization::Planner)
            .with_brain(BrainTier::Planner)
            .with_tools([Tool::MemoryStore, Tool::AgentSpawn])
            .system_prompt(
                "You are Joshua, the master planner. You break down complex tasks \
                 into actionable steps and coordinate other agents to execute them. \
                 You think strategically and consider edge cases.",
            )
            .capabilities(AgentCapabilities::full())
    }

    /// Create Jessica (Coder) template
    pub fn jessica() -> Self {
        Self::new("Jessica")
            .specialization(AgentSpecialization::Coder)
            .with_brain(BrainTier::Worker)
            .with_tools([
                Tool::FileSystem,
                Tool::CodeExecution,
                Tool::Shell,
                Tool::MemoryStore,
            ])
            .system_prompt(
                "You are Jessica, an expert Rust developer. You write clean, \
                 efficient, and well-documented code. You follow best practices \
                 and handle errors gracefully. You love type safety.",
            )
    }

    /// Create Jules (Reviewer) template
    pub fn jules() -> Self {
        Self::new("Jules")
            .specialization(AgentSpecialization::Reviewer)
            .with_brain(BrainTier::Worker)
            .with_tools([Tool::FileSystem, Tool::MemoryStore])
            .system_prompt(
                "You are Jules, the code reviewer. You scrutinize code for bugs, \
                 security issues, and style violations. You provide constructive \
                 feedback and suggest improvements.",
            )
    }

    /// Create Janet (Researcher) template
    pub fn janet() -> Self {
        Self::new("Janet")
            .specialization(AgentSpecialization::Researcher)
            .with_brain(BrainTier::Background)
            .with_tools([Tool::WebBrowse, Tool::MemoryStore])
            .system_prompt(
                "You are Janet, the researcher. You gather information from \
                 documentation, the web, and memory stores. You synthesize \
                 findings into actionable insights.",
            )
            .capabilities(AgentCapabilities {
                network_access: true,
                ..AgentCapabilities::sandbox()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_builder() {
        let agent = AgentBuilder::new("TestAgent")
            .specialization(AgentSpecialization::Coder)
            .with_tool(Tool::FileSystem)
            .build()
            .unwrap();

        assert_eq!(agent.name, "TestAgent");
        assert!(agent.has_tool(&Tool::FileSystem));
    }

    #[test]
    fn test_jessica_template() {
        let jessica = AgentBuilder::jessica().build().unwrap();

        assert_eq!(jessica.name, "Jessica");
        assert_eq!(jessica.brain_tier, BrainTier::Worker);
        assert!(jessica.has_tool(&Tool::CodeExecution));
    }

    #[test]
    fn test_capabilities() {
        let coder_caps = AgentCapabilities::coder("/home/user/project");

        assert!(coder_caps
            .file_read_paths
            .contains(&"/home/user/project".to_string()));
        assert!(!coder_caps.network_access);
    }
}
