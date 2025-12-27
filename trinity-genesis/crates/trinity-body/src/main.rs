//! # Trinity Body (The Avatar)
//!
//! ## Philosophy (Architectonics)
//! "The Body is the primary interface for collaboration. It gives form to the Mind's intent.
//!  It must be fluid, aesthetically profound (Zen Mode), and serve as a transparent window
//!  into the machine's cognition (Antigravity)."
//!
//! ## Instructions for Developers
//! 1. **Aesthetics Matter**: If it looks basic, it is broken. Use gradients, animations, and premium typography.
//! 2. **Feedback Loops**: The user must always know what the Brain is thinking (via Antigravity Panel).
//! 3. **Immersive**: Speech and visuals should blend seamlessly to create a presence, not just a tool.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub mod agent_components;
pub mod agent_systems;
mod audio;
mod avatar;
mod bridge;
mod panels;

use agent_systems::AgentVisualsPlugin;
use audio::AudioResource;
use avatar::AvatarPlugin;
use bridge::{
    ping_brain_system, process_brain_responses, spawn_brain_runtime, BrainConnection, BrainRequest,
};
use panels::{AntigravityPanel, HardwarePanel, TaskPanel};
use trinity_protocol::types::AvatarState as ProtocolAvatarState;

/// Default Brain address (Desktop via Tailscale)
const DEFAULT_BRAIN_ADDR: &str = "100.115.247.4:9000";

/// Application state shared across systems
#[derive(Resource)]
pub struct AppState {
    /// Current chat messages
    pub messages: Vec<ChatMessage>,
    /// Current input text
    pub input_text: String,
    /// Current avatar state
    pub avatar_state: ProtocolAvatarState,
    /// Whether we're waiting for a response
    pub waiting_for_response: bool,
    /// Whether voice response is enabled
    pub voice_mode: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            messages: vec![ChatMessage {
                role: "system".to_string(),
                content: "Welcome to Trinity Genesis. Press 'Connect' to link to the Brain node."
                    .to_string(),
            }],
            input_text: String::new(),
            avatar_state: ProtocolAvatarState::Idle,
            waiting_for_response: false,
            voice_mode: false,
        }
    }
}

impl AppState {
    /// Push a message with rolling window - keeps only last 9 user/assistant messages
    /// (older messages are stored in Brain's memory via embeddings)
    pub fn push_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);

        // Count non-system messages
        let chat_count = self
            .messages
            .iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .count();

        // If more than 18 (9 exchanges), remove oldest non-system message
        if chat_count > 18 {
            if let Some(idx) = self
                .messages
                .iter()
                .position(|m| m.role == "user" || m.role == "assistant")
            {
                self.messages.remove(idx);
            }
        }
    }
}

fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    info!("╔══════════════════════════════════════════════════════════════╗");
    info!("║              TRINITY GENESIS - BODY NODE                    ║");
    info!("╚══════════════════════════════════════════════════════════════╝");

    // Get brain address from env or use default
    let brain_addr = std::env::var("BRAIN_ADDR").unwrap_or_else(|_| DEFAULT_BRAIN_ADDR.to_string());
    info!("Brain address: {}", brain_addr);

    // Spawn the async brain communication runtime
    let (request_tx, response_rx) = spawn_brain_runtime(brain_addr.clone());

    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Trinity Genesis".to_string(),
            resolution: (1400.0, 800.0).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EguiPlugin)
    .add_plugins(AvatarPlugin)
    .add_plugins(AgentVisualsPlugin)
    // Skills are now accessed via Brain RPC, not Bevy plugins
    // Register the bridge events for skill requests
    .add_event::<bridge::RequestCodeGeneration>()
    .add_event::<bridge::RequestWriting>()
    .init_resource::<AppState>()
    .insert_resource(BrainConnection {
        connected: false,
        brain_addr,
        request_tx,
        response_rx,
        model_info: None,
    });

    // Initialize audio player (optional - may fail on systems without audio)
    match AudioResource::new() {
        Ok(audio) => {
            info!("🔊 Audio player initialized");
            app.insert_resource(audio);
        }
        Err(e) => {
            info!("⚠️ Audio unavailable: {} (voice playback disabled)", e);
        }
    }

    app.init_resource::<HardwarePanel>()
        .init_resource::<TaskPanel>()
        .init_resource::<AntigravityPanel>()
        .add_systems(Startup, (setup_scene, auto_connect_brain))
        .add_systems(
            Update,
            (
                ui_system,
                keyboard_input,
                process_brain_responses,
                ping_brain_system,
                ping_brain_system,
                update_avatar_from_state,
                update_local_hardware_stats,
                bridge::handle_skill_requests,
            ),
        )
        .run();
}

