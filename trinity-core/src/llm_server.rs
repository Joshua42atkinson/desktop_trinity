//! LLM Server Manager - Embedded llama-server control for Trinity
//!
//! Provides two modes of operation:
//! 1. **Process mode** (`LlmServer`): Manages llama-server as a subprocess
//! 2. **Native mode** (`LlamaNative`, requires `llama-cpp` feature): Direct Rust bindings
//!
//! Native mode is preferred for Strix Halo (gfx1103) as it eliminates IPC overhead.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// llama-cpp-2 native bindings (feature-gated)
#[cfg(feature = "llama-cpp")]
use llama_cpp_2::context::params::LlamaContextParams;
#[cfg(feature = "llama-cpp")]
use llama_cpp_2::llama_backend::LlamaBackend;
#[cfg(feature = "llama-cpp")]
use llama_cpp_2::model::params::LlamaModelParams;
#[cfg(feature = "llama-cpp")]
use llama_cpp_2::model::LlamaModel;
#[cfg(feature = "llama-cpp")]
use std::num::NonZeroU32;

/// Configuration for the embedded LLM server
#[derive(Clone, Debug)]
pub struct LlmServerConfig {
    /// Path to the llama-server binary
    pub server_binary: PathBuf,
    /// Path to the GGUF model file
    pub model_path: PathBuf,
    /// Port to listen on
    pub port: u16,
    /// Number of GPU layers to offload (-1 = all)
    pub gpu_layers: i32,
    /// Context size
    pub context_size: usize,
    /// Host to bind to
    pub host: String,
}

impl Default for LlmServerConfig {
    fn default() -> Self {
        Self {
            server_binary: PathBuf::from("bin/llama-server"),
            model_path: PathBuf::from(
                "/home/joshua/.lmstudio/models/lmstudio-community/gpt-oss-120b-GGUF/gpt-oss-120b-MXFP4-00001-of-00002.gguf"
            ),
            port: 8081,
            gpu_layers: 99,
            context_size: 8192,
            host: "127.0.0.1".to_string(),
        }
    }
}

impl LlmServerConfig {
    /// Configuration for GPT-OSS-120B
    pub fn gpt_oss_120b() -> Self {
        Self::default()
    }

    /// Configuration for Qwen3-235B (when downloaded)
    pub fn qwen3_235b() -> Self {
        Self {
            model_path: PathBuf::from(
                "/home/joshua/.lmstudio/models/qwen3-235b-q3/Qwen3-235B-A22B-UD-Q3_K_XL.gguf",
            ),
            context_size: 32768,
            ..Self::default()
        }
    }
}

/// Manages an embedded llama-server process
pub struct LlmServer {
    config: LlmServerConfig,
    process: Option<Child>,
    running: Arc<AtomicBool>,
}

impl LlmServer {
    /// Create a new server manager
    pub fn new(config: LlmServerConfig) -> Self {
        Self {
            config,
            process: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the LLM server
    pub fn start(&mut self) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            anyhow::bail!("Server already running");
        }

        tracing::info!("Starting embedded LLM server on port {}", self.config.port);
        tracing::info!("Model: {:?}", self.config.model_path);

        // Verify binary exists
        if !self.config.server_binary.exists() {
            anyhow::bail!(
                "llama-server binary not found: {:?}",
                self.config.server_binary
            );
        }

        // Verify model exists
        if !self.config.model_path.exists() {
            anyhow::bail!("Model file not found: {:?}", self.config.model_path);
        }

        let child = Command::new(&self.config.server_binary)
            .arg("-m")
            .arg(&self.config.model_path)
            .arg("-ngl")
            .arg(self.config.gpu_layers.to_string())
            .arg("-c")
            .arg(self.config.context_size.to_string())
            .arg("--port")
            .arg(self.config.port.to_string())
            .arg("--host")
            .arg(&self.config.host)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn llama-server")?;

        self.process = Some(child);
        self.running.store(true, Ordering::SeqCst);

        tracing::info!("LLM server started successfully");
        Ok(())
    }

