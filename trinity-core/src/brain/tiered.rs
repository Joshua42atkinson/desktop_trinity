//! Tiered Brain System - Multi-Model AI Architecture
//!
//! Provides a three-tier AI system optimized for different workloads:
//! - **Tier 1 (Reflection)**: Large models (Qwen 235B) for deep thinking
//! - **Tier 2 (Tasks)**: Medium models (GPT-OSS 120B) for regular work
//! - **Tier 3 (Swarm)**: Small models (Gemma 3 2B) for parallel function calls

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::brain::{Brain, GenerationConfig};

// ============================================================================
// Tier Definitions
// ============================================================================

/// AI processing tier based on task complexity and requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrainTier {
    /// Deep reflection, complex reasoning, creative synthesis
    /// Uses largest available model (e.g., Qwen 235B)
    /// Best for: Philosophy, novel solutions, multi-step reasoning
    Reflection,

    /// Standard task execution, code generation, analysis
    /// Uses medium model (e.g., GPT-OSS 120B)
    /// Best for: Code writing, document analysis, general tasks
    Tasks,

    /// Lightweight parallel operations, function calling
    /// Uses small fine-tuned models (e.g., Gemma 3 2B)
    /// Best for: Tool calling, data extraction, simple Q&A
    Swarm,
}

impl BrainTier {
    /// Get default generation config for this tier
    pub fn default_config(&self) -> GenerationConfig {
        match self {
            BrainTier::Reflection => GenerationConfig {
                max_tokens: 8192,
                temperature: 0.7,
                top_p: 0.95,
                top_k: 40,
                repetition_penalty: 1.1,
                stop_sequences: vec!["</think>".to_string()],
            },
            BrainTier::Tasks => GenerationConfig {
                max_tokens: 4096,
                temperature: 0.3,
                top_p: 0.9,
                top_k: 30,
                repetition_penalty: 1.05,
                stop_sequences: vec!["```".to_string(), "</output>".to_string()],
            },
            BrainTier::Swarm => GenerationConfig {
                max_tokens: 512,
                temperature: 0.1,
                top_p: 0.85,
                top_k: 20,
                repetition_penalty: 1.0,
                stop_sequences: vec!["}".to_string(), "\n\n".to_string()],
            },
        }
    }

    /// Get suggested model size range for this tier (in GB)
    pub fn model_size_range(&self) -> (f32, f32) {
        match self {
            BrainTier::Reflection => (80.0, 200.0), // 80-200GB models
            BrainTier::Tasks => (20.0, 80.0),       // 20-80GB models
            BrainTier::Swarm => (1.0, 10.0),        // 1-10GB models
        }
    }

    /// Get typical context window for this tier
    pub fn context_window(&self) -> usize {
        match self {
            BrainTier::Reflection => 32768, // Long context for deep thinking
            BrainTier::Tasks => 16384,      // Standard context
            BrainTier::Swarm => 4096,       // Short context for speed
        }
    }

    /// Memory priority (higher = load first, keep longer)
    pub fn priority(&self) -> u8 {
        match self {
            BrainTier::Reflection => 3,
            BrainTier::Tasks => 2,
            BrainTier::Swarm => 1,
        }
    }
}

// ============================================================================
// Model Assignment
// ============================================================================

/// Model configuration for a specific tier
#[derive(Debug, Clone)]
pub struct TierConfig {
    /// Path to the model file
    pub model_path: PathBuf,
    /// Friendly name
    pub name: String,
    /// Model size in GB
    pub size_gb: f32,
    /// Whether to keep loaded in memory
    pub keep_loaded: bool,
    /// Number of GPU layers to offload
    pub gpu_layers: i32,
    /// Custom generation config (or None for tier default)
    pub generation_config: Option<GenerationConfig>,
}

impl TierConfig {
    pub fn new(path: impl Into<PathBuf>, name: impl Into<String>, size_gb: f32) -> Self {
        Self {
            model_path: path.into(),
            name: name.into(),
            size_gb,
            keep_loaded: false,
            gpu_layers: 999, // Force all layers to GPU by default
            generation_config: None,
        }
    }

