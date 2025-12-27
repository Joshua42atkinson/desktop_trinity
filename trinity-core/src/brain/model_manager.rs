//! Model Manager - Model Catalog and Hot-Swap System for Trinity
//!
//! Provides model discovery, metadata extraction, and safe hot-swapping
//! with proper memory management integration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::config::TrinityConfig;
use crate::memory::UnifiedMemoryManager;

// ============================================================================
// Model Metadata
// ============================================================================

/// Quantization type detected from filename
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quantization {
    F16,
    F32,
    Q2K,
    Q3KS,
    Q3KM,
    Q3KL,
    Q4KS,
    Q4KM,
    Q5KS,
    Q5KM,
    Q6K,
    Q8_0,
    IQ1S,
    IQ2XXS,
    IQ2XS,
    IQ3XXS,
    IQ4XS,
    Unknown(String),
}

impl Quantization {
    /// Parse quantization from filename
    pub fn from_filename(filename: &str) -> Self {
        let upper = filename.to_uppercase();

        // Check for IQ (importance quantization) first
        if upper.contains("IQ4_XS") || upper.contains("IQ4XS") {
            return Quantization::IQ4XS;
        }
        if upper.contains("IQ3_XXS") || upper.contains("IQ3XXS") {
            return Quantization::IQ3XXS;
        }
        if upper.contains("IQ2_XS") || upper.contains("IQ2XS") {
            return Quantization::IQ2XS;
        }
        if upper.contains("IQ2_XXS") || upper.contains("IQ2XXS") {
            return Quantization::IQ2XXS;
        }
        if upper.contains("IQ1_S") || upper.contains("IQ1S") {
            return Quantization::IQ1S;
        }

        // Standard quantizations
        if upper.contains("Q8_0") || upper.contains("Q8-0") {
            return Quantization::Q8_0;
        }
        if upper.contains("Q6_K") || upper.contains("Q6K") {
            return Quantization::Q6K;
        }
        if upper.contains("Q5_K_M") || upper.contains("Q5KM") {
            return Quantization::Q5KM;
        }
        if upper.contains("Q5_K_S") || upper.contains("Q5KS") {
            return Quantization::Q5KS;
        }
        if upper.contains("Q4_K_M") || upper.contains("Q4KM") {
            return Quantization::Q4KM;
        }
        if upper.contains("Q4_K_S") || upper.contains("Q4KS") {
            return Quantization::Q4KS;
        }
        if upper.contains("Q3_K_L") || upper.contains("Q3KL") {
            return Quantization::Q3KL;
        }
        if upper.contains("Q3_K_M") || upper.contains("Q3KM") {
            return Quantization::Q3KM;
        }
        if upper.contains("Q3_K_S") || upper.contains("Q3KS") {
            return Quantization::Q3KS;
        }
        if upper.contains("Q2_K") || upper.contains("Q2K") {
            return Quantization::Q2K;
        }
        if upper.contains("F16") || upper.contains("FP16") {
            return Quantization::F16;
        }
        if upper.contains("F32") || upper.contains("FP32") {
            return Quantization::F32;
        }

        Quantization::Unknown("Unknown".to_string())
    }

    /// Estimated bits per weight for memory calculations
    pub fn bits_per_weight(&self) -> f64 {
        match self {
            Quantization::F32 => 32.0,
            Quantization::F16 => 16.0,
            Quantization::Q8_0 => 8.0,
            Quantization::Q6K => 6.5,
            Quantization::Q5KM | Quantization::Q5KS => 5.5,
            Quantization::Q4KM | Quantization::Q4KS => 4.5,
            Quantization::Q3KL | Quantization::Q3KM | Quantization::Q3KS => 3.5,
            Quantization::Q2K => 2.5,
            Quantization::IQ4XS => 4.25,
            Quantization::IQ3XXS => 3.0,
            Quantization::IQ2XS | Quantization::IQ2XXS => 2.3,
            Quantization::IQ1S => 1.5,
            Quantization::Unknown(_) => 4.0, // Assume Q4 as default
        }
    }
}