/// Automatically connect to Brain on startup
fn auto_connect_brain(connection: Res<BrainConnection>, mut state: ResMut<AppState>) {
    info!(
        "🔗 Auto-connecting to Brain at {}...",
        connection.brain_addr
    );
    state.messages.push(ChatMessage {
        role: "system".to_string(),
        content: format!("Connecting to Brain at {}...", connection.brain_addr),
    });
    let _ = connection.request_tx.try_send(BrainRequest::Connect);
}

fn update_local_hardware_stats(
    mut hardware_panel: ResMut<HardwarePanel>,
    time: Res<Time>,
    mut last_update: Local<f64>,
) {
    // Update every 1s
    if time.elapsed_seconds_f64() - *last_update > 1.0 {
        *last_update = time.elapsed_seconds_f64();
        hardware_panel.update_local_stats();
    }
}

/// Setup the 3D scene
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.1, 0.1, 0.15),
        brightness: 200.0,
    });

    // Point light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            color: Color::srgb(0.5, 0.8, 1.0),
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(10.0, 10.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.12),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            ..default()
        }),
        ..default()
    });

    info!("Scene initialized");
}

fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<AppState>,
    connection: Res<BrainConnection>,
    hardware_panel: Res<HardwarePanel>,
    mut task_panel: ResMut<TaskPanel>,
    mut antigravity_panel: ResMut<AntigravityPanel>,
) {
    // Status bar at bottom (always visible)
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(28.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal_centered(|ui| {
                // Brain connection status
                if connection.connected {
                    ui.colored_label(egui::Color32::GREEN, "🟢 Brain");
                } else {
                    ui.colored_label(egui::Color32::RED, "🔴 Brain Offline");
                }

                ui.separator();

                // Model info
                if let Some(ref info) = connection.model_info {
                    ui.label(format!("🧠 {}", info.name));
                    ui.separator();
                }

                // Avatar state
                ui.label(format!("Avatar: {:?}", state.avatar_state));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Antigravity toggle
                    if ui
                        .button(if antigravity_panel.visible {
                            "⬇️ Hide Antigravity"
                        } else {
                            "⬆️ Show Antigravity"
                        })
                        .clicked()
                    {
                        antigravity_panel.visible = !antigravity_panel.visible;
                    }
                    ui.label("(A)");
                });
            });
        });

    // Antigravity Window (bottom panel when visible)
    if antigravity_panel.visible {
        egui::TopBottomPanel::bottom("antigravity_panel")
            .default_height(300.0)
            .resizable(true)
            .show(contexts.ctx_mut(), |ui| {
                antigravity_panel.ui(ui);
            });
    }

    // Left panel - Chat
    egui::SidePanel::left("chat_panel")
        .default_width(450.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("💬 Trinity Chat");
            ui.separator();

            // Messages scroll area
            let _text_height = ui.text_style_height(&egui::TextStyle::Body);
            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 80.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for msg in &state.messages {
                        let (icon, color) = match msg.role.as_str() {
                            "user" => ("👤", egui::Color32::from_rgb(100, 180, 255)),
                            "assistant" => ("🔮", egui::Color32::from_rgb(100, 255, 150)),
                            _ => ("ℹ️", egui::Color32::from_rgb(180, 180, 180)),
                        };

                        ui.horizontal_wrapped(|ui| {
                            ui.label(icon);
                            ui.colored_label(color, &msg.content);
                        });
                        ui.add_space(6.0);
                    }

                    if state.waiting_for_response {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Thinking...");
                        });
                    }
                });

            ui.separator();

            // Input area
            ui.horizontal(|ui| {
                let response = ui.add_sized(
                    [ui.available_width() - 70.0, 30.0],
                    egui::TextEdit::singleline(&mut state.input_text).hint_text("Ask Trinity..."),
                );

                let send_enabled = !state.input_text.is_empty()
                    && connection.connected
                    && !state.waiting_for_response;

                if ui
                    .add_enabled(send_enabled, egui::Button::new("Send"))
                    .clicked()
                    || (response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        && send_enabled)
                {
                    send_message(&mut state, &connection);
                }
            });
        });

    // Right panel - Status & Controls
    egui::SidePanel::right("status_panel")
        .default_width(280.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("📊 Status");
            ui.separator();

            // Avatar state
            ui.label(format!("Avatar: {:?}", state.avatar_state));
            ui.label(format!("Messages: {}", state.messages.len()));
            ui.label(format!("Brain: {}", connection.brain_addr));

            ui.separator();
            ui.heading("🔧 Controls");

            // Voice Mode Toggle
            ui.checkbox(&mut state.voice_mode, "🎙️ Voice Mode");

            // Connect/Disconnect button
            if connection.connected {
                if ui.button("🔌 Disconnect").clicked() {
                    let _ = connection.request_tx.try_send(BrainRequest::Disconnect);
                }
            } else {
                if ui.button("🔗 Connect to Brain").clicked() {
                    let _ = connection.request_tx.try_send(BrainRequest::Connect);
                    state.messages.push(ChatMessage {
                        role: "system".to_string(),
                        content: format!("Connecting to {}...", connection.brain_addr),
                    });
                }
            }

            if ui.button("🧹 Clear Chat").clicked() {
                state.messages.clear();
                state.messages.push(ChatMessage {
                    role: "system".to_string(),
                    content: "Chat cleared.".to_string(),
                });
            }

            ui.separator();
            ui.heading("🎮 Keyboard");
            ui.label("ESC - Exit");
            ui.label("Enter - Send message");

            if let Some(ref info) = connection.model_info {
                ui.separator();
                ui.heading("🧠 Model Info");
                ui.label(format!("Name: {}", info.name));
                ui.label(format!("Quantization: {}", info.quantization));
                ui.label(format!("Context: {} tokens", info.context_size));
            }

            ui.separator();
            hardware_panel.ui(ui);

            ui.separator();
            if let Some((name, _desc)) = task_panel.ui(ui) {
                // Send to Brain
                // Determine task type based on content?
                // For now, if name starts with "edit ", use EditFile.
                // If name starts with "run ", use RunCommand.
                // Otherwise, use Think.

                let task_type = if name.starts_with("edit ") {
                    let parts: Vec<&str> = name.splitn(3, ' ').collect();
                    if parts.len() >= 3 {
                        trinity_protocol::task::TaskType::EditFile {
                            path: parts[1].to_string(),
                            instructions: parts[2].to_string(),
                        }
                    } else {
                        trinity_protocol::task::TaskType::Think {
                            prompt: name.clone(),
                        }
                    }
                } else if name.starts_with("run ") {
                    trinity_protocol::task::TaskType::RunCommand {
                        command: name[4..].to_string(),
                        working_dir: None,
                    }
                } else {
                    trinity_protocol::task::TaskType::Think {
                        prompt: name.clone(),
                    }
                };

                let _ = connection.request_tx.try_send(BrainRequest::SubmitTask {
                    name: name.clone(),
                    task_type,
                    priority: 1, // Normal
                });
            }
        });
}

