pub mod avatar;
pub mod hair;
pub mod movement;
pub mod procedural;
pub mod udim;

// Re-export specific items if needed
pub use procedural::ProceduralAvatarPlugin;

use bevy::prelude::*;

/// Plugin to initialize all high-fidelity visual systems
pub struct TrinityVisualsPlugin;

impl Plugin for TrinityVisualsPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "desktop")]
        {
            _app.add_plugins(udim::UdimPlugin)
                .add_plugins(hair::HairPlugin)
                .add_plugins(avatar::AvatarPlugin)
                .add_plugins(movement::AvatarMovementPlugin);
        }
    }
}
