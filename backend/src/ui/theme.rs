//! Trinity Dark Theme
//! 
//! Custom egui theme with Purdue gold accents

use bevy_egui::egui::{self, Color32, Rounding, Stroke, Visuals};

/// Purdue colors
pub mod colors {
    use super::*;
    
    pub const PURDUE_GOLD: Color32 = Color32::from_rgb(207, 185, 145);
    pub const PURDUE_BLACK: Color32 = Color32::from_rgb(0, 0, 0);
    pub const TRINITY_PURPLE: Color32 = Color32::from_rgb(138, 43, 226);
    
    pub const DARK_BG: Color32 = Color32::from_rgb(20, 20, 25);
    pub const DARKER_BG: Color32 = Color32::from_rgb(15, 15, 18);
    pub const PANEL_BG: Color32 = Color32::from_rgb(30, 30, 38);
    pub const HOVER_BG: Color32 = Color32::from_rgb(45, 45, 55);
    pub const SELECTED_BG: Color32 = Color32::from_rgb(60, 60, 75);
    
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 240, 245);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 170);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 100, 110);
    
    pub const SUCCESS: Color32 = Color32::from_rgb(80, 200, 120);
    pub const WARNING: Color32 = Color32::from_rgb(255, 180, 50);
    pub const ERROR: Color32 = Color32::from_rgb(255, 80, 80);
    pub const INFO: Color32 = Color32::from_rgb(80, 160, 255);
}

/// Trinity theme configuration
pub struct TrinityTheme {
    pub dark_mode: bool,
}

impl Default for TrinityTheme {
    fn default() -> Self {
        Self { dark_mode: true }
    }
}

/// Apply the Trinity theme to egui context
pub fn apply_trinity_theme(ctx: &egui::Context, dark_mode: bool) {
    let mut visuals = if dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    
    if dark_mode {
        // Background colors
        visuals.window_fill = colors::PANEL_BG;
        visuals.panel_fill = colors::DARKER_BG;
        visuals.faint_bg_color = colors::DARK_BG;
        visuals.extreme_bg_color = colors::DARKER_BG;
        
        // Widgets
        visuals.widgets.noninteractive.bg_fill = colors::PANEL_BG;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT_SECONDARY);
        visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
        
        visuals.widgets.inactive.bg_fill = colors::PANEL_BG;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT_PRIMARY);
        visuals.widgets.inactive.rounding = Rounding::same(4.0);
        
        visuals.widgets.hovered.bg_fill = colors::HOVER_BG;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, colors::PURDUE_GOLD);
        visuals.widgets.hovered.rounding = Rounding::same(4.0);
        
        visuals.widgets.active.bg_fill = colors::SELECTED_BG;
        visuals.widgets.active.fg_stroke = Stroke::new(2.0, colors::PURDUE_GOLD);
        visuals.widgets.active.rounding = Rounding::same(4.0);
        
        // Selection
        visuals.selection.bg_fill = colors::TRINITY_PURPLE.gamma_multiply(0.5);
        visuals.selection.stroke = Stroke::new(1.0, colors::TRINITY_PURPLE);
        
        // Window styling
        visuals.window_rounding = Rounding::same(8.0);
        visuals.window_stroke = Stroke::new(1.0, colors::HOVER_BG);
        visuals.window_shadow.color = Color32::from_black_alpha(100);
        
        // Hyperlinks
        visuals.hyperlink_color = colors::PURDUE_GOLD;
    }
    
    ctx.set_visuals(visuals);
    
    // Set custom fonts if needed
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style(style);
}

/// Status indicator colors
pub fn status_color(status: &str) -> Color32 {
    match status.to_lowercase().as_str() {
        "idle" | "ready" => colors::SUCCESS,
        "processing" | "running" => colors::INFO,
        "waiting" => colors::WARNING,
        "error" | "failed" => colors::ERROR,
        _ => colors::TEXT_MUTED,
    }
}

/// Format a status badge
pub fn status_badge(ui: &mut egui::Ui, status: &str) {
    let color = status_color(status);
    let icon = match status.to_lowercase().as_str() {
        "idle" | "ready" => "🟢",
        "processing" | "running" => "🔵",
        "waiting" => "🟡",
        "error" | "failed" => "🔴",
        _ => "⚪",
    };
    
    ui.horizontal(|ui| {
        ui.label(icon);
        ui.colored_label(color, status);
    });
}
