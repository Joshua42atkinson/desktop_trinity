//! Agent Executor - The "Scheduler" of Trinity OS
//!
//! Manages the execution of agent processes (Thinking -> Executing -> Idle).
//! Utilizes Bevy ECS systems to schedule tasks with Brain integration.

#[cfg(feature = "desktop")]
use super::components::AgentRole;
use super::components::{AgentId, AgentState, Message, WorkingMemory};
use super::tools::{ToolCall, ToolRegistry};
use bevy::prelude::*;
use chrono::Utc;

#[cfg(feature = "desktop")]
use crate::brain::brain_resource::{BrainInterface, ThinkRequest};

// ============================================================================
// Agent Execution Event
// ============================================================================

/// Event requesting an agent to process a prompt
#[derive(Event, Clone)]
pub struct AgentThinkRequest {
    /// Target agent ID
    pub agent_id: uuid::Uuid,
    /// The prompt/task to process
    pub prompt: String,
}

/// Event when an agent completes thinking
#[derive(Event, Clone)]
pub struct AgentThinkComplete {
    /// Agent that completed
    pub agent_id: uuid::Uuid,
    /// The response from the brain
    pub response: String,
    /// Any tool calls extracted
    pub tool_calls: Vec<ToolCall>,
    /// Duration in ms
    pub duration_ms: u64,
}

// ============================================================================
// Agent Task Queue
// ============================================================================

/// Resource holding pending tasks for agents
#[derive(Resource, Default)]
pub struct AgentTaskQueue {
    /// Pending prompts by agent ID
    pending: std::collections::HashMap<uuid::Uuid, Vec<String>>,
}

impl AgentTaskQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a task for an agent
    pub fn queue(&mut self, agent_id: uuid::Uuid, prompt: String) {
        self.pending.entry(agent_id).or_default().push(prompt);
    }

    /// Get next task for an agent
    pub fn pop(&mut self, agent_id: uuid::Uuid) -> Option<String> {
        if let Some(tasks) = self.pending.get_mut(&agent_id) {
            if !tasks.is_empty() {
                return Some(tasks.remove(0));
            }
        }
        None
    }

    /// Check if agent has pending tasks
    pub fn has_pending(&self, agent_id: uuid::Uuid) -> bool {
        self.pending
            .get(&agent_id)
            .map(|t| !t.is_empty())
            .unwrap_or(false)
    }
}

// ============================================================================
// Pending Think Requests
// ============================================================================

/// Resource tracking in-flight think requests
#[derive(Resource, Default)]
pub struct PendingThinkRequests {
    /// Map from request ID to agent ID
    requests: std::collections::HashMap<uuid::Uuid, uuid::Uuid>,
}

impl PendingThinkRequests {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, request_id: uuid::Uuid, agent_id: uuid::Uuid) {
        self.requests.insert(request_id, agent_id);
    }

    pub fn resolve(&mut self, request_id: uuid::Uuid) -> Option<uuid::Uuid> {
        self.requests.remove(&request_id)
    }
}

// ============================================================================
// Core Executor Systems
// ============================================================================

/// System: Transitions agents from Thinking to Executing or Idle
pub fn agent_scheduler_system(mut query: Query<(&AgentId, &mut AgentState, &mut WorkingMemory)>) {
    for (id, mut state, mut memory) in query.iter_mut() {
        match *state {
            AgentState::Idle => {
                // Check for new messages in context?
                // In a real OS, this would check an Event Queue.
            }
            AgentState::Thinking => {
                tracing::info!("Agent {} is thinking...", id.0);
                // Simulate thinking process (would typically involve LLM inference)
                // For now, fast-forward to execution
                *state = AgentState::Executing;
            }
            AgentState::Executing => {
                tracing::info!("Agent {} is executing...", id.0);
                // Simulate execution
                // In reality, this would run tools or generate text

                // Log completion
                memory.context_window.push_back(Message {
                    role: "assistant".to_string(),
                    content: "Task completed.".to_string(),
                    timestamp: Utc::now(),
                });

                *state = AgentState::Idle;
            }
            AgentState::WaitingForInput => {
                // Blocked state
            }
            AgentState::Suspended => {
                // Do nothing
            }
        }
    }
}

/// System: Monitor agent health (Heartbeat)
pub fn agent_monitor_system(query: Query<(&AgentId, &AgentState)>) {
    for (id, state) in query.iter() {
        tracing::debug!("Agent {} status: {:?}", id.0, state);
    }
}

// ============================================================================
// Brain-Integrated Systems (desktop feature)
// ============================================================================

