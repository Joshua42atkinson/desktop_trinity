#![allow(unused)]
//! Self-Coding Agent - Autonomous code generation and file manipulation
//!
//! The core "close to metal" self-editing capability for Trinity.
//! This agent can read, write, and modify code files autonomously.
//! Refactored to use Async I/O (Tokio) and non-blocking compute.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

// Trinity Core imports
use trinity_core::brain::{Brain, GenerationConfig};

/// Configuration for the self-coding agent
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
    /// Sudo password for privileged operations (OPTIONAL/DANGEROUS)
    pub sudo_password: Option<String>,
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
            sudo_password: None,
        }
    }
}

impl SelfCoderConfig {
    pub fn with_workspace(mut self, path: impl Into<PathBuf>) -> Self {
        self.workspace_root = path.into();
        self
    }

    pub fn allow_dangerous(mut self) -> Self {
        self.allow_dangerous_ops = true;
        self
    }

    pub fn with_sudo(mut self, password: impl Into<String>) -> Self {
        self.sudo_password = Some(password.into());
        self
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
    fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            content: None,
            path: None,
        }
    }

    fn with_content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            content: None,
            path: None,
        }
    }
}

/// Self-Coding Agent for autonomous code generation and editing
pub struct SelfCodingAgent {
    config: SelfCoderConfig,
    model: Option<Arc<dyn Brain>>,
    /// Operation log for audit trail
    // TODO: Ideally this should also be async-safe or use a channel,
    // but for now we'll just keep it simple or wrap in Mutex if needed.
    // Since this struct is usually instantiated per request or held in Arc,
    // we might need interior mutability if we want to share it.
    // For this refactor, we'll assume the agent is mutable.
    operation_log: Vec<OperationLogEntry>,
}

/// Log entry for tracking operations
#[derive(Debug, Clone)]
pub struct OperationLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub path: Option<PathBuf>,
    pub success: bool,
}

impl SelfCodingAgent {
    /// Create a new self-coding agent without a model (for testing)
    pub fn new(config: SelfCoderConfig) -> Self {
        Self {
            config,
            model: None,
            operation_log: Vec::new(),
        }
    }

    /// Resize agent with a loaded Brain implementation (e.g. Orchestrator)
    pub fn with_brain(config: SelfCoderConfig, brain: Arc<dyn Brain>) -> Self {
        Self {
            config,
            model: Some(brain),
            operation_log: Vec::new(),
        }
    }

    /// Check if a path is within the allowed workspace
    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // canonicalize is a blocking FS op, but usually fast enough.
        // For strict async, we could use tokio::fs::canonicalize, but that requires async here.
        // We'll trust standard lib for path logic for now as it's often cached by OS.
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

    /// Log an operation
    fn log_operation(&mut self, operation: &str, path: Option<&Path>, success: bool) {
        self.operation_log.push(OperationLogEntry {
            timestamp: chrono::Utc::now(),
            operation: operation.to_string(),
            path: path.map(|p| p.to_path_buf()),
            success,
        });
    }

    // =========================================================================
    // FILE OPERATIONS (ASYNC)
    // =========================================================================

    /// Read a file's contents
    pub async fn read_file(&mut self, path: impl AsRef<Path>) -> Result<CodeResult> {
        let path = path.as_ref();
        let validated_path = self.validate_path(path)?;
        self.validate_extension(&validated_path)?;

        let metadata = fs::metadata(&validated_path).await?;
        if metadata.len() as usize > self.config.max_file_size {
            anyhow::bail!("File too large: {} bytes", metadata.len());
        }

        let content = fs::read_to_string(&validated_path)
            .await
            .context("Failed to read file")?;

        self.log_operation("read", Some(&validated_path), true);

        Ok(CodeResult::success("File read successfully")
            .with_content(content)
            .with_path(validated_path))
    }

    /// Write content to a file (creates if doesn't exist)
    pub async fn write_file(
        &mut self,
        path: impl AsRef<Path>,
        content: &str,
    ) -> Result<CodeResult> {
        let path = path.as_ref();
        let validated_path = self.validate_path(path)?;
        self.validate_extension(&validated_path)?;

        // Create parent directories if needed
        if let Some(parent) = validated_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create parent directories")?;
        }

        // Check if overwriting existing file
        if validated_path.exists() && !self.config.allow_dangerous_ops {
            anyhow::bail!("File exists and dangerous ops disabled: {:?}", path);
        }

        fs::write(&validated_path, content)
            .await
            .context("Failed to write file")?;

        self.log_operation("write", Some(&validated_path), true);

