//! Desktop Brain - Native LLM Inference using llama.cpp and ROCm/HIP
//!
//! Provides the primary inference backend for Trinity on AMD Strix Halo hardware.
//! Uses llama-cpp-2 Rust bindings for efficient GGUF model loading and generation.

use crate::brain::{Brain, GenerationConfig, ModelInfo, StreamToken};
use crate::config::TrinityConfig;
use anyhow::{Context, Result};
use async_trait::async_trait;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// ============================================================================
// Brain State
// ============================================================================

/// Internal state holding the loaded model and context
struct BrainState {
    model: &'static LlamaModel,
    context: LlamaContext<'static>,
    model_path: String,
    model_size: u64,
}

// We use 'static lifetimes by leaking the model.
// This is acceptable for a singleton Brain application.
unsafe impl Send for BrainState {}
unsafe impl Sync for BrainState {}

// ============================================================================
// Desktop Brain
// ============================================================================

/// Desktop-native Brain implementation using llama.cpp and ROCm
pub struct DesktopBrain {
    backend: Arc<LlamaBackend>,
    state: Arc<Mutex<Option<BrainState>>>,
    config: TrinityConfig,
    gpu_layers: i32,
}

impl DesktopBrain {
    /// Create a new DesktopBrain with default config
    pub fn new() -> Self {
        let config = TrinityConfig::load()
            .unwrap_or_default()
            .with_env_overrides();
        Self::with_config(config)
    }

    /// Create with specific config
    pub fn with_config(config: TrinityConfig) -> Self {
        // Set HSA override for Strix Halo before initializing backend
        std::env::set_var(
            "HSA_OVERRIDE_GFX_VERSION",
            &config.hardware.hsa_override_version,
        );

        // Initialize the llama.cpp backend
        let backend = Arc::new(LlamaBackend::init().expect("Failed to init llama backend"));

        // Attempt to load default model
        let default_path = config
            .models
            .default_model_path
            .to_string_lossy()
            .to_string();
        let state = if Path::new(&default_path).exists() {
            match Self::init_state(&backend, &default_path, config.models.context_size, 999) {
                Ok(s) => {
                    tracing::info!("✓ Loaded default model: {}", default_path);
                    Some(s)
                }
                Err(e) => {
                    tracing::warn!("Failed to load default model: {}", e);
                    None
                }
            }
        } else {
            tracing::info!("No default model found at: {}", default_path);
            None
        };

        Self {
            backend,
            state: Arc::new(Mutex::new(state)),
            config,
            gpu_layers: 999, // Default to full offload for Strix Halo
        }
    }

    /// Set the number of GPU layers to offload
    pub fn with_gpu_layers(mut self, layers: i32) -> Self {
        self.gpu_layers = layers;
        self
    }

    /// Initialize model state from a file path
    fn init_state(
        backend: &Arc<LlamaBackend>,
        path: &str,
        context_size: u32,
        gpu_layers: i32,
    ) -> Result<BrainState> {
        tracing::info!("Loading model from: {}", path);

        // Get file size
        let model_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        // Configure model params for Strix Halo
        let model_params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers as u32);

        let model = LlamaModel::load_from_file(backend, Path::new(path), &model_params)
            .with_context(|| format!("Failed to load model from {}", path))?;

        // Leak model to get 'static reference (singleton pattern)
        let model_ref: &'static LlamaModel = Box::leak(Box::new(model));

        // Create context with configured size
        let ctx_params = LlamaContextParams::default().with_n_ctx(NonZeroU32::new(context_size));

        let context = model_ref
            .new_context(backend, ctx_params)
            .with_context(|| "Failed to create context")?;

        tracing::info!(
            "✓ Model loaded: {:.1} GB, {} ctx",
            model_size as f64 / (1024.0 * 1024.0 * 1024.0),
            context_size
        );

