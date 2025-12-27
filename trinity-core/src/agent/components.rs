//! Agent Components - The "Process Control Block" of Trinity OS
//!
//! In Trinity OS, every "Agent" is an ECS Entity, analogous to a process in Unix.
//! These components define the state, memory, and capabilities of the agent.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// The core identity of an Agent (Process ID)
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AgentId(pub uuid::Uuid);

/// The current state of the agent process
#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub enum AgentState {
    #[default]
    Idle,
    Thinking,
    Executing,
    WaitingForInput,
    Suspended,
}

/// The Agent's role/specialization (Analogy: Program binary)
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub enum AgentRole {
    /// The Kernel/Root agent (System management)
    Kernel,
    /// General purpose assistant
    Assistant,
    /// Deep research specialist
    Researcher,
    /// Code generation specialist
    Developer,
    /// Creative writer
    Writer,
    /// Custom user-defined role
    Custom(String),
}

/// The Agent's working memory (Analogy: RAM / Heap)
#[derive(Component, Debug, Clone, Default)]
pub struct WorkingMemory {
    /// Short-term conversation history
    pub context_window: VecDeque<Message>,
    /// Currently active tool outputs
    pub scratchpad: String,
}

/// A message in the agent's context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Capabilities/Permissions (Analogy: User Groups / Capabilities)
#[derive(Component, Debug, Clone, Default)]
pub struct AgentCapabilities {
    pub can_read_files: bool,
    pub can_write_files: bool,
    pub can_access_internet: bool,
    pub can_spawn_agents: bool,
}

/// Workflow Node - Connecting Agents into n8n-style graphs
#[derive(Component, Debug, Clone)]
pub struct WorkflowNode {
    /// Workflow this agent belongs to
    pub workflow_id: uuid::Uuid,
    /// Upstream agents (inputs)
    pub inputs: Vec<Entity>,
    /// Downstream agents (outputs)
    pub outputs: Vec<Entity>,
}
