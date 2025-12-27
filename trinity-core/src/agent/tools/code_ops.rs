//! Code Operations Tools
//!
//! Provides code editing, command execution, and compilation tools for agents.

use super::{Tool, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

// ============================================================================
// Edit Code Tool
// ============================================================================

/// Tool for making structured edits to code files
pub struct EditCodeTool {
    /// Root directory for sandboxing (planned for Phase 7: hardening)
    #[allow(dead_code)]
    sandbox_root: Option<PathBuf>,
}

impl EditCodeTool {
    pub fn new() -> Self {
        Self { sandbox_root: None }
    }

    pub fn with_sandbox(root: PathBuf) -> Self {
        Self {
            sandbox_root: Some(root),
        }
    }
}

impl Default for EditCodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct EditCodeParams {
    path: String,
    /// Text to search for (must exist in file)
    search: String,
    /// Text to replace with
    replace: String,
    /// Only replace first occurrence (default: true)
    #[serde(default = "default_true")]
    first_only: bool,
}

fn default_true() -> bool {
    true
}

#[async_trait]
impl Tool for EditCodeTool {
    fn name(&self) -> &str {
        "edit_code"
    }

    fn description(&self) -> &str {
        "Edit a code file by searching for text and replacing it. The search text must exist in the file."
    }

    fn parameters_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "required": ["path", "search", "replace"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "search": {
                    "type": "string",
                    "description": "Exact text to search for in the file"
                },
                "replace": {
                    "type": "string",
                    "description": "Text to replace the search text with"
                },
                "first_only": {
                    "type": "boolean",
                    "description": "Only replace first occurrence",
                    "default": true
                }
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn risk_level(&self) -> u8 {
        6
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let params: EditCodeParams = serde_json::from_value(params)?;

        let path = Path::new(&params.path);
        if !path.exists() {
            return Ok(ToolResult::error(format!(
                "File not found: {}",
                params.path
            )));
        }

        let content = tokio::fs::read_to_string(&path).await?;

        // Check if search text exists
        if !content.contains(&params.search) {
            return Ok(ToolResult::error(
                "Search text not found in file. Make sure to use exact text including whitespace."
                    .to_string(),
            ));
        }

        // Perform replacement
        let new_content = if params.first_only {
            content.replacen(&params.search, &params.replace, 1)
        } else {
            content.replace(&params.search, &params.replace)
        };

        let replacements = if params.first_only {
            1
        } else {
            content.matches(&params.search).count()
        };

        // Write back
        tokio::fs::write(&path, &new_content).await?;

        Ok(ToolResult::success_with_data(
            format!("Made {} replacement(s) in {}", replacements, params.path),
            json!({
                "path": params.path,
                "replacements": replacements,
                "lines_changed": new_content.lines().count()
            }),
        ))
    }
}

// ============================================================================
// Run Command Tool
// ============================================================================

/// Tool for running shell commands
pub struct RunCommandTool {
    /// Working directory for commands
    working_dir: Option<PathBuf>,
    /// Timeout in seconds
    timeout_secs: u64,
    /// Allowed command prefixes (if set)
    allowed_prefixes: Option<Vec<String>>,
}

impl RunCommandTool {
    pub fn new() -> Self {
        Self {
            working_dir: None,
            timeout_secs: 60,
            allowed_prefixes: None,
        }
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_allowed_prefixes(mut self, prefixes: Vec<String>) -> Self {
        self.allowed_prefixes = Some(prefixes);
        self
    }

    /// Create a safe version that only allows specific commands
    pub fn safe() -> Self {
        Self {
            working_dir: None,
            timeout_secs: 30,
            allowed_prefixes: Some(vec![
                "cargo".to_string(),
                "rustc".to_string(),
                "git".to_string(),
                "ls".to_string(),
                "cat".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "head".to_string(),
                "tail".to_string(),
                "wc".to_string(),
                "echo".to_string(),
            ]),
        }
    }
}

impl Default for RunCommandTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct RunCommandParams {
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    cwd: Option<String>,
}

#[async_trait]
impl Tool for RunCommandTool {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Run a shell command and return its output."
    }

    fn parameters_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "required": ["command"],
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to run"
                },
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Command arguments"
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory for the command"
                }
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }

    fn risk_level(&self) -> u8 {
        8
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let params: RunCommandParams = serde_json::from_value(params)?;

        // Check if command is allowed
        if let Some(allowed) = &self.allowed_prefixes {
            if !allowed.iter().any(|p| params.command.starts_with(p)) {
                return Ok(ToolResult::error(format!(
                    "Command '{}' not allowed. Allowed: {:?}",
                    params.command, allowed
                )));
            }
        }

        // Determine working directory
        let cwd = params
            .cwd
            .map(PathBuf::from)
            .or_else(|| self.working_dir.clone())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Build command
        let mut cmd = Command::new(&params.command);
        cmd.args(&params.args)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Execute with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            cmd.output(),
        )
        .await;

        match output {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let combined = if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{}\n\n--- STDERR ---\n{}", stdout, stderr)
                };

                if output.status.success() {
                    Ok(ToolResult::success_with_data(
                        combined,
                        json!({
                            "exit_code": output.status.code(),
                            "command": params.command,
                            "cwd": cwd.to_string_lossy()
                        }),
                    ))
                } else {
                    Ok(ToolResult::error(format!(
                        "Command failed with exit code {:?}\n{}",
                        output.status.code(),
                        combined
                    )))
                }
            }
            Ok(Err(e)) => Ok(ToolResult::error(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => Ok(ToolResult::error(format!(
                "Command timed out after {} seconds",
                self.timeout_secs
            ))),
        }
    }
}