    pub fn keep_loaded(mut self) -> Self {
        self.keep_loaded = true;
        self
    }

    pub fn with_gpu_layers(mut self, layers: i32) -> Self {
        self.gpu_layers = layers;
        self
    }
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            name: "Default".to_string(),
            size_gb: 0.0,
            keep_loaded: false,
            gpu_layers: 999,
            generation_config: None,
        }
    }
}

// ============================================================================
// Swarm Slot
// ============================================================================

/// A slot in the swarm tier for parallel model instances
#[derive(Debug, Clone)]
pub struct SwarmSlot {
    /// Slot identifier
    pub id: usize,
    /// Model path
    pub model_path: PathBuf,
    /// Model specialization (e.g., "code", "json", "chat")
    pub specialization: String,
    /// Whether currently in use
    pub in_use: bool,
}

// ============================================================================
// Tiered Brain Manager
// ============================================================================

/// Manages the three-tier AI architecture
pub struct TieredBrainManager {
    /// Model configurations per tier
    tier_configs: HashMap<BrainTier, TierConfig>,
    /// Brain instances per tier
    brains: RwLock<HashMap<BrainTier, Arc<dyn Brain>>>,
    /// Swarm slots for parallel execution (sync Mutex for non-async access)
    swarm_slots: Mutex<Vec<SwarmSlot>>,
    /// Currently active tier (for memory management)
    active_tier: RwLock<Option<BrainTier>>,
    /// Total available VRAM in bytes
    total_vram: u64,
}

impl TieredBrainManager {
    /// Create a new tiered brain manager
    pub fn new(total_vram_gb: f32) -> Self {
        Self {
            tier_configs: HashMap::new(),
            brains: RwLock::new(HashMap::new()),
            swarm_slots: Mutex::new(Vec::new()),
            active_tier: RwLock::new(None),
            total_vram: (total_vram_gb * 1024.0 * 1024.0 * 1024.0) as u64,
        }
    }

    /// Configure a tier with a specific model
    pub fn configure_tier(&mut self, tier: BrainTier, config: TierConfig) {
        tracing::info!(
            "Configured {:?} tier: {} ({:.1} GB)",
            tier,
            config.name,
            config.size_gb
        );
        self.tier_configs.insert(tier, config);
    }

    /// Add a swarm slot with a specialized model
    pub fn add_swarm_slot(&mut self, model_path: PathBuf, specialization: impl Into<String>) {
        let mut slots = self.swarm_slots.lock().expect("swarm_slots lock poisoned");
        let id = slots.len();
        slots.push(SwarmSlot {
            id,
            model_path,
            specialization: specialization.into(),
            in_use: false,
        });
    }

    /// Get the brain for a specific tier
    pub async fn get_brain(&self, tier: BrainTier) -> Option<Arc<dyn Brain>> {
        let brains = self.brains.read().await;
        brains.get(&tier).cloned()
    }

    /// Check if a tier is loaded
    pub async fn is_tier_loaded(&self, tier: BrainTier) -> bool {
        let brains = self.brains.read().await;
        brains.contains_key(&tier)
    }

    /// Get the currently active tier
    pub async fn active_tier(&self) -> Option<BrainTier> {
        *self.active_tier.read().await
    }

    /// Calculate memory usage for loaded tiers
    pub async fn memory_usage(&self) -> f32 {
        let brains = self.brains.read().await;
        let mut total = 0.0f32;

        for (tier, _) in brains.iter() {
            if let Some(config) = self.tier_configs.get(tier) {
                total += config.size_gb;
            }
        }

        total
    }

    /// Check if we can fit a model in remaining memory
    pub async fn can_fit(&self, size_gb: f32) -> bool {
        let used = self.memory_usage().await;
        let total_gb = self.total_vram as f32 / (1024.0 * 1024.0 * 1024.0);
        (used + size_gb) <= total_gb * 0.95 // Leave 5% buffer
    }

    /// Get tier configuration
    pub fn get_tier_config(&self, tier: BrainTier) -> Option<&TierConfig> {
        self.tier_configs.get(&tier)
    }

