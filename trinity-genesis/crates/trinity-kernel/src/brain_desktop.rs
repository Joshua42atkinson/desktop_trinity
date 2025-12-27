//! Desktop Brain - Native LLM Inference using llama.cpp and ROCm/HIP
//!
//! Provides the primary inference backend for Trinity Genesis on AMD Strix Halo hardware.
//! Uses llama-cpp-2 Rust bindings for efficient GGUF model loading and generation.

use crate::brain::{Brain, GrammarSpec, StreamToken};
use anyhow::{Context, Result};
use async_trait::async_trait;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc;

/// Shared global backend to prevent double-initialization
static GLOBAL_BACKEND: OnceLock<Arc<LlamaBackend>> = OnceLock::new();

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for DesktopBrain
#[derive(Debug, Clone)]
pub struct DesktopBrainConfig {
    /// Path to the model file (GGUF format)
    pub model_path: String,
    /// Context size (tokens)
    pub context_size: u32,
    /// Number of GPU layers to offload (-1 = all)
    pub n_gpu_layers: i32,
    /// HSA override version for AMD GPUs
    pub hsa_override: String,
    /// Maximum tokens to generate
    pub max_tokens: u32,
}

impl Default for DesktopBrainConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            context_size: 32768,
            n_gpu_layers: -1, // -1 = offload ALL layers to GPU
            hsa_override: "11.5.1".to_string(),  // Default for Strix Halo (gfx1151)
            max_tokens: 2048,
        }
    }
}

impl DesktopBrainConfig {
    /// =============================================================================
    /// STRIX HALO PRESET - PRODUCTION BRAIN
    /// =============================================================================
    /// AMD Strix Halo: 128GB unified RAM, 96-124GB VRAM via GTT override
    /// 
    /// ## KEY FIXES (Documented here so we don't repeat troubleshooting!)
    /// 
    /// 1. MULTI-PART MODELS: Point to FIRST file (-00001-of-00002), llama.cpp
    ///    automatically loads subsequent parts from same directory.
    /// 
    /// 2. GPU LAYERS: Use -1 which converts to u32::MAX = offload ALL layers
    /// 
    /// 3. ENVIRONMENT (set in new()):
    ///    - HSA_OVERRIDE_GFX_VERSION=11.5.1 (critical for gfx1151)
    ///    - HIP_VISIBLE_DEVICES=0 
    ///    - ROCR_VISIBLE_DEVICES=0
    /// 
    /// 4. ZOMBIE PROCESSES: Always kill trinity-brain/llama processes before start
    ///    Run: pkill -9 -f trinity-brain
    /// 
    /// 5. SUPPORTED ARCHITECTURES: llama, qwen, glm4 work. mistral3 does NOT.
    /// 
    /// Strix Halo: PLANNER Profile (Llama 4 Scout - High IQ, 17B MoE)
    pub fn planner() -> Self {
        Self {
            model_path: "/home/joshua/antigravity/models/Llama-4-Scout-17B-16E-Instruct-GGUF/Llama-4-Scout-17B-16E-Instruct-Q4_K_M-00001-of-00002.gguf".to_string(),
            context_size: 16384, // 16k Context (Safe for 17B on Strix Halo)
            n_gpu_layers: -1,
            hsa_override: "11.5.1".to_string(),
            max_tokens: 4096, // Planning needs thought, but not novels
        }
    }

    /// Strix Halo: WORKER Profile (Overthinking Rustacean Behemoth - 73B)
    pub fn worker() -> Self {
         Self {
            model_path: "/home/joshua/antigravity/models/Overthinking-Rustacean-Behemoth.Q4_K_M.gguf".to_string(),
            context_size: 32768, // 32k Context (Safe limit for dual 70B setup)
            n_gpu_layers: -1,
            hsa_override: "11.5.1".to_string(),
            max_tokens: 4096, // Coding needs space
        }
    }