        Ok(BrainState {
            model: model_ref,
            context,
            model_path: path.to_string(),
            model_size,
        })
    }

    /// Generate text from the model using greedy sampling
    fn generate_greedy(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        let mut state_guard = self.state.lock().unwrap();
        let state = state_guard
            .as_mut()
            .context("No model loaded. Call load_model() first.")?;

        // Tokenize the prompt
        let tokens = state
            .model
            .str_to_token(prompt, AddBos::Always)
            .context("Failed to tokenize prompt")?;

        tracing::debug!("Prompt tokenized: {} tokens", tokens.len());

        // Clear KV cache for fresh generation
        state.context.clear_kv_cache();

        // Create batch and add prompt tokens
        let mut batch = LlamaBatch::new(512, 1);
        let last_idx = tokens.len() - 1;
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], i == last_idx)?;
        }

        // Process prompt (prefill)
        state.context.decode(&mut batch)?;

        // Generation loop with greedy sampling
        let mut output_text = String::new();
        let n_cur = tokens.len();

        for i in 0..max_tokens as usize {
            // Get token data array for the last position and sample greedily
            let mut candidates = state.context.token_data_array_ith(batch.n_tokens() - 1);
            let new_token = candidates.sample_token_greedy();

            // Check for EOS
            if new_token == state.model.token_eos() {
                tracing::debug!("EOS token reached after {} tokens", i);
                break;
            }

            // Decode token to text
            let token_text = state
                .model
                .token_to_str(new_token, Special::Tokenize)
                .unwrap_or_default();

            output_text.push_str(&token_text);

            // Prepare next batch
            batch.clear();
            batch.add(new_token, (n_cur + i) as i32, &[0], true)?;

            // Decode
            state.context.decode(&mut batch)?;
        }

        Ok(output_text)
    }

    /// Generate text with true token-by-token streaming
    ///
    /// Sends each token through the provided sender as it's generated.
    /// Returns the complete response after generation finishes.
    ///
    /// TODO: Wire up to UI streaming in Phase 6
    #[allow(dead_code)]
    fn generate_streaming(
        &self,
        prompt: &str,
        max_tokens: u32,
        token_sender: &std::sync::mpsc::Sender<(String, bool)>,
    ) -> Result<String> {
        let mut state_guard = self.state.lock().unwrap();
        let state = state_guard
            .as_mut()
            .context("No model loaded. Call load_model() first.")?;

        // Tokenize the prompt
        let tokens = state
            .model
            .str_to_token(prompt, AddBos::Always)
            .context("Failed to tokenize prompt")?;

        tracing::debug!(
            "Streaming generation: {} prompt tokens, max {} output",
            tokens.len(),
            max_tokens
        );

        // Clear KV cache for fresh generation
        state.context.clear_kv_cache();

        // Create batch and add prompt tokens
        let mut batch = LlamaBatch::new(512, 1);
        let last_idx = tokens.len() - 1;
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], i == last_idx)?;
        }

        // Process prompt (prefill)
        state.context.decode(&mut batch)?;

        // Generation loop with greedy sampling + streaming
        let mut output_text = String::new();
        let n_cur = tokens.len();

        for i in 0..max_tokens as usize {
            // Get token data array for the last position and sample greedily
            let mut candidates = state.context.token_data_array_ith(batch.n_tokens() - 1);
            let new_token = candidates.sample_token_greedy();

            // Check for EOS
            let is_eos = new_token == state.model.token_eos();

            if is_eos {
                tracing::debug!("EOS token reached after {} tokens", i);
                // Send final empty token to signal completion
                let _ = token_sender.send((String::new(), true));
                break;
            }

            // Decode token to text
            let token_text = state
                .model
                .token_to_str(new_token, Special::Tokenize)
                .unwrap_or_default();

            output_text.push_str(&token_text);

            // Stream this token (is_final = false since we're not at EOS)
            let is_final = i == (max_tokens as usize - 1);
            if token_sender.send((token_text, is_final)).is_err() {
                // Receiver dropped, stop generation
                tracing::debug!("Token receiver dropped, stopping generation");
                break;
            }

            // Prepare next batch
            batch.clear();
            batch.add(new_token, (n_cur + i) as i32, &[0], true)?;

            // Decode
            state.context.decode(&mut batch)?;
        }

        Ok(output_text)
    }

    /// Get the size of the currently loaded model in bytes
    pub fn model_size(&self) -> Option<u64> {
        let state_guard = self.state.lock().ok()?;
        state_guard.as_ref().map(|s| s.model_size)
    }

    /// Get the path of the currently loaded model
    pub fn model_path(&self) -> Option<String> {
        let state_guard = self.state.lock().ok()?;
        state_guard.as_ref().map(|s| s.model_path.clone())
    }

    /// Unload the current model
    ///
    /// NOTE: Due to llama-cpp-2's lifetime requirements, the actual model memory
    /// is not freed (Box::leak pattern). This method clears the state to allow
    /// loading a new model, but the old model memory remains allocated until
    /// process exit. For true hot-swap, a future version should use a different
    /// memory management strategy.
    pub fn unload_model(&self) -> Option<u64> {
        let mut state_guard = self.state.lock().ok()?;

        if let Some(state) = state_guard.take() {
            let size = state.model_size;
            let path = state.model_path;

            // Clear the state (context will be dropped, but leaked model remains)
            tracing::warn!(
                "Unloading model {} ({:.1} GB) - NOTE: memory not freed due to lifetime constraints",
                path,
                size as f64 / (1024.0 * 1024.0 * 1024.0)
            );

            Some(size)
        } else {
            None
        }
    }

    /// Check if a model is currently loaded
    pub fn has_model(&self) -> bool {
        self.state.lock().map(|s| s.is_some()).unwrap_or(false)
    }
}

impl Default for DesktopBrain {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Brain Trait Implementation
// ============================================================================

#[async_trait]
impl Brain for DesktopBrain {
    async fn think(&self, prompt: &str) -> Result<String> {
        // Check if model is loaded
        {
            let state = self.state.lock().unwrap();
            if state.is_none() {
                return Ok(format!(
                    "[No model loaded] Would process: {}...",
                    &prompt.chars().take(50).collect::<String>()
                ));
            }
        }

        // Run generation (blocking, but fast for small outputs)
        self.generate_greedy(prompt, 2048)
    }