        Ok(
            CodeResult::success(format!("File written: {:?}", validated_path))
                .with_path(validated_path),
        )
    }

    /// Delete a file (requires allow_dangerous_ops)
    pub async fn delete_file(&mut self, path: impl AsRef<Path>) -> Result<CodeResult> {
        if !self.config.allow_dangerous_ops {
            anyhow::bail!("Delete operation requires allow_dangerous_ops");
        }

        let path = path.as_ref();
        let validated_path = self.validate_path(path)?;

        fs::remove_file(&validated_path)
            .await
            .context("Failed to delete file")?;

        self.log_operation("delete", Some(&validated_path), true);

        Ok(CodeResult::success(format!(
            "File deleted: {:?}",
            validated_path
        )))
    }

    /// List files in a directory
    pub async fn list_files(&mut self, path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let path = path.as_ref();
        let validated_path = self.validate_path(path)?;

        let mut read_dir = fs::read_dir(&validated_path).await?;
        let mut entries = Vec::new();

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            entries.push(entry.path());
        }

        self.log_operation("list", Some(&validated_path), true);

        Ok(entries)
    }

    // =========================================================================
    // AI-POWERED OPERATIONS (ASYNC + BLOCKING)
    // =========================================================================

    /// Generate code from a natural language prompt
    pub async fn generate_code(&mut self, prompt: &str, language: &str) -> Result<CodeResult> {
        let brain = self
            .model
            .clone() // Clone Arc for the task
            .ok_or_else(|| anyhow::anyhow!("No brain loaded"))?;

        let full_prompt = format!(
            "You are an expert {} programmer. Generate clean, well-documented code.\n\n\
            Request: {}\n\n\
            Respond with ONLY the code, no explanations:",
            language, prompt
        );

        let config = GenerationConfig::default();

        let code = brain.think_with_config(&full_prompt, &config).await?;

        self.log_operation("generate", None, true);

        Ok(CodeResult::success("Code generated").with_content(code))
    }

    /// Edit existing code based on instructions
    pub async fn edit_code(
        &mut self,
        path: impl AsRef<Path>,
        instructions: &str,
    ) -> Result<CodeResult> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf(); // Clone for async block if usage needed

        // Read existing file (await)
        let read_result = self.read_file(path).await?;
        let original_code = read_result
            .content
            .ok_or_else(|| anyhow::anyhow!("Failed to read file content"))?;

        let brain = self
            .model
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No brain loaded"))?;

        let prompt = format!(
            "Edit the following code according to the instructions.\n\n\
            ORIGINAL CODE:\n```\n{}\n```\n\n\
            INSTRUCTIONS: {}\n\n\
            Return ONLY the complete modified code, no explanations:",
            original_code, instructions
        );

        let config = GenerationConfig::default();

        let edited_code = brain.think_with_config(&prompt, &config).await?;

        // Write back if dangerous ops allowed
        if self.config.allow_dangerous_ops {
            self.write_file(path, &edited_code).await?;
        }

        self.log_operation("edit", Some(path), true);

        Ok(CodeResult::success("Code edited").with_content(edited_code))
    }

    /// Explain code in a file
    pub async fn explain_code(&mut self, path: impl AsRef<Path>) -> Result<CodeResult> {
        let path = path.as_ref();

        let read_result = self.read_file(path).await?;
        let code = read_result
            .content
            .ok_or_else(|| anyhow::anyhow!("Failed to read file content"))?;

        let brain = self
            .model
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No brain loaded"))?;

        let prompt = format!(
            "Explain the following code clearly and concisely:\n\n```\n{}\n```",
            code
        );

        let config = GenerationConfig::default();

        let explanation = brain.think_with_config(&prompt, &config).await?;

        self.log_operation("explain", Some(path), true);

        Ok(CodeResult::success("Code explained").with_content(explanation))
    }

    /// Run a shell command (Dangerous!)
    pub async fn run_shell_command(&mut self, command: &str) -> Result<CodeResult> {
        if !self.config.allow_dangerous_ops {
            anyhow::bail!("Shell commands require allow_dangerous_ops");
        }

        let mut cmd_obj = tokio::process::Command::new("bash");

        // Handle sudo if command starts with sudo and we have a password
        let final_command =
            if command.trim().starts_with("sudo") && self.config.sudo_password.is_some() {
                // Use sudo -S to read password from stdin
                // We need to pipe the password
                // Construct a command that echos password to sudo -S
                // Note: This is fragile but works for simple cases.
                // security warning: password is in process memory temporarily.
                let password = self.config.sudo_password.as_ref().unwrap();
                format!(
                    "echo \"{}\" | sudo -S {}",
                    password,
                    command.trim_start_matches("sudo").trim()
                )
            } else {
                command.to_string()
            };

        cmd_obj.arg("-c").arg(&final_command);

        // Set workdir to workspace path, but default to current if workspace is file or invalid
        let workdir = if self.config.workspace_root.is_dir() {
            self.config.workspace_root.clone()
        } else {
            PathBuf::from(".")
        };
        cmd_obj.current_dir(workdir);

        let output = cmd_obj
            .output()
            .await
            .context("Failed to execute command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let combined = if stderr.is_empty() {
            stdout
        } else {
            format!("{}\nSTDERR:\n{}", stdout, stderr)
        };

        self.log_operation("shell", None, output.status.success());

        if output.status.success() {
            Ok(CodeResult::success("Command executed").with_content(combined))
        } else {
            Ok(CodeResult::error(format!(
                "Command failed (Exit {}): {}",
                output.status, combined
            )))
        }
    }

    /// Get the operation log
    pub fn get_operation_log(&self) -> &[OperationLogEntry] {
        &self.operation_log
    }

    /// Get workspace info
    pub fn workspace_info(&self) -> WorkspaceInfo {
        WorkspaceInfo {
            root: self.config.workspace_root.clone(),
            allowed_extensions: self.config.allowed_extensions.iter().cloned().collect(),
            dangerous_ops_enabled: self.config.allow_dangerous_ops,
            model_loaded: self.model.is_some(),
        }
    }
}

/// Information about the workspace
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub root: PathBuf,
    pub allowed_extensions: Vec<String>,
    pub dangerous_ops_enabled: bool,
    pub model_loaded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_default() {
        let config = SelfCoderConfig::default();
        assert!(config.allowed_extensions.contains("rs"));
        assert!(config.allowed_extensions.contains("py"));
        assert!(!config.allow_dangerous_ops);
    }

    #[test]
    fn test_agent_creation() {
        let config = SelfCoderConfig::default().with_workspace("/tmp/test");
        let agent = SelfCodingAgent::new(config);
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_code_result() {
        let result = CodeResult::success("Test")
            .with_content("code".to_string())
            .with_path(PathBuf::from("/test"));

        assert!(result.success);
        assert_eq!(result.content, Some("code".to_string()));
    }
}
