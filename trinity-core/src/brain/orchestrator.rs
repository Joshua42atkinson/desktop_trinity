//! Brain Orchestrator - Routes Tasks to Appropriate Tier
//!
//! Manages model loading, task routing, and swarm parallelism.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::desktop::DesktopBrain;
use super::tiered::{classify_task, BrainTier, TieredBrainManager};
use super::{Brain, GenerationConfig, StreamToken};

// ============================================================================
// Orchestrator Request/Response
// ============================================================================

/// A request to the orchestrator
#[derive(Debug, Clone)]
pub struct OrchRequest {
    /// Unique request ID
    pub id: uuid::Uuid,
    /// The prompt to process
    pub prompt: String,
    /// Optional tier override (otherwise auto-classified)
    pub tier_override: Option<BrainTier>,
    /// Optional generation config override
    pub config_override: Option<GenerationConfig>,
    /// Whether to use streaming
    pub stream: bool,
}

impl OrchRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            prompt: prompt.into(),
            tier_override: None,
            config_override: None,
            stream: false,
        }
    }

    pub fn with_tier(mut self, tier: BrainTier) -> Self {
        self.tier_override = Some(tier);
        self
    }

    pub fn streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    pub fn with_config(mut self, config: GenerationConfig) -> Self {
        self.config_override = Some(config);
        self
    }
}

/// Response from the orchestrator
#[derive(Debug, Clone)]
pub struct OrchResponse {
    /// Request ID this responds to
    pub request_id: uuid::Uuid,
    /// The generated response
    pub response: String,
    /// Which tier processed this request
    pub tier: BrainTier,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Tokens generated (rough estimate)
    pub tokens: usize,
}

// ============================================================================
// Brain Orchestrator
// ============================================================================

/// Orchestrates AI processing across multiple tiers
pub struct BrainOrchestrator {
    /// The tiered manager
    manager: Arc<RwLock<TieredBrainManager>>,
    /// Currently loaded brains by tier
    brains: RwLock<std::collections::HashMap<BrainTier, Arc<DesktopBrain>>>,
    /// Max concurrent swarm tasks
    #[allow(dead_code)]
    max_swarm_concurrent: usize,
    /// Channel for streaming tokens
    token_tx: Option<mpsc::Sender<StreamToken>>,
}

impl BrainOrchestrator {
    /// Create a new orchestrator with a tiered manager
    pub fn new(manager: TieredBrainManager, max_swarm: usize) -> Self {
        Self {
            manager: Arc::new(RwLock::new(manager)),
            brains: RwLock::new(std::collections::HashMap::new()),
            max_swarm_concurrent: max_swarm,
            token_tx: None,
        }
    }

    /// Set the token streaming channel
    pub fn with_token_channel(mut self, tx: mpsc::Sender<StreamToken>) -> Self {
        self.token_tx = Some(tx);
        self
    }

    /// Process a request, routing to appropriate tier
    pub async fn process(&self, request: OrchRequest) -> Result<OrchResponse> {
        let start = std::time::Instant::now();

        // Determine which tier to use
        let tier = request
            .tier_override
            .unwrap_or_else(|| classify_task(&request.prompt));

        tracing::info!(
            "Routing request {} to {:?} tier",
            &request.id.to_string()[..8],
            tier
        );

        // Get or load the brain for this tier
        let brain = self.get_or_load_brain(tier).await?;

        // Get generation config
        let config = request
            .config_override
            .unwrap_or_else(|| tier.default_config());

        // Execute
        let response = if request.stream {
            if let Some(ref tx) = self.token_tx {
                brain.think_stream(&request.prompt, tx.clone()).await?
            } else {
                brain.think(&request.prompt).await?
            }
        } else {
            brain.think_with_config(&request.prompt, &config).await?
        };

        let duration = start.elapsed();
        let tokens = response.split_whitespace().count(); // Rough estimate

        Ok(OrchResponse {
            request_id: request.id,
            response,
            tier,
            duration_ms: duration.as_millis() as u64,
            tokens,
        })
    }