    async fn think_with_config(&self, prompt: &str, config: &GenerationConfig) -> Result<String> {
        // For now, just use max_tokens from config
        {
            let state = self.state.lock().unwrap();
            if state.is_none() {
                return Ok(format!(
                    "[No model loaded] Would process: {}...",
                    &prompt.chars().take(50).collect::<String>()
                ));
            }
        }

        self.generate_greedy(prompt, config.max_tokens)
    }

    async fn think_stream(
        &self,
        prompt: &str,
        token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String> {
        // Check if model is loaded (drop lock before any await)
        let has_model = {
            let state = self.state.lock().unwrap();
            state.is_some()
        };

        if !has_model {
            let msg = format!(
                "[No model loaded] Would process: {}...",
                &prompt.chars().take(50).collect::<String>()
            );
            let _ = token_tx
                .send(StreamToken {
                    text: msg.clone(),
                    is_final: true,
                    index: 0,
                })
                .await;
            return Ok(msg);
        }

        // Create sync channel for streaming tokens from blocking thread
        let (sync_tx, sync_rx) = std::sync::mpsc::channel::<(String, bool)>();

        // Clone what we need for the blocking task
        let state_clone = self.state.clone();
        let prompt_owned = prompt.to_string();

        // Spawn blocking task for generation
        let generation_handle = tokio::task::spawn_blocking(move || {
            let mut state_guard = state_clone.lock().unwrap();
            let state = match state_guard.as_mut() {
                Some(s) => s,
                None => return Err(anyhow::anyhow!("No model loaded")),
            };

            // Tokenize
            let tokens = state
                .model
                .str_to_token(&prompt_owned, AddBos::Always)
                .context("Failed to tokenize")?;

            state.context.clear_kv_cache();

            let mut batch = LlamaBatch::new(512, 1);
            let last_idx = tokens.len() - 1;
            for (i, token) in tokens.iter().enumerate() {
                batch.add(*token, i as i32, &[0], i == last_idx)?;
            }

            state.context.decode(&mut batch)?;

            let mut output_text = String::new();
            let n_cur = tokens.len();
            let max_tokens = 2048u32;

            for i in 0..max_tokens as usize {
                let mut candidates = state.context.token_data_array_ith(batch.n_tokens() - 1);
                let new_token = candidates.sample_token_greedy();

                if new_token == state.model.token_eos() {
                    let _ = sync_tx.send((String::new(), true));
                    break;
                }

                let token_text = state
                    .model
                    .token_to_str(new_token, Special::Tokenize)
                    .unwrap_or_default();

                output_text.push_str(&token_text);

                let is_final = i == (max_tokens as usize - 1);
                if sync_tx.send((token_text, is_final)).is_err() {
                    break;
                }

                batch.clear();
                batch.add(new_token, (n_cur + i) as i32, &[0], true)?;
                state.context.decode(&mut batch)?;
            }

            Ok(output_text)
        });

        // Forward tokens from sync channel to async channel
        let mut full_response = String::new();
        let mut index = 0;

        loop {
            // Try to receive with a small timeout
            match sync_rx.recv_timeout(std::time::Duration::from_millis(10)) {
                Ok((text, is_final)) => {
                    full_response.push_str(&text);
                    let _ = token_tx
                        .send(StreamToken {
                            text,
                            is_final,
                            index,
                        })
                        .await;
                    index += 1;

                    if is_final {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Check if generation task is done
                    if generation_handle.is_finished() {
                        break;
                    }
                    // Yield to allow other tasks
                    tokio::task::yield_now().await;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        // Wait for generation to complete and get result
        match generation_handle.await {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(anyhow::anyhow!("Generation task panicked: {}", e)),
        }
    }

    async fn load_model(&self, model_path: &str) -> Result<()> {
        let mut state_guard = self.state.lock().unwrap();

        tracing::info!("Loading model: {}", model_path);

        let state = Self::init_state(
            &self.backend,
            model_path,
            self.config.models.context_size,
            self.gpu_layers,
        )?;
        *state_guard = Some(state);

        Ok(())
    }

    fn model_info(&self) -> Option<ModelInfo> {
        let state_guard = self.state.lock().ok()?;
        let state = state_guard.as_ref()?;

        // Extract model name from path
        let name = Path::new(&state.model_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Try to detect quantization from filename
        let quantization = if state.model_path.contains("Q3_K") {
            "Q3_K_L"
        } else if state.model_path.contains("Q4_K") {
            "Q4_K_M"
        } else if state.model_path.contains("Q8") {
            "Q8_0"
        } else {
            "Unknown"
        }
        .to_string();

        Some(ModelInfo {
            name,
            path: state.model_path.clone(),
            size_bytes: state.model_size,
            quantization,
            context_size: self.config.models.context_size,
            loaded: true,
        })
    }

    fn name(&self) -> &'static str {
        "DesktopBrain (llama.cpp + ROCm)"
    }
}
