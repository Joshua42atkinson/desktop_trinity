// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Bridge to Brain Node
//!
//! Tarpc client for connecting to the desktop Brain node.
//! Provides Bevy-compatible async integration via channels.

use anyhow::Result;
use bevy::prelude::*;
use std::net::SocketAddr;

use tarpc::{client, context, tokio_serde::formats::Bincode};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use trinity_protocol::{
    brain::BrainServiceClient,
    types::ModelInfo,
    task::TaskType,
    StreamEvent,
};
// Bevy events for skill integration
// These are defined here to avoid circular dependency issues
// When skills module has proper Bevy support, these can be moved

/// Request to generate code (Bevy Event)
#[derive(bevy::prelude::Event, Debug, Clone)]
pub struct RequestCodeGeneration {
    pub prompt: String,
    pub language: String,
    pub output_path: Option<String>,
}

/// Request to generate written content (Bevy Event)
#[derive(bevy::prelude::Event, Debug, Clone)]
pub struct RequestWriting {
    pub style: String,
    pub topic: String,
    pub target_words: u32,
}

// ============================================================================
// Bevy Resources
// ============================================================================

/// Connection state to the Brain node
#[derive(Resource)]
pub struct BrainConnection {
    /// Whether connected to brain
    pub connected: bool,
    /// Brain address
    pub brain_addr: String,
    /// Channel to send requests to async runtime
    pub request_tx: mpsc::Sender<BrainRequest>,
    /// Channel to receive responses from async runtime
    pub response_rx: mpsc::Receiver<BrainResponse>,
    /// Model info (if connected)
    pub model_info: Option<ModelInfo>,
}

/// Request to send to the Brain
#[derive(Debug, Clone)]
pub enum BrainRequest {
    Connect,
    Disconnect,
    Think { prompt: String, history: Vec<trinity_protocol::types::ChatMessage> },
    /// Think with voice synthesis (returns audio for playback)
    ThinkWithVoice { prompt: String },
    Ping,
    GetModelInfo,
    SubmitTask {
        name: String,
        task_type: trinity_protocol::task::TaskType,
        priority: u8,
    },
    GetQueueStatus,
    ListPendingTasks,
    GetHardwareStats,
    PollEvents { since_id: u64 },
}

/// Response from the Brain
#[derive(Debug, Clone)]
pub enum BrainResponse {
    Connected { model_info: Option<ModelInfo> },
    Disconnected,
    ConnectionFailed { error: String },
    ThinkResult { result: Result<String, String> },
    /// Voice synthesis result with audio data
    VoiceResult {
        text: String,
        audio_data: Option<Vec<u8>>,
        sample_rate: u32,
    },
    PingResult { alive: bool },
    ModelInfo { info: Option<ModelInfo> },
    TaskSubmitted { task_id: uuid::Uuid },
    QueueStatus { status: trinity_protocol::task::QueueStatus },
    PendingTasks { tasks: Vec<trinity_protocol::task::TaskInfo> },
    HardwareStats { stats: trinity_protocol::types::HardwareStats },
    StreamEvents { events: Vec<StreamEvent> },
}

// ============================================================================
// Async Runtime Bridge
// ============================================================================

