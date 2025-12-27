//! Workspace Mode System
//!
//! Provides context-aware UI switching based on the current task type.
//! The UI adapts to show relevant panels for Planning, Creation, Analysis, etc.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Workspace Modes
// ============================================================================

/// Current workspace mode determines which panels are visible and how they're laid out
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Resource)]
pub enum WorkspaceMode {
    /// Default mode - balanced view with all panels
    #[default]
    Default,

    /// Chat-focused mode - maximized chat, minimized other panels
    Chat,

    /// Planning mode - task queue, agent status prominent
    Planning,

    /// Creative mode - media studio, image generation focus  
    Creative,

    /// Code mode - code editor, terminal output focus
    Coding,

    /// Analysis mode - memory, metrics, hardware stats focus
    Analysis,

    /// Presentation mode - avatar prominent, minimal UI
    Presentation,
}

impl WorkspaceMode {
    /// Get display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Chat => "💬 Chat",
            Self::Planning => "📋 Planning",
            Self::Creative => "🎨 Creative",
            Self::Coding => "💻 Coding",
            Self::Analysis => "📊 Analysis",
            Self::Presentation => "🎭 Presentation",
        }
    }

    /// Get all available modes
    pub fn all() -> &'static [WorkspaceMode] {
        &[
            Self::Default,
            Self::Chat,
            Self::Planning,
            Self::Creative,
            Self::Coding,
            Self::Analysis,
            Self::Presentation,
        ]
    }

    /// Suggest mode based on message content
    pub fn suggest_from_message(message: &str) -> Option<WorkspaceMode> {
        let msg = message.to_lowercase();

        if msg.contains("plan") || msg.contains("design") || msg.contains("outline") {
            Some(Self::Planning)
        } else if msg.contains("create")
            || msg.contains("generate")
            || msg.contains("draw")
            || msg.contains("image")
        {
            Some(Self::Creative)
        } else if msg.contains("code")
            || msg.contains("implement")
            || msg.contains("fix")
            || msg.contains("debug")
        {
            Some(Self::Coding)
        } else if msg.contains("analyze") || msg.contains("review") || msg.contains("stats") {
            Some(Self::Analysis)
        } else if msg.contains("present") || msg.contains("demo") || msg.contains("show") {
            Some(Self::Presentation)
        } else {
            None
        }
    }
}

// ============================================================================
// Panel Visibility Configuration
// ============================================================================

/// Which panels are visible in each mode
#[derive(Debug, Clone)]
pub struct PanelVisibility {
    pub chat: bool,
    pub status: bool,
    pub hardware: bool,
    pub tasks: bool,
    pub antigravity: bool,
    pub media_studio: bool,
}

impl PanelVisibility {
    /// Get panel visibility for a workspace mode
    pub fn for_mode(mode: WorkspaceMode) -> Self {
        match mode {
            WorkspaceMode::Default => Self {
                chat: true,
                status: true,
                hardware: true,
                tasks: true,
                antigravity: false,
                media_studio: false,
            },
            WorkspaceMode::Chat => Self {
                chat: true,
                status: true,
                hardware: false,
                tasks: false,
                antigravity: false,
                media_studio: false,
            },
            WorkspaceMode::Planning => Self {
                chat: true,
                status: true,
                hardware: false,
                tasks: true,
                antigravity: true,
                media_studio: false,
            },
            WorkspaceMode::Creative => Self {
                chat: true,
                status: false,
                hardware: false,
                tasks: false,
                antigravity: false,
                media_studio: true,
            },
            WorkspaceMode::Coding => Self {
                chat: true,
                status: true,
                hardware: false,
                tasks: true,
                antigravity: true,
                media_studio: false,
            },
            WorkspaceMode::Analysis => Self {
                chat: false,
                status: true,
                hardware: true,
                tasks: true,
                antigravity: true,
                media_studio: false,
            },
            WorkspaceMode::Presentation => Self {
                chat: false,
                status: false,
                hardware: false,
                tasks: false,
                antigravity: false,
                media_studio: false,
            },
        }
    }
}

// ============================================================================
// Workspace Bevy Resource
// ============================================================================

/// Bevy resource for workspace state
#[derive(Resource, Default)]
pub struct WorkspaceState {
    /// Current mode
    pub mode: WorkspaceMode,
    /// Panel visibility (derived from mode)
    pub visibility: Option<PanelVisibility>,
    /// Whether mode was auto-suggested
    pub auto_suggested: bool,
}

impl WorkspaceState {
    /// Switch to a new mode
    pub fn set_mode(&mut self, mode: WorkspaceMode) {
        self.mode = mode;
        self.visibility = Some(PanelVisibility::for_mode(mode));
        self.auto_suggested = false;
    }

    /// Suggest a mode (user can accept or reject)
    pub fn suggest_mode(&mut self, mode: WorkspaceMode) {
        if self.mode == WorkspaceMode::Default {
            self.mode = mode;
            self.visibility = Some(PanelVisibility::for_mode(mode));
            self.auto_suggested = true;
        }
    }

    /// Get current visibility (or default)
    pub fn get_visibility(&self) -> PanelVisibility {
        self.visibility
            .clone()
            .unwrap_or_else(|| PanelVisibility::for_mode(self.mode))
    }
}

// ============================================================================
// Workspace UI Component
// ============================================================================

use bevy_egui::egui;

/// UI component for workspace mode selector
pub fn workspace_mode_selector(ui: &mut egui::Ui, state: &mut WorkspaceState) {
    ui.horizontal(|ui| {
        ui.label("Mode:");

        egui::ComboBox::from_label("")
            .selected_text(state.mode.display_name())
            .show_ui(ui, |ui: &mut egui::Ui| {
                for mode in WorkspaceMode::all() {
                    if ui
                        .selectable_label(state.mode == *mode, mode.display_name())
                        .clicked()
                    {
                        state.set_mode(*mode);
                    }
                }
            });

        if state.auto_suggested {
            ui.colored_label(egui::Color32::YELLOW, "(suggested)");
        }
    });
}
