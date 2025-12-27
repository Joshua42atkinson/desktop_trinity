//! Trinity Configuration System
//!
//! Provides centralized, runtime-configurable settings for Trinity AI OS.
//! Configuration is loaded from `~/.trinity/config.toml` with environment variable overrides.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration struct for Trinity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct TrinityConfig {
    /// Model configuration
    pub models: ModelConfig,
    /// Hardware configuration
    pub hardware: HardwareConfig,
    /// Agent configuration
    pub agents: AgentConfig,
    /// Logging configuration
    pub logging: LogConfig,
}

/// Model-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    /// Path to the default GGUF model
    pub default_model_path: PathBuf,
    /// Path to tokenizer (if separate from model)
    pub tokenizer_path: Option<PathBuf>,
    /// Context window size in tokens
    pub context_size: u32,
    /// Number of layers to offload to GPU (None = auto)
    pub gpu_layers: Option<u32>,
    /// Directory to scan for available models
    pub models_directory: PathBuf,
}

/// Hardware-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HardwareConfig {
    /// Preferred device type: "gpu", "npu", "cpu", or "auto"
    pub preferred_device: String,
    /// Maximum VRAM to use in GB (None = use all available)
    pub max_vram_gb: Option<f64>,
    /// Enable memory overcommit (use swap if needed)
    pub allow_memory_overcommit: bool,
    /// HSA override version for Strix Halo
    pub hsa_override_version: String,
}

/// Agent-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    /// Default agents to spawn at boot
    pub default_agents: Vec<AgentDefinition>,
    /// Maximum concurrent agents
    pub max_agents: usize,
    /// Default context window per agent
    pub default_context_size: usize,
}

/// Definition for a default agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Agent role: "assistant", "developer", "researcher", "writer"
    pub role: String,
    /// Agent name (optional)
    pub name: Option<String>,
    /// Custom system prompt (optional)
    pub system_prompt: Option<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    /// Log level: "trace", "debug", "info", "warn", "error"
    pub level: String,
    /// Log format: "pretty", "json", "compact"
    pub format: String,
    /// Log file path (None = stdout only)
    pub file: Option<PathBuf>,
}

// ============================================================================
// Default Implementations
// ============================================================================

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            // Use Llama-4-Scout as default (user selected)
            default_model_path: PathBuf::from("/home/joshua/.lmstudio/models/lmstudio-community/Llama-4-Scout-17B-16E-Instruct-GGUF/Llama-4-Scout-17B-16E-Instruct-Q4_K_M-00001-of-00002.gguf"),
            tokenizer_path: None,
            context_size: 8192,
            gpu_layers: None, // Auto-detect
            models_directory: PathBuf::from("/home/joshua/.lmstudio/models"),
        }
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            preferred_device: "auto".to_string(),
            max_vram_gb: Some(96.0), // Strix Halo default
            allow_memory_overcommit: false,
            hsa_override_version: "11.5.1".to_string(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_agents: vec![
                AgentDefinition {
                    role: "assistant".to_string(),
                    name: Some("Core".to_string()),
                    system_prompt: None,
                },
                AgentDefinition {
                    role: "developer".to_string(),
                    name: Some("DevAgent".to_string()),
                    system_prompt: None,
                },
                AgentDefinition {
                    role: "researcher".to_string(),
                    name: Some("ResearchAgent".to_string()),
                    system_prompt: None,
                },
            ],
            max_agents: 16,
            default_context_size: 4096,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            file: None,
        }
    }
}

// ============================================================================
// Configuration Loading
// ============================================================================

impl TrinityConfig {
    /// Load configuration from the default path (~/.trinity/config.toml)
    pub fn load() -> Result<Self> {
        let config_path = Self::default_config_path();
        Self::load_from(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let contents = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config from {:?}", path))?;
            let config: TrinityConfig = toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config from {:?}", path))?;
            tracing::info!("Loaded configuration from {:?}", path);
            Ok(config)
        } else {
            tracing::info!("No config file found at {:?}, using defaults", path);
            Ok(Self::default())
        }
    }

    /// Save configuration to a file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }

        std::fs::write(path, contents)
            .with_context(|| format!("Failed to write config to {:?}", path))?;

        tracing::info!("Saved configuration to {:?}", path);
        Ok(())
    }

    /// Get the default configuration file path
    pub fn default_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".trinity")
            .join("config.toml")
    }

    /// Create default config file if it doesn't exist
    pub fn ensure_default_config() -> Result<PathBuf> {
        let path = Self::default_config_path();
        if !path.exists() {
            let config = Self::default();
            config.save(&path)?;
            tracing::info!("Created default config at {:?}", path);
        }
        Ok(path)
    }

    /// Apply environment variable overrides
    pub fn with_env_overrides(mut self) -> Self {
        // Model path override
        if let Ok(path) = std::env::var("TRINITY_MODEL_PATH") {
            self.models.default_model_path = PathBuf::from(path);
        }

        // VRAM limit override
        if let Ok(vram) = std::env::var("TRINITY_MAX_VRAM_GB") {
            if let Ok(gb) = vram.parse::<f64>() {
                self.hardware.max_vram_gb = Some(gb);
            }
        }

        // Device preference override
        if let Ok(device) = std::env::var("TRINITY_DEVICE") {
            self.hardware.preferred_device = device;
        }

        // Log level override
        if let Ok(level) = std::env::var("TRINITY_LOG_LEVEL") {
            self.logging.level = level;
        }

        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TrinityConfig::default();
        assert_eq!(config.hardware.preferred_device, "auto");
        assert_eq!(config.logging.level, "info");
        assert!(!config.agents.default_agents.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = TrinityConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: TrinityConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.hardware.max_vram_gb, parsed.hardware.max_vram_gb);
    }

    #[test]
    fn test_env_overrides() {
        std::env::set_var("TRINITY_MAX_VRAM_GB", "64");
        let config = TrinityConfig::default().with_env_overrides();
        assert_eq!(config.hardware.max_vram_gb, Some(64.0));
        std::env::remove_var("TRINITY_MAX_VRAM_GB");
    }
}
