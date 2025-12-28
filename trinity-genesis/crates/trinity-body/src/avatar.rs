// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Avatar System for Trinity Body
//!
//! Animated 3D avatars representing AI agents with particle effects
//! and smooth state transitions.

use bevy::prelude::*;

/// Avatar plugin for Trinity Body
pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_avatars).add_systems(
            Update,
            (
                animate_avatar,
                animate_orbital_rings,
                animate_particles,
                update_avatar_materials,
            ),
        );
    }
}

/// Avatar state matching the protocol definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub enum AvatarState {
    #[default]
    Idle,
    Thinking,
    Coding,
    Speaking,
    Sleeping,
}

/// Avatar persona/role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Component)]
pub enum AvatarPersona {
    #[default]
    Trinity, // Main orchestrator - cyan crystal
    Coder,      // Developer - green polyhedron
    Writer,     // Content creator - gold ribbon
    Artist,     // Image gen - rainbow orb
    Researcher, // Information gatherer - blue glass
}

impl AvatarPersona {
    pub fn base_color(&self) -> Color {
        match self {
            AvatarPersona::Trinity => Color::srgb(0.2, 0.8, 1.0), // Cyan
            AvatarPersona::Coder => Color::srgb(0.2, 1.0, 0.4),   // Green
            AvatarPersona::Writer => Color::srgb(1.0, 0.85, 0.3), // Gold
            AvatarPersona::Artist => Color::srgb(0.9, 0.4, 0.9),  // Magenta
            AvatarPersona::Researcher => Color::srgb(0.3, 0.5, 1.0), // Blue
        }
    }

    pub fn emissive_color(&self) -> Color {
        match self {
            AvatarPersona::Trinity => Color::srgb(0.0, 0.5, 0.8),
            AvatarPersona::Coder => Color::srgb(0.0, 0.6, 0.2),
            AvatarPersona::Writer => Color::srgb(0.6, 0.5, 0.1),
            AvatarPersona::Artist => Color::srgb(0.5, 0.2, 0.5),
            AvatarPersona::Researcher => Color::srgb(0.1, 0.2, 0.6),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AvatarPersona::Trinity => "Trinity Prime",
            AvatarPersona::Coder => "Coder",
            AvatarPersona::Writer => "Writer",
            AvatarPersona::Artist => "Artist",
            AvatarPersona::Researcher => "Researcher",
        }
    }
}

/// Marker component for the main Trinity avatar
#[derive(Component)]
pub struct TrinityAvatar;

/// Marker for orbital ring decoration
#[derive(Component)]
pub struct OrbitalRing {
    pub radius: f32,
    pub speed: f32,
    pub offset: f32,
}

/// Marker for particle effects
#[derive(Component)]
pub struct AvatarParticle {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub velocity: Vec3,
}

/// Animation timer for avatar effects
#[derive(Component)]
pub struct AvatarAnimation {
    pub time: f32,
    pub base_y: f32,
    pub state_transition: f32, // 0.0 to 1.0 for smooth transitions
    pub previous_state: AvatarState,
}

/// Handle to the avatar's material for dynamic updates
#[derive(Component)]
pub struct AvatarMaterialHandle(pub Handle<StandardMaterial>);

/// Spawn all avatars
fn spawn_avatars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_trinity_avatar(&mut commands, &mut meshes, &mut materials);
}

/// Spawn the main Trinity avatar (Spirit Crystal)
fn spawn_trinity_avatar(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let persona = AvatarPersona::Trinity;

    // Main crystal material
    let main_material = materials.add(StandardMaterial {
        base_color: persona.base_color(),
        emissive: persona.emissive_color().into(),
        metallic: 0.9,
        perceptual_roughness: 0.1,
        ..default()
    });

    // Core glow material
    let core_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::srgb(1.0, 1.0, 1.0).into(),
        ..default()
    });

    // Ring material
    let ring_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.3, 0.8, 1.0, 0.5),
        emissive: Color::srgb(0.1, 0.4, 0.6).into(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let base_y = 1.5;

    // Main Spirit Crystal
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.5).mesh().ico(4).unwrap()),
            material: main_material.clone(),
            transform: Transform::from_xyz(0.0, base_y, 0.0),
            ..default()
        },
        TrinityAvatar,
        persona,
        AvatarState::Idle,
        AvatarAnimation {
            time: 0.0,
            base_y,
            state_transition: 1.0,
            previous_state: AvatarState::Idle,
        },
        AvatarMaterialHandle(main_material),
    ));

    // Inner glow core
    commands.spawn(PbrBundle {
        mesh: meshes.add(Sphere::new(0.2).mesh().ico(2).unwrap()),
        material: core_material,
        transform: Transform::from_xyz(0.0, base_y, 0.0),
        ..default()
    });

    // Orbital rings
    for (i, (radius, speed)) in [(0.7, 1.0), (0.9, -0.7), (1.1, 0.5)].iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Torus::new(0.02, *radius)),
                material: ring_material.clone(),
                transform: Transform::from_xyz(0.0, base_y, 0.0).with_rotation(Quat::from_euler(
                    EulerRot::XYZ,
                    0.3 * i as f32,
                    0.0,
                    0.5 * i as f32,
                )),
                ..default()
            },
            OrbitalRing {
                radius: *radius,
                speed: *speed,
                offset: i as f32 * 2.0,
            },
        ));
    }

    tracing::info!("Spawned Trinity avatar with orbital rings");
}

