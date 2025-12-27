#![allow(unused)]
//! Agent ECS Components
//!
//! Defines the core components that make up an AI agent in the swarm.
//! Each agent is a Bevy entity with these components attached.

use crate::ai::llm::gguf_loader::ModelType;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// AGENT ROLE
// ============================================================================

/// The specialized role of an agent in the swarm
#[derive(
    Component, Reflect, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
#[reflect(Component)]
pub enum AgentRole {
    /// Fast model - decides which agent should handle a request
    Router,
    /// General conversation and simple tasks
    #[default]
    Core,
    /// Research with RAG and citations
    Research,
    /// Code generation and file operations
    Developer,
    /// Creative writing and narrative
    Writer,
    /// Custom agent with user-defined role
    Custom(String),
}

impl AgentRole {
    /// Returns true if this role should use the smart (120B) model
    pub fn uses_smart_model(&self) -> bool {
        matches!(
            self,
            AgentRole::Research | AgentRole::Developer | AgentRole::Writer
        )
    }

    /// Get a descriptive name for the role
    pub fn name(&self) -> &str {
        match self {
            AgentRole::Router => "Router",
            AgentRole::Core => "Core",
            AgentRole::Research => "Research",
            AgentRole::Developer => "Developer",
            AgentRole::Writer => "Writer",
            AgentRole::Custom(name) => name,
        }
    }
}

// ============================================================================
// AGENT STATE
// ============================================================================

/// Current operational state of an agent
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub enum AgentState {
    /// Agent is ready to accept tasks
    #[default]
    Idle,
    /// Agent is processing a task
    Processing { task_id: u64, task_preview: String },
    /// Agent delegated to another and is waiting
    WaitingForDelegate {
        delegate_to: Entity,
        original_task: String,
    },
    /// Agent completed its task
    Completed {
        task_id: u64,
        result_preview: String,
    },
    /// Agent encountered an error
    Error { message: String },
}

impl AgentState {
    pub fn is_idle(&self) -> bool {
        matches!(self, AgentState::Idle)
    }

    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            AgentState::Processing { .. } | AgentState::WaitingForDelegate { .. }
        )
    }
}

// ============================================================================
// AGENT MEMORY
// ============================================================================

/// Per-agent conversation memory
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct AgentMemory {
    /// Recent context (sliding window)
    pub context_window: VecDeque<MemoryEntry>,
    /// Maximum entries to keep
    pub max_context: usize,
    /// Total tokens used (approximate)
    pub token_count: usize,
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self {
            context_window: VecDeque::new(),
            max_context: 10,
            token_count: 0,
        }
    }
}

/// A single memory entry
#[derive(Reflect, Clone, Debug)]
pub struct MemoryEntry {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub timestamp: f64,
}

impl AgentMemory {
    /// Add a new entry, evicting old ones if necessary
    pub fn push(&mut self, role: impl Into<String>, content: impl Into<String>, timestamp: f64) {
        let entry = MemoryEntry {
            role: role.into(),
            content: content.into(),
            timestamp,
        };

        self.context_window.push_back(entry);

        // Evict oldest if over limit
        while self.context_window.len() > self.max_context {
            self.context_window.pop_front();
        }
    }

    /// Get context as formatted string for LLM
    pub fn as_context_string(&self) -> String {
        self.context_window
            .iter()
            .map(|e| format!("{}: {}", e.role, e.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Clear all memory
    pub fn clear(&mut self) {
        self.context_window.clear();
        self.token_count = 0;
    }
}

// ============================================================================
// AGENT MODEL CONFIG
// ============================================================================

/// Configuration for which LLM this agent uses
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct AgentModel {
    /// Type of model (Smart or Fast)
    pub model_type: ModelType,
    /// Endpoint URL (legacy/optional)
    pub endpoint: String,
    /// Max tokens to generate
    pub max_tokens: usize,
    /// Temperature for generation
    pub temperature: f32,
}

impl Default for AgentModel {
    fn default() -> Self {
        Self {
            model_type: ModelType::Fast,
            endpoint: "http://localhost:11434".to_string(),
            max_tokens: 500,
            temperature: 0.7,
        }
    }
}

impl AgentModel {
    /// Create config for the fast (router) model
    pub fn fast() -> Self {
        Self {
            model_type: ModelType::Fast,
            endpoint: "http://localhost:11434".to_string(),
            max_tokens: 200,
            temperature: 0.1,
        }
    }

    /// Create config for the smart (120B) model
    pub fn smart() -> Self {
        Self {
            model_type: ModelType::Smart,
            endpoint: "http://localhost:1234".to_string(),
            max_tokens: 1500,
            temperature: 0.7,
        }
    }
}

// ============================================================================
// AGENT BUNDLE
// ============================================================================

/// Bundle for spawning a complete agent entity
#[derive(Bundle)]
pub struct AgentBundle {
    pub name: Name,
    pub role: AgentRole,
    pub state: AgentState,
    pub memory: AgentMemory,
    pub model: AgentModel,
}

impl AgentBundle {
    /// Create a router agent (uses fast model)
    pub fn router() -> Self {
        Self {
            name: Name::new("Router"),
            role: AgentRole::Router,
            state: AgentState::Idle,
            memory: AgentMemory::default(),
            model: AgentModel::fast(),
        }
    }

    /// Create a core agent (uses fast model)
    pub fn core() -> Self {
        Self {
            name: Name::new("Core"),
            role: AgentRole::Core,
            state: AgentState::Idle,
            memory: AgentMemory::default(),
            model: AgentModel::fast(),
        }
    }

    /// Create a research agent (uses smart model)
    pub fn research() -> Self {
        Self {
            name: Name::new("Research"),
            role: AgentRole::Research,
            state: AgentState::Idle,
            memory: AgentMemory {
                max_context: 20,
                ..Default::default()
            },
            model: AgentModel::smart(),
        }
    }

    /// Create a developer agent (uses smart model)
    pub fn developer() -> Self {
        Self {
            name: Name::new("Developer"),
            role: AgentRole::Developer,
            state: AgentState::Idle,
            memory: AgentMemory {
                max_context: 15,
                ..Default::default()
            },
            model: AgentModel {
                max_tokens: 2000,
                temperature: 0.4,
                ..AgentModel::smart()
            },
        }
    }

    /// Create a writer agent (uses smart model)
    pub fn writer() -> Self {
        Self {
            name: Name::new("Writer"),
            role: AgentRole::Writer,
            state: AgentState::Idle,
            memory: AgentMemory::default(),
            model: AgentModel {
                max_tokens: 1000,
                temperature: 0.8,
                ..AgentModel::smart()
            },
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_role_uses_smart_model() {
        assert!(!AgentRole::Router.uses_smart_model());
        assert!(!AgentRole::Core.uses_smart_model());
        assert!(AgentRole::Research.uses_smart_model());
        assert!(AgentRole::Developer.uses_smart_model());
        assert!(AgentRole::Writer.uses_smart_model());
    }

    #[test]
    fn test_agent_memory_sliding_window() {
        let mut memory = AgentMemory {
            max_context: 3,
            ..Default::default()
        };

        memory.push("user", "Hello", 1.0);
        memory.push("assistant", "Hi there", 2.0);
        memory.push("user", "How are you?", 3.0);
        memory.push("assistant", "I'm well!", 4.0); // This should evict first

        assert_eq!(memory.context_window.len(), 3);
        assert_eq!(memory.context_window[0].content, "Hi there");
    }
}
