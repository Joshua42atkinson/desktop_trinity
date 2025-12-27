//! LM Studio Brain - Brain trait adapter for LM Studio OpenAI-compatible API
//!
//! Enables Trinity's self-coding agent to use LM Studio's local inference
//! as a drop-in replacement for native llama.cpp inference.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use crate::ai::lmstudio_client::{ChatMessage, LMStudioClient};
use trinity_core::brain::{Brain, GenerationConfig, ModelInfo, StreamToken};

// ============================================================================
// Circuit Breaker (Cascading Failure Prevention)
// ============================================================================

/// Simple circuit breaker for cascading failure prevention
pub struct CircuitBreaker {
    failures: AtomicU32,
    threshold: u32,
    last_failure: Mutex<Option<Instant>>,
    recovery_time: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, recovery_time: Duration) -> Self {
        Self {
            failures: AtomicU32::new(0),
            threshold,
            last_failure: Mutex::new(None),
            recovery_time,
        }
    }

    pub fn is_open(&self) -> bool {
        let failures = self.failures.load(Ordering::Relaxed);
        if failures >= self.threshold {
            // Check if recovery time has passed
            if let Ok(guard) = self.last_failure.lock() {
                if let Some(last) = *guard {
                    if last.elapsed() < self.recovery_time {
                        return true; // Still in recovery
                    }
                }
            }
            // Recovery time passed, allow retry
            self.failures.store(0, Ordering::Relaxed);
        }
        false
    }

    pub fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut guard) = self.last_failure.lock() {
            *guard = Some(Instant::now());
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(3, Duration::from_secs(60))
    }
}

// ============================================================================
// Robust LM Studio Client
// ============================================================================

/// Production-hardened LM Studio client with retries and circuit breaker
pub struct RobustLMStudioClient {
    inner: LMStudioClient,
    max_retries: u32,
    base_delay: Duration,
    circuit_breaker: CircuitBreaker,
}

impl RobustLMStudioClient {
    /// Create a new robust client wrapping an LMStudioClient
    pub fn new(inner: LMStudioClient) -> Self {
        Self {
            inner,
            max_retries: 3,
            base_delay: Duration::from_secs(2),
            circuit_breaker: CircuitBreaker::default(),
        }
    }

    /// Create with default LMStudioClient settings
    pub fn default_client() -> Self {
        Self::new(LMStudioClient::new())
    }

    /// Set model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.inner = self.inner.with_model(model);
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set base delay for exponential backoff
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Complete with exponential backoff retry
    pub async fn complete_with_retry(&self, prompt: &str) -> Result<String> {
        self.with_retry(|| async { self.inner.complete(prompt).await })
            .await
    }

    /// Chat with config and retry
    pub async fn chat_with_config_retry(
        &self,
        messages: &[ChatMessage],
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String> {
        let msgs = messages.to_vec();
        self.with_retry(|| async {
            self.inner
                .chat_with_config(&msgs, temperature, max_tokens)
                .await
        })
        .await
    }

    /// Retry wrapper with exponential backoff
    async fn with_retry<F, Fut>(&self, f: F) -> Result<String>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<String>>,
    {
        // Check circuit breaker
        if self.circuit_breaker.is_open() {
            anyhow::bail!("Circuit breaker open - LM Studio unavailable (too many failures)");
        }

        let mut last_error = None;
        for attempt in 0..self.max_retries {
            match f().await {
                Ok(result) => {
                    self.circuit_breaker.record_success();
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    self.circuit_breaker.record_failure();

                    if attempt + 1 < self.max_retries {
                        let delay = self.base_delay * 2u32.pow(attempt);
                        log::warn!(
                            "LM Studio request failed (attempt {}/{}), retrying in {:?}: {}",
                            attempt + 1,
                            self.max_retries,
                            delay,
                            last_error.as_ref().unwrap()
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Max retries exceeded")))
    }

    /// Health check ping
    pub async fn health_check(&self) -> bool {
        self.inner.is_available().await
    }

    /// Get list of available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        self.inner.list_models().await
    }
}

// ============================================================================
// LM Studio Brain (Brain Trait Implementation)
// ============================================================================

/// LM Studio Brain - implements Brain trait for LM Studio API
pub struct LMStudioBrain {
    client: RobustLMStudioClient,
    model: String,
}

impl LMStudioBrain {
    /// Create a new LMStudioBrain with default settings
    pub fn new() -> Self {
        Self {
            client: RobustLMStudioClient::default_client(),
            model: "openai/gpt-oss-120b".to_string(),
        }
    }

    /// Create with a custom robust client
    pub fn with_client(client: RobustLMStudioClient, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Self {
        let model =
            std::env::var("LM_STUDIO_MODEL").unwrap_or_else(|_| "openai/gpt-oss-120b".to_string());
        let endpoint =
            std::env::var("LM_STUDIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:1234".into());

        let inner_client = LMStudioClient::with_endpoint(endpoint).with_model(model.clone());

        Self {
            client: RobustLMStudioClient::new(inner_client)
                .with_max_retries(3)
                .with_base_delay(Duration::from_secs(2)),
            model,
        }
    }

    /// Check if LM Studio is available
    pub async fn is_available(&self) -> bool {
        self.client.health_check().await
    }
}

impl Default for LMStudioBrain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Brain for LMStudioBrain {
    async fn think(&self, prompt: &str) -> Result<String> {
        log::debug!("LMStudioBrain: thinking on prompt ({} chars)", prompt.len());
        self.client.complete_with_retry(prompt).await
    }

    async fn think_with_config(&self, prompt: &str, config: &GenerationConfig) -> Result<String> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        self.client
            .chat_with_config_retry(&messages, config.temperature, config.max_tokens)
            .await
    }

    async fn think_stream(
        &self,
        prompt: &str,
        token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String> {
        // LM Studio API doesn't support streaming in our current client,
        // so we just return the full response as one token
        let response = self.think(prompt).await?;
        let _ = token_tx
            .send(StreamToken {
                text: response.clone(),
                is_final: true,
                index: 0,
            })
            .await;
        Ok(response)
    }

    async fn load_model(&self, model_path: &str) -> Result<()> {
        // LM Studio manages models via its own UI, so this is a no-op
        // We just log what model would be requested
        log::info!(
            "LMStudioBrain: Model switch requested to '{}' (managed by LM Studio UI)",
            model_path
        );
        Ok(())
    }

    fn model_info(&self) -> Option<ModelInfo> {
        Some(ModelInfo {
            name: self.model.clone(),
            path: "http://localhost:1234".to_string(),
            size_bytes: 0, // Unknown for remote API
            quantization: "API".to_string(),
            context_size: 32768, // Typical for large models
            loaded: true,        // Assume loaded if we got this far
        })
    }

    fn name(&self) -> &'static str {
        "LMStudioBrain"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        assert!(!cb.is_open());
    }

    #[test]
    fn test_circuit_breaker_opens() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_open());
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        assert!(!cb.is_open());
    }

    #[test]
    fn test_lmstudio_brain_creation() {
        let brain = LMStudioBrain::new();
        assert_eq!(brain.name(), "LMStudioBrain");
        assert!(brain.model_info().is_some());
    }
}