    /// Strix Halo: SOLO Profile (Overthinking Rustacean Behemoth - 73B)
    /// Runs a single massive model for both Planning and Coding.
    pub fn solo_coder() -> Self {
         Self {
            model_path: "/home/joshua/antigravity/models/Overthinking-Rustacean-Behemoth.Q4_K_M.gguf".to_string(),
            context_size: 32768, // 32k Context (Safe for 128GB Unified Memory with 73B model)
            n_gpu_layers: -1,
            hsa_override: "11.5.1".to_string(),
            max_tokens: 8192,
        }
    }

    pub fn strix_halo() -> Self {
        Self::solo_coder()
    }
}

// ============================================================================
// Brain State (internal)
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
    config: DesktopBrainConfig,
}

impl DesktopBrain {
    /// Create a new DesktopBrain with the given config
    pub fn new(config: DesktopBrainConfig) -> Self {
        // Set up Strix Halo environment (Vulkan Mode)
        // ROCm env vars commented out to favor Vulkan backend
        // std::env::set_var("HSA_OVERRIDE_GFX_VERSION", &config.hsa_override);
        // std::env::set_var("HIP_VISIBLE_DEVICES", "0");
        // std::env::set_var("ROCR_VISIBLE_DEVICES", "0");
        // std::env::set_var("GGML_CUDA_ENABLE_UNIFIED_MEMORY", "1");
        
        // Vulkan might need this? Usually auto-detects.
        // std::env::set_var("GGML_VULKAN_DEVICE", "0");

        if std::env::var("ROCM_PATH").is_err() {
            std::env::set_var("ROCM_PATH", "/opt/rocm");
        }
        tracing::info!("Using Vulkan backend (ROCm vars disabled)");

        // Initialize the llama.cpp backend (Singleton)
        let backend = GLOBAL_BACKEND.get_or_init(|| {
            tracing::info!("Initializing Global Llama Backend...");
            Arc::new(LlamaBackend::init().expect("Failed to init llama backend"))
        }).clone();
        
        tracing::info!("Acquired shared llama.cpp backend");

        // Attempt to load model if path is provided
        let state = if !config.model_path.is_empty() && Path::new(&config.model_path).exists() {
            match Self::init_state(&backend, &config.model_path, config.context_size, config.n_gpu_layers) {
                Ok(s) => {
                    tracing::info!("✓ Loaded model: {}", config.model_path);
                    Some(s)
                }
                Err(e) => {
                    tracing::warn!("Failed to load model: {}", e);
                    None
                }
            }
        } else {
            if !config.model_path.is_empty() {
                tracing::info!("Model not found at: {}", config.model_path);
            }
            None
        };

        Self {
            backend,
            state: Arc::new(Mutex::new(state)),
            config,
        }
    }

    /// Create with Strix Halo optimized settings
    pub fn strix_halo() -> Self {
        Self::new(DesktopBrainConfig::strix_halo())
    }

    /// Count tokens in text (for pre-flight task validation)
    /// Returns 0 if model not loaded
    pub fn count_tokens(&self, text: &str) -> usize {
        let state_guard = self.state.lock().unwrap();
        if let Some(state) = state_guard.as_ref() {
            state.model
                .str_to_token(text, AddBos::Never)
                .map(|tokens| tokens.len())
                .unwrap_or(0)
        } else {
            // Rough estimate: ~4 chars per token for English
            text.len() / 4
        }
    }

    /// Get the batch limit for this model (max tokens per decode call)
    /// Default is 2048, but context size may limit it
    pub fn get_batch_limit(&self) -> u32 {
        // n_batch is typically 2048, reserve 512 for system prompt
        std::cmp::min(self.config.context_size, 2048).saturating_sub(512)
    }

