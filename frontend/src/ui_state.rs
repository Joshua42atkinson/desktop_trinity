use leptos::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct GlobalUiState {
    pub unique_session_id: uuid::Uuid, // Add session ID for consistent context
    pub zen_mode: RwSignal<bool>,
    pub show_hud: RwSignal<bool>,
    pub hud_content: RwSignal<HudContent>,
    // Voice State
    pub is_listening: RwSignal<bool>,
    pub is_speaking: RwSignal<bool>,
    pub last_transcript: RwSignal<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HudContent {
    Empty,
    Video(String),        // URL
    Code(String, String), // Content, Language
}

impl GlobalUiState {
    pub fn new() -> Self {
        Self {
            unique_session_id: uuid::Uuid::new_v4(),
            zen_mode: RwSignal::new(false),
            show_hud: RwSignal::new(false),
            hud_content: RwSignal::new(HudContent::Empty),
            is_listening: RwSignal::new(false),
            is_speaking: RwSignal::new(false),
            last_transcript: RwSignal::new(String::new()),
        }
    }

    pub fn toggle_zen(&self) {
        self.zen_mode.update(|z| *z = !*z);
    }

    pub fn open_hud(&self, content: HudContent) {
        self.hud_content.set(content);
        self.show_hud.set(true);
    }
}

impl Default for GlobalUiState {
    fn default() -> Self {
        Self::new()
    }
}
