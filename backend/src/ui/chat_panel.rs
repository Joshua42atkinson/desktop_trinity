//! Chat Panel
//!
//! Conversation interface with message history, input, and thinking display

use crate::ui::{theme, UiState};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// Plugin for chat panel
pub struct ChatPanelPlugin;

impl Plugin for ChatPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatState>()
            .add_systems(Update, chat_panel_system);
    }
}

/// Chat panel state
#[derive(Resource, Default)]
pub struct ChatState {
    /// Current input text
    pub input: String,
    /// Message history
    pub messages: Vec<ChatMessage>,
    /// Whether agent is thinking
    pub is_thinking: bool,
    /// Current thinking content (Qwen3)
    pub thinking_content: Option<String>,
    /// Show thinking panel
    pub show_thinking: bool,
}

/// A chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: String,
    pub tool_calls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl ChatState {
    pub fn with_sample_messages() -> Self {
        Self {
            input: String::new(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: "Trinity initialized. Ready to assist.".to_string(),
                    timestamp: "16:30".to_string(),
                    tool_calls: vec![],
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: "List the files in the src directory".to_string(),
                    timestamp: "16:31".to_string(),
                    tool_calls: vec![],
                },
                ChatMessage {
                    role: MessageRole::Assistant,
                    content: "I'll list the files for you.".to_string(),
                    timestamp: "16:31".to_string(),
                    tool_calls: vec!["list_directory(\"src\")".to_string()],
                },
            ],
            is_thinking: false,
            thinking_content: None,
            show_thinking: true,
        }
    }
}

/// Chat panel system
fn chat_panel_system(
    mut contexts: EguiContexts,
    ui_state: Res<UiState>,
    mut chat_state: ResMut<ChatState>,
) {
    if !ui_state.show_chat_panel {
        return;
    }

    // Initialize with sample messages if empty
    if chat_state.messages.is_empty() {
        *chat_state = ChatState::with_sample_messages();
    }

    let ctx = contexts.ctx_mut();

    egui::SidePanel::right("chat_panel")
        .default_width(400.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("💬 Chat");
            ui.separator();

            // Thinking panel (Qwen3)
            if chat_state.show_thinking {
                if let Some(ref thinking) = chat_state.thinking_content {
                    egui::CollapsingHeader::new("🧠 Thinking...")
                        .default_open(false)
                        .show(ui, |ui| {
                            egui::Frame::none()
                                .fill(theme::colors::DARKER_BG)
                                .rounding(4.0)
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.colored_label(theme::colors::TEXT_MUTED, thinking);
                                });
                        });
                    ui.separator();
                }
            }

            // Message area
            let available_height = ui.available_height() - 60.0;
            egui::ScrollArea::vertical()
                .max_height(available_height)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for message in &chat_state.messages {
                        message_bubble(ui, message);
                        ui.add_space(8.0);
                    }

                    // Thinking indicator
                    if chat_state.is_thinking {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Trinity is thinking...");
                        });
                    }
                });

            ui.separator();

            // Input area
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(&mut chat_state.input)
                        .hint_text("Type a message...")
                        .desired_width(ui.available_width() - 60.0)
                        .desired_rows(2),
                );

                ui.vertical(|ui| {
                    if (ui.button("Send").clicked()
                        || (response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)))
                        && !chat_state.input.trim().is_empty()
                    {
                        // Clone input before mutable borrow
                        let input_content = chat_state.input.clone();
                        // Add user message
                        chat_state.messages.push(ChatMessage {
                            role: MessageRole::User,
                            content: input_content,
                            timestamp: "Now".to_string(),
                            tool_calls: vec![],
                        });
                        chat_state.input.clear();
                        chat_state.is_thinking = true;
                    }

                    ui.checkbox(&mut chat_state.show_thinking, "🧠");
                });
            });
        });
}

/// Render a message bubble
fn message_bubble(ui: &mut egui::Ui, message: &ChatMessage) {
    let (bg_color, align, icon) = match message.role {
        MessageRole::User => (theme::colors::SELECTED_BG, egui::Align::Max, "👤"),
        MessageRole::Assistant => (theme::colors::PANEL_BG, egui::Align::Min, "🔮"),
        MessageRole::System => (theme::colors::DARKER_BG, egui::Align::Center, "⚙️"),
        MessageRole::Tool => (theme::colors::HOVER_BG, egui::Align::Min, "🔧"),
    };

    ui.with_layout(egui::Layout::top_down(align), |ui| {
        egui::Frame::none()
            .fill(bg_color)
            .rounding(12.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.set_max_width(300.0);

                // Header
                ui.horizontal(|ui| {
                    ui.label(icon);
                    ui.colored_label(theme::colors::TEXT_MUTED, &message.timestamp);
                });

                // Content
                ui.label(&message.content);

                // Tool calls
                if !message.tool_calls.is_empty() {
                    ui.add_space(4.0);
                    for tool in &message.tool_calls {
                        ui.horizontal(|ui| {
                            ui.label("🔧");
                            ui.code(tool);
                        });
                    }
                }
            });
    });
}
