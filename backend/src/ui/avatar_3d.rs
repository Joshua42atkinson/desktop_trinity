//! Trinity 3D Avatar System
//!
//! Renders a 3D holographic avatar that responds to chat interactions.
//! The avatar appears in a corner of the screen and animates during conversations.

use bevy::prelude::*;
use bevy_egui::egui;
use std::time::Duration;

/// Avatar visibility and animation state
#[derive(Component, Default)]
pub struct TrinityAvatar3D {
    /// Whether the avatar is currently visible
    pub visible: bool,
    /// Current mouth openness (0.0 to 1.0)
    pub mouth_open: f32,
    /// Target mouth openness for animation
    pub mouth_target: f32,
    /// Current eye blink progress
    pub blink_progress: f32,
    /// Time until next blink
    pub blink_timer: f32,
    /// Current speaking text
    pub speaking_text: String,
    /// Characters revealed so far (for typewriter effect)
    pub chars_revealed: usize,
    /// Is currently speaking
    pub is_speaking: bool,
}

/// Marker for avatar head mesh
#[derive(Component)]
pub struct AvatarHead;

/// Marker for avatar eyes
#[derive(Component)]
pub struct AvatarEye {
    pub is_left: bool,
}

/// Marker for avatar mouth
#[derive(Component)]
pub struct AvatarMouth;

/// Avatar mood affects appearance
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AvatarMood {
    #[default]
    Neutral,
    Happy,
    Thinking,
    Excited,
}

/// Resource holding avatar configuration
#[derive(Resource)]
pub struct AvatarConfig {
    pub base_color: Color,
    pub glow_color: Color,
    pub eye_color: Color,
    pub scale: f32,
}

impl Default for AvatarConfig {
    fn default() -> Self {
        Self {
            base_color: Color::srgba(0.4, 0.2, 0.8, 0.8), // Purple hologram
            glow_color: Color::srgba(0.6, 0.3, 1.0, 0.5),
            eye_color: Color::srgba(0.0, 1.0, 1.0, 1.0), // Cyan eyes
            scale: 1.0,
        }
    }
}

/// Plugin for the 3D avatar system
pub struct Avatar3DPlugin;

impl Plugin for Avatar3DPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvatarConfig>()
            .add_systems(Startup, spawn_avatar)
            .add_systems(
                Update,
                (
                    animate_avatar,
                    update_mouth_animation,
                    update_blink_animation,
                    update_typewriter_effect,
                ),
            );
    }
}

/// Spawn the 3D avatar entities
fn spawn_avatar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<AvatarConfig>,
) {
    // Create avatar parent entity
    let avatar_entity = commands
        .spawn((
            TrinityAvatar3D::default(),
            SpatialBundle {
                transform: Transform::from_xyz(3.0, 1.0, -2.0)
                    .with_scale(Vec3::splat(config.scale)),
                visibility: Visibility::Visible,
                ..default()
            },
        ))
        .id();

    // Head - stylized sphere
    let head_mesh = meshes.add(Sphere::new(0.5));
    let head_material = materials.add(StandardMaterial {
        base_color: config.base_color,
        emissive: LinearRgba::from(config.glow_color),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let head_entity = commands
        .spawn((
            AvatarHead,
            PbrBundle {
                mesh: head_mesh,
                material: head_material,
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
        ))
        .id();

    // Left Eye
    let eye_mesh = meshes.add(Sphere::new(0.1));
    let eye_material = materials.add(StandardMaterial {
        base_color: config.eye_color,
        emissive: LinearRgba::from(config.eye_color) * 2.0,
        ..default()
    });

    let left_eye = commands
        .spawn((
            AvatarEye { is_left: true },
            PbrBundle {
                mesh: eye_mesh.clone(),
                material: eye_material.clone(),
                transform: Transform::from_xyz(-0.15, 0.1, 0.4),
                ..default()
            },
        ))
        .id();

    // Right Eye
    let right_eye = commands
        .spawn((
            AvatarEye { is_left: false },
            PbrBundle {
                mesh: eye_mesh,
                material: eye_material,
                transform: Transform::from_xyz(0.15, 0.1, 0.4),
                ..default()
            },
        ))
        .id();

    // Mouth - stretched sphere
    let mouth_mesh = meshes.add(Sphere::new(0.08));
    let mouth_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.1, 0.4, 0.9),
        ..default()
    });

    let mouth_entity = commands
        .spawn((
            AvatarMouth,
            PbrBundle {
                mesh: mouth_mesh,
                material: mouth_material,
                transform: Transform::from_xyz(0.0, -0.15, 0.4)
                    .with_scale(Vec3::new(1.5, 0.3, 0.5)),
                ..default()
            },
        ))
        .id();

    // Parent all parts to avatar
    commands
        .entity(avatar_entity)
        .push_children(&[head_entity, left_eye, right_eye, mouth_entity]);

    // Add a light for the avatar
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::srgba(0.6, 0.3, 1.0, 1.0),
            intensity: 1000.0,
            range: 10.0,
            ..default()
        },
        transform: Transform::from_xyz(3.0, 2.0, 0.0),
        ..default()
    });

    log::info!("🔮 Trinity 3D Avatar spawned");
}

