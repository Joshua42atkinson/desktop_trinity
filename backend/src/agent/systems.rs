#![allow(unused)]
//! Agent Systems
//!
//! Bevy systems for agent lifecycle, routing, and task processing.

use super::components::*;
use super::events::*;
use bevy::prelude::*;

// ============================================================================
// RESOURCES
// ============================================================================

/// Global swarm coordinator resource
#[derive(Resource, Default)]
pub struct SwarmCoordinator {
    /// Maximum allowed delegation depth
    pub max_delegation_depth: u8,
    /// Next task ID
    pub next_task_id: u64,
    /// Total tasks completed
    pub tasks_completed: u64,
}

impl SwarmCoordinator {
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_task_id;
        self.next_task_id += 1;
        id
    }
}

/// Task queue for pending user requests
#[derive(Resource, Default)]
pub struct TaskQueue {
    pub pending: Vec<PendingTask>,
}

pub struct PendingTask {
    pub task_id: u64,
    pub content: String,
    pub created_at: f64,
}

// ============================================================================
// SWARM PLUGIN
// ============================================================================

/// Plugin that registers all agent systems and events
pub struct AgentSwarmPlugin;

impl Plugin for AgentSwarmPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<SwarmCoordinator>()
            .init_resource::<TaskQueue>()
            // Events
            .add_event::<UserRequest>()
            .add_event::<UserResponse>()
            .add_event::<AgentTaskRequest>()
            .add_event::<AgentTaskComplete>()
            .add_event::<AgentTaskError>()
            .add_event::<AgentDelegation>()
            .add_event::<DelegationComplete>()
            .add_event::<RoutingDecision>()
            .add_event::<SwarmStatusUpdate>()
            // Systems
            .add_systems(Startup, spawn_default_agents)
            .add_systems(
                Update,
                (
                    route_user_requests,
                    process_routing_decisions,
                    handle_delegations,
                    handle_task_completions,
                    emit_swarm_status,
                )
                    .chain(),
            );
    }
}

// ============================================================================
// STARTUP SYSTEMS
// ============================================================================

/// Spawn the default agent swarm on startup
fn spawn_default_agents(mut commands: Commands) {
    info!("🔮 Spawning agent swarm...");

    // Spawn router (fast model)
    commands.spawn(AgentBundle::router());

    // Spawn core agent (fast model)
    commands.spawn(AgentBundle::core());

    // Spawn specialized agents (smart model)
    commands.spawn(AgentBundle::research());
    commands.spawn(AgentBundle::developer());
    commands.spawn(AgentBundle::writer());

    info!("✅ Agent swarm ready: Router, Core, Research, Developer, Writer");
}

// ============================================================================
// ROUTING SYSTEM
// ============================================================================

/// Route incoming user requests to the router agent
fn route_user_requests(
    mut user_requests: EventReader<UserRequest>,
    mut agent_requests: EventWriter<AgentTaskRequest>,
    mut coordinator: ResMut<SwarmCoordinator>,
    agents: Query<(Entity, &AgentRole, &AgentState)>,
) {
    for request in user_requests.read() {
        // Find the router agent
        let router = agents
            .iter()
            .find(|(_, role, _)| matches!(role, AgentRole::Router));

        if let Some((entity, _, _)) = router {
            let task_id = coordinator.next_id();

            agent_requests.send(AgentTaskRequest {
                agent: entity,
                task_id,
                task: format!("ROUTE: {}", request.content),
                context: None,
                requester: None,
                depth: 0,
            });

            debug!("📨 Routed request {} to Router agent", task_id);
        } else {
            warn!("⚠️ No router agent found!");
        }
    }
}

