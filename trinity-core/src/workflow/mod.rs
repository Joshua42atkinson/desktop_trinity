//! Workflow Engine - The "Userland" of Trinity OS
//!
//! Manages n8n-style node-graph execution for Agents.
//! Handles data flow between agents via `TaskCompleted` events.

use crate::agent::{AgentId, TaskRequest, WorkflowNode};
use bevy::prelude::*;
use uuid::Uuid;

/// Event: An agent has completed a task
#[derive(Event, Debug, Clone)]
pub struct TaskCompleted {
    /// The ID of the agent that finished
    pub agent_id: AgentId,
    /// The workflow this task belongs to (if any)
    pub workflow_id: Option<Uuid>,
    /// The result/output of the task
    pub result: String,
}

/// System: Propagates execution through the workflow graph
pub fn workflow_execution_system(
    mut completed_events: EventReader<TaskCompleted>,
    // Query to look up node connections
    nodes: Query<(Entity, &AgentId, &WorkflowNode)>,
    // Query to get AgentId from Entity (for outputs)
    agent_ids: Query<&AgentId>,
    // To trigger next steps
    mut task_requests: EventWriter<TaskRequest>,
) {
    for event in completed_events.read() {
        tracing::debug!(
            "Workflow: Processing completion for Agent {}",
            event.agent_id.0
        );

        // 1. Find the node corresponding to the completed agent
        let mut current_node_outputs = None;

        for (_entity, id, node) in nodes.iter() {
            if id.0 == event.agent_id.0 {
                current_node_outputs = Some(&node.outputs);
                break;
            }
        }

        // 2. If node found, trigger downstream agents
        if let Some(outputs) = current_node_outputs {
            if outputs.is_empty() {
                tracing::debug!(
                    "Workflow: No downstream nodes for Agent {}",
                    event.agent_id.0
                );
                continue;
            }

            tracing::info!("Workflow: Triggering {} downstream agents", outputs.len());

            for &output_entity in outputs {
                // Get the AgentId of the downstream agent
                if let Ok(downstream_id) = agent_ids.get(output_entity) {
                    tracing::info!(
                        "Workflow: Propagation -> Agent {} (Input: '{}')",
                        downstream_id.0,
                        event.result
                    );

                    // 3. Send TaskRequest to downstream agent
                    task_requests.send(TaskRequest {
                        content: format!("Input from previous step: {}", event.result),
                        // We target the specific agent by ID via preferred_agent string
                        preferred_agent: Some(downstream_id.0.to_string()),
                    });
                } else {
                    tracing::warn!(
                        "Workflow: Downstream entity {:?} missing AgentId",
                        output_entity
                    );
                }
            }
        }
    }
}