/// Animate the avatar based on state and time
fn animate_avatar(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &mut AvatarAnimation,
            &AvatarState,
            &AvatarPersona,
        ),
        With<TrinityAvatar>,
    >,
) {
    for (mut transform, mut anim, state, _persona) in query.iter_mut() {
        anim.time += time.delta_seconds();

        // Handle state transitions
        if *state != anim.previous_state {
            anim.state_transition = 0.0;
            anim.previous_state = *state;
        }
        anim.state_transition = (anim.state_transition + time.delta_seconds() * 2.0).min(1.0);

        // Smooth easing
        let t = ease_out_cubic(anim.state_transition);

        match state {
            AvatarState::Idle => {
                // Gentle floating and slow rotation
                let y_offset = (anim.time * 0.5).sin() * 0.1 * t;
                transform.translation.y = anim.base_y + y_offset;
                transform.translation.x = lerp(transform.translation.x, 0.0, 0.1);
                transform.rotate_y(time.delta_seconds() * 0.2);
                transform.scale = Vec3::lerp(transform.scale, Vec3::ONE, 0.1);
            }
            AvatarState::Thinking => {
                // Faster rotation, pulsing scale, slight elevation
                let scale_pulse = 1.0 + (anim.time * 3.0).sin() * 0.08 * t;
                let y_offset = 0.2 * t;
                transform.translation.y = anim.base_y + y_offset;
                transform.scale = Vec3::splat(scale_pulse);
                transform.rotate_y(time.delta_seconds() * 1.5);
            }
            AvatarState::Coding => {
                // Rapid vibration effect + fast rotation
                let x_offset = (anim.time * 25.0).sin() * 0.015 * t;
                let y_offset = (anim.time * 30.0).cos() * 0.01 * t;
                transform.translation.x = x_offset;
                transform.translation.y = anim.base_y + y_offset;
                transform.rotate_y(time.delta_seconds() * 3.0);
                transform.scale = Vec3::lerp(transform.scale, Vec3::splat(1.1), 0.1);
            }
            AvatarState::Speaking => {
                // Bouncing motion synced to "speech"
                let bounce = (anim.time * 6.0).sin().abs() * 0.15 * t;
                transform.translation.y = anim.base_y + bounce;
                transform.translation.x = lerp(transform.translation.x, 0.0, 0.1);
                transform.rotate_y(time.delta_seconds() * 0.5);

                // Breathing scale
                let scale = 1.0 + (anim.time * 4.0).sin() * 0.05 * t;
                transform.scale = Vec3::splat(scale);
            }
            AvatarState::Sleeping => {
                // Very slow breathing, dim
                let scale = 0.9 + (anim.time * 0.3).sin() * 0.03 * t;
                transform.translation.y = anim.base_y - 0.1 * t;
                transform.scale = Vec3::splat(scale);
                transform.rotate_y(time.delta_seconds() * 0.05);
            }
        }
    }
}

/// Animate orbital rings
fn animate_orbital_rings(
    time: Res<Time>,
    avatars: Query<(&Transform, &AvatarState), With<TrinityAvatar>>,
    mut rings: Query<(&mut Transform, &OrbitalRing), Without<TrinityAvatar>>,
) {
    // Get avatar state for ring behavior
    let (avatar_tf, avatar_state) = match avatars.get_single() {
        Ok(a) => a,
        Err(_) => return,
    };

    let speed_multiplier = match avatar_state {
        AvatarState::Idle => 1.0,
        AvatarState::Thinking => 2.5,
        AvatarState::Coding => 4.0,
        AvatarState::Speaking => 1.5,
        AvatarState::Sleeping => 0.2,
    };

    for (mut transform, ring) in rings.iter_mut() {
        // Follow avatar Y position
        transform.translation.y = avatar_tf.translation.y;

        // Rotate based on state
        let rotation_speed = ring.speed * speed_multiplier * time.delta_seconds();
        transform.rotate_y(rotation_speed);
        transform.rotate_x(rotation_speed * 0.3);
    }
}

/// Animate floating particles (placeholder - would spawn/despawn)
fn animate_particles(time: Res<Time>, mut particles: Query<(&mut Transform, &mut AvatarParticle)>) {
    for (mut transform, mut particle) in particles.iter_mut() {
        particle.lifetime += time.delta_seconds();

        // Move particle
        transform.translation += particle.velocity * time.delta_seconds();

        // Fade out (would need material access)
        let alpha = 1.0 - (particle.lifetime / particle.max_lifetime);
        transform.scale = Vec3::splat(alpha.max(0.0));
    }
}

/// Update avatar materials based on state
fn update_avatar_materials(
    avatars: Query<(&AvatarState, &AvatarPersona, &AvatarMaterialHandle), Changed<AvatarState>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (state, persona, handle) in avatars.iter() {
        if let Some(material) = materials.get_mut(&handle.0) {
            // Adjust emissive intensity based on state
            let intensity = match state {
                AvatarState::Idle => 1.0,
                AvatarState::Thinking => 2.0,
                AvatarState::Coding => 2.5,
                AvatarState::Speaking => 1.8,
                AvatarState::Sleeping => 0.3,
            };

            let base_emissive = persona.emissive_color();
            material.emissive = Color::srgb(
                base_emissive.to_srgba().red * intensity,
                base_emissive.to_srgba().green * intensity,
                base_emissive.to_srgba().blue * intensity,
            )
            .into();
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
