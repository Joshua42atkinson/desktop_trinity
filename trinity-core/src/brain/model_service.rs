//! Model Service - Centralized Model Management for Trinity
//!
//! Provides GGUF model discovery, cataloging, loading, and hot-swap capabilities.
//! Replaces LM Studio as the model management layer.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::desktop::DesktopBrain;
use super::Brain;

// ============================================================================
// Model Metadata
// ============================================================================

/// Quantization type parsed from filename
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Quantization {
    Q2K,
    Q3KS,
    Q3KM,
    Q3KL,
    Q4_0,
    Q4KS,
    Q4KM,
    Q5KS,
    Q5KM,
    Q6K,
    Q8_0,
    F16,
    F32,
    MXFP4,
    Unknown(String),
}

impl Quantization {
    /// Parse quantization from filename
    pub fn from_filename(filename: &str) -> Self {
        let upper = filename.to_uppercase();

        if upper.contains("Q2_K") || upper.contains("Q2K") {
            Self::Q2K
        } else if upper.contains("Q3_K_S") {
            Self::Q3KS
        } else if upper.contains("Q3_K_M") {
            Self::Q3KM
        } else if upper.contains("Q3_K_L") {
            Self::Q3KL
        } else if upper.contains("Q4_0") {
            Self::Q4_0
        } else if upper.contains("Q4_K_S") {
            Self::Q4KS
        } else if upper.contains("Q4_K_M") {
            Self::Q4KM
        } else if upper.contains("Q5_K_S") {
            Self::Q5KS
        } else if upper.contains("Q5_K_M") {
            Self::Q5KM
        } else if upper.contains("Q6_K") {
            Self::Q6K
        } else if upper.contains("Q8_0") {
            Self::Q8_0
        } else if upper.contains("F16") {
            Self::F16
        } else if upper.contains("F32") {
            Self::F32
        } else if upper.contains("MXFP4") {
            Self::MXFP4
        } else {
            Self::Unknown("unknown".to_string())
        }
    }

    /// Bits per weight (approximate)
    pub fn bits_per_weight(&self) -> f32 {
        match self {
            Self::Q2K => 2.5,
            Self::Q3KS | Self::Q3KM | Self::Q3KL => 3.5,
            Self::Q4_0 | Self::Q4KS | Self::Q4KM | Self::MXFP4 => 4.5,
            Self::Q5KS | Self::Q5KM => 5.5,
            Self::Q6K => 6.5,
            Self::Q8_0 => 8.0,
            Self::F16 => 16.0,
            Self::F32 => 32.0,
            Self::Unknown(_) => 4.5, // Assume Q4
        }
    }
}

impl std::fmt::Display for Quantization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Q2K => write!(f, "Q2_K"),
            Self::Q3KS => write!(f, "Q3_K_S"),
            Self::Q3KM => write!(f, "Q3_K_M"),
            Self::Q3KL => write!(f, "Q3_K_L"),
            Self::Q4_0 => write!(f, "Q4_0"),
            Self::Q4KS => write!(f, "Q4_K_S"),
            Self::Q4KM => write!(f, "Q4_K_M"),
            Self::Q5KS => write!(f, "Q5_K_S"),
            Self::Q5KM => write!(f, "Q5_K_M"),
            Self::Q6K => write!(f, "Q6_K"),
            Self::Q8_0 => write!(f, "Q8_0"),
            Self::F16 => write!(f, "F16"),
            Self::F32 => write!(f, "F32"),
            Self::MXFP4 => write!(f, "MXFP4"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

/// Metadata about a discovered model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Unique identifier (hash of path)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Full path to the model file (first shard for split models)
    pub path: PathBuf,
    /// All shard paths for split models
    pub shards: Vec<PathBuf>,
    /// Total size in bytes (all shards)
    pub size_bytes: u64,
    /// Quantization type
    pub quantization: Quantization,
    /// Whether this is a split model
    pub is_split: bool,
    /// Number of shards
    pub shard_count: usize,
    /// Parent directory name (often contains model family info)
    pub family: String,
}

impl ModelEntry {
    /// Get size in GB
    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

impl std::fmt::Display for ModelEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}, {:.1} GB{})",
            self.name,
            self.quantization,
            self.size_gb(),
            if self.is_split {
                format!(", {} shards", self.shard_count)
            } else {
                String::new()
            }
        )
    }
}

// ============================================================================
// Model Catalog
// ============================================================================