/// Process routing decisions and dispatch to target agents
fn process_routing_decisions(
    mut routing_decisions: EventReader<RoutingDecision>,
    mut agent_requests: EventWriter<AgentTaskRequest>,
    agents: Query<(Entity, &AgentRole, &AgentState)>,
) {
    for decision in routing_decisions.read() {
        // Find agent by role name
        let target = agents
            .iter()
            .find(|(_, role, _)| role.name().eq_ignore_ascii_case(&decision.target_role));

        if let Some((entity, _, _)) = target {
            agent_requests.send(AgentTaskRequest {
                agent: entity,
                task_id: decision.task_id,
                task: decision.original_request.clone(),
                context: Some(decision.reasoning.clone()),
                requester: None,
                depth: 1,
            });

            info!(
                "🔀 Dispatched task {} to {} agent",
                decision.task_id, decision.target_role
            );
        } else {
            warn!("⚠️ Unknown target role: {}", decision.target_role);
        }
    }
}

// ============================================================================
// DELEGATION SYSTEM
// ============================================================================

/// Handle agent-to-agent delegations
fn handle_delegations(
    mut delegations: EventReader<AgentDelegation>,
    mut agent_requests: EventWriter<AgentTaskRequest>,
    coordinator: Res<SwarmCoordinator>,
    agents: Query<(Entity, &AgentRole)>,
) {
    for delegation in delegations.read() {
        // Check depth limit
        if delegation.depth >= coordinator.max_delegation_depth {
            warn!(
                "⚠️ Max delegation depth reached for task {}",
                delegation.task_id
            );
            continue;
        }

        // Find target agent by role
        let target = agents
            .iter()
            .find(|(_, role)| role.name().eq_ignore_ascii_case(&delegation.to_role));

        if let Some((entity, _)) = target {
            agent_requests.send(AgentTaskRequest {
                agent: entity,
                task_id: delegation.task_id,
                task: delegation.task.clone(),
                context: Some(delegation.context.clone()),
                requester: Some(delegation.from),
                depth: delegation.depth + 1,
            });

            debug!(
                "🔄 Delegation: task {} depth {}",
                delegation.task_id,
                delegation.depth + 1
            );
        }
    }
}

// ============================================================================
// COMPLETION SYSTEM
// ============================================================================

/// Handle task completions and send responses
fn handle_task_completions(
    mut completions: EventReader<AgentTaskComplete>,
    mut user_responses: EventWriter<UserResponse>,
    mut delegation_complete: EventWriter<DelegationComplete>,
    mut coordinator: ResMut<SwarmCoordinator>,
    agents: Query<(&Name, &AgentRole)>,
) {
    for complete in completions.read() {
        coordinator.tasks_completed += 1;

        // Get agent info for response
        let (name, role) = agents
            .get(complete.agent)
            .map(|(n, r)| (n.as_str().to_string(), r.name().to_string()))
            .unwrap_or(("Unknown".to_string(), "Unknown".to_string()));

        if let Some(requester) = complete.requester {
            // This was a delegation - return to requester
            delegation_complete.send(DelegationComplete {
                requester,
                handler: complete.agent,
                result: complete.result.clone(),
                task_id: complete.task_id,
            });
        } else {
            // This was a user request - send response
            user_responses.send(UserResponse {
                content: complete.result.clone(),
                from_agent: complete.agent,
                agent_role: role,
                citations: vec![],
            });
        }

        info!("✅ Task {} completed by {}", complete.task_id, name);
    }
}

// ============================================================================
// STATUS SYSTEM
// ============================================================================

/// Emit swarm status updates periodically
fn emit_swarm_status(
    mut status_updates: EventWriter<SwarmStatusUpdate>,
    agents: Query<&AgentState>,
    coordinator: Res<SwarmCoordinator>,
    task_queue: Res<TaskQueue>,
    mut last_update: Local<f64>,
    time: Res<Time>,
) {
    // Update every second
    let now = time.elapsed_seconds_f64();
    if now - *last_update < 1.0 {
        return;
    }
    *last_update = now;

    let active = agents.iter().filter(|s| s.is_busy()).count();
    let idle = agents.iter().filter(|s| s.is_idle()).count();

    status_updates.send(SwarmStatusUpdate {
        active_agents: active,
        idle_agents: idle,
        queued_tasks: task_queue.pending.len(),
        completed_tasks: coordinator.tasks_completed as usize,
    });
}
