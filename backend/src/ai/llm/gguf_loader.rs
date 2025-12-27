//! GGUF Model Loader - Native quantized model loading for Trinity
//!
//! Loads GGUF quantized models directly into Candle for close-to-metal inference.
//! Supports AMD ROCm acceleration via HIP backend.

use anyhow::{Context, Result};
use bevy::reflect::Reflect;
use candle_core::{quantized::gguf_file, Device, Tensor};
use candle_transformers::models::quantized_llama::ModelWeights;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokenizers::Tokenizer;

/// Configuration for GGUF model loading
#[derive(Clone, Debug)]
pub struct GgufConfig {
    /// Path to the .gguf model file
    pub model_path: PathBuf,
    /// Path to tokenizer.json (or will attempt to extract from GGUF)
    pub tokenizer_path: Option<PathBuf>,
    /// Device to run inference on
    pub device: DeviceConfig,
    /// Maximum context length
    pub max_context_length: usize,
    /// Random seed for sampling
    pub seed: u64,
}

/// Device configuration for inference
#[derive(Clone, Debug, Default)]
pub enum DeviceConfig {
    /// Automatic detection (CUDA -> Metal -> CPU)
    #[default]
    Auto,
    /// Force CPU
    Cpu,
    /// Force CUDA with device ID
    Cuda(usize),
    /// Force Metal (macOS)
    Metal,
}

impl Default for GgufConfig {
    fn default() -> Self {
        let model_path = std::env::var("GGUF_MODEL_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("models/model.gguf"));

        Self {
            model_path,
            tokenizer_path: None,
            device: DeviceConfig::Auto,
            max_context_length: 4096,
            seed: 42,
        }
    }
}

#[derive(Clone, Debug, Copy, Reflect)]
pub enum ModelType {
    Smart, // 120B+ models
    Fast,  // <10B models
}

impl GgufConfig {
    /// Load config from environment variables based on model type
    pub fn from_env(model_type: ModelType) -> Self {
        let (env_var, default_path) = match model_type {
            ModelType::Smart => ("GGUF_MODEL_PATH_SMART", "models/smart_model.gguf"),
            ModelType::Fast => ("GGUF_MODEL_PATH_FAST", "models/fast_model.gguf"),
        };

        let model_path = std::env::var(env_var)
            .map(PathBuf::from)
            // Fallback to legacy env var if specific one not set
            .or_else(|_| std::env::var("GGUF_MODEL_PATH").map(PathBuf::from))
            .unwrap_or_else(|_| PathBuf::from(default_path));

        let tokenizer_path = std::env::var("TOKENIZER_PATH").map(PathBuf::from).ok();

        // Smart models need much more context
        let max_context_length = match model_type {
            ModelType::Smart => 8192,
            ModelType::Fast => 4096,
        };

        Self {
            model_path,
            tokenizer_path,
            device: DeviceConfig::Auto,
            max_context_length,
            seed: 42,
        }
    }

    /// Create config for a specific GGUF file
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self {
            model_path: path.into(),
            ..Default::default()
        }
    }

    /// Preset for Qwen3-Next-80B-A3B running on Trinity (AMD Strix Halo)
    /// Uses the symlinked model from LM Studio and downloaded tokenizer
    pub fn qwen3_80b_trinity() -> Self {
        Self {
            model_path: PathBuf::from("models/qwen3-80b.gguf"),
            tokenizer_path: Some(PathBuf::from("models/tokenizer.json")),
            device: DeviceConfig::Auto,
            max_context_length: 32768, // Qwen3 supports long context
            seed: 42,
        }
    }

    /// Preset for Intel Qwen3-235B-A22B (Mixture of Experts)
    /// This is a 235B parameter MoE model with ~22B active parameters
    /// Optimized for AMD Strix Halo with 128GB unified memory
    pub fn qwen3_235b() -> Self {
        Self {
            model_path: PathBuf::from("models/Qwen3-235B-A22B-q2ks-mixed-ar.gguf"),
            tokenizer_path: Some(PathBuf::from("models/tokenizer.json")),
            device: DeviceConfig::Auto,
            max_context_length: 32768, // Qwen3 supports long context
            seed: 42,
        }
    }

    /// Preset for GPT-OSS 120B
    pub fn gpt_oss_120b() -> Self {
        Self {
            model_path: PathBuf::from("models/gpt-oss-120b.gguf"),
            tokenizer_path: None,
            device: DeviceConfig::Auto,
            max_context_length: 8192,
            seed: 42,
        }
    }

    /// Preset for smaller/faster models (Llama 8B, etc.)
    pub fn fast_model(path: impl Into<PathBuf>) -> Self {
        Self {
            model_path: path.into(),
            tokenizer_path: None,
            device: DeviceConfig::Auto,
            max_context_length: 4096,
            seed: 42,
        }
    }

    /// Set tokenizer path
    pub fn with_tokenizer(mut self, path: impl Into<PathBuf>) -> Self {
        self.tokenizer_path = Some(path.into());
        self
    }