/// System: Check for pending tasks and submit to brain
#[cfg(feature = "desktop")]
pub fn agent_task_dispatch_system(
    mut task_queue: ResMut<AgentTaskQueue>,
    mut pending: ResMut<PendingThinkRequests>,
    brain: Res<BrainInterface>,
    mut query: Query<(&AgentId, &AgentRole, &mut AgentState, &WorkingMemory)>,
) {
    for (agent_id, role, mut state, memory) in query.iter_mut() {
        // Only process idle agents with pending tasks
        if *state != AgentState::Idle {
            continue;
        }

        if let Some(prompt) = task_queue.pop(agent_id.0) {
            // Build full prompt with context
            let full_prompt = build_prompt(role, memory, &prompt);

            // Create think request
            let request = ThinkRequest::new(full_prompt);
            let request_id = request.id;

            // Submit to brain
            if brain.submit(request) {
                pending.register(request_id, agent_id.0);
                *state = AgentState::Thinking;
                tracing::info!(
                    "Agent {} started thinking on: {}",
                    agent_id.0,
                    &prompt[..prompt.len().min(50)]
                );
            } else {
                tracing::warn!("Failed to submit think request for agent {}", agent_id.0);
            }
        }
    }
}

/// System: Poll for brain responses and update agents
#[cfg(feature = "desktop")]
pub fn agent_response_handler_system(
    brain: Res<BrainInterface>,
    mut pending: ResMut<PendingThinkRequests>,
    mut query: Query<(&AgentId, &mut AgentState, &mut WorkingMemory)>,
    mut complete_events: EventWriter<AgentThinkComplete>,
) {
    // Poll for responses
    while let Some(response) = brain.poll_response() {
        if let Some(agent_id) = pending.resolve(response.request_id) {
            // Find the agent
            for (id, mut state, mut memory) in query.iter_mut() {
                if id.0 == agent_id {
                    // Parse tool calls from response
                    let tool_calls = ToolCall::parse_from_output(&response.response);

                    // Update working memory
                    memory.context_window.push_back(Message {
                        role: "assistant".to_string(),
                        content: response.response.clone(),
                        timestamp: Utc::now(),
                    });

                    // Transition to executing (if there are tool calls) or idle
                    if tool_calls.is_empty() {
                        *state = AgentState::Idle;
                    } else {
                        *state = AgentState::Executing;
                    }

                    // Emit completion event
                    complete_events.send(AgentThinkComplete {
                        agent_id,
                        response: response.response.clone(),
                        tool_calls,
                        duration_ms: response.duration_ms,
                    });

                    tracing::info!(
                        "Agent {} completed thinking in {}ms",
                        agent_id,
                        response.duration_ms
                    );
                    break;
                }
            }
        }
    }
}

// ============================================================================
// Prompt Building
// ============================================================================

/// Build a full prompt with role and context
#[cfg(feature = "desktop")]
fn build_prompt(role: &AgentRole, memory: &WorkingMemory, task: &str) -> String {
    let system_prompt = match role {
        AgentRole::Kernel => "You are the Trinity Kernel, the core orchestrator of the Trinity AI OS.",
        AgentRole::Assistant => "You are a helpful AI assistant.",
        AgentRole::Researcher => "You are a research specialist. Find and synthesize information.",
        AgentRole::Developer => "You are a software developer. Write, edit, and debug code. Use tools to interact with the filesystem.",
        AgentRole::Writer => "You are a content writer. Create and edit text content.",
        AgentRole::Custom(name) => &format!("You are {}, a specialized agent.", name),
    };

    let mut prompt = format!("SYSTEM: {}\n\n", system_prompt);

    // Add context from working memory
    let context_messages: Vec<_> = memory.context_window.iter().collect();
    let recent = context_messages.iter().rev().take(10).rev();

    for msg in recent {
        prompt.push_str(&format!("{}: {}\n", msg.role.to_uppercase(), msg.content));
    }

    // Add scratchpad if present
    if !memory.scratchpad.is_empty() {
        prompt.push_str("\nSCRATCHPAD:\n");
        prompt.push_str(&memory.scratchpad);
        prompt.push('\n');
    }

    // Add current task
    prompt.push_str(&format!("\nUSER: {}\n\nASSISTANT:", task));

    prompt
}

// ============================================================================
// Tool Execution System
// ============================================================================

/// Resource for tool registry
#[derive(Resource)]
pub struct ToolRegistryResource(pub ToolRegistry);

impl Default for ToolRegistryResource {
    fn default() -> Self {
        Self(ToolRegistry::with_defaults())
    }
}

/// System: Execute tool calls for agents in Executing state
pub fn agent_tool_execution_system(
    tool_registry: Option<Res<ToolRegistryResource>>,
    mut query: Query<(&AgentId, &mut AgentState, &mut WorkingMemory)>,
) {
    let Some(_registry) = tool_registry else {
        return;
    };

    for (agent_id, mut state, memory) in query.iter_mut() {
        if *state != AgentState::Executing {
            continue;
        }

        // Check for pending tool calls in scratchpad
        // This is a simplified version - in production you'd track tool calls properly
        if let Some(last_msg) = memory.context_window.back() {
            let tool_calls = ToolCall::parse_from_output(&last_msg.content);

            if tool_calls.is_empty() {
                *state = AgentState::Idle;
                continue;
            }

            // For now, just log the tool calls (actual execution is async)
            for call in &tool_calls {
                tracing::info!(
                    "Agent {} would execute tool '{}' with params: {}",
                    agent_id.0,
                    call.tool_name,
                    serde_json::to_string(&call.parameters).unwrap_or_default()
                );
            }

            // Mark as idle (tool execution is async)
            *state = AgentState::Idle;
        }
    }
}