// ============================================================================
// Cargo Build Tool
// ============================================================================

/// Tool specifically for Cargo operations (safer than raw commands)
pub struct CargoBuildTool {
    /// Working directory (project root)
    project_root: Option<PathBuf>,
}

impl CargoBuildTool {
    pub fn new() -> Self {
        Self { project_root: None }
    }

    pub fn with_project(root: PathBuf) -> Self {
        Self {
            project_root: Some(root),
        }
    }
}

impl Default for CargoBuildTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct CargoBuildParams {
    /// Cargo subcommand (check, build, test, clippy)
    subcommand: String,
    /// Package to target (optional)
    #[serde(default)]
    package: Option<String>,
    /// Features to enable
    #[serde(default)]
    features: Vec<String>,
    /// Release mode
    #[serde(default)]
    release: bool,
}

#[async_trait]
impl Tool for CargoBuildTool {
    fn name(&self) -> &str {
        "cargo_build"
    }

    fn description(&self) -> &str {
        "Run Cargo commands (check, build, test, clippy) on the project."
    }

    fn parameters_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "required": ["subcommand"],
            "properties": {
                "subcommand": {
                    "type": "string",
                    "enum": ["check", "build", "test", "clippy", "fmt"],
                    "description": "Cargo subcommand to run"
                },
                "package": {
                    "type": "string",
                    "description": "Specific package to target"
                },
                "features": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Features to enable"
                },
                "release": {
                    "type": "boolean",
                    "description": "Build in release mode"
                }
            }
        })
    }

    fn risk_level(&self) -> u8 {
        3 // Lower risk than arbitrary commands
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let params: CargoBuildParams = serde_json::from_value(params)?;

        // Validate subcommand
        let allowed = ["check", "build", "test", "clippy", "fmt"];
        if !allowed.contains(&params.subcommand.as_str()) {
            return Ok(ToolResult::error(format!(
                "Invalid subcommand. Allowed: {:?}",
                allowed
            )));
        }

        let cwd = self
            .project_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let mut cmd = Command::new("cargo");
        cmd.arg(&params.subcommand)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(pkg) = &params.package {
            cmd.args(["-p", pkg]);
        }

        if !params.features.is_empty() {
            cmd.args(["--features", &params.features.join(",")]);
        }

        if params.release {
            cmd.arg("--release");
        }

        // Color output for readability
        cmd.args(["--color", "always"]);

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minute timeout for builds
            cmd.output(),
        )
        .await;

        match output {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Cargo outputs to stderr for status
                let combined = format!("{}{}", stdout, stderr);

                if output.status.success() {
                    Ok(ToolResult::success_with_data(
                        combined,
                        json!({
                            "success": true,
                            "subcommand": params.subcommand
                        }),
                    ))
                } else {
                    Ok(ToolResult::error(format!(
                        "cargo {} failed:\n{}",
                        params.subcommand, combined
                    )))
                }
            }
            Ok(Err(e)) => Ok(ToolResult::error(format!("Failed to run cargo: {}", e))),
            Err(_) => Ok(ToolResult::error("Cargo command timed out after 5 minutes")),
        }
    }
}

// ============================================================================
// Search Code Tool
// ============================================================================

/// Tool for searching code using ripgrep
pub struct SearchCodeTool {
    /// Root directory for searches
    root: Option<PathBuf>,
}

impl SearchCodeTool {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self { root: Some(root) }
    }
}

impl Default for SearchCodeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct SearchCodeParams {
    /// Pattern to search for
    pattern: String,
    /// File extension filter
    #[serde(default)]
    extension: Option<String>,
    /// Max results
    #[serde(default = "default_max_results")]
    max_results: usize,
    /// Case insensitive
    #[serde(default)]
    case_insensitive: bool,
}

fn default_max_results() -> usize {
    50
}

#[async_trait]
impl Tool for SearchCodeTool {
    fn name(&self) -> &str {
        "search_code"
    }

    fn description(&self) -> &str {
        "Search for patterns in code files using ripgrep."
    }

    fn parameters_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "required": ["pattern"],
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Search pattern (regex supported)"
                },
                "extension": {
                    "type": "string",
                    "description": "Filter by file extension (e.g., 'rs', 'py')"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results",
                    "default": 50
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "Case insensitive search"
                }
            }
        })
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let params: SearchCodeParams = serde_json::from_value(params)?;

        let root = self
            .root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let mut cmd = Command::new("rg");
        cmd.arg(&params.pattern)
            .arg("--line-number")
            .arg("--color=never")
            .args(["--max-count", &params.max_results.to_string()])
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(ext) = &params.extension {
            cmd.args(["-g", &format!("*.{}", ext)]);
        }

        if params.case_insensitive {
            cmd.arg("-i");
        }

        let output = cmd.output().await;

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                if stdout.is_empty() {
                    Ok(ToolResult::success("No matches found"))
                } else {
                    let match_count = stdout.lines().count();
                    Ok(ToolResult::success_with_data(
                        stdout.to_string(),
                        json!({
                            "matches": match_count,
                            "pattern": params.pattern
                        }),
                    ))
                }
            }
            Err(e) => Ok(ToolResult::error(format!(
                "Search failed (is ripgrep installed?): {}",
                e
            ))),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_safe() {
        let tool = RunCommandTool::safe();
        assert!(tool.requires_confirmation());
        assert!(tool.allowed_prefixes.is_some());
    }

    #[test]
    fn test_cargo_build_risk_level() {
        let tool = CargoBuildTool::new();
        // Cargo is safer than arbitrary commands
        assert!(tool.risk_level() < RunCommandTool::new().risk_level());
    }
}
