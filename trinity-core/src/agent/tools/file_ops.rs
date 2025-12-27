//! File Operations Tools
//!
//! Provides sandboxed file system operations for agents.

use super::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::path::{Path, PathBuf};

// ============================================================================
// Read File Tool
// ============================================================================

/// Tool for reading file contents
pub struct ReadFileTool {
    /// Root directory for sandboxing (if set)
    sandbox_root: Option<PathBuf>,
}

impl ReadFileTool {
    pub fn new() -> Self {
        Self { sandbox_root: None }
    }

    pub fn with_sandbox(root: PathBuf) -> Self {
        Self {
            sandbox_root: Some(root),
        }
    }

    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        let canonical = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        // If sandboxed, ensure path is within sandbox
        if let Some(root) = &self.sandbox_root {
            let abs_root = root.canonicalize()?;
            if !canonical.starts_with(&abs_root) {
                anyhow::bail!("Path {} is outside sandbox", path.display());
            }
        }

        Ok(canonical)
    }
}

impl Default for ReadFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct ReadFileParams {
    path: String,
    #[serde(default)]
    start_line: Option<usize>,
    #[serde(default)]
    end_line: Option<usize>,
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Optionally specify start_line and end_line for partial reads."
    }

    fn parameters_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Optional starting line number (1-indexed)"
                },
                "end_line": {
                    "type": "integer",
                    "description": "Optional ending line number (1-indexed, inclusive)"
                }
            }
        })
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let params: ReadFileParams = serde_json::from_value(params)?;

        let path = self.validate_path(Path::new(&params.path))?;

        if !path.exists() {
            return Ok(ToolResult::error(format!(
                "File not found: {}",
                params.path
            )));
        }

        let content = tokio::fs::read_to_string(&path).await?;

        // Apply line filtering if specified
        let output = match (params.start_line, params.end_line) {
            (Some(start), Some(end)) => content
                .lines()
                .skip(start.saturating_sub(1))
                .take(end.saturating_sub(start.saturating_sub(1)))
                .collect::<Vec<_>>()
                .join("\n"),
            (Some(start), None) => content
                .lines()
                .skip(start.saturating_sub(1))
                .collect::<Vec<_>>()
                .join("\n"),
            _ => content,
        };

        let line_count = output.lines().count();
        let byte_count = output.len();

        Ok(ToolResult::success_with_data(
            output,
            json!({
                "path": params.path,
                "lines": line_count,
                "bytes": byte_count
            }),
        ))
    }
}

// ============================================================================
// Write File Tool
// ============================================================================

/// Tool for writing file contents
pub struct WriteFileTool {
    /// Root directory for sandboxing (if set)
    sandbox_root: Option<PathBuf>,
    /// Allowed extensions (if set)
    allowed_extensions: Option<Vec<String>>,
}

impl WriteFileTool {
    pub fn new() -> Self {
        Self {
            sandbox_root: None,
            allowed_extensions: None,
        }
    }

    pub fn with_sandbox(root: PathBuf) -> Self {
        Self {
            sandbox_root: Some(root),
            allowed_extensions: None,
        }
    }

    pub fn with_allowed_extensions(mut self, exts: Vec<String>) -> Self {
        self.allowed_extensions = Some(exts);
        self
    }

    fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // Check extension
        if let Some(allowed) = &self.allowed_extensions {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !allowed.iter().any(|a| a == ext) {
                anyhow::bail!("Extension .{} not allowed. Allowed: {:?}", ext, allowed);
            }
        }

        let canonical = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        // If sandboxed, ensure path is within sandbox
        if let Some(root) = &self.sandbox_root {
            let abs_root = root.canonicalize()?;
            // For new files, check the parent directory
            let check_path = if canonical.exists() {
                canonical.clone()
            } else {
                canonical.parent().unwrap_or(&canonical).to_path_buf()
            };
            if check_path.exists() && !check_path.starts_with(&abs_root) {
                anyhow::bail!("Path {} is outside sandbox", path.display());
            }
        }