    /// Force CPU device
    pub fn cpu(mut self) -> Self {
        self.device = DeviceConfig::Cpu;
        self
    }

    /// Set context length (Qwen3 supports up to 128K)
    pub fn with_context_length(mut self, len: usize) -> Self {
        self.max_context_length = len;
        self
    }
}

/// Configuration for text generation
#[derive(Clone, Debug)]
pub struct GenerateConfig {
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature for sampling (0.0 = greedy, 1.0 = diverse)
    pub temperature: f64,
    /// Top-p nucleus sampling threshold
    pub top_p: f64,
    /// Repetition penalty
    pub repeat_penalty: f32,
    /// Last N tokens to consider for repeat penalty
    pub repeat_last_n: usize,
}

impl Default for GenerateConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
        }
    }
}

impl GenerateConfig {
    /// Create config for code generation (lower temperature)
    pub fn for_code() -> Self {
        Self {
            temperature: 0.3,
            max_tokens: 1024,
            ..Default::default()
        }
    }

    /// Create config for creative writing (higher temperature)
    pub fn for_creative() -> Self {
        Self {
            temperature: 0.9,
            top_p: 0.95,
            ..Default::default()
        }
    }
}

/// Native GGUF model for close-to-metal inference
pub struct GgufModel {
    model: ModelWeights,
    tokenizer: Arc<Tokenizer>,
    device: Device,
    config: GgufConfig,
    /// EOS token ID
    eos_token_id: u32,
}

impl GgufModel {
    /// Load a GGUF model from disk
    pub fn load(config: GgufConfig) -> Result<Self> {
        log::info!("Loading GGUF model from {:?}", config.model_path);

        // 1. Initialize device
        let device = match &config.device {
            DeviceConfig::Auto => Self::detect_best_device()?,
            DeviceConfig::Cpu => Device::Cpu,
            DeviceConfig::Cuda(id) => {
                Device::new_cuda(*id).context("Failed to initialize CUDA device")?
            }
            DeviceConfig::Metal => {
                Device::new_metal(0).context("Failed to initialize Metal device")?
            }
        };

        log::info!("Using device: {:?}", device);

        // 2. Open and parse GGUF file
        let model_path = &config.model_path;
        if !model_path.exists() {
            anyhow::bail!("Model file not found: {:?}", model_path);
        }

        let mut file = File::open(model_path).context("Failed to open GGUF file")?;

        let gguf_content =
            gguf_file::Content::read(&mut file).context("Failed to parse GGUF file")?;

        log::info!(
            "GGUF file loaded: {} tensors, architecture: {:?}",
            gguf_content.tensor_infos.len(),
            gguf_content.metadata.get("general.architecture")
        );

        // 3. Load model weights
        let model = ModelWeights::from_gguf(gguf_content, &mut file, &device)
            .context("Failed to load model weights from GGUF")?;

        log::info!("Model weights loaded successfully");

        // 4. Load tokenizer
        let tokenizer = Self::load_tokenizer(&config)?;

        // 5. Get EOS token
        let eos_token_id = tokenizer
            .token_to_id("</s>")
            .or_else(|| tokenizer.token_to_id("<|end_of_text|>"))
            .or_else(|| tokenizer.token_to_id("<|eot_id|>"))
            .unwrap_or(2);

        log::info!("Model ready for inference (EOS token: {})", eos_token_id);

        Ok(Self {
            model,
            tokenizer: Arc::new(tokenizer),
            device,
            config,
            eos_token_id,
        })
    }

    /// Detect the best available device
    fn detect_best_device() -> Result<Device> {
        // Try CUDA first (includes ROCm/HIP)
        if candle_core::utils::cuda_is_available() {
            log::info!("CUDA/ROCm GPU detected");
            return Device::new_cuda(0).context("Failed to init CUDA");
        }

        // Try Metal on macOS
        if candle_core::utils::metal_is_available() {
            log::info!("Metal GPU detected");
            return Device::new_metal(0).context("Failed to init Metal");
        }

        // Fallback to CPU
        log::warn!("No GPU detected, using CPU (inference will be slow)");
        Ok(Device::Cpu)
    }

