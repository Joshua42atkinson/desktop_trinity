//! Hardware Monitor Panel
//!
//! Displays system resource usage from both Brain and Body nodes.

use bevy::prelude::*;
use bevy_egui::egui;

/// Hardware statistics from a node
#[derive(Debug, Clone, Default)]
pub struct HardwareStats {
    pub cpu_usage: f32, // 0.0 - 100.0
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub vram_used_gb: f32,
    pub vram_total_gb: f32,
    pub gpu_temp_c: Option<f32>,
    pub inference_tokens_per_sec: Option<f32>,
}

/// Hardware panel state
#[derive(Resource, Default)]
pub struct HardwarePanel {
    /// Stats from the Brain node (if connected)
    pub brain_stats: Option<HardwareStats>,
    /// Stats from local Body node
    pub body_stats: HardwareStats,
    /// Last update time
    pub last_update: f64,
}

impl HardwarePanel {
    /// Update local stats (would read from sysinfo)
    pub fn update_local_stats(&mut self) {
        // Placeholder - would use sysinfo crate
        self.body_stats = HardwareStats {
            cpu_usage: 25.0,
            ram_used_gb: 8.0,
            ram_total_gb: 32.0,
            vram_used_gb: 2.0,
            vram_total_gb: 8.0,
            gpu_temp_c: Some(45.0),
            inference_tokens_per_sec: None,
        };
    }

    /// Render the hardware panel UI
    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.heading("🖥️ Hardware");
        ui.separator();

        // Brain node stats
        ui.collapsing("🧠 Brain (Desktop)", |ui| {
            if let Some(stats) = &self.brain_stats {
                Self::render_stats(ui, stats, true);
            } else {
                ui.colored_label(egui::Color32::GRAY, "Not connected");
            }
        });

        ui.add_space(4.0);

        // Body node stats
        ui.collapsing("👤 Body (Local)", |ui| {
            Self::render_stats(ui, &self.body_stats, false);
        });
    }

    fn render_stats(ui: &mut egui::Ui, stats: &HardwareStats, is_brain: bool) {
        // CPU
        ui.horizontal(|ui| {
            ui.label("CPU:");
            let color = if stats.cpu_usage > 80.0 {
                egui::Color32::RED
            } else if stats.cpu_usage > 50.0 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::GREEN
            };
            ui.colored_label(color, format!("{:.1}%", stats.cpu_usage));
        });

        // RAM
        let ram_percent = (stats.ram_used_gb / stats.ram_total_gb) * 100.0;
        ui.horizontal(|ui| {
            ui.label("RAM:");
            ui.label(format!(
                "{:.1} / {:.1} GB ({:.0}%)",
                stats.ram_used_gb, stats.ram_total_gb, ram_percent
            ));
        });
        ui.add(egui::ProgressBar::new(ram_percent / 100.0).show_percentage());

        // VRAM
        if stats.vram_total_gb > 0.0 {
            let vram_percent = (stats.vram_used_gb / stats.vram_total_gb) * 100.0;
            ui.horizontal(|ui| {
                ui.label("VRAM:");
                ui.label(format!(
                    "{:.1} / {:.1} GB",
                    stats.vram_used_gb, stats.vram_total_gb
                ));
            });
            ui.add(
                egui::ProgressBar::new(vram_percent / 100.0)
                    .fill(egui::Color32::from_rgb(100, 200, 255)),
            );
        }

        // GPU Temp
        if let Some(temp) = stats.gpu_temp_c {
            ui.horizontal(|ui| {
                ui.label("GPU Temp:");
                let color = if temp > 80.0 {
                    egui::Color32::RED
                } else if temp > 60.0 {
                    egui::Color32::YELLOW
                } else {
                    egui::Color32::GREEN
                };
                ui.colored_label(color, format!("{}°C", temp as i32));
            });
        }

        // Inference speed (brain only)
        if is_brain {
            if let Some(tps) = stats.inference_tokens_per_sec {
                ui.horizontal(|ui| {
                    ui.label("Inference:");
                    ui.colored_label(egui::Color32::LIGHT_GREEN, format!("{:.1} t/s", tps));
                });
            }
        }
    }
}
