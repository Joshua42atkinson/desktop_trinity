// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

use bevy::prelude::*;

use crate::AppState;

pub struct PlayPlugin;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Play), setup_play_ui)
            .add_systems(OnExit(AppState::Play), cleanup_play_ui);
    }
}

#[derive(Component)]
struct PlayRoot;

fn setup_play_ui(mut commands: Commands) {
    // Canvas Overlay
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                ..default()
            },
            PlayRoot,
        ))
        .with_children(|parent| {
            // Left Gauge (Coal)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(50.0),
                        height: Val::Percent(60.0),
                        background_color: BackgroundColor(Color::BLACK),
                        border_color: BorderColor(Color::ORANGE),
                        border_width: UiRect::all(Val::Px(2.0)),
                        align_self: AlignSelf::FlexEnd,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|gauge| {
                    gauge.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(80.0), // Mock 80% coal
                            background_color: BackgroundColor(Color::ORANGE),
                            align_self: AlignSelf::FlexEnd,
                            ..default()
                        },
                        ..default()
                    });
                });

            // Right Gauge (Steam)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(50.0),
                        height: Val::Percent(60.0),
                        background_color: BackgroundColor(Color::BLACK),
                        border_color: BorderColor(Color::CYAN),
                        border_width: UiRect::all(Val::Px(2.0)),
                        align_self: AlignSelf::FlexEnd,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|gauge| {
                    gauge.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(40.0), // Mock 40% steam
                            background_color: BackgroundColor(Color::CYAN),
                            align_self: AlignSelf::FlexEnd,
                            ..default()
                        },
                        ..default()
                    });
                });
        });
}

fn cleanup_play_ui(mut commands: Commands, query: Query<Entity, With<PlayRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