    /// Load tokenizer from file or embedded in GGUF
    fn load_tokenizer(config: &GgufConfig) -> Result<Tokenizer> {
        // Try explicit tokenizer path first
        if let Some(ref path) = config.tokenizer_path {
            if path.exists() {
                return Tokenizer::from_file(path)
                    .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e));
            }
        }

        // Try to find tokenizer next to model
        let model_dir = config.model_path.parent().unwrap_or(Path::new("."));
        let tokenizer_path = model_dir.join("tokenizer.json");

        if tokenizer_path.exists() {
            return Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e));
        }

        anyhow::bail!(
            "Tokenizer not found. Please provide tokenizer.json next to the model file or set tokenizer_path."
        )
    }

    /// Generate text from a prompt
    pub fn generate(&mut self, prompt: &str, gen_config: &GenerateConfig) -> Result<String> {
        log::debug!("Generating from prompt ({} chars)", prompt.len());

        // 1. Tokenize prompt
        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

        let prompt_tokens: Vec<u32> = encoding.get_ids().to_vec();
        let prompt_len = prompt_tokens.len();

        if prompt_len > self.config.max_context_length {
            anyhow::bail!(
                "Prompt too long: {} tokens (max: {})",
                prompt_len,
                self.config.max_context_length
            );
        }

        log::debug!("Prompt tokenized to {} tokens", prompt_len);

        // 2. Initialize generation
        let mut all_tokens = prompt_tokens.clone();
        let mut generated_tokens: Vec<u32> = Vec::new();

        // 3. Generation loop
        for i in 0..gen_config.max_tokens {
            // Get input for this step (last token or full context for first step)
            let input_tokens = if i == 0 {
                &all_tokens[..]
            } else {
                &all_tokens[all_tokens.len() - 1..]
            };

            // Create input tensor
            let input = Tensor::new(input_tokens, &self.device)?.unsqueeze(0)?;

            // Forward pass
            let logits = self.model.forward(&input, i)?;
            let logits = logits.squeeze(0)?.squeeze(0)?;

            // Sample next token
            let next_token = self.sample_token(&logits, gen_config, &all_tokens)?;

            // Check for EOS
            if next_token == self.eos_token_id {
                log::debug!("EOS token generated at position {}", i);
                break;
            }

            all_tokens.push(next_token);
            generated_tokens.push(next_token);
        }

        // 4. Decode tokens to text
        let output = self
            .tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| anyhow::anyhow!("Decoding failed: {}", e))?;

        log::debug!("Generated {} tokens", generated_tokens.len());

        Ok(output)
    }

    /// Sample a token from logits
    fn sample_token(
        &self,
        logits: &Tensor,
        config: &GenerateConfig,
        context: &[u32],
    ) -> Result<u32> {
        let logits = logits.to_dtype(candle_core::DType::F32)?;
        let logits = logits.to_vec1::<f32>()?;

        // Apply repetition penalty
        let mut logits = logits;
        let penalty_start = context.len().saturating_sub(config.repeat_last_n);
        for &token_id in &context[penalty_start..] {
            if let Some(logit) = logits.get_mut(token_id as usize) {
                *logit /= config.repeat_penalty;
            }
        }

        // Temperature scaling
        if config.temperature > 0.0 {
            for logit in &mut logits {
                *logit /= config.temperature as f32;
            }
        }

        // Softmax to get probabilities
        let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = logits.iter().map(|x| (x - max_logit).exp()).sum();
        let probs: Vec<f32> = logits
            .iter()
            .map(|x| (x - max_logit).exp() / exp_sum)
            .collect();

        // Top-p sampling
        if config.top_p < 1.0 {
            let mut sorted: Vec<(usize, f32)> = probs.iter().cloned().enumerate().collect();
            sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            let mut cumsum = 0.0;
            let mut cutoff_idx = sorted.len();
            for (i, (_, p)) in sorted.iter().enumerate() {
                cumsum += p;
                if cumsum >= config.top_p as f32 {
                    cutoff_idx = i + 1;
                    break;
                }
            }

            // Sample from top-p tokens
            let top_tokens: Vec<(usize, f32)> = sorted[..cutoff_idx].to_vec();
            let sum: f32 = top_tokens.iter().map(|(_, p)| p).sum();

            let mut rng_val: f32 = rand_value() * sum;
            for (idx, p) in &top_tokens {
                rng_val -= p;
                if rng_val <= 0.0 {
                    return Ok(*idx as u32);
                }
            }

            return Ok(top_tokens[0].0 as u32);
        }

        // Greedy fallback
        let max_idx = probs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        Ok(max_idx as u32)
    }

    /// Get model information
    pub fn info(&self) -> ModelInfo {
        ModelInfo {
            model_path: self.config.model_path.clone(),
            device: format!("{:?}", self.device),
            max_context: self.config.max_context_length,
        }
    }
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub model_path: PathBuf,
    pub device: String,
    pub max_context: usize,
}

/// Simple random value generator (replace with proper RNG in production)
fn rand_value() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f32) / (u32::MAX as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gguf_config_default() {
        let config = GgufConfig::default();
        assert_eq!(config.max_context_length, 4096);
    }

    #[test]
    fn test_generate_config_presets() {
        let code_config = GenerateConfig::for_code();
        assert!(code_config.temperature < 0.5);

        let creative_config = GenerateConfig::for_creative();
        assert!(creative_config.temperature > 0.8);
    }

    #[test]
    fn test_device_config() {
        let config = GgufConfig::from_path("/path/to/model.gguf").cpu();
        matches!(config.device, DeviceConfig::Cpu);
    }
}
