//! Agent Router - The "Switchboard" of Trinity OS
//!
//! Directs user input or internal events to the appropriate Agent process.
//! Uses a lightweight classifier (or regex/rules for now) to route tasks.

use crate::agent::{AgentId, AgentRole};
use bevy::prelude::*;

/// Event: A new task request that needs routing
#[derive(Event, Debug, Clone)]
pub struct TaskRequest {
    pub content: String,
    pub preferred_agent: Option<String>,
}

/// System: Routes task requests to the best available agent
pub fn task_router_system(
    mut events: EventReader<TaskRequest>,
    agents: Query<(Entity, &AgentId, &AgentRole)>,
    // mut commands: Commands, // Would use this to spawn tasks
) {
    for event in events.read() {
        tracing::info!("Router received task: '{}'", event.content);

        // 1. Check for explicit preference
        if let Some(ref name) = event.preferred_agent {
            tracing::info!("Routing to requested agent: {}", name);
            // logic to find agent by name/role
            continue;
        }

        // 2. Simple Keyword Matching (Heuristic Routing)
        let target_role = if event.content.contains("code") || event.content.contains("function") {
            AgentRole::Developer
        } else if event.content.contains("research") || event.content.contains("find") {
            AgentRole::Researcher
        } else if event.content.contains("story") || event.content.contains("chapter") {
            AgentRole::Writer
        } else {
            AgentRole::Assistant
        };

        // 3. Find an agent with that role
        let mut found = false;
        for (_entity, id, role) in agents.iter() {
            // Manual matching because PartialEq can be tricky with enums carrying data
            let match_role = matches!(
                (role, &target_role),
                (AgentRole::Developer, AgentRole::Developer)
                    | (AgentRole::Researcher, AgentRole::Researcher)
                    | (AgentRole::Writer, AgentRole::Writer)
                    | (AgentRole::Assistant, AgentRole::Assistant)
            );

            if match_role {
                tracing::info!("Routed task to Agent {} ({:?})", id.0, role);
                found = true;
                break;
            }
        }

        if !found {
            tracing::warn!("No suitable agent found for role: {:?}", target_role);
        }
    }
}