    /// Process multiple swarm requests in parallel
    pub async fn process_swarm(&self, requests: Vec<OrchRequest>) -> Vec<Result<OrchResponse>> {
        let mut results = Vec::with_capacity(requests.len());

        // Get swarm brain once
        let brain = match self.get_or_load_brain(BrainTier::Swarm).await {
            Ok(b) => b,
            Err(e) => {
                // Return error for all requests
                for req in requests {
                    results.push(Err(anyhow::anyhow!(
                        "Failed to load swarm brain: {} (request {})",
                        e,
                        req.id
                    )));
                }
                return results;
            }
        };

        // Process requests sequentially for now
        // TODO: True parallel execution with separate model instances
        for request in requests {
            let start = std::time::Instant::now();
            let config = BrainTier::Swarm.default_config();

            match brain.think_with_config(&request.prompt, &config).await {
                Ok(response) => {
                    let duration = start.elapsed();
                    let tokens = response.split_whitespace().count();
                    results.push(Ok(OrchResponse {
                        request_id: request.id,
                        response,
                        tier: BrainTier::Swarm,
                        duration_ms: duration.as_millis() as u64,
                        tokens,
                    }));
                }
                Err(e) => {
                    results.push(Err(e));
                }
            }
        }

        results
    }

    /// Get or load a brain for the specified tier
    async fn get_or_load_brain(&self, tier: BrainTier) -> Result<Arc<DesktopBrain>> {
        // Check if already loaded
        {
            let brains = self.brains.read().await;
            if let Some(brain) = brains.get(&tier) {
                return Ok(brain.clone());
            }
        }

        // Need to load - check config and clone the data we need
        let (model_path, model_name, size_gb, can_fit, gpu_layers) = {
            let manager = self.manager.read().await;
            let config = manager
                .get_tier_config(tier)
                .ok_or_else(|| anyhow::anyhow!("No model configured for {:?} tier", tier))?;

            let can_fit = manager.can_fit(config.size_gb).await;
            (
                config.model_path.clone(),
                config.name.clone(),
                config.size_gb,
                can_fit,
                config.gpu_layers,
            )
        };

        tracing::info!("Loading model for {:?} tier: {}", tier, model_name);

        // Check memory and unload if needed
        if !can_fit {
            self.unload_lower_priority(tier).await?;
        }

        // Load the model using the standard DesktopBrain API
        let brain = DesktopBrain::new().with_gpu_layers(gpu_layers);

        let model_path_str = model_path.to_string_lossy();
        brain.load_model(&model_path_str).await?;
        let brain = Arc::new(brain);

        // Store
        {
            let mut brains = self.brains.write().await;
            brains.insert(tier, brain.clone());
        }

        tracing::info!("Loaded {:?} tier ({:.1} GB)", tier, size_gb);
        Ok(brain)
    }

    /// Unload tiers with lower priority to make room
    async fn unload_lower_priority(&self, target_tier: BrainTier) -> Result<()> {
        let target_priority = target_tier.priority();

        let mut brains = self.brains.write().await;

        // Find tiers to unload (lower priority)
        let to_unload: Vec<BrainTier> = brains
            .keys()
            .filter(|t| t.priority() < target_priority)
            .copied()
            .collect();

        for tier in to_unload {
            tracing::info!("Unloading {:?} tier to make room", tier);
            if let Some(brain) = brains.remove(&tier) {
                brain.unload_model();
            }
        }

        Ok(())
    }

    /// Check which tiers are currently loaded
    pub async fn loaded_tiers(&self) -> Vec<BrainTier> {
        let brains = self.brains.read().await;
        brains.keys().copied().collect()
    }

    /// Force load a specific tier
    pub async fn preload(&self, tier: BrainTier) -> Result<()> {
        self.get_or_load_brain(tier).await?;
        Ok(())
    }

    /// Unload a specific tier
    pub async fn unload(&self, tier: BrainTier) {
        let mut brains = self.brains.write().await;
        if let Some(brain) = brains.remove(&tier) {
            brain.unload_model();
            tracing::info!("Unloaded {:?} tier", tier);
        }
    }

    /// Get memory usage summary
    pub async fn memory_summary(&self) -> MemorySummary {
        let manager = self.manager.read().await;
        let loaded: Vec<_> = self.loaded_tiers().await;

        let mut total_used = 0.0f32;
        let mut tier_usage = std::collections::HashMap::new();

        for tier in &loaded {
            if let Some(config) = manager.get_tier_config(*tier) {
                tier_usage.insert(*tier, config.size_gb);
                total_used += config.size_gb;
            }
        }

        MemorySummary {
            total_vram_gb: manager.total_vram_gb(),
            used_gb: total_used,
            tier_usage,
        }
    }
}