    /// Get available swarm slots
    pub fn available_swarm_slots(&self) -> Vec<usize> {
        let slots = self.swarm_slots.lock().expect("swarm_slots lock poisoned");
        slots.iter().filter(|s| !s.in_use).map(|s| s.id).collect()
    }

    /// Reserve a swarm slot
    pub fn reserve_swarm_slot(&self, specialization: Option<&str>) -> Option<usize> {
        let mut slots = self.swarm_slots.lock().expect("swarm_slots lock poisoned");

        // Find matching specialization if requested
        if let Some(spec) = specialization {
            for slot in slots.iter_mut() {
                if !slot.in_use && slot.specialization == spec {
                    slot.in_use = true;
                    return Some(slot.id);
                }
            }
        }

        // Otherwise, get any available slot
        for slot in slots.iter_mut() {
            if !slot.in_use {
                slot.in_use = true;
                return Some(slot.id);
            }
        }

        None
    }

    /// Release a swarm slot
    pub fn release_swarm_slot(&self, id: usize) {
        let mut slots = self.swarm_slots.lock().expect("swarm_slots lock poisoned");
        if let Some(slot) = slots.get_mut(id) {
            slot.in_use = false;
        }
    }

    /// Get total VRAM in GB
    pub fn total_vram_gb(&self) -> f32 {
        self.total_vram as f32 / (1024.0 * 1024.0 * 1024.0)
    }
}

// ============================================================================
// Task Classification
// ============================================================================

/// Classify a task to determine which tier should handle it
pub fn classify_task(_prompt: &str) -> BrainTier {
    // ENFORCED SINGLE BRAIN ARCHITECTURE (Llama 4 Scout)
    // All tasks route to the main model for maximum capability and memory stability.
    BrainTier::Tasks
}

// ============================================================================
// Preset Configurations
// ============================================================================

/// Create a tiered manager with Strix Halo optimized presets
pub fn strix_halo_presets() -> TieredBrainManager {
    println!("DEBUG: strix_halo_presets called! Configuring Single-Brain (Llama 4 Scout)...");
    let mut manager = TieredBrainManager::new(124.0); // 124GB VRAM

    // Tier: Tasks - Llama 4 Scout 17B (Actual size ~64GB on disk / 70GB VRAM)
    // The Heart of Trinity: One powerful model for all inference.
    manager.configure_tier(
        BrainTier::Tasks,
        TierConfig::new(
            "/home/joshua/.lmstudio/models/lmstudio-community/Llama-4-Scout-17B-16E-Instruct-GGUF/Llama-4-Scout-17B-16E-Instruct-Q4_K_M-00001-of-00002.gguf",
            "Llama 4 Scout (Trinity Core)",
            65.0, 
        )
        .keep_loaded(),
    );

    // Note: Reflection and Swarm tiers are intentionally not configured.
    // The classify_task function ensures everything routes to Tasks.

    manager
}


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_classification() {
        // All should return Tasks in Single-Brain mode
        assert_eq!(classify_task("Reflect deeply"), BrainTier::Tasks);
        assert_eq!(classify_task("Extract JSON"), BrainTier::Tasks);
    }

    #[test]
    fn test_tier_configs() {
        assert_eq!(BrainTier::Reflection.context_window(), 32768);
        assert_eq!(BrainTier::Tasks.context_window(), 16384);
        assert_eq!(BrainTier::Swarm.context_window(), 4096);

        assert!(BrainTier::Reflection.priority() > BrainTier::Tasks.priority());
        assert!(BrainTier::Tasks.priority() > BrainTier::Swarm.priority());
    }

    #[test]
    fn test_swarm_slots() {
        let mut manager = TieredBrainManager::new(128.0);
        manager.add_swarm_slot(PathBuf::from("/test/model"), "code");
        manager.add_swarm_slot(PathBuf::from("/test/model"), "json");

        let available = manager.available_swarm_slots();
        assert_eq!(available.len(), 2);

        let slot = manager.reserve_swarm_slot(Some("code"));
        assert!(slot.is_some());

        let available = manager.available_swarm_slots();
        assert_eq!(available.len(), 1);

        manager.release_swarm_slot(slot.unwrap());
        let available = manager.available_swarm_slots();
        assert_eq!(available.len(), 2);
    }
}
