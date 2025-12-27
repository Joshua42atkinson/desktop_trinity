//! Chat Panel - Streaming Conversation Interface
//!
//! Provides a rich chat interface with streaming token display.

use bevy_egui::egui::{self, Color32, RichText, ScrollArea, Ui, Vec2};
use chrono::{DateTime, Utc};

/// A message in the chat
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_streaming: bool,
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

impl MessageRole {
    fn color(&self) -> Color32 {
        match self {
            MessageRole::User => Color32::from_rgb(100, 180, 255),
            MessageRole::Assistant => Color32::from_rgb(180, 255, 100),
            MessageRole::System => Color32::from_rgb(180, 180, 180),
            MessageRole::Error => Color32::from_rgb(255, 100, 100),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            MessageRole::User => "You",
            MessageRole::Assistant => "Trinity",
            MessageRole::System => "System",
            MessageRole::Error => "Error",
        }
    }
}

/// State for the chat panel
#[derive(Default)]
pub struct ChatPanelState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub scroll_to_bottom: bool,
    pub is_generating: bool,
    pub current_streaming: String,
}

impl ChatPanelState {
    pub fn new() -> Self {
        Self {
            messages: vec![ChatMessage {
                role: MessageRole::System,
                content: "Trinity AI OS initialized. How can I help you?".to_string(),
                timestamp: Utc::now(),
                is_streaming: false,
            }],
            input: String::new(),
            scroll_to_bottom: true,
            is_generating: false,
            current_streaming: String::new(),
        }
    }

    /// Add a new message
    pub fn add_message(&mut self, role: MessageRole, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role,
            content: content.into(),
            timestamp: Utc::now(),
            is_streaming: false,
        });
        self.scroll_to_bottom = true;
    }

    /// Start streaming a new assistant message
    pub fn start_streaming(&mut self) {
        self.is_generating = true;
        self.current_streaming.clear();
    }

    /// Append a token to the streaming message
    pub fn append_token(&mut self, token: &str) {
        self.current_streaming.push_str(token);
        self.scroll_to_bottom = true;
    }

    /// Finalize the streaming message
    pub fn finish_streaming(&mut self) {
        if !self.current_streaming.is_empty() {
            let content = std::mem::take(&mut self.current_streaming);
            self.add_message(MessageRole::Assistant, content);
        }
        self.is_generating = false;
    }
}

/// Chat panel display
pub struct ChatPanel;

impl ChatPanel {
    /// Show the chat panel
    pub fn show(ui: &mut Ui, state: &mut ChatPanelState, on_submit: impl FnOnce(&str)) -> bool {
        let mut submitted = false;

        // Header
        ui.horizontal(|ui| {
            ui.heading("💬 Chat");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if state.is_generating {
                    ui.label(
                        RichText::new("● Generating...").color(Color32::from_rgb(255, 200, 50)),
                    );
                } else {
                    ui.label(RichText::new("● Ready").color(Color32::from_rgb(100, 255, 100)));
                }
            });
        });

        ui.separator();

        // Messages area
        let available_height = ui.available_height() - 80.0;

        ScrollArea::vertical()
            .max_height(available_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                for msg in &state.messages {
                    Self::render_message(ui, msg);
                    ui.add_space(8.0);
                }

                // Show streaming message if generating
                if state.is_generating && !state.current_streaming.is_empty() {
                    let streaming_msg = ChatMessage {
                        role: MessageRole::Assistant,
                        content: state.current_streaming.clone(),
                        timestamp: Utc::now(),
                        is_streaming: true,
                    };
                    Self::render_message(ui, &streaming_msg);
                }

                // Auto-scroll
                if state.scroll_to_bottom {
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                    state.scroll_to_bottom = false;
                }
            });

        ui.separator();

        // Input area
        ui.horizontal(|ui| {
            let input_width = ui.available_width() - 80.0;
            let response = ui.add_sized(
                Vec2::new(input_width, 36.0),
                egui::TextEdit::singleline(&mut state.input)
                    .hint_text("Type a message...")
                    .font(egui::TextStyle::Body),
            );

            let can_submit = !state.input.trim().is_empty() && !state.is_generating;

            if ui
                .add_enabled(can_submit, egui::Button::new("Send"))
                .clicked()
                || (response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && can_submit)
            {
                let input = std::mem::take(&mut state.input);
                state.add_message(MessageRole::User, &input);
                on_submit(&input);
                submitted = true;
            }
        });

        submitted
    }

    fn render_message(ui: &mut Ui, msg: &ChatMessage) {
        let is_user = msg.role == MessageRole::User;

        let bg_color = if is_user {
            Color32::from_rgba_unmultiplied(100, 150, 255, 30)
        } else {
            Color32::from_rgba_unmultiplied(100, 255, 100, 20)
        };

        egui::Frame::none()
            .fill(bg_color)
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                // Header with role and time
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(msg.role.label())
                            .color(msg.role.color())
                            .strong(),
                    );
                    ui.label(
                        RichText::new(msg.timestamp.format("%H:%M").to_string())
                            .small()
                            .weak(),
                    );
                    if msg.is_streaming {
                        ui.label(RichText::new("●").color(Color32::from_rgb(255, 200, 50)));
                    }
                });

                ui.add_space(4.0);

                // Content
                ui.label(&msg.content);
            });
    }
}
