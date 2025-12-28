use bevy::prelude::*;

pub struct PlayPlugin;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, play_ui);
    }
}

fn play_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // Only show play UI when in Play state (TODO)
) {
    // Placeholder UI
    // commands.spawn(NodeBundle {
    //     style: Style {
    //         width: Val::Percent(100.0),
    //         height: Val::Percent(100.0),
    //         justify_content: JustifyContent::Center,
    //         align_items: AlignItems::Center,
    //         ..default()
    //     },
    //     ..default()
    // }).with_children(|parent| {
    //     parent.spawn(TextBundle::from_section(
    //         "Play Mode",
    //         TextStyle {
    //             font_size: 40.0,
    //             color: Color::WHITE,
    //             ..default()
    //         },
    //     ));
    // });
}
