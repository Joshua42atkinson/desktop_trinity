//! Tool System - Structured Agent Capabilities
//!
//! Provides a trait-based interface for agent tools (file ops, code ops, web search, etc.)
//! Tools are registered and dispatched based on LLM output parsing.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

pub mod code_ops;
pub mod file_ops;

// ============================================================================
// Tool Result
// ============================================================================

/// Result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool succeeded
    pub success: bool,
    /// Output from the tool
    pub output: String,
    /// Optional structured data
    pub data: Option<JsonValue>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a success result
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            data: None,
            error: None,
        }
    }

    /// Create a success result with data
    pub fn success_with_data(output: impl Into<String>, data: JsonValue) -> Self {
        Self {
            success: true,
            output: output.into(),
            data: Some(data),
            error: None,
        }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        let msg = message.into();
        Self {
            success: false,
            output: String::new(),
            data: None,
            error: Some(msg),
        }
    }
}

// ============================================================================
// Tool Trait
// ============================================================================

/// Core trait for all agent tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name for this tool
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// JSON Schema for tool parameters
    fn parameters_schema(&self) -> JsonValue;

    /// Execute the tool with given parameters
    async fn execute(&self, params: JsonValue) -> Result<ToolResult>;

    /// Whether this tool requires user confirmation before execution
    fn requires_confirmation(&self) -> bool {
        false
    }

    /// Risk level (0-10) for UI display
    fn risk_level(&self) -> u8 {
        0
    }
}

// ============================================================================
// Tool Registry
// ============================================================================

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register file operations
        registry.register(Arc::new(file_ops::ReadFileTool::new()));
        registry.register(Arc::new(file_ops::WriteFileTool::new()));
        registry.register(Arc::new(file_ops::ListDirectoryTool::new()));

        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Get all tool names
    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Execute a tool by name
    pub async fn execute(&self, name: &str, params: JsonValue) -> Result<ToolResult> {
        if let Some(tool) = self.tools.get(name) {
            tool.execute(params).await
        } else {
            Ok(ToolResult::error(format!("Unknown tool: {}", name)))
        }
    }

    /// Generate tool documentation for system prompts
    pub fn generate_docs(&self) -> String {
        let mut docs = String::from("# Available Tools\n\n");

        for (name, tool) in &self.tools {
            docs.push_str(&format!("## {}\n", name));
            docs.push_str(&format!("{}\n\n", tool.description()));
            docs.push_str("**Parameters:**\n```json\n");
            docs.push_str(
                &serde_json::to_string_pretty(&tool.parameters_schema()).unwrap_or_default(),
            );
            docs.push_str("\n```\n\n");

            if tool.requires_confirmation() {
                docs.push_str("⚠️ *Requires user confirmation*\n\n");
            }
        }

        docs
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tool Call Parsing
// ============================================================================

/// A parsed tool call from LLM output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the tool to execute
    pub tool_name: String,
    /// Parameters for the tool
    pub parameters: JsonValue,
}

impl ToolCall {
    /// Parse tool calls from LLM output
    ///
    /// Looks for JSON blocks in markdown code fences:
    /// ```json
    /// {"tool": "read_file", "params": {"path": "/path/to/file"}}
    /// ```
    pub fn parse_from_output(output: &str) -> Vec<ToolCall> {
        let mut calls = Vec::new();

        // Find JSON code blocks
        let json_pattern = regex::Regex::new(r"```json\s*([\s\S]*?)```").ok();

        if let Some(re) = json_pattern {
            for cap in re.captures_iter(output) {
                if let Some(json_str) = cap.get(1) {
                    if let Ok(value) = serde_json::from_str::<JsonValue>(json_str.as_str().trim()) {
                        // Check for tool call format
                        if let (Some(tool), Some(params)) = (
                            value.get("tool").and_then(|v| v.as_str()),
                            value.get("params"),
                        ) {
                            calls.push(ToolCall {
                                tool_name: tool.to_string(),
                                parameters: params.clone(),
                            });
                        }
                        // Also check for function_call format
                        else if let (Some(name), Some(arguments)) = (
                            value.get("name").and_then(|v| v.as_str()),
                            value.get("arguments"),
                        ) {
                            calls.push(ToolCall {
                                tool_name: name.to_string(),
                                parameters: arguments.clone(),
                            });
                        }
                    }
                }
            }
        }

        calls
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_parsing() {
        let output = r#"
I'll read the file for you.

```json
{"tool": "read_file", "params": {"path": "/home/user/test.txt"}}
```

Let me know if you need anything else.
        "#;

        let calls = ToolCall::parse_from_output(output);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_name, "read_file");
    }

    #[test]
    fn test_tool_result() {
        let success = ToolResult::success("File read successfully");
        assert!(success.success);

        let error = ToolResult::error("File not found");
        assert!(!error.success);
    }
}