/// Spawns the async runtime that handles Brain communication
pub fn spawn_brain_runtime(brain_addr: String) -> (mpsc::Sender<BrainRequest>, mpsc::Receiver<BrainResponse>) {
    let (request_tx, mut request_rx) = mpsc::channel::<BrainRequest>(32);
    let (response_tx, response_rx) = mpsc::channel::<BrainResponse>(32);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        
        rt.block_on(async move {
            let mut client: Option<BrainServiceClient> = None;

            while let Some(request) = request_rx.recv().await {
                match request {
                    BrainRequest::Connect => {
                        match connect_to_brain(&brain_addr).await {
                            Ok(new_client) => {
                                // Get model info
                                let model_info = new_client
                                    .model_info(context::current())
                                    .await
                                    .ok()
                                    .flatten();
                                
                                client = Some(new_client);
                                let _ = response_tx.send(BrainResponse::Connected { model_info }).await;
                            }
                            Err(e) => {
                                let _ = response_tx.send(BrainResponse::ConnectionFailed {
                                    error: e.to_string(),
                                }).await;
                            }
                        }
                    }
                    
                    BrainRequest::Disconnect => {
                        client = None;
                        let _ = response_tx.send(BrainResponse::Disconnected).await;
                    }
                    
                    BrainRequest::Think { prompt, history } => {
                        if let Some(ref c) = client {
                            // Use chat() with conversation history
                            let message = trinity_protocol::types::ChatMessage {
                                role: "user".to_string(),
                                content: prompt,
                                timestamp: chrono::Utc::now().timestamp(),
                            };

                            // Set long deadline (5 min) for LLM inference
                            let mut ctx = context::current();
                            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
                            
                            tracing::info!("Sending chat with deadline: {:?}", ctx.deadline);
                            
                            let result = c
                                .chat(ctx, message, history)
                                .await;
                            
                            let result = match result {
                                Ok(text) => Ok(text),
                                Err(e) => Err(e.to_string()),
                            };
                            
                            let _ = response_tx.send(BrainResponse::ThinkResult { result }).await;
                        } else {
                            let _ = response_tx.send(BrainResponse::ThinkResult {
                                result: Err("Not connected to Brain".to_string()),
                            }).await;
                        }
                    }
                    
                    BrainRequest::ThinkWithVoice { prompt } => {
                        if let Some(ref c) = client {
                            let message = trinity_protocol::types::ChatMessage {
                                role: "user".to_string(),
                                content: prompt,
                                timestamp: chrono::Utc::now().timestamp(),
                            };

                            // Call chat_with_voice with audio synthesis enabled
                            match c.chat_with_voice(context::current(), message, true).await {
                                Ok(voice_resp) => {
                                    let (audio_data, sample_rate) = if let Some(ref audio) = voice_resp.audio {
                                        (Some(audio.audio_data.clone()), audio.sample_rate)
                                    } else {
                                        (None, 22050)
                                    };
                                    
                                    let _ = response_tx.send(BrainResponse::VoiceResult { 
                                        text: voice_resp.text,
                                        audio_data,
                                        sample_rate,
                                    }).await;
                                }
                                Err(e) => {
                                    let _ = response_tx.send(BrainResponse::ThinkResult {
                                        result: Err(e.to_string()),
                                    }).await;
                                }
                            }
                        } else {
                            let _ = response_tx.send(BrainResponse::ThinkResult {
                                result: Err("Not connected to Brain".to_string()),
                            }).await;
                        }
                    }
                    
                    BrainRequest::Ping => {
                        let alive = if let Some(ref c) = client {
                            c.ping(context::current()).await.unwrap_or(false)
                        } else {
                            false
                        };
                        let _ = response_tx.send(BrainResponse::PingResult { alive }).await;
                    }
                    
                    BrainRequest::GetModelInfo => {
                        let info = if let Some(ref c) = client {
                            c.model_info(context::current()).await.ok().flatten()
                        } else {
                            None
                        };
                        let _ = response_tx.send(BrainResponse::ModelInfo { info }).await;
                    }

                    BrainRequest::SubmitTask { name, task_type, priority } => {
                        if let Some(ref c) = client {
                            match c.submit_task(context::current(), name, task_type, priority).await {
                                Ok(Ok(task_id)) => {
                                    let _ = response_tx.send(BrainResponse::TaskSubmitted { task_id }).await;
                                }
                                Ok(Err(e)) => {
                                    tracing::error!("Brain protocol error submitting task: {:?}", e);
                                    let _ = response_tx.send(BrainResponse::ConnectionFailed { 
                                        error: format!("Protocol Error: {:?}", e) 
                                    }).await;
                                }
                                Err(e) => {
                                    tracing::error!("RPC error submitting task: {}", e);
                                    let _ = response_tx.send(BrainResponse::ConnectionFailed { 
                                        error: e.to_string() 
                                    }).await;
                                }
                            }
                        } else {
                             let _ = response_tx.send(BrainResponse::ConnectionFailed {
                                error: "Not connected".to_string(),
                            }).await;
                        }
                    }

                    BrainRequest::GetQueueStatus => {
                        if let Some(ref c) = client {
                            match c.get_queue_status(context::current()).await {
                                Ok(status) => {
                                    let _ = response_tx.send(BrainResponse::QueueStatus { status }).await;
                                }
                                Err(e) => {
                                    // Don't spam errors for polling
                                    tracing::debug!("Failed to get queue status: {}", e);
                                }
                            }
                        }
                    }

                    BrainRequest::ListPendingTasks => {
                        if let Some(ref c) = client {
                            match c.list_pending_tasks(context::current()).await {
                                Ok(tasks) => {
                                    let _ = response_tx.send(BrainResponse::PendingTasks { tasks }).await;
                                }
                                Err(e) => {
                                    tracing::debug!("Failed to list pending tasks: {}", e);
                                }
                            }
                        }
                    }
                    
                    BrainRequest::GetHardwareStats => {
                        if let Some(ref c) = client {
                            match c.get_hardware_stats(context::current()).await {
                                Ok(stats) => {
                                    let _ = response_tx.send(BrainResponse::HardwareStats { stats }).await;
                                }
                                Err(e) => {
                                    tracing::debug!("Failed to get hardware stats: {}", e);
                                }
                            }
                        }
                    }


                    BrainRequest::PollEvents { since_id } => {
                        if let Some(ref c) = client {
                            match c.poll_events(context::current(), since_id).await {
                                Ok(events) => {
                                    let _ = response_tx.send(BrainResponse::StreamEvents { events }).await;
                                }
                                Err(_e) => {
                                    // Quietly ignore polling errors
                                }
                            }
                        }
                    }
                }
            } // Close while loop
        });
    });

    (request_tx, response_rx)
}