    /// Stop the LLM server
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            tracing::info!("Stopping LLM server...");
            child.kill().context("Failed to kill llama-server")?;
            child.wait().context("Failed to wait for llama-server")?;
            self.running.store(false, Ordering::SeqCst);
            tracing::info!("LLM server stopped");
        }
        Ok(())
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get the API base URL
    pub fn api_url(&self) -> String {
        format!("http://{}:{}/v1", self.config.host, self.config.port)
    }

    /// Wait for server to be ready (health check)
    pub async fn wait_ready(&self, timeout: Duration) -> Result<()> {
        let health_url = format!("http://{}:{}/health", self.config.host, self.config.port);
        let client = reqwest::Client::new();
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match client.get(&health_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!("LLM server is ready");
                    return Ok(());
                }
                Ok(resp) if resp.status().as_u16() == 503 => {
                    // Still loading model
                    tracing::debug!("Server loading model...");
                }
                _ => {}
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        anyhow::bail!("Server failed to become ready within {:?}", timeout)
    }
}

impl Drop for LlmServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// ============================================================================
// Native llama-cpp-2 bindings (feature-gated)
// ============================================================================

/// Configuration for native llama-cpp-2 inference
#[cfg(feature = "llama-cpp")]
#[derive(Clone, Debug)]
pub struct LlamaNativeConfig {
    /// Path to the GGUF model file
    pub model_path: PathBuf,
    /// Number of layers to offload to GPU (-1 = all layers)
    pub n_gpu_layers: i32,
    /// Context window size
    pub n_ctx: u32,
    /// Random seed for sampling
    pub seed: u32,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature for sampling (0.0 = greedy)
    pub temperature: f32,
}

#[cfg(feature = "llama-cpp")]
impl Default for LlamaNativeConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/model.gguf"),
            n_gpu_layers: -1, // Offload all layers to GPU
            n_ctx: 4096,
            seed: 42,
            max_tokens: 512,
            temperature: 0.7,
        }
    }
}

#[cfg(feature = "llama-cpp")]
impl LlamaNativeConfig {
    /// Preset for Qwen 7B on 32GB system (fast, fits easily)
    pub fn qwen_7b() -> Self {
        Self {
            model_path: PathBuf::from("models/Qwen2.5-7B-Instruct-Q4_K_M.gguf"),
            n_gpu_layers: -1,
            n_ctx: 8192,
            seed: 42,
            max_tokens: 1024,
            temperature: 0.7,
        }
    }

    /// Preset for Qwen 14B on 32GB system (good balance)
    pub fn qwen_14b() -> Self {
        Self {
            model_path: PathBuf::from("models/Qwen2.5-14B-Instruct-Q4_K_M.gguf"),
            n_gpu_layers: -1,
            n_ctx: 4096,
            seed: 42,
            max_tokens: 1024,
            temperature: 0.7,
        }
    }

    /// Preset for Qwen 32B on 96GB VRAM (fits with large context)
    pub fn qwen_32b() -> Self {
        Self {
            model_path: PathBuf::from("models/Qwen2.5-32B-Instruct-Q4_K_M.gguf"),
            n_gpu_layers: -1,
            n_ctx: 8192, // Full context with 96GB VRAM
            seed: 42,
            max_tokens: 1024,
            temperature: 0.7,
        }
    }

    /// Preset for Qwen 72B on 96GB VRAM (~45GB model)
    pub fn qwen_72b() -> Self {
        Self {
            model_path: PathBuf::from("models/Qwen2.5-72B-Instruct-Q4_K_M.gguf"),
            n_gpu_layers: -1,
            n_ctx: 8192,
            seed: 42,
            max_tokens: 1024,
            temperature: 0.7,
        }
    }

    /// Preset for Qwen 120B/GPT-OSS-120B on 96GB VRAM (~75GB Q4)
    pub fn qwen_120b() -> Self {
        Self {
            model_path: PathBuf::from("models/gpt-oss-120b.gguf"),
            n_gpu_layers: -1,
            n_ctx: 4096, // Reduced context for memory headroom
            seed: 42,
            max_tokens: 512,
            temperature: 0.7,
        }
    }

    /// Preset for Qwen 235B Q3_K_M on 112GB+ VRAM (~111GB model)
    /// Requires BIOS configured for maximum VRAM allocation
    pub fn qwen_235b() -> Self {
        Self {
            model_path: PathBuf::from("models/Qwen2.5-235B-Instruct-Q3_K_M.gguf"),
            n_gpu_layers: -1,
            n_ctx: 2048, // Minimal context to maximize model fit
            seed: 42,
            max_tokens: 256,
            temperature: 0.7,
        }
    }
}