    /// Initialize model state from a file path
    fn init_state(
        backend: &Arc<LlamaBackend>,
        path: &str,
        context_size: u32,
        n_gpu_layers: i32,
    ) -> Result<BrainState> {
        tracing::info!("Loading model from: {}", path);

        // Get file size
        let model_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        tracing::info!(
            "Model size: {:.1} GB",
            model_size as f64 / (1024.0 * 1024.0 * 1024.0)
        );

        // Configure model params for GPU offloading
        // -1 means offload ALL layers (use u32::MAX)
        let gpu_layers = if n_gpu_layers < 0 {
            u32::MAX
        } else {
            n_gpu_layers as u32
        };
        let mut model_params = LlamaModelParams::default();
        // STRIX HALO HARDWARE MASTER (64GB MODEL OPTIMIZED):
        // 1. mmap = false: Avoid double-buffering (64GB file map + 64GB ROCm buffer).
        //    This is REQUIRED for a 64GB model on a 128GB system.
        // 2. mlock = false: Disable memory pinning to prevent kernel deadlocks (verified fix)
        // 3. n_gpu_layers = MAX: Full offload to Unified VRAM.
        model_params = model_params
            .with_n_gpu_layers(gpu_layers)
            .with_use_mmap(false)
            .with_use_mlock(false);

        tracing::info!("STABILITY MASTER ACTIVE: single-buffer allocation engaged. (mmap=false, mlock=false)");

        let model = LlamaModel::load_from_file(backend, Path::new(path), &model_params)
            .with_context(|| format!("Failed to load model from {}", path))?;

        // Leak model to get 'static reference (singleton pattern)
        let model_ref: &'static LlamaModel = Box::leak(Box::new(model));

        // Create context with configured size and FLASH ATTENTION (critical for Strix Halo)
        // Flash attention is REQUIRED on Strix Halo unified memory to prevent crashes
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(context_size))
            .with_flash_attention_policy(1); // 1 = ENABLED (critical for Strix Halo)
        
        tracing::info!("FLASH ATTENTION ENABLED for Strix Halo stability");

        let context = model_ref
            .new_context(backend, ctx_params)
            .with_context(|| "Failed to create context")?;

        tracing::info!(
            "✓ Model ready: {:.1} GB, {} context",
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

        // Create batch and add prompt tokens (4096 to handle longer prompts)
        let mut batch = LlamaBatch::new(32768, 1);
        let last_idx = tokens.len() - 1;
        for (i, token) in tokens.iter().enumerate() {
            batch.add(*token, i as i32, &[0], i == last_idx)?;
        }

        // Process prompt (prefill)
        let prefill_start = std::time::Instant::now();
        state.context.decode(&mut batch)?;
        let prefill_time = prefill_start.elapsed();
        tracing::info!(
            "Prefill: {} tokens in {:.2}s ({:.1} t/s)",
            tokens.len(),
            prefill_time.as_secs_f64(),
            tokens.len() as f64 / prefill_time.as_secs_f64()
        );

        // Generation loop with greedy sampling
        let gen_start = std::time::Instant::now();
        let mut output_text = String::new();
        let n_cur = tokens.len();
        let mut gen_tokens = 0usize;

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
            gen_tokens += 1;

            // Prepare next batch
            batch.clear();
            batch.add(new_token, (n_cur + i) as i32, &[0], true)?;

            // Decode
            state.context.decode(&mut batch)?;
        }

        let gen_time = gen_start.elapsed();
        tracing::info!(
            "Generation: {} tokens in {:.2}s ({:.1} t/s)",
            gen_tokens,
            gen_time.as_secs_f64(),
            gen_tokens as f64 / gen_time.as_secs_f64()
        );

        Ok(output_text)
    }

    /// Get model info
    pub fn get_model_info(&self) -> Option<ModelInfo> {
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
            context_size: self.config.context_size,
        })
    }

    /// Check if a model is loaded
    pub fn has_model(&self) -> bool {
        self.state.lock().map(|s| s.is_some()).unwrap_or(false)
    }
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub quantization: String,
    pub context_size: u32,
}

