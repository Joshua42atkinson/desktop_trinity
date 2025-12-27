pub mod demo;
pub mod systems;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Resource)]
pub struct SharedWorkflowStateResource(pub Arc<RwLock<SharedWorkflowState>>);

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SharedWorkflowState {
    pub active_executions: Vec<WorkflowExecution>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub nodes: HashMap<Uuid, WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub created_at: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: Uuid,
    pub label: String,
    pub kind: NodeKind,
    pub position: Vec2, // UI position
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeKind {
    /// Entry point for the workflow
    Trigger(TriggerType),
    /// An AI Agent processing step
    Agent(AgentConfig),
    /// A structured tool execution (e.g. search, file IO)
    Tool(ToolConfig),
    /// Logic flow control
    Router(RouterConfig),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TriggerType {
    Manual,
    Webhook { path: String },
    Schedule { cron: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    pub role_name: String, // e.g., "Research", "Writer"
    pub system_prompt_override: Option<String>,
    pub model_override: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    pub tool_name: String,
    pub parameters: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouterConfig {
    // JavaScript/Rhai compatible expression, e.g., "input.contains('error')"
    pub condition: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
    pub label: Option<String>,
}

// ============================================================================
// RUNTIME STRUCTURES
// ============================================================================

/// Represents a single execution instance of a workflow
/// Represents a single execution instance of a workflow
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub workflow_id: Uuid,
    pub execution_id: Uuid,
    pub status: ExecutionStatus,
    pub context: HashMap<String, serde_json::Value>,
    /// Tokens representing active execution heads
    pub tokens: Vec<WorkflowToken>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowToken {
    pub id: Uuid,
    pub current_node: Uuid,
    pub data: serde_json::Value,
    pub history: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Paused,
    Completed,
    Failed(String),
}
