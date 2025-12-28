// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! # Agent Systems - ECS Update Logic
//!
//! ## Philosophy
//! "The visualization IS the simulation. When an agent thinks, the world reacts.
//!  These systems bridge the gap between kernel events and visual feedback."
//!
//! ## Systems
//!
//! - `sync_agents_system` - Process orchestrator events → update ECS state
//! - `update_agent_visuals_system` - Sync visual state from cognitive state
//! - `animate_agents_system` - Smooth animations and particle effects

use bevy::prelude::*;

use crate::agent_components::{
    AgentCognitiveState, AgentId, AgentParticleEmitter, AgentRole, AgentStatus, AgentUiEvent,
    AgentVisualState, CurrentTask,
};

// ============================================================================
// Agent Plugin
// ============================================================================

/// Plugin for agent visualization systems
pub struct AgentVisualsPlugin;

impl Plugin for AgentVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AgentUiEvent>().add_systems(
            Update,
            (
                process_agent_events,
                update_agent_visuals,
                animate_agent_meshes,
            )
                .chain(),
        );
    }
}

// ============================================================================
// Event Processing
// ============================================================================

/// Process events from the orchestrator and update agent states
pub fn process_agent_events(
    mut events: EventReader<AgentUiEvent>,
    mut agents: Query<(&AgentId, &mut AgentCognitiveState)>,
    time: Res<Time>,
) {
    for event in events.read() {
        match event {
            AgentUiEvent::TaskStarted {
                agent_id,
                task_id,
                task_name,
            } => {
                for (id, mut state) in agents.iter_mut() {
                    if id.id == *agent_id {
                        state.status = AgentStatus::Working;
                        state.current_task = Some(CurrentTask {
                            id: *task_id,
                            name: task_name.clone(),
                            started_at: time.elapsed_seconds_f64(),
                        });
                        state.push_thought(format!("Starting: {}", task_name));
                        tracing::debug!("🎮 Agent {} started task: {}", id.name, task_name);
                    }
                }
            }

            AgentUiEvent::Thinking { agent_id, thought } => {
                for (id, mut state) in agents.iter_mut() {
                    if id.id == *agent_id {
                        state.status = AgentStatus::Thinking;
                        state.push_thought(thought.clone());
                        state.confidence = 0.7; // Moderate confidence while thinking
                    }
                }
            }

            AgentUiEvent::CodeGenerated {
                agent_id,
                file_path,
                line_count,
            } => {
                for (id, mut state) in agents.iter_mut() {
                    if id.id == *agent_id {
                        state.resource_usage = 0.8; // High resource usage during generation
                        state.push_thought(format!(
                            "Generated {} lines in {}",
                            line_count, file_path
                        ));
                    }
                }
            }

            AgentUiEvent::TaskCompleted {
                agent_id,
                task_id: _,
                duration_ms,
            } => {
                for (id, mut state) in agents.iter_mut() {
                    if id.id == *agent_id {
                        state.status = AgentStatus::Completed;
                        state.confidence = 1.0;
                        state.resource_usage = 0.0;
                        state.push_thought(format!("Completed in {}ms", duration_ms));
                        state.current_task = None;
                        tracing::debug!("🎮 Agent {} completed task", id.name);
                    }
                }
            }

            AgentUiEvent::TaskFailed {
                agent_id,
                task_id: _,
                error,
            } => {
                for (id, mut state) in agents.iter_mut() {
                    if id.id == *agent_id {
                        state.status = AgentStatus::Failed;
                        state.confidence = 0.2;
                        state.push_thought(format!("Error: {}", error));
                        state.current_task = None;
                    }
                }
            }

            AgentUiEvent::AgentIdle { agent_id } => {
                for (id, mut state) in agents.iter_mut() {
                    if id.id == *agent_id {
                        state.status = AgentStatus::Idle;
                        state.resource_usage = 0.0;
                        state.current_task = None;
                    }
                }
            }
        }
    }
}

// ============================================================================
// Visual State Sync
// ============================================================================

/// Sync visual state from cognitive state
pub fn update_agent_visuals(
    mut agents: Query<(&AgentCognitiveState, &mut AgentVisualState), Changed<AgentCognitiveState>>,
) {
    for (cognitive, mut visual) in agents.iter_mut() {
        visual.sync_from_cognitive(cognitive);
    }
}

// ============================================================================
// Animation
// ============================================================================

/// Animate agent meshes based on visual state
pub fn animate_agent_meshes(
    time: Res<Time>,
    mut agents: Query<(&mut AgentVisualState, &mut Transform), With<AgentId>>,
) {
    let dt = time.delta_seconds();

    for (mut visual, mut transform) in agents.iter_mut() {
        // Update visual state (lerp toward targets)
        visual.update(dt);

        // Apply scale
        transform.scale = Vec3::splat(visual.scale);

        // Apply subtle pulse for idle animation
        let pulse = (visual.pulse_phase.sin() * 0.5 + 0.5) * 0.05;
        transform.scale *= 1.0 + pulse;

        // Gentle rotation when working
        if visual.glow_intensity > 0.5 {
            transform.rotate_y(dt * 0.5 * visual.glow_intensity);
        }
    }
}

// ============================================================================
// Particle Emission (placeholder)
// ============================================================================

/// Update particle emitters based on visual state
pub fn update_particle_emitters(
    time: Res<Time>,
    agents: Query<(&AgentVisualState, &AgentId)>,
    mut emitters: Query<(&mut AgentParticleEmitter, &Parent)>,
) {
    let dt = time.delta_seconds();

    for (mut emitter, parent) in emitters.iter_mut() {
        // Find parent agent's visual state
        if let Ok((visual, _)) = agents.get(parent.get()) {
            emitter.rate = visual.particle_rate;
            emitter.accumulated += emitter.rate * dt;

            // Spawn particles when accumulated >= 1.0
            while emitter.accumulated >= 1.0 {
                emitter.accumulated -= 1.0;
                // TODO: Actually spawn particle entities
            }
        }
    }
}

// ============================================================================
// Spawning Helpers
// ============================================================================

/// Spawn agent entities with 3D meshes
pub fn spawn_agent_meshes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    agents: &[(&str, &str, AgentRole, Vec3)],
) {
    for (id, name, role, position) in agents {
        // Create material
        let material = materials.add(StandardMaterial {
            base_color: role.base_color(),
            emissive: role.emissive_color().into(),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            ..default()
        });

        // Create mesh based on role
        let mesh = match role {
            AgentRole::Planner => meshes.add(Sphere::new(0.4)),
            AgentRole::Coder => meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
            AgentRole::Reviewer => meshes.add(Cylinder::new(0.3, 0.6)),
            AgentRole::Researcher => meshes.add(Torus::new(0.3, 0.15)),
        };

        // Spawn entity with all components
        commands.spawn((
            crate::agent_components::AgentBundle::new(*id, *name, *role),
            PbrBundle {
                mesh,
                material,
                transform: Transform::from_translation(*position),
                ..default()
            },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        // Just verify the plugin can be instantiated
        let _plugin = AgentVisualsPlugin;
    }
}
