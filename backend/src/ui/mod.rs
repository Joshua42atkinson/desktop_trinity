#![allow(unused)]
//! Trinity Native UI Module
//!
//! Native Bevy+egui overlay for agent monitoring and interaction.
//! Provides agent status, task queue, and chat panels.

pub mod agent_panel;
pub mod chat_panel;
pub mod task_panel;
pub mod theme;

use bevy::prelude::*;
use bevy_egui::egui::Rounding;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

pub use agent_panel::AgentPanelPlugin;
pub use chat_panel::ChatPanelPlugin;
pub use task_panel::TaskPanelPlugin;
pub use theme::TrinityTheme;

/// Global UI state
#[derive(Resource, Default)]
pub struct UiState {
    /// Show agent status panel
    pub show_agent_panel: bool,
    /// Show task queue panel
    pub show_task_panel: bool,
    /// Show chat panel
    pub show_chat_panel: bool,
    /// Show hardware monitor
    pub show_hardware_panel: bool,
    /// Dark mode enabled
    pub dark_mode: bool,
    /// Startup progress (0.0 to 1.0)
    pub startup_progress: f32,
    /// Status message
    pub status_message: String,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            show_agent_panel: true,
            show_task_panel: true,
            show_chat_panel: true,
            show_hardware_panel: false,
            dark_mode: true,
            startup_progress: 0.0,
            status_message: "Initializing...".to_string(),
        }
    }
}

/// Application startup state
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum StartupState {
    #[default]
    Loading,
    Ready,
}

/// Main Trinity UI plugin - adds all UI systems
pub struct TrinityUiPlugin;

impl Plugin for TrinityUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<UiState>()
            .init_state::<StartupState>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    splash_screen.run_if(in_state(StartupState::Loading)),
                    main_menu_bar.run_if(in_state(StartupState::Ready)),
                    toggle_panels.run_if(in_state(StartupState::Ready)),
                    update_startup_progress.run_if(in_state(StartupState::Loading)),
                ),
            );
    }
}

/// Setup UI on startup
fn setup_ui(mut ui_state: ResMut<UiState>) {
    *ui_state = UiState::new();
    log::info!("Trinity UI initialized");
}

/// Update progress (fake loading for effect)
fn update_startup_progress(
    time: Res<Time>,
    mut ui_state: ResMut<UiState>,
    mut next_state: ResMut<NextState<StartupState>>,
) {
    // Fake loading progress
    ui_state.startup_progress += time.delta_seconds() * 0.5; // 2 seconds to load

    if ui_state.startup_progress < 0.3 {
        ui_state.status_message = "Initializing Neural Core...".to_string();
    } else if ui_state.startup_progress < 0.6 {
        ui_state.status_message = "Loading Memory Systems...".to_string();
    } else if ui_state.startup_progress < 0.9 {
        ui_state.status_message = "Connecting to Dream Stream...".to_string();
    } else if ui_state.startup_progress >= 1.0 {
        ui_state.startup_progress = 1.0;
        ui_state.status_message = "Ready".to_string();
        next_state.set(StartupState::Ready);
    }
}

/// Draw the Splash Screen
fn splash_screen(mut contexts: EguiContexts, ui_state: Res<UiState>) {
    let ctx = contexts.ctx_mut();

    // Apply theme
    theme::apply_trinity_theme(ctx, ui_state.dark_mode);

    egui::CentralPanel::default().show(ctx, |ui| {
        let available_size = ui.available_size();
        let center = egui::pos2(available_size.x / 2.0, available_size.y / 2.0);

        ui.vertical_centered(|ui| {
            ui.add_space(available_size.y * 0.3);
            ui.heading(
                egui::RichText::new("TRINITY AI OS")
                    .size(48.0)
                    .color(theme::colors::PURDUE_GOLD),
            );
            ui.label(
                egui::RichText::new("Autonomous Cognitive Architecture")
                    .size(16.0)
                    .color(theme::colors::TEXT_SECONDARY),
            );

            ui.add_space(32.0);

            // Custom progress bar
            let progress_rect = egui::Rect::from_center_size(
                egui::pos2(center.x, center.y + 20.0),
                egui::vec2(300.0, 6.0),
            );

            ui.painter()
                .rect_filled(progress_rect, Rounding::same(3.0), theme::colors::DARKER_BG);

            let fill_width = 300.0 * ui_state.startup_progress;
            let fill_rect =
                egui::Rect::from_min_size(progress_rect.min, egui::vec2(fill_width, 6.0));

            ui.painter().rect_filled(
                fill_rect,
                Rounding::same(3.0),
                theme::colors::TRINITY_PURPLE,
            );

            ui.add_space(16.0);
            ui.label(
                egui::RichText::new(&ui_state.status_message).color(theme::colors::TEXT_MUTED),
            );
        });
    });
}

/// Main menu bar at the top
fn main_menu_bar(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>) {
    let ctx = contexts.ctx_mut();

    // Apply Trinity theme
    theme::apply_trinity_theme(ctx, ui_state.dark_mode);

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("🔮 Trinity", |ui| {
                if ui.button("About").clicked() {
                    // TODO: Show about dialog
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });

            ui.menu_button("View", |ui| {
                ui.checkbox(&mut ui_state.show_agent_panel, "Agent Status");
                ui.checkbox(&mut ui_state.show_task_panel, "Task Queue");
                ui.checkbox(&mut ui_state.show_chat_panel, "Chat");
                ui.checkbox(&mut ui_state.show_hardware_panel, "Hardware");
                ui.separator();
                ui.checkbox(&mut ui_state.dark_mode, "Dark Mode");
            });

            ui.menu_button("Help", |ui| {
                if ui.button("Documentation").clicked() {
                    ui.close_menu();
                }
                if ui.button("Keyboard Shortcuts").clicked() {
                    ui.close_menu();
                }
            });

            // Right-aligned status
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("🟢 Ready");
            });
        });
    });
}

/// Handle keyboard shortcuts for panel toggles
fn toggle_panels(keyboard: Res<ButtonInput<KeyCode>>, mut ui_state: ResMut<UiState>) {
    // Ctrl+1: Toggle agent panel
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::Digit1) {
        ui_state.show_agent_panel = !ui_state.show_agent_panel;
    }

    // Ctrl+2: Toggle task panel
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::Digit2) {
        ui_state.show_task_panel = !ui_state.show_task_panel;
    }

    // Ctrl+3: Toggle chat panel
    if keyboard.pressed(KeyCode::ControlLeft) && keyboard.just_pressed(KeyCode::Digit3) {
        ui_state.show_chat_panel = !ui_state.show_chat_panel;
    }
}
