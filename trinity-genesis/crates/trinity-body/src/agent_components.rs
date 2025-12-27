//! # Agent Components - ECS Visualization Layer
//!
//! ## Philosophy
//! "Each agent is a living entity in the Trinity world—not a background service,
//!  but a teammate with visible state, theory of mind, and visual feedback.
//!  The UI is not a wrapper around the OS; the UI IS the simulation."
//!
//! ## Architecture
//!
//! ```text
//!    ┌─────────────────────────────────────────────────────────────────┐
//!    │                    Bevy ECS World                               │
//!    │                                                                 │
//!    │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
//!    │   │  Entity #1  │  │  Entity #2  │  │  Entity #3  │            │
//!    │   │   Joshua    │  │   Jessica   │  │    Jules    │            │
//!    │   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘            │
//!    │          │                │                │                    │
//!    │   ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐            │
//!    │   │ AgentState  │  │ AgentState  │  │ AgentState  │            │
//!    │   │ AgentVisual │  │ AgentVisual │  │ AgentVisual │            │
//!    │   │ Transform   │  │ Transform   │  │ Transform   │            │
//!    │   └─────────────┘  └─────────────┘  └─────────────┘            │
//!    └─────────────────────────────────────────────────────────────────┘
//! ```

use bevy::prelude::*;
use std::collections::VecDeque;
use uuid::Uuid;

// ============================================================================
// Agent Identity
// ============================================================================

/// Unique identifier linking ECS entity to orchestrator agent
#[derive(Component, Debug, Clone)]
pub struct AgentId {
    /// Matches agent_id in AgentEvent
    pub id: String,
    /// Human-readable name
    pub name: String,
}

/// Agent specialization (mirrors kernel definition)
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentRole {
    /// Strategic planning (Joshua)
    Planner,
    /// Code generation (Jessica)
    #[default]
    Coder,
    /// Code review (Jules)
    Reviewer,
    /// Research (Janet)
    Researcher,
}

impl AgentRole {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentRole::Planner => "Joshua",
            AgentRole::Coder => "Jessica",
            AgentRole::Reviewer => "Jules",
            AgentRole::Researcher => "Janet",
        }
    }

    /// Get base color for this role
    pub fn base_color(&self) -> Color {
        match self {
            AgentRole::Planner => Color::srgb(0.6, 0.4, 1.0), // Purple
            AgentRole::Coder => Color::srgb(0.2, 1.0, 0.6),   // Green
            AgentRole::Reviewer => Color::srgb(1.0, 0.8, 0.2), // Gold
            AgentRole::Researcher => Color::srgb(0.3, 0.7, 1.0), // Blue
        }
    }

    /// Get emissive glow color
    pub fn emissive_color(&self) -> Color {
        match self {
            AgentRole::Planner => Color::srgb(0.8, 0.5, 1.5),
            AgentRole::Coder => Color::srgb(0.3, 1.5, 0.8),
            AgentRole::Reviewer => Color::srgb(1.5, 1.0, 0.3),
            AgentRole::Researcher => Color::srgb(0.4, 0.9, 1.5),
        }
    }
}

// ============================================================================
// Agent State (Cognitive)
// ============================================================================

/// Current cognitive state of the agent
#[derive(Component, Debug, Clone, Default)]
pub struct AgentCognitiveState {
    /// Current activity status
    pub status: AgentStatus,
    /// Current task being worked on
    pub current_task: Option<CurrentTask>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
    /// Resource utilization (0.0 - 1.0)
    pub resource_usage: f32,
    /// Recent thoughts (for UI display)
    pub thought_history: VecDeque<String>,
}

/// Agent activity status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentStatus {
    #[default]
    Idle,
    Thinking,
    Working,
    Completed,
    Failed,
}

/// Currently active task
#[derive(Debug, Clone)]
pub struct CurrentTask {
    pub id: Uuid,
    pub name: String,
    pub started_at: f64, // Time since startup
}

impl AgentCognitiveState {
    /// Push a thought to history (keeps last 5)
    pub fn push_thought(&mut self, thought: impl Into<String>) {
        self.thought_history.push_back(thought.into());
        while self.thought_history.len() > 5 {
            self.thought_history.pop_front();
        }
    }

    /// Get latest thought
    pub fn latest_thought(&self) -> Option<&String> {
        self.thought_history.back()
    }
}

// ============================================================================
// Agent Visual State
// ============================================================================

/// Visual representation state for rendering
#[derive(Component, Debug, Clone)]
pub struct AgentVisualState {
    /// Base color (from role)
    pub base_color: Color,
    /// Current glow intensity (0.0 - 2.0)
    pub glow_intensity: f32,
    /// Target glow intensity (for smooth transitions)
    pub target_glow: f32,
    /// Pulse phase (for idle animation)
    pub pulse_phase: f32,
    /// Scale multiplier (for emphasis)
    pub scale: f32,
    /// Target scale (for smooth transitions)
    pub target_scale: f32,
    /// Particle emission rate
    pub particle_rate: f32,
}

impl Default for AgentVisualState {
    fn default() -> Self {
        Self {
            base_color: Color::WHITE,
            glow_intensity: 0.3,
            target_glow: 0.3,
            pulse_phase: 0.0,
            scale: 1.0,
            target_scale: 1.0,
            particle_rate: 0.0,
        }
    }
}