/// Native llama.cpp inference server for Trinity
///
/// Uses llama-cpp-2 Rust bindings for direct inference without subprocess overhead.
/// Optimized for AMD Strix Halo (gfx1103) with HSA environment configuration.
#[cfg(feature = "llama-cpp")]
pub struct LlamaNative {
    backend: LlamaBackend,
    model: LlamaModel,
    config: LlamaNativeConfig,
}

#[cfg(feature = "llama-cpp")]
impl LlamaNative {
    /// Initialize the llama.cpp backend and load the model
    ///
    /// This sets up HSA environment variables for gfx1103 (Strix Halo)
    /// and initializes the HIP/ROCm backend.
    pub fn new(config: LlamaNativeConfig) -> Result<Self> {
        // Set HSA override for gfx1103 Strix Halo
        setup_strix_halo_env();

        tracing::info!("Initializing native llama.cpp backend for Strix Halo (gfx1103)");

        // Initialize backend
        let backend = LlamaBackend::init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize llama.cpp backend: {:?}", e))?;

        // Configure model parameters with builder pattern
        // Note: n_gpu_layers takes u32, so we convert (negative means all layers)
        let gpu_layers = if config.n_gpu_layers < 0 {
            u32::MAX
        } else {
            config.n_gpu_layers as u32
        };
        let model_params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);

        tracing::info!("Loading model from {:?}", config.model_path);

        if !config.model_path.exists() {
            anyhow::bail!(
                "Model file not found: {:?}\nPlease download a GGUF model to this path.",
                config.model_path
            );
        }

        // Load the model
        let model = LlamaModel::load_from_file(&backend, &config.model_path, &model_params)
            .map_err(|e| anyhow::anyhow!("Failed to load model: {:?}", e))?;

        tracing::info!(
            "Model loaded successfully with {} GPU layers",
            config.n_gpu_layers
        );

        Ok(Self {
            backend,
            model,
            config,
        })
    }

    /// Generate text from a prompt (placeholder - full implementation requires sampling loop)
    pub fn generate(&self, prompt: &str) -> Result<String> {
        tracing::debug!("Generating from prompt ({} chars)", prompt.len());

        // Create context parameters with builder pattern
        let ctx_params =
            LlamaContextParams::default().with_n_ctx(NonZeroU32::new(self.config.n_ctx));

        // Create context
        let _ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| anyhow::anyhow!("Failed to create context: {:?}", e))?;

        // NOTE: Full tokenization, evaluation, and sampling loop would go here
        // The llama-cpp-2 crate provides these primitives, but implementation
        // requires careful handling of the generation loop.

        Ok(format!(
            "[LlamaNative] Ready: {:?}, Context: {} tokens",
            self.config.model_path.file_name().unwrap_or_default(),
            self.config.n_ctx
        ))
    }

    /// Get model configuration
    pub fn config(&self) -> &LlamaNativeConfig {
        &self.config
    }

    /// Get model info string
    pub fn info(&self) -> String {
        format!(
            "LlamaNative: {} (GPU layers: {}, ctx: {})",
            self.config.model_path.display(),
            self.config.n_gpu_layers,
            self.config.n_ctx
        )
    }
}

/// Set up environment variables for Strix Halo gfx1103 architecture
pub fn setup_strix_halo_env() {
    // HSA override for gfx1151 architecture (official Strix Halo)
    std::env::set_var("HSA_OVERRIDE_GFX_VERSION", "11.5.1");

    // Force single GPU device
    std::env::set_var("HIP_VISIBLE_DEVICES", "0");

    // ROCm path (if not already set)
    if std::env::var("ROCM_PATH").is_err() {
        std::env::set_var("ROCM_PATH", "/opt/rocm");
    }

    tracing::debug!("Strix Halo environment configured for gfx1103");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = LlmServerConfig::default();
        assert_eq!(config.port, 8081);
        assert_eq!(config.gpu_layers, 99);
    }

    #[test]
    fn test_api_url() {
        let server = LlmServer::new(LlmServerConfig::default());
        assert_eq!(server.api_url(), "http://127.0.0.1:8081/v1");
    }
}
