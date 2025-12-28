// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

use bevy::prelude::*;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_avatar)
            .add_systems(Update, animate_avatar);
    }
}

#[derive(Component)]
pub struct Avatar;

fn spawn_avatar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 3D Avatar (Placeholder Cube)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb(0.1, 0.8, 0.8)), // Cyan
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        Avatar,
    ));
}

fn animate_avatar(time: Res<Time>, mut query: Query<&mut Transform, With<Avatar>>) {
    for mut transform in &mut query {
        // Bobbing animation
        transform.translation.y = 1.0 + (time.elapsed_seconds() * 2.0).sin() * 0.1;
        // Rotation
        transform.rotate_y(0.5 * time.delta_seconds());
    }
}
