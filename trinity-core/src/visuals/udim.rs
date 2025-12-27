use bevy::prelude::*;

pub struct UdimPlugin;

impl Plugin for UdimPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_udim_support);
    }
}

fn setup_udim_support(_commands: Commands) {
    // Placeholder for setting up texture arrays/materials for UDIM
    tracing::info!("Initializing UDIM texture support for 8K assets");
}
