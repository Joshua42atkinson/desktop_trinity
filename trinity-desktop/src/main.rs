use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use trinity_core::kernel::Kernel;

pub mod panels;
use panels::agent_viz::AgentVizPanel;
use panels::dashboard::DashboardPanel;
use panels::log_console::LogConsolePanel;
use panels::task_input::TaskInputPanel;

// ----------------------------------------------------------------------------
// Communications Bridge
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub enum KernelCommand {
    SubmitTask(String),
    LoadModel(String),
    UnloadModel,
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum KernelEvent {
    /// General log message
    Log(String),
    /// Agent count status update
    StatusUpdate { agents: usize },
    /// Task completed with result
    TaskComplete(String),
    /// Streaming token from LLM
    StreamingToken { token: String, is_final: bool },
    /// Model loading progress
    ModelLoading { progress: f32, model_name: String },
    /// Model loaded successfully
    ModelLoaded { name: String, size_gb: f32 },
    /// Memory pressure update
    MemoryPressure {
        vram_used: u64,
        vram_total: u64,
        system_used: u64,
        system_total: u64,
    },
    /// Agent thinking status
    AgentThinking { agent_id: String, preview: String },
    /// Agent completed response
    AgentResponse {
        agent_id: String,
        response: String,
        duration_ms: u64,
    },
    /// Error occurred
    Error(String),
}

#[derive(Resource)]
pub struct KernelBridge {
    pub _runtime: Runtime,
    pub command_tx: mpsc::Sender<KernelCommand>,
    pub event_rx: Arc<Mutex<mpsc::Receiver<KernelEvent>>>, // Mutex for Bevy system access
    pub last_status: String,
}

#[derive(Resource, Default)]
struct UiState {
    prompt: String,
    logs: Vec<String>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .init_resource::<UiState>()
        .add_systems(Startup, setup_kernel)
        .add_systems(Update, (ui_system, update_bridge_system))
        .run();
}

fn setup_kernel(mut commands: Commands) {
    // 1. Create Tokio Runtime
    let rt = Runtime::new().unwrap();

    // 2. Create Channels
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<KernelCommand>(32);
    let (event_tx, _event_rx) = mpsc::channel::<KernelEvent>(32);

    // 3. Spawn Kernel Thread
    let event_tx_clone = event_tx.clone();
    rt.spawn(async move {
        // Init Kernel
        let _ = event_tx_clone
            .send(KernelEvent::Log("Booting Trinity Kernel...".into()))
            .await;

        match Kernel::boot().await {
            Ok(mut kernel) => {
                let _ = event_tx_clone
                    .send(KernelEvent::Log("Kernel Ready.".into()))
                    .await;

                // Kernel Loop
                loop {
                    // Check commands
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            KernelCommand::SubmitTask(t) => {
                                let _ = event_tx_clone
                                    .send(KernelEvent::Log(format!("Processing task: {}", t)))
                                    .await;
                                kernel.submit_task(t);
                            }
                            KernelCommand::LoadModel(path) => {
                                let _ = event_tx_clone
                                    .send(KernelEvent::Log(format!("Loading model: {}", path)))
                                    .await;
                                // TODO: Integrate with ModelManager
                            }
                            KernelCommand::UnloadModel => {
                                let _ = event_tx_clone
                                    .send(KernelEvent::Log("Unloading model...".into()))
                                    .await;
                                // TODO: Integrate with ModelManager
                            }
                            KernelCommand::Shutdown => break,
                        }
                    }

                    // Tick Kernel
                    kernel.tick();

                    // Collect Metrics/Events (Placeholder)
                    // In a real implementation we would inspect kernel.world for events
                    let agent_count = kernel.world.entities().len();
                    let _ = event_tx_clone.try_send(KernelEvent::StatusUpdate {
                        agents: agent_count as usize,
                    });

                    // throttle
                    tokio::time::sleep(Duration::from_millis(16)).await;
                }
            }
            Err(e) => {
                let _ = event_tx_clone
                    .send(KernelEvent::Log(format!("Kernel Panic: {}", e)))
                    .await;
            }
        }
    });

    // 4. Store Bridge Resource
    commands.insert_resource(KernelBridge {
        _runtime: rt,
        command_tx: cmd_tx,
        event_rx: Arc::new(Mutex::new(_event_rx)),
        last_status: "Initializing...".to_string(),
    });
}

fn update_bridge_system(mut bridge: ResMut<KernelBridge>, mut ui_state: ResMut<UiState>) {
    // Poll events from Kernel
    let mut events = Vec::new();
    if let Ok(mut rx) = bridge.event_rx.try_lock() {
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }
    }

    for event in events {
        match event {
            KernelEvent::Log(msg) => ui_state.logs.push(msg),
            KernelEvent::StatusUpdate { agents } => {
                bridge.last_status = format!("Active Agents: {}", agents);
            }
            KernelEvent::TaskComplete(res) => ui_state.logs.push(format!("✓ {}", res)),
            KernelEvent::StreamingToken { token, is_final } => {
                ui_state.logs.push(format!(
                    "[Token] {}{}",
                    token,
                    if is_final { " ✓" } else { "" }
                ));
            }
            KernelEvent::ModelLoading {
                progress,
                model_name,
            } => {
                bridge.last_status = format!("Loading {} ({:.0}%)", model_name, progress * 100.0);
            }
            KernelEvent::ModelLoaded { name, size_gb } => {
                ui_state
                    .logs
                    .push(format!("✓ Loaded model: {} ({:.1} GB)", name, size_gb));
            }
            KernelEvent::MemoryPressure {
                vram_used,
                vram_total,
                ..
            } => {
                let pct = (vram_used as f64 / vram_total as f64) * 100.0;
                bridge.last_status = format!(
                    "VRAM: {:.1}% ({}/{} GB)",
                    pct,
                    vram_used / (1024 * 1024 * 1024),
                    vram_total / (1024 * 1024 * 1024)
                );
            }
            KernelEvent::AgentThinking { agent_id, preview } => {
                ui_state.logs.push(format!(
                    "🤔 Agent {} thinking: {}...",
                    &agent_id[..8],
                    &preview[..preview.len().min(30)]
                ));
            }
            KernelEvent::AgentResponse {
                agent_id,
                duration_ms,
                ..
            } => {
                ui_state.logs.push(format!(
                    "✓ Agent {} responded in {}ms",
                    &agent_id[..8],
                    duration_ms
                ));
            }
            KernelEvent::Error(msg) => {
                ui_state.logs.push(format!("❌ ERROR: {}", msg));
            }
        }
    }
}

fn ui_system(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>, bridge: Res<KernelBridge>) {
    egui::Window::new("Trinity AI OS").show(contexts.ctx_mut(), |ui| {
        ui.heading("Trinity Digital Familiar");
        ui.separator();

        // 1. Dashboard
        DashboardPanel::show(ui, &bridge.last_status);
        ui.separator();

        ui.separator();

        // 2. Task Input
        TaskInputPanel::show(ui, &mut ui_state.prompt, &bridge);
        ui.separator();

        // 3. Agent Viz
        AgentVizPanel::show(ui, &bridge.last_status);
        ui.separator();

        // 4. Log Console
        LogConsolePanel::show(ui, &ui_state.logs);
    });
}