/// Catalog of all discovered GGUF models
#[derive(Debug, Clone, Default)]
pub struct ModelCatalog {
    /// All discovered models, indexed by ID
    entries: HashMap<String, ModelEntry>,
    /// Search paths used for discovery
    search_paths: Vec<PathBuf>,
}

impl ModelCatalog {
    /// Create an empty catalog
    pub fn new() -> Self {
        Self::default()
    }

    /// Create catalog with default search paths
    pub fn with_default_paths() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        let search_paths = vec![
            PathBuf::from(&home).join(".lmstudio/models"),
            PathBuf::from(&home).join("antigravity"),
            PathBuf::from(&home).join("models"),
            PathBuf::from("/models"),
        ];
        Self {
            entries: HashMap::new(),
            search_paths,
        }
    }

    /// Add a custom search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Scan all search paths for GGUF models
    pub fn scan(&mut self) -> Result<usize> {
        let mut found = 0;
        let mut all_ggufs: Vec<(PathBuf, u64)> = Vec::new();

        // Collect all GGUF files
        for search_path in &self.search_paths.clone() {
            if search_path.exists() {
                self.find_gguf_recursive(search_path, &mut all_ggufs);
            }
        }

        // Group by base name (for split models)
        let mut groups: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();
        for (path, size) in all_ggufs {
            let base_name = self.get_base_name(&path);
            groups.entry(base_name).or_default().push((path, size));
        }

        // Create entries
        for (base_name, mut files) in groups {
            // Sort by shard number (for consistent ordering)
            files.sort_by(|a, b| a.0.cmp(&b.0));

            let first_path = &files[0].0;
            let total_size: u64 = files.iter().map(|(_, s)| s).sum();
            let is_split = files.len() > 1;

            // Skip mmproj files (multimodal projectors)
            if base_name.to_lowercase().contains("mmproj") {
                continue;
            }

            let family = first_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let id = format!("{:x}", md5_hash(&first_path.to_string_lossy()));

            let entry = ModelEntry {
                id: id.clone(),
                name: base_name.clone(),
                path: first_path.clone(),
                shards: files.iter().map(|(p, _)| p.clone()).collect(),
                size_bytes: total_size,
                quantization: Quantization::from_filename(&base_name),
                is_split,
                shard_count: files.len(),
                family,
            };

            self.entries.insert(id, entry);
            found += 1;
        }

        tracing::info!("ModelCatalog: Found {} models", found);
        Ok(found)
    }

    /// Get base name for grouping split models
    fn get_base_name(&self, path: &Path) -> String {
        let filename = path.file_stem().unwrap_or_default().to_string_lossy();

        // Remove shard suffix like "-00001-of-00003"
        let re = regex::Regex::new(r"-\d{5}-of-\d{5}$").unwrap();
        re.replace(&filename, "").to_string()
    }

    /// Find GGUF files recursively
    fn find_gguf_recursive(&self, dir: &Path, results: &mut Vec<(PathBuf, u64)>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.find_gguf_recursive(&path, results);
                } else if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                    if let Ok(meta) = std::fs::metadata(&path) {
                        results.push((path, meta.len()));
                    }
                }
            }
        }
    }

    /// Get all models sorted by size (largest first)
    pub fn all_models(&self) -> Vec<&ModelEntry> {
        let mut models: Vec<_> = self.entries.values().collect();
        models.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        models
    }

    /// Get a model by ID
    pub fn get(&self, id: &str) -> Option<&ModelEntry> {
        self.entries.get(id)
    }

    /// Find models by name (substring match)
    pub fn find_by_name(&self, query: &str) -> Vec<&ModelEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .values()
            .filter(|e| e.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get models that fit in the given VRAM (GB)
    pub fn models_for_vram(&self, vram_gb: f64) -> Vec<&ModelEntry> {
        let vram_bytes = (vram_gb * 1024.0 * 1024.0 * 1024.0) as u64;
        let mut models: Vec<_> = self
            .entries
            .values()
            .filter(|e| e.size_bytes <= vram_bytes)
            .collect();
        models.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        models
    }

    /// Count of models
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Simple hash for ID generation
fn md5_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

// ============================================================================
// Model Service
// ============================================================================

/// Centralized model loading and management service
pub struct ModelService {
    /// Model catalog
    catalog: RwLock<ModelCatalog>,
    /// Currently loaded brain (if any)
    active_brain: RwLock<Option<Arc<DesktopBrain>>>,
    /// Currently loaded model entry
    active_model: RwLock<Option<ModelEntry>>,
}

impl ModelService {
    /// Create a new model service with default paths
    pub fn new() -> Self {
        let mut catalog = ModelCatalog::with_default_paths();
        let _ = catalog.scan(); // Best-effort scan on creation

        Self {
            catalog: RwLock::new(catalog),
            active_brain: RwLock::new(None),
            active_model: RwLock::new(None),
        }
    }

    /// Rescan for models
    pub async fn rescan(&self) -> Result<usize> {
        let mut catalog = self.catalog.write().await;
        catalog.scan()
    }

    /// Get the model catalog
    pub async fn catalog(&self) -> ModelCatalog {
        self.catalog.read().await.clone()
    }

    /// List all available models
    pub async fn list_models(&self) -> Vec<ModelEntry> {
        let catalog = self.catalog.read().await;
        catalog.all_models().into_iter().cloned().collect()
    }

    /// Get currently active model
    pub async fn active_model(&self) -> Option<ModelEntry> {
        self.active_model.read().await.clone()
    }

    /// Get the active brain (if loaded)
    pub async fn brain(&self) -> Option<Arc<DesktopBrain>> {
        self.active_brain.read().await.clone()
    }

    /// Load a model by ID
    pub async fn load_by_id(&self, id: &str) -> Result<()> {
        let catalog = self.catalog.read().await;
        let entry = catalog
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", id))?;
        drop(catalog);

        self.load_entry(&entry).await
    }

    /// Load a model by path
    pub async fn load_by_path(&self, path: &Path) -> Result<()> {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let entry = ModelEntry {
            id: format!("{:x}", md5_hash(&path.to_string_lossy())),
            name: filename.clone(),
            path: path.to_path_buf(),
            shards: vec![path.to_path_buf()],
            size_bytes: size,
            quantization: Quantization::from_filename(&filename),
            is_split: false,
            shard_count: 1,
            family: path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
        };

        self.load_entry(&entry).await
    }

    /// Load a model entry
    async fn load_entry(&self, entry: &ModelEntry) -> Result<()> {
        tracing::info!("Loading model: {}", entry);

        // Set HSA override for Strix Halo
        std::env::set_var("HSA_OVERRIDE_GFX_VERSION", "11.5.1");

        let path_str = entry.path.to_string_lossy().to_string();

        // Create and load brain
        let brain = DesktopBrain::new();
        brain
            .load_model(&path_str)
            .await
            .with_context(|| format!("Failed to load model: {}", entry.name))?;

        // Update state
        {
            let mut active = self.active_brain.write().await;
            *active = Some(Arc::new(brain));
        }
        {
            let mut model = self.active_model.write().await;
            *model = Some(entry.clone());
        }

        tracing::info!("Model loaded successfully: {}", entry.name);
        Ok(())
    }

    /// Unload the current model
    pub async fn unload(&self) -> Option<ModelEntry> {
        let brain = {
            let mut active = self.active_brain.write().await;
            active.take()
        };

        if let Some(brain) = brain {
            brain.unload_model();
        }

        let mut model = self.active_model.write().await;
        model.take()
    }

    /// Check if a model is loaded
    pub async fn is_loaded(&self) -> bool {
        self.active_brain.read().await.is_some()
    }

    /// Think with the active brain
    pub async fn think(&self, prompt: &str) -> Result<String> {
        let brain = self.active_brain.read().await;
        let brain = brain
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No model loaded"))?;

        brain.think(prompt).await
    }
}

impl Default for ModelService {
    fn default() -> Self {
        Self::new()
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
            Quantization::from_filename("model-Q3_K_L-00001-of-00003.gguf"),
            Quantization::Q3KL
        );
        assert_eq!(
            Quantization::from_filename("gpt-oss-120b-MXFP4-00001.gguf"),
            Quantization::MXFP4
        );
    }

    #[test]
    fn test_catalog_creation() {
        let catalog = ModelCatalog::new();
        assert!(catalog.is_empty());
    }

    #[test]
    fn test_base_name_extraction() {
        let catalog = ModelCatalog::new();
        let path = Path::new("/models/Qwen3-235B-Q3_K_L-00001-of-00003.gguf");
        let base = catalog.get_base_name(path);
        assert_eq!(base, "Qwen3-235B-Q3_K_L");
    }
}
