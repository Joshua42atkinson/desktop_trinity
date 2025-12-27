use super::avatar::TrinityAvatar;
use bevy::prelude::*;

pub struct AvatarMovementPlugin;

impl Plugin for AvatarMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (hover_animation, rotate_avatar));
    }
}

/// Makes the avatar hover up and down slightly to look "alive"
fn hover_animation(time: Res<Time>, mut query: Query<(&mut Transform, &TrinityAvatar)>) {
    for (mut transform, _avatar) in query.iter_mut() {
        // Simple sine wave bobbing
        let bob_offset = (time.elapsed_seconds() * 2.0).sin() * 0.005;
        transform.translation.y += bob_offset;
    }
}

/// Slowly rotates the avatar to show off the 3D model
fn rotate_avatar(time: Res<Time>, mut query: Query<(&mut Transform, &TrinityAvatar)>) {
    for (mut transform, _) in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_y(0.5 * time.delta_seconds());
    }
}