impl std::fmt::Display for Quantization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Quantization::F16 => write!(f, "F16"),
            Quantization::F32 => write!(f, "F32"),
            Quantization::Q2K => write!(f, "Q2_K"),
            Quantization::Q3KS => write!(f, "Q3_K_S"),
            Quantization::Q3KM => write!(f, "Q3_K_M"),
            Quantization::Q3KL => write!(f, "Q3_K_L"),
            Quantization::Q4KS => write!(f, "Q4_K_S"),
            Quantization::Q4KM => write!(f, "Q4_K_M"),
            Quantization::Q5KS => write!(f, "Q5_K_S"),
            Quantization::Q5KM => write!(f, "Q5_K_M"),
            Quantization::Q6K => write!(f, "Q6_K"),
            Quantization::Q8_0 => write!(f, "Q8_0"),
            Quantization::IQ1S => write!(f, "IQ1_S"),
            Quantization::IQ2XXS => write!(f, "IQ2_XXS"),
            Quantization::IQ2XS => write!(f, "IQ2_XS"),
            Quantization::IQ3XXS => write!(f, "IQ3_XXS"),
            Quantization::IQ4XS => write!(f, "IQ4_XS"),
            Quantization::Unknown(s) => write!(f, "{}", s),
        }
    }
}

/// Metadata about a discovered model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Display name (derived from filename)
    pub name: String,
    /// Full path to the model file
    pub path: PathBuf,
    /// File size in bytes
    pub size_bytes: u64,
    /// Detected quantization type
    pub quantization: Quantization,
    /// Whether this is part of a split model (multiple files)
    pub is_split: bool,
    /// Part number if split (e.g., 1 of 3)
    pub split_part: Option<(u32, u32)>,
    /// All files for this model (if split)
    pub all_files: Vec<PathBuf>,
}

impl ModelMetadata {
    /// Get size in GB
    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Get total size including all split files
    pub fn total_size_bytes(&self) -> u64 {
        if self.all_files.is_empty() {
            self.size_bytes
        } else {
            self.all_files
                .iter()
                .filter_map(|p| std::fs::metadata(p).ok())
                .map(|m| m.len())
                .sum()
        }
    }

    /// Get total size in GB
    pub fn total_size_gb(&self) -> f64 {
        self.total_size_bytes() as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

impl std::fmt::Display for ModelMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}, {:.1} GB)",
            self.name,
            self.quantization,
            self.total_size_gb()
        )
    }
}

// ============================================================================
// Model Presets
// ============================================================================

/// Predefined model presets for different use cases
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelPreset {
    /// Fast model for quick responses (smaller, ~7-20B)
    Fast,
    /// Smart/capable model for complex tasks (~70-235B)
    Smart,
    /// Optimized for code generation
    Code,
    /// Creative writing model
    Creative,
    /// Custom user-defined preset
    Custom(String),
}

impl std::fmt::Display for ModelPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelPreset::Fast => write!(f, "Fast"),
            ModelPreset::Smart => write!(f, "Smart"),
            ModelPreset::Code => write!(f, "Code"),
            ModelPreset::Creative => write!(f, "Creative"),
            ModelPreset::Custom(name) => write!(f, "Custom: {}", name),
        }
    }
}

// ============================================================================
// Model Catalog
// ============================================================================

/// Catalog of available models discovered in the models directory
#[derive(Debug, Clone)]
pub struct ModelCatalog {
    /// All discovered models, keyed by normalized name
    models: HashMap<String, ModelMetadata>,
    /// Preset assignments
    presets: HashMap<ModelPreset, String>,
    /// Root directory for model scanning
    root_directory: PathBuf,
}

