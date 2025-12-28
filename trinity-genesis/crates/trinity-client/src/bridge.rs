// Trinity Bridge - Connects Client (Body) to Brain (Mind)
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use trinity_protocol::types::{ChatMessage, ModelInfo};
use uuid::Uuid;

// Web-Specific Imports
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

// Native Imports
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;

/// Resource to hold the connection state
#[derive(Resource)]
pub struct BrainConnection {
    pub connected: bool,
    pub brain_addr: String,
    pub model_info: Option<ModelInfo>,
    // Channel to send requests to the async runtime
    pub request_tx: crossbeam_channel::Sender<BrainRequest>,
    // Channel to receive responses from the async runtime
    pub response_rx: crossbeam_channel::Receiver<BrainResponse>,
}

#[derive(Debug, Clone)]
pub enum BrainRequest {
    Connect,
    Disconnect,
    Think {
        prompt: String,
        history: Vec<ChatMessage>,
    },
    // Add other requests as needed
}

#[derive(Debug, Clone)]
pub enum BrainResponse {
    Connected(bool),
    Snapshot(String), // Token streaming
    ThoughtComplete(String),
    Error(String),
}

/// Spawns the async runtime logic
pub fn spawn_brain_runtime(brain_addr: String) -> (crossbeam_channel::Sender<BrainRequest>, crossbeam_channel::Receiver<BrainResponse>) {
    let (req_tx, req_rx) = crossbeam_channel::unbounded();
    let (res_tx, res_rx) = crossbeam_channel::unbounded();

    let brain_addr_clone = brain_addr.clone();

    #[cfg(target_arch = "wasm32")]
    {
        spawn_local(async move {
            web_brain_loop(brain_addr_clone, req_rx, res_tx).await;
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                native_brain_loop(brain_addr_clone, req_rx, res_tx).await;
            });
        });
    }

    (req_tx, res_rx)
}

// ----------------------------------------------------------------------------
// Web Implementation (HTTP/Fetch)
// ----------------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
async fn web_brain_loop(
    addr: String,
    req_rx: crossbeam_channel::Receiver<BrainRequest>,
    res_tx: crossbeam_channel::Sender<BrainResponse>,
) {
    info!("🌐 Starting Web Brain Loop targeting {}", addr);
    
    // In WASM, we can't block on recv, so we need a different approach OR
    // we use a channel specific to WASM. Crossbeam might block.
    // Actually, for Bevy WASM, we usually polling system.
    // For now, let's assume we consume all events per frame or similar.
    // Ideally this loop runs continuously.
    
    // NOTE: Crossbeam recv() blocks thread, which panics in WASM main thread.
    // We need an async channel or polling.
    // Refactor: Simplification for prototype -> Direct HTTP calls?
    // Let's implement a simple poller.
    
    loop {
        // Non-blocking try_recv
        match req_rx.try_recv() {
            Ok(req) => {
                match req {
                    BrainRequest::Connect => {
                        // Ping HTTP endpoint
                        let client = reqwest::Client::new();
                        let url = format!("http://{}/health", addr); // Assuming health endpoint
                        match client.get(&url).send().await {
                            Ok(_) => {
                                let _ = res_tx.send(BrainResponse::Connected(true));
                            }
                             Err(e) => {
                                let _ = res_tx.send(BrainResponse::Error(e.to_string()));
                            }
                        }
                    }
                    BrainRequest::Think { prompt, .. } => {
                        // Use Axum API
                        // ... implementation ...
                    }
                    _ => {}
                }
            }
            Err(_) => {
                // Sleep specifically for WASM
                // gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Native Implementation (Tarpc)
// ----------------------------------------------------------------------------
#[cfg(not(target_arch = "wasm32"))]
async fn native_brain_loop(
    addr: String,
    req_rx: crossbeam_channel::Receiver<BrainRequest>,
    res_tx: crossbeam_channel::Sender<BrainResponse>,
) {
    info!("🖥️ Starting Native Brain Loop targeting {}", addr);
    
    // Native loop logic (reuse from trinity-body)
    loop {
        if let Ok(req) = req_rx.recv() {
             match req {
                BrainRequest::Connect => {
                    // Tarpc connect logic
                     let _ = res_tx.send(BrainResponse::Connected(true));
                }
                BrainRequest::Think { prompt, .. } => {
                     // Tarpc think logic
                     let _ = res_tx.send(BrainResponse::ThoughtComplete("Thinking...".to_string()));
                }
                _ => {}
             }
        }
    }
}
