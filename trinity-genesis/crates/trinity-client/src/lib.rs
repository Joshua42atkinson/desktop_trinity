// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

use bevy::prelude::*;
use bevy_web_asset::WebAssetPlugin;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Dashboard,
    Oracle,
    Studio,
    Authoring, // Added Authoring
    Play,
}

mod bridge;
mod editor;
mod game;
mod ui;

use editor::EditorPlugin;
use game::avatar::AvatarPlugin;
use game::vaam::VaaMPlugin;

use game::vaam::{CognitiveWeight, SemanticSocket, VocabularyItem};
use game::weigh_station::WeighStationPlugin; // Added plugin import
use iron_road_physics::VocabularyTier;
use ui::authoring::AuthoringPlugin;
use ui::glass::GlassUiPlugin;
use ui::play::PlayPlugin;

pub fn run() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Trinity Genesis".to_string(),
                    canvas: Some("#canvas".to_string()),
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            WebAssetPlugin::default(),
        ))
        .init_state::<AppState>()
        .add_plugins((
            AuthoringPlugin,
            PlayPlugin,
            AvatarPlugin,
            EditorPlugin,
            VaaMPlugin,         // Physics of Language
            GlassUiPlugin,      // Body UI
            WeighStationPlugin, // Brain UI/Link
        ))
        .insert_resource(bridge::BrainConnection {
            connected: false,
            brain_addr: "127.0.0.1:9000".to_string(),
            model_info: None,
            request_tx: crossbeam_channel::unbounded().0,
            response_rx: crossbeam_channel::unbounded().1,
        })
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, setup_main_menu)
        .add_systems(Startup, setup_debug_vocab)
        .run();
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn(TextBundle::from_section(
        "Trinity Genesis Client (Unified)\nPress 'A' for Authoring, 'P' for Play\nEditor Mode Active",
        TextStyle {
            font_size: 30.0,
            color: Color::WHITE,
            ..default()
        },
    ));
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            color: Color::srgb(0.0, 1.0, 1.0),
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.0, 0.0, 1.0),
        brightness: 0.2,
    });

    commands.spawn((PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgba(0.1, 0.1, 0.8, 1.0)),
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        ..default()
    },));
}

fn setup_debug_vocab(mut commands: Commands) {
    // Tier 1: Basic Word
    commands.spawn((
        VocabularyItem {
            word: "Apple".to_string(),
            definition: "A red or green fruit that keeps doctors away.".to_string(),
            tier: VocabularyTier::Basic,
            tags: vec!["Food".to_string(), "Nature".to_string()],
        },
        CognitiveWeight {
            base_mass: 5.0,
            effective_mass: 5.0,
        },
    ));

    // Tier 2: Academic Word
    commands.spawn((
        VocabularyItem {
            word: "Photosynthesis".to_string(),
            definition: "Process by which plants use sunlight to synthesize foods.".to_string(),
            tier: VocabularyTier::Academic,
            tags: vec!["Biology".to_string(), "Process".to_string()],
        },
        CognitiveWeight {
            base_mass: 35.0,
            effective_mass: 35.0,
        },
    ));

    // Tier 3: Hazardous Word
    commands.spawn((
        VocabularyItem {
            word: "Eigenvector".to_string(),
            definition: "A non-zero vector that changes by a scalar factor.".to_string(),
            tier: VocabularyTier::Hazardous,
            tags: vec![
                "Math".to_string(),
                "Linear Algebra".to_string(),
                "Scary".to_string(),
            ],
        },
        CognitiveWeight {
            base_mass: 85.0,
            effective_mass: 85.0,
        },
    ));

    // Debug Sockets
    commands.spawn(SemanticSocket {
        required_tags: vec!["Food".to_string()],
        difficulty_class: 5,
        is_solved: false,
    });

    commands.spawn(SemanticSocket {
        required_tags: vec!["Math".to_string()],
        difficulty_class: 15,
        is_solved: false,
    });
}
