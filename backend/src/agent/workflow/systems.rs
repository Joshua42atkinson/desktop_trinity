use super::*;
use crate::agent::components::{AgentRole, AgentState};
use crate::agent::events::*;
use bevy::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Resource, Default)]
pub struct WorkflowRunner {
    pub active_workflows: HashMap<Uuid, WorkflowExecution>,
    pub definitions: HashMap<Uuid, Workflow>,
    /// Maps task_id -> (execution_id, token_id)
    pub task_map: HashMap<u64, (Uuid, Uuid)>,
}

pub struct WorkflowPlugin;

impl Plugin for WorkflowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorkflowRunner>()
            .add_event::<WorkflowStepEvent>()
            .add_systems(Startup, load_demo_workflow)
            .add_systems(
                Update,
                (
                    workflow_execution_loop,
                    handle_workflow_events,
                    sync_workflow_state,
                ),
            );
    }
}

pub fn load_demo_workflow(mut runner: ResMut<WorkflowRunner>) {
    let demo = super::demo::get_demo_workflow();
    info!(
        "🚀 Loaded Demo Workflow: '{}' with {} nodes",
        demo.name,
        demo.nodes.len()
    );

    // Store definition
    runner.definitions.insert(demo.id, demo.clone());

    // Initialize standard execution state
    let mut execution = WorkflowExecution {
        workflow_id: demo.id,
        execution_id: Uuid::new_v4(),
        status: ExecutionStatus::Running,
        context: HashMap::new(),
        tokens: Vec::new(),
    };

    // Find trigger node
    if let Some((trigger_id, _)) = demo
        .nodes
        .iter()
        .find(|(_, n)| matches!(n.kind, NodeKind::Trigger(_)))
    {
        execution.tokens.push(WorkflowToken {
            id: Uuid::new_v4(),
            current_node: *trigger_id,
            data: serde_json::json!({ "init": true }),
            history: vec![],
        });
        info!("▶ Started Workflow Execution: {:?}", execution.execution_id);
    }

    runner
        .active_workflows
        .insert(execution.execution_id, execution);
}

#[derive(Event)]
pub struct WorkflowStepEvent {
    pub execution_id: Uuid,
    pub token_id: Uuid,
    pub node_id: Uuid,
}

use crate::agent::systems::SwarmCoordinator;

