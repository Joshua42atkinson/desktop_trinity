#![allow(unused)]
//! LM Studio Client - OpenAI-compatible API for local LLM inference
//!
//! Connects to LM Studio running locally at http://localhost:1234
//! for GPT-OSS 120B model inference.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LM Studio client for local LLM inference
pub struct LMStudioClient {
    endpoint: String,
    model: String,
    timeout: Duration,
}

/// Chat message for the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Request body for chat completions
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

/// Response from chat completions
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

/// Models list response
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
}

impl LMStudioClient {
    /// Create a new LM Studio client with default settings
    pub fn new() -> Self {
        Self {
            endpoint: "http://localhost:1234".to_string(),
            model: "gpt-oss-120b".to_string(),
            timeout: Duration::from_secs(120), // 2 min for large model
        }
    }

    /// Create client with custom endpoint
    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            ..Self::new()
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if LM Studio is reachable
    pub async fn is_available(&self) -> bool {
        self.list_models().await.is_ok()
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/v1/models", self.endpoint);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let response: ModelsResponse = client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to LM Studio")?
            .json()
            .await
            .context("Failed to parse models response")?;

        Ok(response.data.into_iter().map(|m| m.id).collect())
    }

    /// Generate a completion for a single prompt
    pub async fn complete(&self, prompt: &str) -> Result<String> {
        self.chat(&[ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }])
        .await
    }

    /// Generate a chat completion
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String> {
        self.chat_with_config(messages, 0.7, 1500).await
    }

    /// Generate a chat completion with custom parameters
    pub async fn chat_with_config(
        &self,
        messages: &[ChatMessage],
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String> {
        let url = format!("{}/v1/chat/completions", self.endpoint);

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            temperature,
            max_tokens,
            stream: false,
        };

        let client = reqwest::Client::builder().timeout(self.timeout).build()?;

        let response: ChatCompletionResponse = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LM Studio")?
            .json()
            .await
            .context("Failed to parse completion response")?;

        response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No completion returned"))
    }

    /// Generate code for a given prompt
    pub async fn generate_code(&self, prompt: &str, language: &str) -> Result<String> {
        let system_prompt = format!(
            "You are an expert {} programmer. Generate clean, well-documented code. \
            Only output code, no explanations unless requested.",
            language
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ];

        self.chat_with_config(&messages, 0.4, 2000).await
    }

    /// Edit existing code based on instructions
    pub async fn edit_code(&self, code: &str, instructions: &str) -> Result<String> {
        let prompt = format!(
            "Here is the code to edit:\n```\n{}\n```\n\nInstructions: {}\n\n\
            Return ONLY the complete modified code, no explanations.",
            code, instructions
        );

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a precise code editor. Apply the requested changes exactly."
                    .to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        self.chat_with_config(&messages, 0.2, 3000).await
    }

    /// Explain code
    pub async fn explain_code(&self, code: &str) -> Result<String> {
        let prompt = format!(
            "Explain the following code clearly and concisely:\n```\n{}\n```",
            code
        );

        self.chat(&[ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }])
        .await
    }
}

impl Default for LMStudioClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LMStudioClient::new();
        assert_eq!(client.endpoint, "http://localhost:1234");
        assert_eq!(client.model, "gpt-oss-120b");
    }

    #[test]
    fn test_builder_pattern() {
        let client = LMStudioClient::with_endpoint("http://custom:5000")
            .with_model("llama3.1:8b")
            .with_timeout(Duration::from_secs(60));

        assert_eq!(client.endpoint, "http://custom:5000");
        assert_eq!(client.model, "llama3.1:8b");
        assert_eq!(client.timeout, Duration::from_secs(60));
    }
}