fn send_message(state: &mut AppState, connection: &BrainConnection) {
    let user_msg = state.input_text.clone();

    // Build history from existing messages (filter out system messages)
    let history: Vec<trinity_protocol::types::ChatMessage> = state
        .messages
        .iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| trinity_protocol::types::ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        })
        .collect();

    state.messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_msg.clone(),
    });

    // Send to brain with history
    if state.voice_mode {
        let _ = connection
            .request_tx
            .try_send(BrainRequest::ThinkWithVoice { prompt: user_msg });
    } else {
        let _ = connection.request_tx.try_send(BrainRequest::Think {
            prompt: user_msg,
            history,
        });
    }

    state.input_text.clear();
    state.waiting_for_response = true;
    state.avatar_state = ProtocolAvatarState::Thinking;
}

/// Update avatar component from app state
fn update_avatar_from_state(
    state: Res<AppState>,
    mut avatars: Query<&mut avatar::AvatarState, With<avatar::TrinityAvatar>>,
) {
    for mut avatar_state in avatars.iter_mut() {
        *avatar_state = match state.avatar_state {
            ProtocolAvatarState::Idle => avatar::AvatarState::Idle,
            ProtocolAvatarState::Thinking => avatar::AvatarState::Thinking,
            ProtocolAvatarState::Coding => avatar::AvatarState::Coding,
            ProtocolAvatarState::Speaking => avatar::AvatarState::Speaking,
            ProtocolAvatarState::Sleeping => avatar::AvatarState::Sleeping,
        };
    }
}

/// Handle keyboard input
fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut antigravity: ResMut<AntigravityPanel>,
    mut contexts: EguiContexts,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }

    // Only toggle Antigravity with 'A' key when NOT typing in a text field
    if keys.just_pressed(KeyCode::KeyA) && !contexts.ctx_mut().wants_keyboard_input() {
        antigravity.visible = !antigravity.visible;
        info!(
            "Antigravity Window: {}",
            if antigravity.visible {
                "OPEN"
            } else {
                "CLOSED"
            }
        );
    }
}

/// A chat message
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}