impl ModelCatalog {
    /// Create a new catalog by scanning a directory
    pub fn scan(directory: impl AsRef<Path>) -> Result<Self> {
        let root = directory.as_ref().to_path_buf();
        let mut models = HashMap::new();

        tracing::info!("Scanning for models in: {:?}", root);

        Self::scan_directory(&root, &mut models)?;

        tracing::info!("Found {} model(s)", models.len());

        Ok(Self {
            models,
            presets: HashMap::new(),
            root_directory: root,
        })
    }

    /// Recursively scan directory for .gguf files
    fn scan_directory(dir: &Path, models: &mut HashMap<String, ModelMetadata>) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir).context("Failed to read directory")?;

        // First pass: collect all GGUF files
        let mut gguf_files: Vec<PathBuf> = Vec::new();
        let mut subdirs: Vec<PathBuf> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                subdirs.push(path);
            } else if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "gguf" {
                    gguf_files.push(path);
                }
            }
        }

        // Process GGUF files, grouping split models
        let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();

        for path in &gguf_files {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();

            // Check if this is a split file (contains -00001-of-00003 pattern)
            if let Some((base_name, part, total)) = Self::parse_split_filename(&filename) {
                if processed.contains(&base_name) {
                    continue;
                }

                // Find all parts
                let all_parts: Vec<PathBuf> = gguf_files
                    .iter()
                    .filter(|p| {
                        let f = p.file_name().unwrap_or_default().to_string_lossy();
                        Self::parse_split_filename(&f)
                            .map(|(n, _, _)| n == base_name)
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();

                let metadata = std::fs::metadata(path)?;
                let model_name = Self::extract_model_name(&base_name);

                models.insert(
                    model_name.clone(),
                    ModelMetadata {
                        name: model_name,
                        path: path.clone(),
                        size_bytes: metadata.len(),
                        quantization: Quantization::from_filename(&filename),
                        is_split: true,
                        split_part: Some((part, total)),
                        all_files: all_parts,
                    },
                );

                processed.insert(base_name);
            } else {
                // Single file model
                let metadata = std::fs::metadata(path)?;
                let model_name = Self::extract_model_name(&filename);

                models.insert(
                    model_name.clone(),
                    ModelMetadata {
                        name: model_name,
                        path: path.clone(),
                        size_bytes: metadata.len(),
                        quantization: Quantization::from_filename(&filename),
                        is_split: false,
                        split_part: None,
                        all_files: vec![path.clone()],
                    },
                );
            }
        }

        // Recurse into subdirectories
        for subdir in subdirs {
            Self::scan_directory(&subdir, models)?;
        }

        Ok(())
    }

    /// Parse split filename pattern (e.g., "model-00001-of-00003.gguf")
    fn parse_split_filename(filename: &str) -> Option<(String, u32, u32)> {
        // Pattern: -NNNNN-of-NNNNN.gguf
        let re_pattern = "-([0-9]+)-of-([0-9]+)\\.gguf$";
        let re = regex::Regex::new(re_pattern).ok()?;

        if let Some(caps) = re.captures(filename) {
            let part: u32 = caps.get(1)?.as_str().parse().ok()?;
            let total: u32 = caps.get(2)?.as_str().parse().ok()?;
            let base = filename[..caps.get(0)?.start()].to_string();
            return Some((base, part, total));
        }

        None
    }

    /// Extract a clean model name from filename
    fn extract_model_name(filename: &str) -> String {
        let name = filename.trim_end_matches(".gguf").trim_end_matches(".GGUF");

        // Remove split suffix if present
        if let Some(idx) = name.find("-00001-of-") {
            return name[..idx].to_string();
        }

        name.to_string()
    }

    /// Get all available models
    pub fn list(&self) -> Vec<&ModelMetadata> {
        self.models.values().collect()
    }

    /// Get a model by name
    pub fn get(&self, name: &str) -> Option<&ModelMetadata> {
        self.models.get(name)
    }

    /// Find models matching a search query (fuzzy)
    pub fn search(&self, query: &str) -> Vec<&ModelMetadata> {
        let query_lower = query.to_lowercase();
        self.models
            .values()
            .filter(|m| m.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Set a preset to use a specific model
    pub fn set_preset(&mut self, preset: ModelPreset, model_name: &str) {
        if self.models.contains_key(model_name) {
            self.presets.insert(preset, model_name.to_string());
        }
    }

    /// Get the model for a preset
    pub fn get_preset(&self, preset: &ModelPreset) -> Option<&ModelMetadata> {
        self.presets
            .get(preset)
            .and_then(|name| self.models.get(name))
    }

    /// Rescan the models directory
    pub fn refresh(&mut self) -> Result<()> {
        let new_catalog = Self::scan(&self.root_directory)?;
        self.models = new_catalog.models;
        Ok(())
    }
}

// ============================================================================
// Model Manager
// ============================================================================

/// Current load state of a model
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelLoadState {
    /// No model loaded
    Unloaded,
    /// Model is currently loading
    Loading { progress: u8 },
    /// Model is loaded and ready
    Loaded,
    /// Load failed with error
    Failed { error: String },
}

/// Model Manager - Orchestrates model loading/unloading with memory integration
pub struct ModelManager {
    /// Model catalog
    catalog: RwLock<ModelCatalog>,
    /// Currently loaded model name (if any)
    current_model: RwLock<Option<String>>,
    /// Load state
    load_state: RwLock<ModelLoadState>,
    /// Memory manager for allocation tracking
    memory_manager: Arc<UnifiedMemoryManager>,
    /// Configuration
    config: TrinityConfig,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new(memory_manager: Arc<UnifiedMemoryManager>) -> Result<Self> {
        let config = TrinityConfig::load()
            .unwrap_or_default()
            .with_env_overrides();

        let catalog = ModelCatalog::scan(&config.models.models_directory)?;

        Ok(Self {
            catalog: RwLock::new(catalog),
            current_model: RwLock::new(None),
            load_state: RwLock::new(ModelLoadState::Unloaded),
            memory_manager,
            config,
        })
    }

    /// Create with specific config
    pub fn with_config(
        config: TrinityConfig,
        memory_manager: Arc<UnifiedMemoryManager>,
    ) -> Result<Self> {
        let catalog = ModelCatalog::scan(&config.models.models_directory)?;

        Ok(Self {
            catalog: RwLock::new(catalog),
            current_model: RwLock::new(None),
            load_state: RwLock::new(ModelLoadState::Unloaded),
            memory_manager,
            config,
        })
    }

    /// Get the model catalog (read-only)
    pub fn catalog(&self) -> std::sync::RwLockReadGuard<'_, ModelCatalog> {
        self.catalog.read().unwrap()
    }

    /// Refresh the model catalog
    pub fn refresh_catalog(&self) -> Result<()> {
        let mut catalog = self.catalog.write().unwrap();
        catalog.refresh()
    }

    /// Get current load state
    pub fn load_state(&self) -> ModelLoadState {
        self.load_state.read().unwrap().clone()
    }

    /// Get currently loaded model name
    pub fn current_model(&self) -> Option<String> {
        self.current_model.read().unwrap().clone()
    }

    /// Check if we can load a model of the given size
    pub fn can_load(&self, size_bytes: u64) -> bool {
        self.memory_manager.can_allocate(size_bytes)
    }

    /// Check if a specific model can be loaded
    pub fn can_load_model(&self, model_name: &str) -> bool {
        let catalog = self.catalog.read().unwrap();
        if let Some(model) = catalog.get(model_name) {
            self.can_load(model.total_size_bytes())
        } else {
            false
        }
    }

    /// Get the path for a model (for DesktopBrain to load)
    pub fn get_model_path(&self, model_name: &str) -> Option<PathBuf> {
        let catalog = self.catalog.read().unwrap();
        catalog.get(model_name).map(|m| m.path.clone())
    }

    /// Mark model as loading
    pub fn set_loading(&self, model_name: &str) {
        let mut state = self.load_state.write().unwrap();
        *state = ModelLoadState::Loading { progress: 0 };

        let mut current = self.current_model.write().unwrap();
        *current = Some(model_name.to_string());

        tracing::info!("Model loading started: {}", model_name);
    }

    /// Update loading progress
    pub fn set_loading_progress(&self, progress: u8) {
        let mut state = self.load_state.write().unwrap();
        if matches!(*state, ModelLoadState::Loading { .. }) {
            *state = ModelLoadState::Loading { progress };
        }
    }

    /// Mark model as loaded
    pub fn set_loaded(&self, model_name: &str, size_bytes: u64) {
        // Track allocation in memory manager
        self.memory_manager.try_allocate(size_bytes);

        let mut state = self.load_state.write().unwrap();
        *state = ModelLoadState::Loaded;

        let mut current = self.current_model.write().unwrap();
        *current = Some(model_name.to_string());

        tracing::info!(
            "Model loaded: {} ({:.1} GB)",
            model_name,
            size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
        );
    }

    /// Mark model as failed
    pub fn set_failed(&self, error: String) {
        let mut state = self.load_state.write().unwrap();
        *state = ModelLoadState::Failed {
            error: error.clone(),
        };

        let mut current = self.current_model.write().unwrap();
        *current = None;

        tracing::error!("Model load failed: {}", error);
    }

    /// Mark model as unloaded and free memory
    pub fn set_unloaded(&self, freed_bytes: u64) {
        // Free allocation in memory manager
        self.memory_manager.free(freed_bytes);

        let mut state = self.load_state.write().unwrap();
        *state = ModelLoadState::Unloaded;

        let mut current = self.current_model.write().unwrap();
        let model_name = current.take();

        if let Some(name) = model_name {
            tracing::info!(
                "Model unloaded: {} ({:.1} GB freed)",
                name,
                freed_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            );
        }
    }

    /// Get recommended default model path
    pub fn default_model_path(&self) -> PathBuf {
        self.config.models.default_model_path.clone()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantization_parsing() {
        assert_eq!(
            Quantization::from_filename("model-Q4_K_M.gguf"),
            Quantization::Q4KM
        );
        assert_eq!(
            Quantization::from_filename("Qwen3-235B-Q3_K_L-00001-of-00003.gguf"),
            Quantization::Q3KL
        );
        assert_eq!(
            Quantization::from_filename("model-IQ4_XS.gguf"),
            Quantization::IQ4XS
        );
    }

    #[test]
    fn test_split_filename_parsing() {
        let result = ModelCatalog::parse_split_filename("model-00001-of-00003.gguf");
        assert_eq!(result, Some(("model".to_string(), 1, 3)));

        let result = ModelCatalog::parse_split_filename("Qwen3-235B-Q3_K_L-00002-of-00003.gguf");
        assert_eq!(result, Some(("Qwen3-235B-Q3_K_L".to_string(), 2, 3)));

        let result = ModelCatalog::parse_split_filename("single-model.gguf");
        assert_eq!(result, None);
    }

    #[test]
    fn test_model_name_extraction() {
        assert_eq!(
            ModelCatalog::extract_model_name("Qwen3-235B-Q3_K_L-00001-of-00003.gguf"),
            "Qwen3-235B-Q3_K_L"
        );
        assert_eq!(
            ModelCatalog::extract_model_name("simple-model.gguf"),
            "simple-model"
        );
    }

    #[test]
    fn test_bits_per_weight() {
        assert_eq!(Quantization::F16.bits_per_weight(), 16.0);
        assert_eq!(Quantization::Q4KM.bits_per_weight(), 4.5);
        assert_eq!(Quantization::Q3KL.bits_per_weight(), 3.5);
    }
}
