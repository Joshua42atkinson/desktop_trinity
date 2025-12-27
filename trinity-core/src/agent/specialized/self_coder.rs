//! Self-Coding Specialist - The "Developer" Agent Logic
//!
//! Provides capabilities for reading, writing, and generating code.
//! This module is intended to be used by the Agent Executor system when
//! an agent has the `Developer` role or `can_write_files` capability.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

// use crate::inference::{GenerateConfig, GgufModel};

/// Configuration for the self-coding capabilities
#[derive(Clone, Debug)]
pub struct SelfCoderConfig {
    /// Root directory for workspace operations
    pub workspace_root: PathBuf,
    /// Allowed file extensions for editing
    pub allowed_extensions: HashSet<String>,
    /// Maximum file size to process (bytes)
    pub max_file_size: usize,
    /// Enable dangerous operations (delete, overwrite)
    pub allow_dangerous_ops: bool,
}

impl Default for SelfCoderConfig {
    fn default() -> Self {
        let mut allowed = HashSet::new();
        // Safe text/code extensions
        allowed.insert("rs".to_string());
        allowed.insert("py".to_string());
        allowed.insert("js".to_string());
        allowed.insert("ts".to_string());
        allowed.insert("md".to_string());
        allowed.insert("txt".to_string());
        allowed.insert("toml".to_string());
        allowed.insert("json".to_string());
        allowed.insert("yaml".to_string());
        allowed.insert("yml".to_string());
        allowed.insert("html".to_string());
        allowed.insert("css".to_string());
        allowed.insert("sh".to_string());

        Self {
            workspace_root: PathBuf::from("."),
            allowed_extensions: allowed,
            max_file_size: 1024 * 1024, // 1MB
            allow_dangerous_ops: false,
        }
    }
}

/// Result of a code operation
#[derive(Debug, Clone)]
pub struct CodeResult {
    pub success: bool,
    pub message: String,
    pub content: Option<String>,
    pub path: Option<PathBuf>,
}

impl CodeResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            content: None,
            path: None,
        }
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            content: None,
            path: None,
        }
    }
}

/// The Self-Coder Logic Engine
pub struct SelfCoder {
    config: SelfCoderConfig,
    // model: Option<Arc<Mutex<GgufModel>>>, // TODO: Re-integrate with Brain trait
}

impl SelfCoder {
    pub fn new(config: SelfCoderConfig) -> Self {
        Self {
            config,
            // model: None,
        }
    }

    // pub fn with_model(config: SelfCoderConfig, model: Arc<Mutex<GgufModel>>) -> Self {
    //     Self {
    //         config,
    //         model: Some(model),
    //     }
    // }

    /// Check if a path is within the allowed workspace
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        let canonical = path
            .canonicalize()
            .or_else(|_| -> Result<PathBuf> {
                // Path might not exist yet
                let parent = path.parent().unwrap_or(Path::new("."));
                Ok(parent
                    .canonicalize()?
                    .join(path.file_name().unwrap_or_default()))
            })
            .context("Invalid path")?;

        let workspace = self
            .config
            .workspace_root
            .canonicalize()
            .context("Invalid workspace root")?;

        if !canonical.starts_with(&workspace) {
            anyhow::bail!("Path outside workspace: {:?}", path);
        }

        Ok(canonical)
    }

    /// Check if file extension is allowed
    fn validate_extension(&self, path: &Path) -> Result<()> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if !self.config.allowed_extensions.contains(ext) {
            anyhow::bail!("File extension not allowed: .{}", ext);
        }

        Ok(())
    }

    // =========================================================================
    // FILE OPERATIONS
    // =========================================================================

    /// Read a file's contents
    pub fn read_file(&self, path: impl AsRef<Path>) -> Result<CodeResult> {
        let path = path.as_ref();
        let validated_path = self.validate_path(path)?;
        self.validate_extension(&validated_path)?;

        let metadata = fs::metadata(&validated_path)?;
        if metadata.len() as usize > self.config.max_file_size {
            anyhow::bail!("File too large: {} bytes", metadata.len());
        }

        let content = fs::read_to_string(&validated_path).context("Failed to read file")?;

        Ok(CodeResult::success("File read successfully")
            .with_content(content)
            .with_path(validated_path))
    }

    /// Write content to a file
    pub fn write_file(&self, path: impl AsRef<Path>, content: &str) -> Result<CodeResult> {
        let path = path.as_ref();
        let validated_path = self.validate_path(path)?;
        self.validate_extension(&validated_path)?;

        // Create parent directories if needed
        if let Some(parent) = validated_path.parent() {
            fs::create_dir_all(parent).context("Failed to create parent directories")?;
        }

        // Check if overwriting existing file
        if validated_path.exists() && !self.config.allow_dangerous_ops {
            anyhow::bail!("File exists and dangerous ops disabled: {:?}", path);
        }

        fs::write(&validated_path, content).context("Failed to write file")?;

        Ok(
            CodeResult::success(format!("File written: {:?}", validated_path))
                .with_path(validated_path),
        )
    }

    /// List files in a directory
    pub fn list_files(&self, path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let path = path.as_ref();
        let validated_path = self.validate_path(path)?;

        let entries: Vec<PathBuf> = fs::read_dir(&validated_path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        Ok(entries)
    }

    // =========================================================================
    // AI-POWERED OPERATIONS
    // =========================================================================

    /// Generate code using the loaded LLM
    pub async fn generate_code(&self, instruction: &str, context: &str) -> Result<String> {
        tracing::info!("🧠 SelfCoder: Generating code for '{}'", instruction);

        // initialize brain (this will use the default model configured in TrinityConfig)
        let brain = crate::brain::create_brain().await?;

        // Construct a coding-focused prompt
        let prompt = format!(
            "You are a high-performance Rust coding assistant optimized for Strix Halo.\n\
            \n\
            CONTEXT:\n\
            {}\n\
            \n\
            INSTRUCTION:\n\
            {}\n\
            \n\
            Provide only the Rust code required. Do not include markdown fencing or explanation.\n\
            CODE:",
            context, instruction
        );

        let response = brain.think(&prompt).await?;

        Ok(response)
    }
}