        Ok(canonical)
    }
}

impl Default for WriteFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct WriteFileParams {
    path: String,
    content: String,
    #[serde(default)]
    create_dirs: bool,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates the file if it doesn't exist."
    }

    fn parameters_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "required": ["path", "content"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "create_dirs": {
                    "type": "boolean",
                    "description": "Create parent directories if they don't exist",
                    "default": false
                }
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        true // Writing files should require confirmation
    }

    fn risk_level(&self) -> u8 {
        5
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let params: WriteFileParams = serde_json::from_value(params)?;

        let path = self.validate_path(Path::new(&params.path))?;

        // Create parent directories if requested
        if params.create_dirs {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        let bytes = params.content.len();
        tokio::fs::write(&path, &params.content).await?;

        Ok(ToolResult::success_with_data(
            format!("Wrote {} bytes to {}", bytes, params.path),
            json!({
                "path": params.path,
                "bytes": bytes
            }),
        ))
    }
}

// ============================================================================
// List Directory Tool
// ============================================================================

/// Tool for listing directory contents
pub struct ListDirectoryTool {
    /// Root directory for sandboxing (planned for Phase 7: hardening)
    #[allow(dead_code)]
    sandbox_root: Option<PathBuf>,
}

impl ListDirectoryTool {
    pub fn new() -> Self {
        Self { sandbox_root: None }
    }

    pub fn with_sandbox(root: PathBuf) -> Self {
        Self {
            sandbox_root: Some(root),
        }
    }
}

impl Default for ListDirectoryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct ListDirParams {
    path: String,
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    max_depth: Option<usize>,
}

#[async_trait]
impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List files and directories in a path."
    }

    fn parameters_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory to list"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List recursively",
                    "default": false
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum depth for recursive listing"
                }
            }
        })
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let params: ListDirParams = serde_json::from_value(params)?;

        let path = Path::new(&params.path);
        if !path.exists() {
            return Ok(ToolResult::error(format!(
                "Directory not found: {}",
                params.path
            )));
        }

        if !path.is_dir() {
            return Ok(ToolResult::error(format!(
                "Not a directory: {}",
                params.path
            )));
        }

        let mut entries = Vec::new();
        let mut dir_stack = vec![(path.to_path_buf(), 0)];

        while let Some((current_dir, depth)) = dir_stack.pop() {
            if let Some(max) = params.max_depth {
                if depth > max {
                    continue;
                }
            }

            let mut read_dir = tokio::fs::read_dir(&current_dir).await?;

            while let Some(entry) = read_dir.next_entry().await? {
                let entry_path = entry.path();
                let is_dir = entry_path.is_dir();
                let relative = entry_path.strip_prefix(path).unwrap_or(&entry_path);

                entries.push(json!({
                    "name": entry.file_name().to_string_lossy(),
                    "path": relative.to_string_lossy(),
                    "is_dir": is_dir,
                    "size": if is_dir { 0 } else {
                        entry.metadata().await.map(|m| m.len()).unwrap_or(0)
                    }
                }));

                if params.recursive && is_dir {
                    dir_stack.push((entry_path, depth + 1));
                }
            }
        }

        let output = entries
            .iter()
            .map(|e| {
                let prefix = if e["is_dir"].as_bool().unwrap_or(false) {
                    "📁 "
                } else {
                    "📄 "
                };
                format!("{}{}", prefix, e["path"].as_str().unwrap_or(""))
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolResult::success_with_data(
            output,
            json!({
                "path": params.path,
                "count": entries.len(),
                "entries": entries
            }),
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_file() {
        let tool = ReadFileTool::new();

        // Try to read Cargo.toml (should exist)
        let result = tool
            .execute(json!({
                "path": "Cargo.toml"
            }))
            .await;

        // This test will pass if run from the crate root
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_file_requires_confirmation() {
        let tool = WriteFileTool::new();
        assert!(tool.requires_confirmation());
    }
}
