#![allow(unused)]
//! Qwen3 Model Support - Specific handling for Qwen3 architecture
//!
//! Qwen3 is a Mixture of Experts (MoE) model with special features:
//! - Thinking mode with <think>...</think> tags
//! - Long context support (up to 128K tokens)
//! - MoE routing (~22B active out of 235B total)

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Qwen3-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qwen3Config {
    /// Enable thinking mode (parse <think> tags)
    pub enable_thinking: bool,
    /// Show thinking content to user
    pub show_thinking: bool,
    /// Maximum thinking tokens before cutoff
    pub max_thinking_tokens: usize,
    /// Context length (Qwen3 supports up to 128K)
    pub context_length: usize,
}

impl Default for Qwen3Config {
    fn default() -> Self {
        Self {
            enable_thinking: true,
            show_thinking: false,
            max_thinking_tokens: 4096,
            context_length: 32768,
        }
    }
}

impl Qwen3Config {
    /// Full thinking mode - show all reasoning
    pub fn full_thinking() -> Self {
        Self {
            enable_thinking: true,
            show_thinking: true,
            max_thinking_tokens: 8192,
            context_length: 32768,
        }
    }

    /// Fast mode - no thinking overhead
    pub fn fast() -> Self {
        Self {
            enable_thinking: false,
            show_thinking: false,
            max_thinking_tokens: 0,
            context_length: 8192,
        }
    }
}

/// Parsed Qwen3 response with thinking separated
#[derive(Debug, Clone)]
pub struct Qwen3Response {
    /// The thinking/reasoning content (if present)
    pub thinking: Option<String>,
    /// The actual response content
    pub response: String,
    /// Whether tool calls were detected
    pub has_tool_calls: bool,
}

/// Parse a Qwen3 response, extracting thinking content
pub fn parse_qwen3_response(text: &str) -> Qwen3Response {
    let mut thinking = None;
    let mut response = text.to_string();

    // Extract <think>...</think> content
    if let Some(start) = text.find("<think>") {
        if let Some(end) = text.find("</think>") {
            let think_content = &text[start + 7..end];
            thinking = Some(think_content.trim().to_string());

            // Remove thinking from response
            response = format!("{}{}", text[..start].trim(), text[end + 8..].trim())
                .trim()
                .to_string();
        }
    }

    // Check for tool calls
    let has_tool_calls = response.contains("<tool_call>");

    Qwen3Response {
        thinking,
        response,
        has_tool_calls,
    }
}

/// Format a prompt for Qwen3 with ChatML format
pub fn format_qwen3_prompt(
    system: &str,
    messages: &[(String, String)], // (role, content) pairs
    enable_thinking: bool,
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str(&format!("<|im_start|>system\n{}<|im_end|>\n", system));

    // Conversation history
    for (role, content) in messages {
        let role_tag = match role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => "user",
        };
        prompt.push_str(&format!(
            "<|im_start|>{}\n{}<|im_end|>\n",
            role_tag, content
        ));
    }

    // Start assistant response
    prompt.push_str("<|im_start|>assistant\n");

    // Add thinking prompt if enabled
    if enable_thinking {
        prompt.push_str("<think>\n");
    }

    prompt
}

/// Memory estimation for Qwen3-235B on AMD Strix Halo
pub fn estimate_memory_usage(context_length: usize, quantization: &str) -> MemoryEstimate {
    // Base model size estimates by quantization
    // Qwen3-235B with q2_k_s is approximately 60-70GB
    let base_model_gb = match quantization {
        "q2_k_s" => 65.0,
        "q3_k_m" => 85.0,
        "q4_k_m" => 120.0,
        "q5_k_m" => 150.0,
        "q8_0" => 235.0,
        _ => 100.0,
    };

    // KV cache per token (MoE models are more efficient)
    // Qwen3 has 22B active params, so KV cache is smaller
    let kv_cache_per_token_mb = 0.3;
    let kv_cache_gb = (context_length as f64 * kv_cache_per_token_mb) / 1024.0;

    // Total with overhead (CUDA/ROCm runtime, etc.)
    let overhead_gb = 4.0;
    let total_gb = base_model_gb + kv_cache_gb + overhead_gb;

    // AMD Strix Halo has 128GB unified memory
    let strix_halo_memory_gb = 128.0;

    MemoryEstimate {
        model_gb: base_model_gb,
        kv_cache_gb,
        overhead_gb,
        total_gb,
        fits_in_memory: total_gb <= strix_halo_memory_gb,
        recommended_context: if total_gb <= strix_halo_memory_gb {
            context_length
        } else {
            // Calculate max context that fits
            let available_for_kv = (strix_halo_memory_gb - base_model_gb - overhead_gb) * 1024.0;
            (available_for_kv / kv_cache_per_token_mb) as usize
        },
    }
}

/// Memory usage estimate
#[derive(Debug, Clone)]
pub struct MemoryEstimate {
    pub model_gb: f64,
    pub kv_cache_gb: f64,
    pub overhead_gb: f64,
    pub total_gb: f64,
    pub fits_in_memory: bool,
    pub recommended_context: usize,
}

impl std::fmt::Display for MemoryEstimate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Model: {:.1}GB + KV Cache: {:.1}GB + Overhead: {:.1}GB = Total: {:.1}GB ({})",
            self.model_gb,
            self.kv_cache_gb,
            self.overhead_gb,
            self.total_gb,
            if self.fits_in_memory {
                "✓ Fits in 128GB"
            } else {
                "✗ Exceeds memory"
            }
        )
    }
}

/// Qwen3 special token IDs
pub mod tokens {
    pub const IM_START: u32 = 151644;
    pub const IM_END: u32 = 151645;
    pub const EOS: u32 = 151643;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_thinking() {
        let text = "<think>\nLet me analyze this...\n</think>\n\nHere is my response.";
        let parsed = parse_qwen3_response(text);

        assert!(parsed.thinking.is_some());
        assert!(parsed.thinking.unwrap().contains("analyze"));
        assert_eq!(parsed.response, "Here is my response.");
    }

    #[test]
    fn test_no_thinking() {
        let text = "Just a normal response without thinking.";
        let parsed = parse_qwen3_response(text);

        assert!(parsed.thinking.is_none());
        assert_eq!(parsed.response, text);
    }

    #[test]
    fn test_memory_estimate() {
        let estimate = estimate_memory_usage(32768, "q2_k_s");

        assert!(estimate.fits_in_memory);
        assert!(estimate.total_gb < 128.0);
    }

    #[test]
    fn test_prompt_format() {
        let prompt = format_qwen3_prompt(
            "You are a helpful assistant.",
            &[("user".to_string(), "Hello".to_string())],
            true,
        );

        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("<think>"));
    }
}