impl ModelInfo {
    /// Size in GB
    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

// ============================================================================
// Brain Trait Implementation
// ============================================================================

#[async_trait]
impl Brain for DesktopBrain {
    async fn think(&self, prompt: &str) -> Result<String> {
        // Check if model is loaded
        if !self.has_model() {
            return Ok(format!(
                "[No model loaded] Would process: {}...",
                &prompt.chars().take(50).collect::<String>()
            ));
        }

        // Run generation (blocking, wrapped for async)
        let config = self.config.clone();
        let state = self.state.clone();
        let prompt_owned = prompt.to_string();

        tokio::task::spawn_blocking(move || {
            let mut state_guard = state.lock().unwrap();
            let state = state_guard
                .as_mut()
                .context("No model loaded")?;

            // Tokenize
            let tokens = state
                .model
                .str_to_token(&prompt_owned, AddBos::Always)
                .context("Failed to tokenize")?;

            state.context.clear_kv_cache();

            let mut batch = LlamaBatch::new(32768, 1);
            let last_idx = tokens.len() - 1;
            for (i, token) in tokens.iter().enumerate() {
                batch.add(*token, i as i32, &[0], i == last_idx)?;
            }

            state.context.decode(&mut batch)?;

            let mut output_text = String::new();
            let n_cur = tokens.len();

            for i in 0..config.max_tokens as usize {
                let mut candidates = state.context.token_data_array_ith(batch.n_tokens() - 1);
                let new_token = candidates.sample_token_greedy();

                if new_token == state.model.token_eos() {
                    break;
                }

                let token_text = state
                    .model
                    .token_to_str(new_token, Special::Tokenize)
                    .unwrap_or_default();

                output_text.push_str(&token_text);

                batch.clear();
                batch.add(new_token, (n_cur + i) as i32, &[0], true)?;
                state.context.decode(&mut batch)?;
            }

            Ok(output_text)
        })
        .await
        .context("Generation task failed")?
    }

    async fn think_with_grammar(&self, prompt: &str, grammar: GrammarSpec) -> Result<String> {
        // If no grammar specified, fall back to regular think
        if grammar == GrammarSpec::None {
            return self.think(prompt).await;
        }

        if !self.has_model() {
            return Ok(format!(
                "[No model loaded] Would process: {}...",
                &prompt.chars().take(50).collect::<String>()
            ));
        }

        // Load grammar file
        let grammar_content = match grammar.grammar_path() {
            Some(rel_path) => {
                // Try multiple possible locations for the grammar file
                let possible_paths = [
                    format!("/home/joshua/antigravity/trinity-genesis/assets/{}", rel_path),
                    format!("assets/{}", rel_path),
                    rel_path.to_string(),
                ];
                
                let mut content = None;
                for path in &possible_paths {
                    if let Ok(c) = std::fs::read_to_string(path) {
                        tracing::info!("Loaded grammar from: {}", path);
                        content = Some(c);
                        break;
                    }
                }
                
                match content {
                    Some(c) => c,
                    None => {
                        tracing::warn!("Grammar file not found for {:?}, falling back to unconstrained", grammar);
                        return self.think(prompt).await;
                    }
                }
            }
            None => return self.think(prompt).await,
        };

        let config = self.config.clone();
        let state = self.state.clone();
        let prompt_owned = prompt.to_string();

        tokio::task::spawn_blocking(move || {
            let mut state_guard = state.lock().unwrap();
            let state = state_guard
                .as_mut()
                .context("No model loaded")?;

            // Tokenize
            let tokens = state
                .model
                .str_to_token(&prompt_owned, AddBos::Always)
                .context("Failed to tokenize")?;

            state.context.clear_kv_cache();

            let mut batch = LlamaBatch::new(32768, 1);
            let last_idx = tokens.len() - 1;
            for (i, token) in tokens.iter().enumerate() {
                batch.add(*token, i as i32, &[0], i == last_idx)?;
            }

            state.context.decode(&mut batch)?;

            // Create grammar-constrained sampler chain
            let grammar_sampler = match LlamaSampler::grammar(state.model, &grammar_content, "root") {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("Failed to create grammar sampler: {:?}, falling back to greedy", e);
                    // Fall back to greedy sampling without grammar
                    LlamaSampler::greedy()
                }
            };

            // Build sampler chain: grammar -> greedy selection
            let mut sampler = LlamaSampler::chain_simple([
                grammar_sampler,
                LlamaSampler::greedy(),
            ]);

            let mut output_text = String::new();
            let n_cur = tokens.len();

            for i in 0..config.max_tokens as usize {
                // Sample with grammar constraint
                let new_token = sampler.sample(&state.context, (batch.n_tokens() - 1) as i32);
                sampler.accept(new_token);

                if new_token == state.model.token_eos() {
                    break;
                }

                let token_text = state
                    .model
                    .token_to_str(new_token, Special::Tokenize)
                    .unwrap_or_default();

                output_text.push_str(&token_text);

                batch.clear();
                batch.add(new_token, (n_cur + i) as i32, &[0], true)?;
                state.context.decode(&mut batch)?;
            }

            Ok(output_text)
        })
        .await
        .context("Grammar-constrained generation task failed")?
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Safe check: return zeros if model isn't loaded, though for hashing it doesn't strictly matter.
        if !self.has_model() {
            return Ok(vec![0.0; 384]);
        }