async fn connect_to_brain(addr: &str) -> Result<BrainServiceClient> {
    let socket_addr: SocketAddr = addr.parse()?;
    let stream = TcpStream::connect(&socket_addr).await?;
    let codec = tarpc::tokio_util::codec::LengthDelimitedCodec::new();
    let framed = tarpc::tokio_util::codec::Framed::new(stream, codec);

    let transport = tarpc::serde_transport::new(framed, Bincode::default());

    let client = BrainServiceClient::new(client::Config::default(), transport).spawn();
    Ok(client)
}

// ============================================================================
// Bevy Systems
// ============================================================================

/// System to process responses from the Brain runtime
pub fn process_brain_responses(
    mut connection: ResMut<BrainConnection>,
    mut app_state: ResMut<crate::AppState>,
    mut task_panel: ResMut<crate::panels::TaskPanel>,
    mut hardware_panel: ResMut<crate::panels::HardwarePanel>,
    mut antigravity_panel: ResMut<crate::panels::AntigravityPanel>,
    audio: Option<Res<crate::audio::AudioResource>>,
) {
    while let Ok(response) = connection.response_rx.try_recv() {
        match response {
            BrainResponse::Connected { model_info } => {
                connection.connected = true;
                connection.model_info = model_info.clone();
                
                let model_name = model_info
                    .as_ref()
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                app_state.messages.push(crate::ChatMessage {
                    role: "system".to_string(),
                    content: format!("✅ Connected to Brain! Model: {}", model_name),
                });
                
                tracing::info!("Connected to Brain node");
            }
            
            BrainResponse::Disconnected => {
                connection.connected = false;
                connection.model_info = None;
                
                app_state.messages.push(crate::ChatMessage {
                    role: "system".to_string(),
                    content: "🔌 Disconnected from Brain".to_string(),
                });
            }
            
            BrainResponse::ConnectionFailed { error } => {
                connection.connected = false;
                
                app_state.messages.push(crate::ChatMessage {
                    role: "system".to_string(),
                    content: format!("❌ Connection failed: {}", error),
                });
                
                tracing::warn!("Failed to connect to Brain: {}", error);
            }
            
            BrainResponse::ThinkResult { result } => {
                app_state.avatar_state = crate::ProtocolAvatarState::Speaking;
                app_state.waiting_for_response = false;
                
                match result {
                    Ok(text) => {
                        app_state.messages.push(crate::ChatMessage {
                            role: "assistant".to_string(),
                            content: text,
                        });
                    }
                    Err(e) => {
                        app_state.messages.push(crate::ChatMessage {
                            role: "system".to_string(),
                            content: format!("⚠️ Error: {}", e),
                        });
                    }
                }
            }
            
            BrainResponse::VoiceResult { text, audio_data, sample_rate } => {
                app_state.avatar_state = crate::ProtocolAvatarState::Speaking;
                app_state.waiting_for_response = false;
                
                // Display message
                app_state.messages.push(crate::ChatMessage {
                    role: "assistant".to_string(),
                    content: text,
                });
                
                // Play audio if available
                if let Some(data) = audio_data {
                    if let Some(ref audio_res) = audio {
                        tracing::info!("Playing TTS audio: {} bytes at {} Hz", data.len(), sample_rate);
                        audio_res.play(data, sample_rate);
                    }
                }
            }
            
            BrainResponse::PingResult { alive } => {
                if !alive && connection.connected {
                    connection.connected = false;
                    app_state.messages.push(crate::ChatMessage {
                        role: "system".to_string(),
                        content: "⚠️ Lost connection to Brain".to_string(),
                    });
                }
            }
            
            BrainResponse::ModelInfo { info } => {
                connection.model_info = info;
            }

            BrainResponse::TaskSubmitted { task_id } => {
                 app_state.messages.push(crate::ChatMessage {
                    role: "system".to_string(),
                    content: format!("✅ Task Submitted! ID: {}", task_id),
                });
                // We could optimistically add it, but polling will catch it.
                // Trigger a poll immediately?
                let _ = connection.request_tx.try_send(BrainRequest::GetQueueStatus);
            }

            BrainResponse::QueueStatus { status: _ } => {
                // We could use this for summary counts, but PendingTasks is better for full list
            }

            BrainResponse::PendingTasks { tasks } => {
                task_panel.update_from_tasks(tasks);
            }

            BrainResponse::HardwareStats { stats } => {
                // Map protocol stats to UI panel stats
                hardware_panel.brain_stats = Some(crate::panels::hardware::HardwareStats {
                    cpu_usage: stats.cpu_percent,
                    ram_used_gb: stats.memory_used_bytes as f32 / (1024.0 * 1024.0 * 1024.0),
                    ram_total_gb: (stats.memory_used_bytes + stats.memory_available_bytes) as f32 / (1024.0 * 1024.0 * 1024.0), // Approx total
                    vram_used_gb: 0.0, // TODO: Get from GPU
                    vram_total_gb: 0.0,
                    gpu_temp_c: None,
                    inference_tokens_per_sec: None,
                });

            }

            BrainResponse::StreamEvents { events } => {
                for event in events.clone() {
                     antigravity_panel.process_event(event);
                }
                
                // Sync avatar state from agent events
                // Avatar reflects the most "active" state from any agent
                for event in &events {
                    match event {
                        trinity_protocol::StreamEvent::Thinking { .. } => {
                            if app_state.avatar_state != crate::ProtocolAvatarState::Coding {
                                app_state.avatar_state = crate::ProtocolAvatarState::Thinking;
                            }
                        }
                        trinity_protocol::StreamEvent::CodeGenerated { .. } => {
                            app_state.avatar_state = crate::ProtocolAvatarState::Coding;
                        }
                        trinity_protocol::StreamEvent::CommandRunning { .. } => {
                            app_state.avatar_state = crate::ProtocolAvatarState::Coding;
                        }
                        trinity_protocol::StreamEvent::TaskCompleted { .. } => {
                            // Only go idle if nothing else is running
                            let any_busy = antigravity_panel.agents.iter().any(|a| a.is_busy);
                            if !any_busy {
                                app_state.avatar_state = crate::ProtocolAvatarState::Idle;
                            }
                        }
                        trinity_protocol::StreamEvent::TaskFailed { .. } => {
                            // Briefly show idle on failure
                            app_state.avatar_state = crate::ProtocolAvatarState::Idle;
                        }
                        _ => {}
                    }
                }
            }

        }
    }
}

