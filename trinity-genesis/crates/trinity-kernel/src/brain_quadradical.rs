// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Quadradical Brain Implementation (Trinity Jr.)
//!
//! Connects to the local Quadradical AI server (formerly llama_llama).
//! This acts as the "Junior" partner to Trinity, handling inference tasks.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::brain::{Brain, StreamToken};

/// Quadradical API client (OpenAI-compatible)
pub struct QuadradicalBrain {
    /// Quadradical API base URL (default: http://localhost:8081)
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
    /// Model identifier
    model: Option<String>,
}

impl Default for QuadradicalBrain {
    fn default() -> Self {
        Self::new("http://localhost:8081")
    }
}

impl QuadradicalBrain {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
            model: None,
        }
    }

    /// Create with specific model identifier
    pub fn with_model(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
            model: Some(model.to_string()),
        }
    }

    /// Set the model ID to use for requests
    pub fn set_model(&mut self, model: String) {
        self.model = Some(model);
    }
}

// OpenAI-compatible API types
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[async_trait]
impl Brain for QuadradicalBrain {
    fn name(&self) -> &'static str {
        "Quadradical (Trinity Jr.)"
    }

    fn is_ready(&self) -> bool {
        // Always report ready - actual availability checked via API
        true
    }

    async fn think(&self, prompt: &str) -> Result<String> {
        // Use the model name from config, or discover it
        let model_name = self.model.clone()
            .unwrap_or_else(|| "local-model".to_string()); // LM Studio often ignores this but 'local-model' is standard

        let request = ChatRequest {
            model: model_name,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: Some(4096),
            temperature: Some(0.7),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Quadradical API error: {} - {}",
                status,
                error_text
            ));
        }

        let chat_response: ChatResponse = response.json().await?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No response from Quadradical"))
    }

    async fn think_stream(
        &self,
        prompt: &str,
        token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String> {
        // For now, get full response and send as single token
        let result = self.think(prompt).await?;

        let _ = token_tx
            .send(StreamToken {
                text: result.clone(),
                index: 0,
                is_final: true,
            })
            .await;

        Ok(result)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Try to fetch real embedding from API
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: self.model.clone().unwrap_or_else(|| "text-embedding-nomic-embed-text-v1.5".to_string()),
        };

        match self.client
            .post(format!("{}/v1/embeddings", self.base_url))
            .json(&request)
            .send()
            .await 
        {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(emb_response) = response.json::<EmbeddingResponse>().await {
                        if let Some(data) = emb_response.data.first() {
                            return Ok(data.embedding.clone());
                        }
                    }
                }
            }
            Err(_) => {} // Fallback to hash
        }

        // Fallback: Deterministic Hash (borrowed from DesktopBrain)
        // This ensures the system doesn't crash if embeddings aren't supported
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::Hasher;
        hasher.write(text.as_bytes());
        let seed = hasher.finish();
        
        let mut vec = Vec::with_capacity(384);
        let mut val = seed;
        for _ in 0..384 {
            val = val.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            vec.push((val as f32 / u64::MAX as f32) * 2.0 - 1.0);
        }
        Ok(vec)
    }
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadradical_brain_default() {
        let brain = QuadradicalBrain::default();
        assert_eq!(brain.base_url, "http://localhost:8081");
        assert_eq!(brain.name(), "Quadradical (Trinity Jr.)");
        assert!(brain.is_ready());
    }
}
