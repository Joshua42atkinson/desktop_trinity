//! Brain Resource - Bevy ECS Integration for Brain Trait
//!
//! Provides the Brain as a Bevy Resource for access from ECS systems.
//! Handles async/sync bridging for agent execution.

use bevy::prelude::*;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::brain::{Brain, GenerationConfig, StreamToken};

// ============================================================================
// Brain Resource
// ============================================================================

/// Bevy Resource wrapper for the Brain
///
/// Provides access to the AI inference backend from Bevy ECS systems.
/// Use `BrainInterface` for sync interactions in Bevy systems.
#[derive(Resource, Clone)]
pub struct BrainResource {
    brain: Arc<dyn Brain>,
}

impl BrainResource {
    /// Create a new BrainResource from an Arc<dyn Brain>
    pub fn new(brain: Arc<dyn Brain>) -> Self {
        Self { brain }
    }

    /// Get the underlying brain (for async contexts)
    pub fn brain(&self) -> &Arc<dyn Brain> {
        &self.brain
    }

    /// Check if a model is loaded
    pub fn is_ready(&self) -> bool {
        self.brain.is_ready()
    }

    /// Get model info
    pub fn model_info(&self) -> Option<crate::brain::ModelInfo> {
        self.brain.model_info()
    }

    /// Get the brain implementation name
    pub fn name(&self) -> &'static str {
        self.brain.name()
    }
}

// ============================================================================
// Brain Request/Response for Agent Communication
// ============================================================================

/// A request for the Brain to process
#[derive(Debug, Clone)]
pub struct ThinkRequest {
    /// Unique request ID
    pub id: uuid::Uuid,
    /// The prompt to process
    pub prompt: String,
    /// Optional generation config
    pub config: Option<GenerationConfig>,
    /// Whether to stream the response
    pub stream: bool,
}

impl ThinkRequest {
    /// Create a new think request
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            prompt: prompt.into(),
            config: None,
            stream: false,
        }
    }

    /// Set streaming mode
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    /// Set generation config
    pub fn with_config(mut self, config: GenerationConfig) -> Self {
        self.config = Some(config);
        self
    }
}

/// Response from the Brain
#[derive(Debug, Clone)]
pub struct ThinkResponse {
    /// Request ID this is responding to
    pub request_id: uuid::Uuid,
    /// The generated response
    pub response: String,
    /// Whether this was a streaming response
    pub streamed: bool,
    /// Processing time in milliseconds
    pub duration_ms: u64,
}

// ============================================================================
// Brain Interface (Sync Wrapper for Bevy Systems)
// ============================================================================

/// Sync interface for Brain operations from Bevy systems
///
/// Provides non-blocking request submission and polling for responses.
#[derive(Resource)]
pub struct BrainInterface {
    /// Channel to submit think requests
    request_tx: mpsc::Sender<ThinkRequest>,
    /// Channel to receive completed responses
    response_rx: std::sync::Mutex<mpsc::Receiver<ThinkResponse>>,
    /// Channel to receive streaming tokens
    token_rx: std::sync::Mutex<mpsc::Receiver<StreamToken>>,
}

impl BrainInterface {
    /// Create a new BrainInterface with async processing
    pub fn new(brain: Arc<dyn Brain>) -> Self {
        let (request_tx, mut request_rx) = mpsc::channel::<ThinkRequest>(32);
        let (response_tx, response_rx) = mpsc::channel::<ThinkResponse>(32);
        let (_token_tx, token_rx) = mpsc::channel::<StreamToken>(256);

        // Spawn async task to process requests
        let brain_clone = brain.clone();
        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                let start = std::time::Instant::now();

                let response = if request.stream {
                    // Streaming mode
                    let (stream_tx, _stream_rx) = mpsc::channel(64);
                    brain_clone.think_stream(&request.prompt, stream_tx).await
                } else if let Some(config) = &request.config {
                    // With config
                    brain_clone.think_with_config(&request.prompt, config).await
                } else {
                    // Simple think
                    brain_clone.think(&request.prompt).await
                };

                match response {
                    Ok(text) => {
                        let _ = response_tx
                            .send(ThinkResponse {
                                request_id: request.id,
                                response: text,
                                streamed: request.stream,
                                duration_ms: start.elapsed().as_millis() as u64,
                            })
                            .await;
                    }
                    Err(e) => {
                        tracing::error!("Brain error: {}", e);
                        let _ = response_tx
                            .send(ThinkResponse {
                                request_id: request.id,
                                response: format!("[Error: {}]", e),
                                streamed: request.stream,
                                duration_ms: start.elapsed().as_millis() as u64,
                            })
                            .await;
                    }
                }
            }
        });

        Self {
            request_tx,
            response_rx: std::sync::Mutex::new(response_rx),
            token_rx: std::sync::Mutex::new(token_rx),
        }
    }

    /// Submit a think request (non-blocking)
    pub fn submit(&self, request: ThinkRequest) -> bool {
        self.request_tx.try_send(request).is_ok()
    }

    /// Poll for completed responses (non-blocking)
    pub fn poll_response(&self) -> Option<ThinkResponse> {
        if let Ok(mut rx) = self.response_rx.try_lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }

    /// Poll for streaming tokens (non-blocking)
    pub fn poll_tokens(&self) -> Vec<StreamToken> {
        let mut tokens = Vec::new();
        if let Ok(mut rx) = self.token_rx.try_lock() {
            while let Ok(token) = rx.try_recv() {
                tokens.push(token);
            }
        }
        tokens
    }
}

// ============================================================================
// Bevy Plugin
// ============================================================================

/// Plugin to add Brain resources to a Bevy app
pub struct BrainPlugin {
    brain: Arc<dyn Brain>,
}

impl BrainPlugin {
    pub fn new(brain: Arc<dyn Brain>) -> Self {
        Self { brain }
    }
}

impl Plugin for BrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BrainResource::new(self.brain.clone()));
        app.insert_resource(BrainInterface::new(self.brain.clone()));
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::brain::MockBrain;

    #[test]
    fn test_think_request_builder() {
        let request = ThinkRequest::new("Hello").with_streaming();

        assert!(request.stream);
        assert_eq!(request.prompt, "Hello");
    }
}
