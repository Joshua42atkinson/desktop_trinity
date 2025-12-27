//! GGUF Model Inference Engine for Trinity
//!
//! Native quantized model loading optimized for AMD Strix Halo.

use anyhow::Result;
use candle_core::quantized::gguf_file;
use candle_transformers::models::quantized_llama::ModelWeights;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;

use crate::device::TrinityDevice;

/// Configuration for model inference
#[derive(Clone, Debug)]
pub struct InferenceConfig {
    pub model_path: PathBuf,
    pub tokenizer_path: Option<PathBuf>,
    pub max_context_length: usize,
    pub seed: u64,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/model.gguf"),
            tokenizer_path: None,
            max_context_length: 32768,
            seed: 42,
        }
    }
}

impl InferenceConfig {
    pub fn qwen3_80b() -> Self {
        Self {
            model_path: PathBuf::from("models/qwen3-80b.gguf"),
            tokenizer_path: Some(PathBuf::from("models/tokenizer.json")),
            max_context_length: 32768,
            seed: 42,
        }
    }

    /// Preset for GPT-OSS 120B (Q4_K_M) - Recommended for logic/reasoning
    /// Fits in 96GB VRAM (approx 72GB) allowing room for context
    pub fn gpt_oss_120b() -> Self {
        Self {
            model_path: PathBuf::from("models/gpt-oss-120b.gguf"),
            tokenizer_path: None, // Usually embedded
            max_context_length: 8192,
            seed: 42,
        }
    }
}

/// Generation configuration
#[derive(Clone, Debug)]
pub struct GenerateConfig {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: f64,
    pub repeat_penalty: f32,
}

impl Default for GenerateConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
        }
    }
}

/// Native GGUF model for AMD Strix Halo
pub struct GgufModel {
    model: ModelWeights,
    tokenizer: Arc<Tokenizer>,
    device: TrinityDevice,
    config: InferenceConfig,
    eos_token_id: u32,
}

impl GgufModel {
    /// Load a GGUF model from disk
    pub fn load(config: InferenceConfig) -> Result<Self> {
        let device = TrinityDevice::new()?;
        Self::load_with_device(config, device)
    }

    /// Load with a specific device
    pub fn load_with_device(config: InferenceConfig, device: TrinityDevice) -> Result<Self> {
        tracing::info!("Loading GGUF model from {:?}", config.model_path);

        if !config.model_path.exists() {
            anyhow::bail!("Model file not found: {:?}", config.model_path);
        }

        let mut file = File::open(&config.model_path)?;
        let gguf_content = gguf_file::Content::read(&mut file)?;

        tracing::info!("GGUF: {} tensors", gguf_content.tensor_infos.len());

        let model = ModelWeights::from_gguf(gguf_content, &mut file, device.device())?;
        let tokenizer = Self::load_tokenizer(&config)?;

        let eos_token_id = tokenizer
            .token_to_id("</s>")
            .or_else(|| tokenizer.token_to_id("<|end_of_text|>"))
            .unwrap_or(2);

        Ok(Self {
            model,
            tokenizer: Arc::new(tokenizer),
            device,
            config,
            eos_token_id,
        })
    }

    fn load_tokenizer(config: &InferenceConfig) -> Result<Tokenizer> {
        if let Some(ref path) = config.tokenizer_path {
            if path.exists() {
                return Tokenizer::from_file(path)
                    .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e));
            }
        }

        let model_dir = config
            .model_path
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let tokenizer_path = model_dir.join("tokenizer.json");

        if tokenizer_path.exists() {
            Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))
        } else {
            anyhow::bail!("Tokenizer not found")
        }
    }

    /// Generate text from a prompt
    pub fn generate(&mut self, prompt: &str, gen_config: &GenerateConfig) -> Result<String> {
        tracing::debug!("Generating from prompt ({} chars)", prompt.len());

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
            let input =
                candle_core::Tensor::new(input_tokens, self.device.device())?.unsqueeze(0)?;

            // Forward pass
            let logits = self
                .model
                .forward(&input, all_tokens.len().saturating_sub(input_tokens.len()))?;
            let logits = logits.squeeze(0)?.squeeze(0)?;

            // Sample next token
            let next_token = self.sample_token(&logits, gen_config, &all_tokens)?;

            // Check for EOS
            if next_token == self.eos_token_id {
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

        Ok(output)
    }

    /// Sample a token from logits
    fn sample_token(
        &self,
        logits: &candle_core::Tensor,
        config: &GenerateConfig,
        context: &[u32],
    ) -> Result<u32> {
        let logits = logits.to_dtype(candle_core::DType::F32)?;
        let logits = logits.to_vec1::<f32>()?;

        // Apply repetition penalty
        let mut logits = logits;
        let penalty_start = context.len().saturating_sub(64); // Hardcoded 64 context window for penalty
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
}

/// Simple random value generator
fn rand_value() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f32) / (u32::MAX as f32)
}

/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub model_path: PathBuf,
    pub device: String,
    pub max_context: usize,
}
