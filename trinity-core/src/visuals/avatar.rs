use bevy::prelude::*;

/// Marker component for the primary user avatar
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TrinityAvatar {
    pub name: String,
    pub state: AvatarState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Default)]
pub enum AvatarState {
    #[default]
    Idle,
    Thinking,
    Coding,
    Speaking,
}

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TrinityAvatar>()
            .register_type::<AvatarState>()
            .add_systems(Startup, spawn_bootstrap_avatar);
    }
}

fn spawn_bootstrap_avatar(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawns a temporary "Spirit Crystal" avatar for immediate visual feedback
    // The "Self-Coding" agent will be tasked to replace this with a full RIG later.
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::default().mesh().ico(5).unwrap()),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 1.0, 1.0),        // Cyan
                emissive: LinearRgba::new(0.0, 5.0, 5.0, 1.0), // Glowing
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.5, 0.0),
            ..default()
        },
        TrinityAvatar {
            name: "Trinity Prime".to_string(),
            state: AvatarState::Idle,
        },
        Name::new("TrinityAvatar"),
    ));

    // Add a light so we can see it
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            color: Color::WHITE,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
