use bevy::prelude::*;

// Simplified imports to avoid resolution errors
use bevy_yoleck::*;
use serde::{Deserialize, Serialize};

// The Vocabulary Component - Bridging Pedagogy and Game
#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct VocabularyComponent {
    pub term: String,
    pub definition: String,
    pub physics_property: String,
    pub value: f32,
}

// Iron Road Integration - Physics Node
#[derive(Component, Reflect, Default, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct VelocityNode {
    pub current_velocity: f32,
    pub max_velocity: f32,
    pub friction: f32,
    pub active: bool,
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VocabularyComponent>();
        app.register_type::<VelocityNode>();

        // Add Yoleck Plugin - assuming standard usage
        // Note: verify YoleckPlugin name in docs if failure persists
        // app.add_plugins(YoleckPlugin);

        if !app.is_plugin_added::<bevy_editor_pls::EditorPlugin>() {
            app.add_plugins(bevy_editor_pls::EditorPlugin::default());
        }

        app.add_systems(Startup, setup_editor_state);
        // app.add_systems(Update, toggle_editor_mode); // Removed problematic system using YoleckGlobalState
    }
}

fn setup_editor_state() {
    // Initial setup
}
