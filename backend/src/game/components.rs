#![allow(unused)]
use bevy::prelude::Resource;
use bevy::prelude::*;
use std::sync::{Arc, RwLock};

// Define wrapper resources for Bevy
#[derive(Resource)]
pub struct SharedResearchLogResource(pub Arc<RwLock<ResearchLog>>);

#[derive(Resource)]
pub struct SharedVirtuesResource(pub Arc<RwLock<VirtueTopology>>);

use serde::{Deserialize, Serialize};

// --- Core Identity Components ---

// The Archetype Component
#[derive(
    Component, Reflect, Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize,
)]
#[reflect(Component)]
pub enum Archetype {
    #[default]
    Novice,
    Sage,
    Hero,
    Jester,
    // Add more as needed
}

// Persona: The mask the player wears
#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Persona {
    pub archetype: Archetype,
    pub shadow_trait: String, // e.g., "Arrogance" for Sage, "Cowardice" for Hero
    pub projective_dissonance: f32, // 0.0 to 1.0, how much the persona conflicts with the self
}

// --- Psychological State Components ---

// VirtueTopology: Tracks ethical alignment based on Self-Determination Theory
#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct VirtueTopology {
    // Core Pedagogical Values
    pub self_efficacy: f32,   // Willingness to make a decision (Agency)
    pub self_esteem: f32,     // Worthiness to receive experience (Resilience)
    pub interdependence: f32, // Understanding of connection (Relatedness)

    // SDT & Traditional Virtues (Secondary)
    pub autonomy: f32,
    pub competence: f32,
    pub relatedness: f32,
    pub honesty: f32,
    pub compassion: f32,
    pub valor: f32,
    pub justice: f32,
    pub sacrifice: f32,
    pub honor: f32,
    pub spirituality: f32,
    pub humility: f32,
}

// CognitiveLoad: Tracks mental effort (Sweller's Cognitive Load Theory)
#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct CognitiveLoad {
    pub intrinsic: f32,  // Inherent difficulty of the task
    pub extraneous: f32, // Unnecessary mental effort (bad UI, distractions)
    pub germane: f32,    // Effort dedicated to processing and learning (the "good" load)
}

// --- Narrative Components ---

// StoryNode: Represents a point in the narrative graph
#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct StoryNode {
    pub id: String,
    pub content: String,
    pub choices: Vec<String>, // IDs of connected nodes
    pub visited: bool,
}

// StoryProgress: Tracks the player's position in the narrative
#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct StoryProgress {
    pub current_quest_id: Option<String>,
    pub current_step_id: Option<String>,
    pub current_step_description: String,
    pub history: Vec<String>, // List of visited node IDs
    pub inventory: Vec<String>,
    pub quest_flags: std::collections::HashMap<String, bool>,
    pub learned_vocab: std::collections::HashSet<String>,
}

// --- Legacy / LitRPG Components (Kept for compatibility) ---

#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Level(pub u32);

#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Experience(pub u32);

#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ResearchLog {
    pub events: Vec<ResearchEvent>,
}

#[derive(Reflect, Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchEvent {
    pub timestamp: f64,     // Seconds since start
    pub event_type: String, // e.g., "DECISION", "VIRTUE_UPDATE"
    pub data: String,       // JSON payload
}

// The Bundle used to spawn a new student entity
#[derive(Bundle)]
pub struct StudentBundle {
    pub persona: Persona,
    pub virtue_topology: VirtueTopology,
    pub cognitive_load: CognitiveLoad,
    pub story_progress: StoryProgress,
    pub research_log: ResearchLog,
    pub name: Name,
    pub level: Level,
    pub xp: Experience,
}
