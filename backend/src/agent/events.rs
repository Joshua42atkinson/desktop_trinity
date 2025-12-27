#![allow(unused)]
//! Agent Events
//!
//! Bevy events for agent-to-agent communication and task routing.

use bevy::prelude::*;

// ============================================================================
// USER INPUT EVENTS
// ============================================================================

/// Event: User submitted a request
#[derive(Event, Clone, Debug)]
pub struct UserRequest {
    /// The user's input text
    pub content: String,
    /// Optional session ID for context
    pub session_id: Option<String>,
}

/// Event: Response ready for user
#[derive(Event, Clone, Debug)]
pub struct UserResponse {
    /// The response content
    pub content: String,
    /// Which agent produced this response
    pub from_agent: Entity,
    /// Agent role name for display
    pub agent_role: String,
    /// Citations if from research agent
    pub citations: Vec<String>,
}

// ============================================================================
// AGENT TASK EVENTS
// ============================================================================

/// Event: Request sent to a specific agent
#[derive(Event, Clone, Debug)]
pub struct AgentTaskRequest {
    /// Target agent entity
    pub agent: Entity,
    /// Task ID for tracking
    pub task_id: u64,
    /// The task/prompt to process
    pub task: String,
    /// Additional context
    pub context: Option<String>,
    /// Who requested this (for delegation chains)
    pub requester: Option<Entity>,
    /// Current delegation depth
    pub depth: u8,
}

/// Event: Agent completed a task
#[derive(Event, Clone, Debug)]
pub struct AgentTaskComplete {
    /// Which agent completed
    pub agent: Entity,
    /// Task ID
    pub task_id: u64,
    /// The result
    pub result: String,
    /// Original requester (if delegated)
    pub requester: Option<Entity>,
}

/// Event: Agent encountered an error
#[derive(Event, Clone, Debug)]
pub struct AgentTaskError {
    /// Which agent errored
    pub agent: Entity,
    /// Task ID
    pub task_id: u64,
    /// Error message
    pub error: String,
}

// ============================================================================
// DELEGATION EVENTS
// ============================================================================

/// Event: Agent wants to delegate to another agent
#[derive(Event, Clone, Debug)]
pub struct AgentDelegation {
    /// Requesting agent
    pub from: Entity,
    /// Target agent role (will be resolved to entity)
    pub to_role: String,
    /// The task to delegate
    pub task: String,
    /// Context from the delegating agent
    pub context: String,
    /// Current depth (to prevent infinite loops)
    pub depth: u8,
    /// Original task ID
    pub task_id: u64,
}

/// Event: Delegation completed, result returned to original agent
#[derive(Event, Clone, Debug)]
pub struct DelegationComplete {
    /// Original requesting agent
    pub requester: Entity,
    /// Agent that handled the delegation
    pub handler: Entity,
    /// The result
    pub result: String,
    /// Original task ID
    pub task_id: u64,
}

// ============================================================================
// ROUTING EVENTS
// ============================================================================

/// Event: Router decided which agent should handle a request
#[derive(Event, Clone, Debug)]
pub struct RoutingDecision {
    /// Task ID
    pub task_id: u64,
    /// Original user request
    pub original_request: String,
    /// Chosen agent role
    pub target_role: String,
    /// Router's reasoning
    pub reasoning: String,
}

// ============================================================================
// SWARM STATUS EVENTS
// ============================================================================

/// Event: Swarm status update (for UI)
#[derive(Event, Clone, Debug)]
pub struct SwarmStatusUpdate {
    /// Active agents count
    pub active_agents: usize,
    /// Idle agents count
    pub idle_agents: usize,
    /// Tasks in queue
    pub queued_tasks: usize,
    /// Tasks completed this session
    pub completed_tasks: usize,
}