impl AgentVisualState {
    /// Create visual state from agent role
    pub fn from_role(role: AgentRole) -> Self {
        Self {
            base_color: role.base_color(),
            ..Default::default()
        }
    }

    /// Update visual based on cognitive state
    pub fn sync_from_cognitive(&mut self, state: &AgentCognitiveState) {
        match state.status {
            AgentStatus::Idle => {
                self.target_glow = 0.3;
                self.target_scale = 1.0;
                self.particle_rate = 0.0;
            }
            AgentStatus::Thinking => {
                self.target_glow = 0.8;
                self.target_scale = 1.1;
                self.particle_rate = 2.0;
            }
            AgentStatus::Working => {
                self.target_glow = 1.2 + state.resource_usage * 0.8;
                self.target_scale = 1.15;
                self.particle_rate = 5.0 + state.resource_usage * 10.0;
            }
            AgentStatus::Completed => {
                self.target_glow = 1.5;
                self.target_scale = 1.2;
                self.particle_rate = 15.0;
            }
            AgentStatus::Failed => {
                self.target_glow = 0.5;
                self.target_scale = 0.9;
                self.particle_rate = 1.0;
            }
        }
    }

    /// Smooth lerp toward target values
    pub fn update(&mut self, dt: f32) {
        let lerp_speed = 3.0;
        self.glow_intensity += (self.target_glow - self.glow_intensity) * lerp_speed * dt;
        self.scale += (self.target_scale - self.scale) * lerp_speed * dt;
        self.pulse_phase += dt * 2.0;
    }
}

// ============================================================================
// Bundle for spawning agents
// ============================================================================

/// Complete bundle for spawning an agent entity
#[derive(Bundle)]
pub struct AgentBundle {
    pub id: AgentId,
    pub role: AgentRole,
    pub cognitive: AgentCognitiveState,
    pub visual: AgentVisualState,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl AgentBundle {
    /// Create a new agent bundle
    pub fn new(id: impl Into<String>, name: impl Into<String>, role: AgentRole) -> Self {
        Self {
            id: AgentId {
                id: id.into(),
                name: name.into(),
            },
            role,
            cognitive: AgentCognitiveState::default(),
            visual: AgentVisualState::from_role(role),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }

    /// Create Joshua (Planner)
    pub fn joshua() -> Self {
        Self::new("joshua-planner", "Joshua", AgentRole::Planner)
    }

    /// Create Jessica (Coder)
    pub fn jessica() -> Self {
        Self::new("jessica-coder", "Jessica", AgentRole::Coder)
    }

    /// Create Jules (Reviewer)
    pub fn jules() -> Self {
        Self::new("jules-reviewer", "Jules", AgentRole::Reviewer)
    }

    /// Create Janet (Researcher)
    pub fn janet() -> Self {
        Self::new("janet-researcher", "Janet", AgentRole::Researcher)
    }
}

// ============================================================================
// Agent Events (from orchestrator)
// ============================================================================

/// Events received from the Brain's orchestrator
#[derive(Event, Debug, Clone)]
pub enum AgentUiEvent {
    /// Agent started a task
    TaskStarted {
        agent_id: String,
        task_id: Uuid,
        task_name: String,
    },
    /// Agent is thinking
    Thinking { agent_id: String, thought: String },
    /// Agent generated code
    CodeGenerated {
        agent_id: String,
        file_path: String,
        line_count: usize,
    },
    /// Task completed
    TaskCompleted {
        agent_id: String,
        task_id: Uuid,
        duration_ms: u64,
    },
    /// Task failed
    TaskFailed {
        agent_id: String,
        task_id: Uuid,
        error: String,
    },
    /// Agent became idle
    AgentIdle { agent_id: String },
}

// ============================================================================
// Marker Components
// ============================================================================

/// Marker for the agent panel UI
#[derive(Component)]
pub struct AgentPanelMarker;

/// Marker for agent 3D mesh
#[derive(Component)]
pub struct AgentMesh;

/// Marker for agent particle emitter
#[derive(Component)]
pub struct AgentParticleEmitter {
    pub rate: f32,
    pub accumulated: f32,
}

impl Default for AgentParticleEmitter {
    fn default() -> Self {
        Self {
            rate: 0.0,
            accumulated: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_bundle_creation() {
        let joshua = AgentBundle::joshua();
        assert_eq!(joshua.id.name, "Joshua");
        assert_eq!(joshua.role, AgentRole::Planner);
    }

    #[test]
    fn test_visual_sync() {
        let mut visual = AgentVisualState::default();
        let mut cognitive = AgentCognitiveState::default();

        cognitive.status = AgentStatus::Working;
        cognitive.resource_usage = 0.5;
        visual.sync_from_cognitive(&cognitive);

        assert!(visual.target_glow > 1.0);
        assert!(visual.particle_rate > 5.0);
    }

    #[test]
    fn test_thought_history() {
        let mut state = AgentCognitiveState::default();
        for i in 0..10 {
            state.push_thought(format!("Thought {}", i));
        }
        assert_eq!(state.thought_history.len(), 5);
        assert_eq!(state.latest_thought(), Some(&"Thought 9".to_string()));
    }
}