        let text_owned = text.to_string();

        // Run purely on CPU, no llama.cpp interaction needed for the proxy
        // CRITICAL FIX: Removed LlamaBatch/decode logic that was causing crashes
        // The previous code allocated a 32768-token batch and ran decode(), 
        // which exceeded llama.cpp's internal limits and crashed.
        // Since we're using a hash proxy anyway, there's no need to touch the model.
        tokio::task::spawn_blocking(move || {
            // TEMPORARY PROXY: Hashed embedding
            // This allows the memory system to function "mechanically" even if semantically weak
            // until we load a proper BERT model or enable embedding mode.
            
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::Hasher;
            hasher.write(text_owned.as_bytes());
            let seed = hasher.finish();
            
            let mut vec = Vec::with_capacity(384);
            let mut val = seed;
            for _ in 0..384 {
                val = val.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                vec.push((val as f32 / u64::MAX as f32) * 2.0 - 1.0);
            }
            Ok(vec)
        })
        .await
        .context("Embedding task failed")?
    }

    async fn think_stream(
        &self,
        prompt: &str,
        token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String> {
        if !self.has_model() {
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

        let state_clone = self.state.clone();
        let prompt_owned = prompt.to_string();
        let max_tokens = self.config.max_tokens;

        // Spawn blocking task for generation
        let generation_handle = tokio::task::spawn_blocking(move || {
            let mut state_guard = state_clone.lock().unwrap();
            let state = match state_guard.as_mut() {
                Some(s) => s,
                None => return Err(anyhow::anyhow!("No model loaded")),
            };

            let tokens = state
                .model
                .str_to_token(&prompt_owned, AddBos::Always)
                .context("Failed to tokenize")?;

            state.context.clear_kv_cache();

            let mut batch = LlamaBatch::new(32768, 1);
            let last_idx = tokens.len() - 1;
            for (i, token) in tokens.iter().enumerate() {
                batch.add(*token, i as i32, &[0], i == last_idx)?;
            }

            state.context.decode(&mut batch)?;

            let mut output_text = String::new();
            let n_cur = tokens.len();

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
                    if generation_handle.is_finished() {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }

        match generation_handle.await {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(anyhow::anyhow!("Generation task panicked: {}", e)),
        }
    }

    fn is_ready(&self) -> bool {
        self.has_model()
    }

    fn name(&self) -> &'static str {
        "DesktopBrain (llama.cpp + ROCm)"
    }
}
