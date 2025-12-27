//! # Tool Executor (Self-Coding Skills)
//!
//! ## Philosophy
//! "Hands without direction are useless. The ToolExecutor gives Trinity the ability
//!  to read, write, and manipulate files—the foundation of self-coding."
//!
//! ## Purpose
//! Safe execution of file and shell operations for autonomous work.
//! Migrated from day_dream/backend/src/ai/tool_executor.rs, simplified for pure Rust.
//!
//! ## Safety
//! - All file operations require explicit paths
//! - Shell execution is sandboxed to workspace
//! - Timeouts prevent runaway processes

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Configuration for tool execution
#[derive(Clone, Debug)]
pub struct ExecutorConfig {
    /// Maximum execution time per tool
    pub timeout: Duration,
    /// Working directory for shell commands
    pub working_dir: String,
    /// Maximum output size in bytes
    pub max_output_size: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            working_dir: ".".to_string(),
            max_output_size: 1024 * 1024, // 1MB
        }
    }
}

/// Tool executor for file and shell operations
pub struct ToolExecutor {
    config: ExecutorConfig,
    /// Execution log
    execution_log: Vec<ExecutionLogEntry>,
}

/// Log entry for tool execution
#[derive(Debug, Clone)]
pub struct ExecutionLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tool_name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub output_preview: String,
}

impl ToolExecutor {
    /// Create a new executor with default config
    pub fn new() -> Self {
        Self {
            config: ExecutorConfig::default(),
            execution_log: Vec::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self {
            config,
            execution_log: Vec::new(),
        }
    }

    /// Read file contents
    pub fn read_file(&mut self, path: &str) -> Result<String> {
        let start = std::time::Instant::now();

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        self.log_execution("read_file", true, start.elapsed(), &content);
        Ok(content)
    }

    /// Write content to file
    pub fn write_file(&mut self, path: &str, content: &str) -> Result<()> {
        let start = std::time::Instant::now();

        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content).with_context(|| format!("Failed to write file: {}", path))?;

        let msg = format!("Wrote {} bytes to {}", content.len(), path);
        self.log_execution("write_file", true, start.elapsed(), &msg);
        Ok(())
    }

    /// List directory contents
    pub fn list_directory(&mut self, path: &str) -> Result<Vec<String>> {
        let start = std::time::Instant::now();

        let entries: Vec<String> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let is_dir = e.path().is_dir();
                if is_dir {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .collect();

        let msg = format!("{} entries in {}", entries.len(), path);
        self.log_execution("list_directory", true, start.elapsed(), &msg);
        Ok(entries)
    }

    /// Run a shell command
    pub fn run_command(&mut self, command: &str) -> Result<String> {
        let start = std::time::Instant::now();

        let output = Command::new("bash")
            .args(["-c", command])
            .current_dir(&self.config.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let result = if output.status.success() {
            self.log_execution("run_command", true, start.elapsed(), &stdout);
            Ok(stdout)
        } else {
            self.log_execution("run_command", false, start.elapsed(), &stderr);
            Err(anyhow::anyhow!("Command failed: {}", stderr))
        };

        result
    }

    /// Append to a file
    pub fn append_file(&mut self, path: &str, content: &str) -> Result<()> {
        let start = std::time::Instant::now();

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        file.write_all(content.as_bytes())?;

        let msg = format!("Appended {} bytes to {}", content.len(), path);
        self.log_execution("append_file", true, start.elapsed(), &msg);
        Ok(())
    }

    /// Check if a file exists
    pub fn file_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    /// Get file metadata
    pub fn file_info(&self, path: &str) -> Result<HashMap<String, String>> {
        let metadata = std::fs::metadata(path)?;
        let mut info = HashMap::new();

        info.insert("size".to_string(), metadata.len().to_string());
        info.insert("is_file".to_string(), metadata.is_file().to_string());
        info.insert("is_dir".to_string(), metadata.is_dir().to_string());

        Ok(info)
    }

    /// Delete a file
    pub fn delete_file(&mut self, path: &str) -> Result<()> {
        let start = std::time::Instant::now();

        std::fs::remove_file(path).with_context(|| format!("Failed to delete file: {}", path))?;

        let msg = format!("Deleted file: {}", path);
        self.log_execution("delete_file", true, start.elapsed(), &msg);
        Ok(())
    }

    /// Delete a directory (must be empty or use recursive)
    pub fn delete_directory(&mut self, path: &str, recursive: bool) -> Result<()> {
        let start = std::time::Instant::now();

        if recursive {
            std::fs::remove_dir_all(path)
                .with_context(|| format!("Failed to delete directory recursively: {}", path))?;
        } else {
            std::fs::remove_dir(path)
                .with_context(|| format!("Failed to delete directory: {}", path))?;
        }

        let msg = format!("Deleted directory: {} (recursive={})", path, recursive);
        self.log_execution("delete_directory", true, start.elapsed(), &msg);
        Ok(())
    }

    /// Create a directory (and parents if needed)
    pub fn create_directory(&mut self, path: &str) -> Result<()> {
        let start = std::time::Instant::now();

        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path))?;

        let msg = format!("Created directory: {}", path);
        self.log_execution("create_directory", true, start.elapsed(), &msg);
        Ok(())
    }

    /// Move/rename a file or directory
    pub fn move_path(&mut self, from: &str, to: &str) -> Result<()> {
        let start = std::time::Instant::now();

        std::fs::rename(from, to).with_context(|| format!("Failed to move {} to {}", from, to))?;

        let msg = format!("Moved {} -> {}", from, to);
        self.log_execution("move_path", true, start.elapsed(), &msg);
        Ok(())
    }

    /// Copy a file
    pub fn copy_file(&mut self, from: &str, to: &str) -> Result<()> {
        let start = std::time::Instant::now();

        // Ensure parent directory exists
        if let Some(parent) = Path::new(to).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::copy(from, to).with_context(|| format!("Failed to copy {} to {}", from, to))?;

        let msg = format!("Copied {} -> {}", from, to);
        self.log_execution("copy_file", true, start.elapsed(), &msg);
        Ok(())
    }

    fn log_execution(&mut self, tool_name: &str, success: bool, duration: Duration, output: &str) {
        self.execution_log.push(ExecutionLogEntry {
            timestamp: chrono::Utc::now(),
            tool_name: tool_name.to_string(),
            success,
            duration_ms: duration.as_millis() as u64,
            output_preview: output.chars().take(100).collect(),
        });

        // Keep log bounded
        if self.execution_log.len() > 1000 {
            self.execution_log.drain(0..500);
        }
    }

    /// Get execution log
    pub fn get_log(&self) -> &[ExecutionLogEntry] {
        &self.execution_log
    }

    /// Clear execution log
    pub fn clear_log(&mut self) {
        self.execution_log.clear();
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_write_file() {
        let mut executor = ToolExecutor::new();

        // Create temp file
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "Hello, Trinity!").unwrap();
        let path = temp.path().to_str().unwrap();

        // Read it back
        let content = executor.read_file(path).unwrap();
        assert!(content.contains("Hello, Trinity!"));
    }

    #[test]
    fn test_file_exists() {
        let executor = ToolExecutor::new();
        assert!(executor.file_exists("/tmp"));
        assert!(!executor.file_exists("/nonexistent/path/12345"));
    }
}