/// Animate the avatar (gentle floating motion)
fn animate_avatar(time: Res<Time>, mut query: Query<&mut Transform, With<TrinityAvatar3D>>) {
    for mut transform in query.iter_mut() {
        // Gentle floating animation
        let t = time.elapsed_seconds();
        transform.translation.y = 1.0 + (t * 0.5).sin() * 0.1;
        transform.rotation = Quat::from_rotation_y((t * 0.3).sin() * 0.1);
    }
}

/// Update mouth animation based on speaking state
fn update_mouth_animation(
    time: Res<Time>,
    mut avatar_query: Query<&TrinityAvatar3D>,
    mut mouth_query: Query<&mut Transform, With<AvatarMouth>>,
) {
    if let Ok(avatar) = avatar_query.get_single() {
        if let Ok(mut mouth_transform) = mouth_query.get_single_mut() {
            // Animate mouth opening
            let target_y = if avatar.is_speaking {
                0.3 + (time.elapsed_seconds() * 15.0).sin().abs() * 0.7
            } else {
                0.3
            };

            // Smooth interpolation
            let current_y = mouth_transform.scale.y;
            mouth_transform.scale.y =
                current_y + (target_y - current_y) * 10.0 * time.delta_seconds();
        }
    }
}

/// Update eye blink animation
fn update_blink_animation(
    time: Res<Time>,
    mut avatar_query: Query<&mut TrinityAvatar3D>,
    mut eye_query: Query<&mut Transform, With<AvatarEye>>,
) {
    if let Ok(mut avatar) = avatar_query.get_single_mut() {
        avatar.blink_timer -= time.delta_seconds();

        if avatar.blink_timer <= 0.0 {
            // Trigger blink
            avatar.blink_progress = 1.0;
            avatar.blink_timer = 3.0 + rand::random::<f32>() * 2.0; // Random 3-5 seconds
        }

        // Animate blink
        if avatar.blink_progress > 0.0 {
            avatar.blink_progress -= time.delta_seconds() * 8.0;

            for mut eye_transform in eye_query.iter_mut() {
                let blink_scale = if avatar.blink_progress > 0.5 {
                    1.0 - (avatar.blink_progress - 0.5) * 2.0
                } else {
                    avatar.blink_progress * 2.0
                };
                eye_transform.scale.y = 1.0 - blink_scale.clamp(0.0, 1.0) * 0.9;
            }
        }
    }
}

/// Update typewriter effect for speaking text
fn update_typewriter_effect(time: Res<Time>, mut avatar_query: Query<&mut TrinityAvatar3D>) {
    if let Ok(mut avatar) = avatar_query.get_single_mut() {
        if avatar.is_speaking && avatar.chars_revealed < avatar.speaking_text.len() {
            // Reveal ~30 characters per second
            avatar.chars_revealed = (avatar.chars_revealed + 1).min(avatar.speaking_text.len());
        }
    }
}

/// Public API: Make the avatar speak
pub fn avatar_speak(avatar: &mut TrinityAvatar3D, text: String) {
    avatar.speaking_text = text;
    avatar.chars_revealed = 0;
    avatar.is_speaking = true;
}

/// Public API: Stop avatar speaking
pub fn avatar_stop_speaking(avatar: &mut TrinityAvatar3D) {
    avatar.is_speaking = false;
    avatar.chars_revealed = avatar.speaking_text.len();
}
