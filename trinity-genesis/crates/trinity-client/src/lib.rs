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
}

mod game;
mod ui;

use game::avatar::AvatarPlugin;
use ui::authoring::AuthoringPlugin;
use ui::play::PlayPlugin;

pub fn run() {
    App::new()
        // Networking & Assets
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Trinity Genesis".to_string(),
                    canvas: Some("canvas".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            WebAssetPlugin::default(),
        ))
        // State
        .init_state::<AppState>()
        // Core Plugins
        .add_plugins((AuthoringPlugin, PlayPlugin, AvatarPlugin))
        // Systems
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, setup_main_menu)
        .run();
}

fn setup_main_menu(mut commands: Commands) {
    // Simple key press to switch modes for now
    commands.spawn(TextBundle::from_section(
        "Press 'A' for Authoring, 'P' for Play",
        TextStyle {
            font_size: 30.0,
            color: Color::WHITE,
            ..default()
        },
    ));
}

/// 3D Scene Setup (The "Cockpit")
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 1.5, 4.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ..default()
    });

    // Lights
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            color: Color::CYAN,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Ambient Light (Sci-Fi Blue)
    commands.insert_resource(AmbientLight {
        color: Color::BLUE,
        brightness: 0.2,
    });

    // Placeholder Avatar (Rotating Cube)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb(0.1, 0.1, 0.8)),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        Rotator,
    ));
}