/// Main loop that advances tokens through the graph
pub fn workflow_execution_loop(
    mut runner: ResMut<WorkflowRunner>,
    mut agent_requests: EventWriter<AgentTaskRequest>,
    mut coordinator: ResMut<SwarmCoordinator>,
    mut step_events: EventWriter<WorkflowStepEvent>,
    agents: Query<(Entity, &AgentRole, &AgentState)>,
    mut completions: EventReader<AgentTaskComplete>,
) {
    // Split borrows to satisfy the borrow checker
    let WorkflowRunner {
        active_workflows,
        definitions,
        task_map,
    } = &mut *runner;

    // 1. Handle Task Completions first (resume waiting tokens)
    for complete in completions.read() {
        if let Some((exec_id, token_id)) = task_map.remove(&complete.task_id) {
            if let Some(execution) = active_workflows.get_mut(&exec_id) {
                if let Some(token) = execution.tokens.iter_mut().find(|t| t.id == token_id) {
                    // Clear waiting state
                    token
                        .data
                        .as_object_mut()
                        .unwrap()
                        .remove("waiting_for_task");

                    // Store result
                    token.data["result"] = serde_json::json!(complete.result);

                    info!(
                        "✅ Workflow Task {} Completed. Result stored in token.",
                        complete.task_id
                    );

                    // Advance node
                    if let Some(workflow) = definitions.get(&execution.workflow_id) {
                        if let Some(edge) = workflow
                            .edges
                            .iter()
                            .find(|e| e.source == token.current_node)
                        {
                            token.history.push(token.current_node);
                            token.current_node = edge.target;
                            info!(
                                "➡️ Token advanced after task completion to {:?}",
                                edge.target
                            );

                            step_events.send(WorkflowStepEvent {
                                execution_id: exec_id,
                                token_id,
                                node_id: edge.target,
                            });
                        } else {
                            execution.status = ExecutionStatus::Completed;
                            info!("🏁 Workflow Completed after task.");
                        }
                    }
                }
            }
        }
    }

    // 2. Process Token Execution
    let mut tasks_to_dispatch = Vec::new();
    let mut tokens_to_advance = Vec::new();

    for (execution_id, execution) in active_workflows.iter_mut() {
        if execution.status != ExecutionStatus::Running {
            continue;
        }

        let workflow = if let Some(w) = definitions.get(&execution.workflow_id) {
            w
        } else {
            warn!("Workflow definition not found: {:?}", execution.workflow_id);
            continue;
        };

        for token in execution.tokens.iter_mut() {
            // Check if token is already waiting for a task
            if token.data.get("waiting_for_task").is_some() {
                continue;
            }

            let node = if let Some(n) = workflow.nodes.get(&token.current_node) {
                n
            } else {
                error!("Node not found: {:?}", token.current_node);
                continue;
            };

            match &node.kind {
                NodeKind::Agent(config) => {
                    // Dispatch agent task
                    let task_id = coordinator.next_id();
                    let role_name = config.role_name.clone();

                    // Add waiting state to token *before* dispatching to avoid loops
                    token.data["waiting_for_task"] = serde_json::json!(task_id);

                    tasks_to_dispatch.push((
                        *execution_id,
                        token.id,
                        task_id,
                        role_name,
                        token.data.clone(),
                        node.id,
                    ));
                }
                NodeKind::Trigger(_) => {
                    // Move past trigger immediately if just started
                    if token.history.is_empty() {
                        tokens_to_advance.push((*execution_id, token.id, node.id));
                    }
                }
                _ => {
                    // For now, auto-advance other nodes (placeholders)
                    tokens_to_advance.push((*execution_id, token.id, node.id));
                }
            }
        }
    }

    // 3. Dispatch Tasks
    for (execution_id, token_id, task_id, role_name, token_data, _node_id) in tasks_to_dispatch {
        // Find agent by role
        let target_agent = agents
            .iter()
            .find(|(_, role, _)| role.name().eq_ignore_ascii_case(&role_name));

        if let Some((entity, _, _)) = target_agent {
            // Register map
            task_map.insert(task_id, (execution_id, token_id));

            let task_content = token_data
                .get("task")
                .and_then(|v| v.as_str())
                .unwrap_or("Perform your role based on available context.")
                .to_string();

            agent_requests.send(AgentTaskRequest {
                agent: entity,
                task_id,
                task: task_content,
                context: Some(format!("Workflow Context: {}", token_data)),
                requester: None,
                depth: 0,
            });

            info!(
                "🤖 Dispatched Workflow Task {} to Agent {:?} ({})",
                task_id, entity, role_name
            );
        } else {
            error!(
                "⚠️ Could not find agent with role '{}' for workflow task {}",
                role_name, task_id
            );
            // In a real system we'd set status to Failed
            // For now, we leave it stuck "waiting" so we can debug
        }
    }

    // 4. process advancements
    for (exec_id, token_id, current_node_id) in tokens_to_advance {
        if let Some(execution) = active_workflows.get_mut(&exec_id) {
            if let Some(workflow) = definitions.get(&execution.workflow_id) {
                if let Some(token) = execution.tokens.iter_mut().find(|t| t.id == token_id) {
                    if let Some(edge) = workflow.edges.iter().find(|e| e.source == current_node_id)
                    {
                        token.history.push(current_node_id);
                        token.current_node = edge.target;
                        info!(
                            "➡️ Token moved from {:?} to {:?}",
                            current_node_id, edge.target
                        );

                        step_events.send(WorkflowStepEvent {
                            execution_id: exec_id,
                            token_id,
                            node_id: edge.target,
                        });
                    } else {
                        info!(
                            "🏁 Workflow Reached End (No outgoing edges from {:?})",
                            current_node_id
                        );
                        execution.status = ExecutionStatus::Completed;
                    }
                }
            }
        }
    }
}

pub fn handle_workflow_events(mut events: EventReader<WorkflowStepEvent>) {
    for event in events.read() {
        info!(
            "Workflow Step: Exec {:?} Node {:?}",
            event.execution_id, event.node_id
        );
    }
}

pub fn sync_workflow_state(
    runner: Res<WorkflowRunner>,
    shared_state: Res<super::SharedWorkflowStateResource>,
) {
    if let Ok(mut state) = shared_state.0.write() {
        state.active_executions = runner.active_workflows.values().cloned().collect();
    }
}
