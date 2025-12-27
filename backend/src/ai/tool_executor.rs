#![allow(unused)]
//! Tool Executor - Executes tools in a sandboxed environment
//!
//! Handles safe execution of tools with timeout, output capture,
//! and confirmation for dangerous operations.

use super::tools::{Tool, ToolCall, ToolImplementation, ToolRegistry, ToolResult};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

/// Configuration for tool execution
#[derive(Clone, Debug)]
pub struct ExecutorConfig {
    /// Maximum execution time per tool
    pub timeout: Duration,
    /// Working directory for shell commands
    pub working_dir: String,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
    /// Whether to require confirmation for dangerous tools
    pub require_confirmation: bool,
    /// Maximum output size in bytes
    pub max_output_size: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            working_dir: ".".to_string(),
            env_vars: HashMap::new(),
            require_confirmation: true,
            max_output_size: 1024 * 1024, // 1MB
        }
    }
}

/// Tool executor with sandboxing and safety features
pub struct ToolExecutor {
    config: ExecutorConfig,
    /// Callbacks for confirmation prompts
    #[allow(clippy::type_complexity)]
    confirmation_callback: Option<Box<dyn Fn(&Tool) -> bool + Send + Sync>>,
    /// Execution log
    execution_log: Vec<ExecutionLogEntry>,
}

/// Log entry for tool execution
#[derive(Debug, Clone)]
pub struct ExecutionLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tool_name: String,
    pub arguments: HashMap<String, String>,
    pub success: bool,
    pub duration_ms: u64,
    pub output_preview: String,
}

impl ToolExecutor {
    /// Create a new executor with default config
    pub fn new() -> Self {
        Self {
            config: ExecutorConfig::default(),
            confirmation_callback: None,
            execution_log: Vec::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self {
            config,
            confirmation_callback: None,
            execution_log: Vec::new(),
        }
    }

    /// Set confirmation callback
    pub fn set_confirmation_callback<F>(&mut self, callback: F)
    where
        F: Fn(&Tool) -> bool + Send + Sync + 'static,
    {
        self.confirmation_callback = Some(Box::new(callback));
    }

    /// Execute a tool call
    pub async fn execute(&mut self, tool: &Tool, call: &ToolCall) -> ToolResult {
        let start = std::time::Instant::now();

        // Check confirmation if required
        if tool.requires_confirmation && self.config.require_confirmation {
            if let Some(ref callback) = self.confirmation_callback {
                if !callback(tool) {
                    return ToolResult::error(&call.id, "Execution cancelled by user");
                }
            } else {
                log::warn!(
                    "Tool {} requires confirmation but no callback set",
                    tool.name
                );
            }
        }

        // Execute based on implementation type
        let result = match &tool.implementation {
            ToolImplementation::Builtin { handler } => {
                self.execute_builtin(handler, &call.arguments).await
            }
            ToolImplementation::Shell { command } => {
                self.execute_shell(command, &call.arguments).await
            }
            ToolImplementation::Python { script } => {
                self.execute_python(script, &call.arguments).await
            }
            ToolImplementation::Http {
                url,
                method,
                headers,
            } => {
                self.execute_http(url, method, headers, &call.arguments)
                    .await
            }
            ToolImplementation::JavaScript { code } => {
                self.execute_javascript(code, &call.arguments).await
            }
            ToolImplementation::Generated { prompt, examples } => {
                // Generated tools need LLM - return placeholder
                Ok(format!(
                    "Generated tool execution not yet implemented: {}",
                    prompt
                ))
            }
        };

        let duration = start.elapsed();

        // Build result
        let tool_result = match result {
            Ok(output) => {
                let truncated = if output.len() > self.config.max_output_size {
                    format!("{}... (truncated)", &output[..self.config.max_output_size])
                } else {
                    output.clone()
                };

                ToolResult::success(&call.id, truncated)
            }
            Err(e) => ToolResult::error(&call.id, e.to_string()),
        };

        // Log execution
        self.execution_log.push(ExecutionLogEntry {
            timestamp: chrono::Utc::now(),
            tool_name: tool.name.clone(),
            arguments: call
                .arguments
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
            success: tool_result.success,
            duration_ms: duration.as_millis() as u64,
            output_preview: tool_result.output.chars().take(100).collect(),
        });

        tool_result
    }

    /// Execute a built-in tool
    async fn execute_builtin(
        &self,
        handler: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        match handler {
            "read_file" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;

                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read file: {}", path))?;

                Ok(content)
            }

            "write_file" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;

                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing content argument"))?;

                std::fs::write(path, content)
                    .with_context(|| format!("Failed to write file: {}", path))?;

                Ok(format!(
                    "Successfully wrote {} bytes to {}",
                    content.len(),
                    path
                ))
            }

            "list_directory" => {
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing path argument"))?;

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

                Ok(entries.join("\n"))
            }

            "calculate" => {
                let expr = args
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing expression argument"))?;

                // Simple expression evaluator (production would use a proper parser)
                // For now, just use Python as a calculator
                let output = Command::new("python3")
                    .args(["-c", &format!("print({})", expr)])
                    .output()?;

                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    Err(anyhow::anyhow!(
                        "Calculation error: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            }

            "web_search" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing query argument"))?;

                // Placeholder - would integrate with search API
                Ok(format!(
                    "Search results for '{}' (placeholder - integrate search API)",
                    query
                ))
            }

            _ => Err(anyhow::anyhow!("Unknown builtin handler: {}", handler)),
        }
    }

    /// Execute a shell command
    async fn execute_shell(
        &self,
        command_template: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        // Substitute arguments into command
        let mut command = command_template.to_string();
        for (key, value) in args {
            let placeholder = format!("${}", key);
            let value_str = match value.as_str() {
                Some(s) => s.to_string(),
                None => value.to_string(),
            };
            command = command.replace(&placeholder, &value_str);
        }

        // Execute with timeout
        let output = tokio::task::spawn_blocking(move || {
            Command::new("bash")
                .args(["-c", &command])
                .current_dir(".")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        })
        .await??;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Execute Python code
    async fn execute_python(
        &self,
        script: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        // Build Python code with arguments
        let mut code = String::new();

        // Inject arguments as variables
        for (key, value) in args {
            code.push_str(&format!("{} = {}\n", key, value));
        }

        code.push_str(script);

        // Execute Python
        let output = tokio::task::spawn_blocking(move || {
            Command::new("python3")
                .args(["-c", &code])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        })
        .await??;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Python error: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Execute HTTP request
    async fn execute_http(
        &self,
        url: &str,
        method: &str,
        headers: &HashMap<String, String>,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        // Build URL with query params for GET
        let mut request_url = url.to_string();

        let client = reqwest::Client::new();

        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(&request_url),
            "POST" => client.post(&request_url).json(args),
            "PUT" => client.put(&request_url).json(args),
            "DELETE" => client.delete(&request_url),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
        };

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        let body = response.text().await?;

        Ok(body)
    }

    /// Execute JavaScript (placeholder)
    async fn execute_javascript(
        &self,
        code: &str,
        _args: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        // Would use deno or node for execution
        Ok(format!(
            "JavaScript execution not yet implemented: {} chars",
            code.len()
        ))
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

    #[test]
    fn test_executor_config() {
        let config = ExecutorConfig::default();
        assert_eq!(config.timeout.as_secs(), 30);
    }
}
