//! Agent Status Panel
//!
//! Displays agent roles, states, and memory usage

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::ui::{UiState, theme};

/// Plugin for agent status panel
pub struct AgentPanelPlugin;

impl Plugin for AgentPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, agent_panel_system);
    }
}

/// Agent display info (UI view model)
#[derive(Debug, Clone)]
pub struct AgentDisplayInfo {
    pub name: String,
    pub role: String,
    pub status: String,
    pub current_task: Option<String>,
    pub memory_used_mb: f32,
    pub tokens_processed: u64,
}

impl Default for AgentDisplayInfo {
    fn default() -> Self {
        Self {
            name: "Core Agent".to_string(),
            role: "Core".to_string(),
            status: "Idle".to_string(),
            current_task: None,
            memory_used_mb: 0.0,
            tokens_processed: 0,
        }
    }
}

/// Agent panel system
fn agent_panel_system(
    mut contexts: EguiContexts,
    ui_state: Res<UiState>,
) {
    if !ui_state.show_agent_panel {
        return;
    }
    
    let ctx = contexts.ctx_mut();
    
    // Sample agents for display
    let agents = vec![
        AgentDisplayInfo {
            name: "Router".to_string(),
            role: "Router".to_string(),
            status: "Idle".to_string(),
            current_task: None,
            memory_used_mb: 12.5,
            tokens_processed: 1024,
        },
        AgentDisplayInfo {
            name: "Core".to_string(),
            role: "Core".to_string(),
            status: "Processing".to_string(),
            current_task: Some("Analyzing code...".to_string()),
            memory_used_mb: 256.0,
            tokens_processed: 15432,
        },
        AgentDisplayInfo {
            name: "Developer".to_string(),
            role: "Developer".to_string(),
            status: "Idle".to_string(),
            current_task: None,
            memory_used_mb: 128.0,
            tokens_processed: 8721,
        },
    ];
    
    egui::SidePanel::left("agent_panel")
        .default_width(280.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("🤖 Agents");
            ui.separator();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for agent in &agents {
                    agent_card(ui, agent);
                    ui.add_space(8.0);
                }
            });
        });
}

/// Render an agent status card
fn agent_card(ui: &mut egui::Ui, agent: &AgentDisplayInfo) {
    egui::Frame::none()
        .fill(theme::colors::PANEL_BG)
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Role icon
                let icon = match agent.role.as_str() {
                    "Router" => "🔀",
                    "Core" => "🧠",
                    "Research" => "🔬",
                    "Developer" => "💻",
                    "Writer" => "✍️",
                    _ => "🤖",
                };
                ui.heading(icon);
                ui.heading(&agent.name);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::status_badge(ui, &agent.status);
                });
            });
            
            ui.add_space(4.0);
            
            // Current task
            if let Some(ref task) = agent.current_task {
                ui.horizontal(|ui| {
                    ui.label("📋");
                    ui.colored_label(theme::colors::TEXT_SECONDARY, task);
                });
            }
            
            // Stats
            ui.horizontal(|ui| {
                ui.label(format!("💾 {:.1} MB", agent.memory_used_mb));
                ui.separator();
                ui.label(format!("🔤 {} tokens", agent.tokens_processed));
            });
        });
}
