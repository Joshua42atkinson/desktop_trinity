//! Model Selector Panel - Load/Unload LLM Models
//!
//! Provides UI for browsing and loading models from the catalog.

use bevy_egui::egui::{self, Color32, ProgressBar, RichText, Ui, Vec2};

/// Model information for display
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub size_gb: f32,
    pub quantization: String,
    pub is_loaded: bool,
}

/// State for the model selector
#[derive(Default)]
pub struct ModelSelectorState {
    pub available_models: Vec<ModelInfo>,
    pub current_model: Option<String>,
    pub loading_progress: Option<(String, f32)>, // (name, progress 0.0-1.0)
    pub memory_usage: MemoryUsage,
}

/// Memory usage info
#[derive(Default, Clone)]
pub struct MemoryUsage {
    pub vram_used_gb: f32,
    pub vram_total_gb: f32,
    pub system_used_gb: f32,
    pub system_total_gb: f32,
}

impl MemoryUsage {
    pub fn vram_percent(&self) -> f32 {
        if self.vram_total_gb > 0.0 {
            (self.vram_used_gb / self.vram_total_gb) * 100.0
        } else {
            0.0
        }
    }

    pub fn system_percent(&self) -> f32 {
        if self.system_total_gb > 0.0 {
            (self.system_used_gb / self.system_total_gb) * 100.0
        } else {
            0.0
        }
    }
}

/// Model action callbacks
pub enum ModelAction {
    Load(String),
    Unload,
    Refresh,
}

/// Model selector panel
pub struct ModelSelectorPanel;

impl ModelSelectorPanel {
    /// Show the model selector panel
    pub fn show(ui: &mut Ui, state: &mut ModelSelectorState) -> Option<ModelAction> {
        let mut action = None;

        // Header
        ui.horizontal(|ui| {
            ui.heading("🧠 Models");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄").clicked() {
                    action = Some(ModelAction::Refresh);
                }
            });
        });

        ui.separator();

        // Memory usage
        Self::show_memory_usage(ui, &state.memory_usage);

        ui.separator();

        // Current model status
        if let Some(current) = &state.current_model {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Loaded:").strong());
                ui.label(RichText::new(current).color(Color32::from_rgb(100, 255, 100)));
                if ui.button("Unload").clicked() {
                    action = Some(ModelAction::Unload);
                }
            });
            ui.separator();
        }

        // Loading progress
        if let Some((name, progress)) = &state.loading_progress {
            ui.horizontal(|ui| {
                ui.label(format!("Loading {}...", name));
            });
            ui.add(ProgressBar::new(*progress).text(format!("{:.0}%", progress * 100.0)));
            ui.add_space(8.0);
        }

        // Available models list
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for model in &state.available_models {
                    let is_current = state.current_model.as_ref() == Some(&model.name);
                    let is_loading = state
                        .loading_progress
                        .as_ref()
                        .map(|(n, _)| n == &model.name)
                        .unwrap_or(false);

                    Self::show_model_row(ui, model, is_current, is_loading, &mut action);
                }

                if state.available_models.is_empty() {
                    ui.label(RichText::new("No models found").weak());
                    ui.label(RichText::new("Place .gguf files in ~/.trinity/models/").small());
                }
            });

        action
    }

    fn show_memory_usage(ui: &mut Ui, memory: &MemoryUsage) {
        ui.horizontal(|ui| {
            let vram_color = if memory.vram_percent() > 90.0 {
                Color32::from_rgb(255, 100, 100)
            } else if memory.vram_percent() > 70.0 {
                Color32::from_rgb(255, 200, 100)
            } else {
                Color32::from_rgb(100, 255, 100)
            };

            ui.label("VRAM:");
            ui.add(
                ProgressBar::new(memory.vram_percent() / 100.0).text(format!(
                    "{:.1}/{:.1} GB",
                    memory.vram_used_gb, memory.vram_total_gb
                )),
            );
            ui.colored_label(vram_color, format!("{:.0}%", memory.vram_percent()));
        });

        ui.horizontal(|ui| {
            ui.label("RAM: ");
            ui.add(
                ProgressBar::new(memory.system_percent() / 100.0).text(format!(
                    "{:.1}/{:.1} GB",
                    memory.system_used_gb, memory.system_total_gb
                )),
            );
            ui.label(format!("{:.0}%", memory.system_percent()));
        });
    }

    fn show_model_row(
        ui: &mut Ui,
        model: &ModelInfo,
        is_current: bool,
        is_loading: bool,
        action: &mut Option<ModelAction>,
    ) {
        let bg_color = if is_current {
            Color32::from_rgba_unmultiplied(100, 255, 100, 30)
        } else if is_loading {
            Color32::from_rgba_unmultiplied(255, 200, 100, 30)
        } else {
            Color32::from_rgba_unmultiplied(100, 100, 100, 20)
        };

        egui::Frame::none()
            .fill(bg_color)
            .rounding(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Model info
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&model.name).strong());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{:.1} GB", model.size_gb)).small());
                            ui.label(
                                RichText::new(&model.quantization)
                                    .small()
                                    .color(Color32::from_rgb(150, 150, 255)),
                            );
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_current {
                            ui.label(
                                RichText::new("✓ Loaded").color(Color32::from_rgb(100, 255, 100)),
                            );
                        } else if is_loading {
                            ui.label(
                                RichText::new("Loading...").color(Color32::from_rgb(255, 200, 100)),
                            );
                        } else if ui
                            .add_sized(Vec2::new(60.0, 24.0), egui::Button::new("Load"))
                            .clicked()
                        {
                            *action = Some(ModelAction::Load(model.path.clone()));
                        }
                    });
                });
            });

        ui.add_space(4.0);
    }
}