/// System to send periodic pings to the Brain
pub fn ping_brain_system(
    connection: Res<BrainConnection>,
    time: Res<Time>,
    mut last_ping: Local<f32>,
) {
    const PING_INTERVAL: f32 = 2.0; // Ping every 2 seconds (faster for hardware stats)
    
    *last_ping += time.delta_seconds();
    
    if *last_ping >= PING_INTERVAL && connection.connected {
        *last_ping = 0.0;
        let _ = connection.request_tx.try_send(BrainRequest::Ping);
        let _ = connection.request_tx.try_send(BrainRequest::GetQueueStatus);
        let _ = connection.request_tx.try_send(BrainRequest::ListPendingTasks);
        let _ = connection.request_tx.try_send(BrainRequest::GetHardwareStats);
        let _ = connection.request_tx.try_send(BrainRequest::PollEvents { since_id: 0 }); // Todo: Track since_id
    }
}
/// Handle requests from Skill Plugins (e.g. Coder, Writer)
pub fn handle_skill_requests(
    connection: Res<BrainConnection>,
    mut code_requests: EventReader<RequestCodeGeneration>,
    mut write_requests: EventReader<RequestWriting>,
) {
    if !connection.connected {
        return;
    }

    for req in code_requests.read() {
        tracing::info!("Bridge: Forwarding Code Generation Request to Brain");
        let _ = connection.request_tx.try_send(BrainRequest::SubmitTask {
            name: format!("Generate {} Code", req.language),
            task_type: TaskType::GenerateCode {
                prompt: req.prompt.clone(),
                language: req.language.clone(),
                output_path: req.output_path.clone(),
            },
            priority: 1, // Normal
        });
    }

    for req in write_requests.read() {
        tracing::info!("Bridge: Forwarding Writing Request to Brain");
        let prompt = format!(
            "Write a {} document about '{}'. Target word count: {}.",
            req.style, req.topic, req.target_words
        );

        let _ = connection.request_tx.try_send(BrainRequest::SubmitTask {
            name: format!("Write: {}", req.topic),
            task_type: TaskType::Think { prompt },
            priority: 1,
        });
    }
}
