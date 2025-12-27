//! Streaming Protocol - Agent Event Types for RPC
//!
//! These types are used to stream agent events from Brain to Body
//! for the Antigravity Window visualization.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::artifact::Artifact;

/// Model tier for agent task routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    /// Fastest tier - small models for quick tasks (Gemma 2B, etc)
    Fast,
    /// Standard tier - balanced models for routine work (Llama 8B, etc)
    Standard,
    /// Powerful tier - large models for complex reasoning (Llama 70B, etc)
    Powerful,
    /// Reflection tier - the big model for deep thinking (Llama 4 Scout, Qwen 235B)
    Reflection,
}

impl Default for ModelTier {
    fn default() -> Self {
        Self::Standard
    }
}

/// Agent status for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub id: String,
    pub name: String,
    pub model_tier: ModelTier,
    pub is_busy: bool,
    pub current_task: Option<String>,
}

/// Events streamed from Brain to Body for Antigravity Window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    /// Agent started working on a task
    TaskStarted {
        agent_id: String,
        task_id: Uuid,
        task_name: String,
    },
    /// Agent is thinking/reasoning (stream thought tokens)
    Thinking { agent_id: String, thought: String },
    /// Agent generated code
    CodeGenerated {
        agent_id: String,
        file_path: String,
        code_snippet: String,
        line_count: usize,
    },
    /// Agent is running a command
    CommandRunning { agent_id: String, command: String },
    /// Command output
    CommandOutput {
        agent_id: String,
        stdout: String,
        stderr: String,
    },
    /// Task completed successfully
    TaskCompleted {
        agent_id: String,
        task_id: Uuid,
        result: String,
        duration_ms: u64,
        tokens_consumed: u32,
    },
    /// Task failed
    TaskFailed {
        agent_id: String,
        task_id: Uuid,
        error: String,
        tokens_consumed: u32,
    },
    /// Agent status update
    AgentStatusUpdate { agents: Vec<AgentStatus> },
    /// Agent generated a structured artifact (for rich UI rendering)
    ArtifactGenerated {
        agent_id: String,
        artifact: Artifact,
    },
    /// Agent mode changed (Planning vs Fast)
    ModeChanged {
        agent_id: String,
        mode: crate::artifact::AgentMode,
        reason: Option<String>,
    },
}

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub model_tier: ModelTier,
    pub specialization: String,
    pub enabled: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            id: "agent-0".to_string(),
            name: "Coder 1".to_string(),
            model_tier: ModelTier::Standard,
            specialization: "general".to_string(),
            enabled: true,
        }
    }
}

/// Orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub agents: Vec<AgentConfig>,
    /// Always use Reflection tier for evaluation/verification
    pub use_reflection_for_eval: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            agents: vec![
                AgentConfig {
                    id: "agent-0".to_string(),
                    name: "Coder 1".to_string(),
                    model_tier: ModelTier::Standard,
                    specialization: "rust".to_string(),
                    enabled: true,
                },
                AgentConfig {
                    id: "agent-1".to_string(),
                    name: "Coder 2".to_string(),
                    model_tier: ModelTier::Standard,
                    specialization: "general".to_string(),
                    enabled: true,
                },
            ],
            use_reflection_for_eval: true,
        }
    }
}