/// Memory usage summary
#[derive(Debug, Clone)]
pub struct MemorySummary {
    pub total_vram_gb: f32,
    pub used_gb: f32,
    pub tier_usage: std::collections::HashMap<BrainTier, f32>,
}

impl MemorySummary {
    pub fn usage_percent(&self) -> f32 {
        if self.total_vram_gb > 0.0 {
            (self.used_gb / self.total_vram_gb) * 100.0
        } else {
            0.0
        }
    }

    pub fn remaining_gb(&self) -> f32 {
        self.total_vram_gb - self.used_gb
    }
}

// ============================================================================
// Convenience Builder
// ============================================================================

/// Builder for creating a configured orchestrator
pub struct OrchestratorBuilder {
    manager: TieredBrainManager,
    max_swarm: usize,
    token_tx: Option<mpsc::Sender<StreamToken>>,
}

impl OrchestratorBuilder {
    /// Create with default Strix Halo configuration
    pub fn strix_halo() -> Self {
        Self {
            manager: super::tiered::strix_halo_presets(),
            max_swarm: 4, // 4 parallel Gemma instances
            token_tx: None,
        }
    }

    /// Create with custom tiered manager
    pub fn custom(manager: TieredBrainManager) -> Self {
        Self {
            manager,
            max_swarm: 4,
            token_tx: None,
        }
    }

    /// Set max concurrent swarm tasks
    pub fn max_swarm(mut self, max: usize) -> Self {
        self.max_swarm = max;
        self
    }

    /// Set token streaming channel
    pub fn with_token_channel(mut self, tx: mpsc::Sender<StreamToken>) -> Self {
        self.token_tx = Some(tx);
        self
    }

    /// Build the orchestrator
    pub fn build(self) -> BrainOrchestrator {
        let mut orch = BrainOrchestrator::new(self.manager, self.max_swarm);
        if let Some(tx) = self.token_tx {
            orch = orch.with_token_channel(tx);
        }
        orch
    }
}

// ============================================================================
// Brain Trait Implementation
// ============================================================================

use async_trait::async_trait;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Brain for BrainOrchestrator {
    async fn think(&self, prompt: &str) -> Result<String> {
        // Auto-routing via process
        let request = OrchRequest::new(prompt);
        let response = self.process(request).await?;
        Ok(response.response)
    }

    async fn think_with_config(&self, prompt: &str, config: &GenerationConfig) -> Result<String> {
        // Route with config override
        let request = OrchRequest::new(prompt).with_config(config.clone());
        let response = self.process(request).await?;
        Ok(response.response)
    }

    async fn think_stream(
        &self,
        prompt: &str,
        _token_tx: mpsc::Sender<StreamToken>,
    ) -> Result<String> {
        // Streaming support in Orchestrator currently uses the internal channel.
        // For trait compliance, we use the standard process, which might use the internal channel if configured.
        let request = OrchRequest::new(prompt).streaming();
        let response = self.process(request).await?;
        Ok(response.response)
    }

    async fn load_model(&self, _model_path: &str) -> Result<()> {
        tracing::warn!("load_model called on BrainOrchestrator - ignored (managed via tiers)");
        Ok(())
    }

    fn model_info(&self) -> Option<super::ModelInfo> {
        Some(super::ModelInfo {
            name: "Tiered Brain Orchestrator".to_string(),
            path: "multi-tier".to_string(),
            size_bytes: 0,
            quantization: "mixed".to_string(),
            context_size: 131072, // Aggregate or Max
            loaded: true,
        })
    }

    fn name(&self) -> &'static str {
        "BrainOrchestrator"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_summary() {
        let summary = MemorySummary {
            total_vram_gb: 124.0,
            used_gb: 60.0,
            tier_usage: std::collections::HashMap::new(),
        };

        assert!((summary.usage_percent() - 48.39).abs() < 0.1);
        assert!((summary.remaining_gb() - 64.0).abs() < 0.1);
    }
}
