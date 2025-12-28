use crate::AppState;
use bevy::prelude::*;

pub struct AuthoringPlugin;

impl Plugin for AuthoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Authoring), setup_authoring_ui)
            .add_systems(OnExit(AppState::Authoring), cleanup_authoring_ui);
    }
}

#[derive(Component)]
struct AuthoringRoot;

fn setup_authoring_ui(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.2, 0.0, 0.0, 0.5)),
                ..default()
            },
            AuthoringRoot,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "AUTHORING MODE // THE TRAIN YARD",
                TextStyle {
                    font_size: 40.0,
                    color: Color::srgb(1.0, 0.84, 0.0), // GOLD
                    ..default()
                },
            ));
            parent.spawn(TextBundle::from_section(
                "[Drag and Drop Nodes Here]",
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgb(0.5, 0.5, 0.5), // GRAY
                    ..default()
                },
            ));
        });
}

fn cleanup_authoring_ui(mut commands: Commands, query: Query<Entity, With<AuthoringRoot>>) {
    // Correct way to despawn recursively
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
