//! Task Queue Panel
//!
//! Displays task queue with priority, progress, and management controls

use crate::ui::{theme, UiState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Plugin for task queue panel
pub struct TaskPanelPlugin;

impl Plugin for TaskPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TaskPanelState>()
            .add_systems(Update, task_panel_system);
    }
}

/// Panel state
#[derive(Resource, Default)]
pub struct TaskPanelState {
    pub new_task_name: String,
    pub new_task_prompt: String,
}

/// Task display info
#[derive(Debug, Clone)]
pub struct TaskDisplayInfo {
    pub id: String,
    pub name: String,
    pub priority: String,
    pub status: String,
    pub progress: Option<f32>,
    pub created: String,
}

/// Task panel system
fn task_panel_system(
    mut contexts: EguiContexts,
    ui_state: Res<UiState>,
    mut panel_state: ResMut<TaskPanelState>,
) {
    if !ui_state.show_task_panel {
        return;
    }

    let ctx = contexts.ctx_mut();

    // Sample tasks
    let tasks = vec![
        TaskDisplayInfo {
            id: "task-001".to_string(),
            name: "Generate API handlers".to_string(),
            priority: "High".to_string(),
            status: "Running".to_string(),
            progress: Some(0.45),
            created: "2 min ago".to_string(),
        },
        TaskDisplayInfo {
            id: "task-002".to_string(),
            name: "Analyze dependencies".to_string(),
            priority: "Normal".to_string(),
            status: "Pending".to_string(),
            progress: None,
            created: "5 min ago".to_string(),
        },
        TaskDisplayInfo {
            id: "task-003".to_string(),
            name: "Memory consolidation".to_string(),
            priority: "Low".to_string(),
            status: "Scheduled".to_string(),
            progress: None,
            created: "1 hour".to_string(),
        },
    ];

    egui::Window::new("📋 Task Queue")
        .default_pos([300.0, 100.0])
        .default_size([400.0, 350.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            // Quick add task
            ui.horizontal(|ui| {
                ui.label("Quick task:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut panel_state.new_task_name)
                        .hint_text("Enter task description...")
                        .desired_width(200.0),
                );

                if (ui.button("➕ Add").clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                    && !panel_state.new_task_name.is_empty()
                {
                    log::info!("Adding task: {}", panel_state.new_task_name);
                    panel_state.new_task_name.clear();
                }
            });

            ui.separator();

            // Task list
            egui::ScrollArea::vertical().show(ui, |ui| {
                for task in &tasks {
                    task_row(ui, task);
                }
            });

            ui.separator();

            // Footer stats
            ui.horizontal(|ui| {
                ui.label(format!("Total: {} tasks", tasks.len()));
                ui.separator();
                ui.label(format!(
                    "Running: {}",
                    tasks.iter().filter(|t| t.status == "Running").count()
                ));
                ui.separator();
                ui.label(format!(
                    "Pending: {}",
                    tasks.iter().filter(|t| t.status == "Pending").count()
                ));
            });
        });
}

/// Render a task row
fn task_row(ui: &mut egui::Ui, task: &TaskDisplayInfo) {
    egui::Frame::none()
        .fill(theme::colors::PANEL_BG)
        .rounding(6.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Priority indicator
                let priority_color = match task.priority.as_str() {
                    "Critical" => theme::colors::ERROR,
                    "High" => theme::colors::WARNING,
                    "Normal" => theme::colors::INFO,
                    "Low" => theme::colors::TEXT_MUTED,
                    _ => theme::colors::TEXT_SECONDARY,
                };

                ui.colored_label(priority_color, "●");

                // Task name
                ui.strong(&task.name);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Cancel button
                    if ui.small_button("✕").clicked() {
                        log::info!("Cancel task: {}", task.id);
                    }

                    // Status
                    theme::status_badge(ui, &task.status);
                });
            });

            // Progress bar (if running)
            if let Some(progress) = task.progress {
                ui.add(
                    egui::ProgressBar::new(progress)
                        .text(format!("{:.0}%", progress * 100.0))
                        .animate(true),
                );
            }

            // Created time
            ui.horizontal(|ui| {
                ui.colored_label(theme::colors::TEXT_MUTED, format!("⏰ {}", task.created));
            });
        });

    ui.add_space(4.0);
}
