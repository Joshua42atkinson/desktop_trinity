#![allow(unused)]
//! Tool System for Trinity GGUF Models
//!
//! Implements function/tool calling for local GGUF models like GPT-OSS 120B.
//! Tools can be dynamically created, stored, and executed.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

// ============================================================================
// TOOL DEFINITION
// ============================================================================

/// A tool that can be called by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique tool identifier
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Parameter definitions
    pub parameters: Vec<ToolParameter>,
    /// Tool category for organization
    #[serde(default)]
    pub category: ToolCategory,
    /// Whether this tool requires confirmation before execution
    #[serde(default)]
    pub requires_confirmation: bool,
    /// Tool implementation type
    pub implementation: ToolImplementation,
}

/// Parameter for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub param_type: ParamType,
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub enum_values: Option<Vec<String>>,
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    File,
    Code,
}

/// Tool categories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    #[default]
    General,
    FileSystem,
    Code,
    Research,
    Web,
    Database,
    System,
    Custom,
}

/// How the tool is implemented
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolImplementation {
    /// Built-in Rust function
    Builtin { handler: String },
    /// Python script
    Python { script: String },
    /// Shell command
    Shell { command: String },
    /// HTTP API call
    Http {
        url: String,
        method: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    /// JavaScript (for web tools)
    JavaScript { code: String },
    /// LLM-generated (dynamic tool)
    Generated {
        prompt: String,
        #[serde(default)]
        examples: Vec<String>,
    },
}

impl Tool {
    /// Create a new tool with basic info
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: Vec::new(),
            category: ToolCategory::default(),
            requires_confirmation: false,
            implementation: ToolImplementation::Builtin {
                handler: "default".to_string(),
            },
        }
    }

    /// Add a parameter
    pub fn with_param(mut self, param: ToolParameter) -> Self {
        self.parameters.push(param);
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: ToolCategory) -> Self {
        self.category = category;
        self
    }

    /// Set implementation
    pub fn with_implementation(mut self, impl_: ToolImplementation) -> Self {
        self.implementation = impl_;
        self
    }

    /// Require confirmation before execution
    pub fn require_confirmation(mut self) -> Self {
        self.requires_confirmation = true;
        self
    }

    /// Generate OpenAI-compatible function schema
    pub fn to_openai_schema(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &self.parameters {
            let type_str = match param.param_type {
                ParamType::String | ParamType::File | ParamType::Code => "string",
                ParamType::Integer => "integer",
                ParamType::Float => "number",
                ParamType::Boolean => "boolean",
                ParamType::Array => "array",
                ParamType::Object => "object",
            };

            let mut prop = serde_json::json!({
                "type": type_str,
                "description": param.description
            });

            if let Some(ref enums) = param.enum_values {
                prop["enum"] = serde_json::json!(enums);
            }

            properties.insert(param.name.clone(), prop);

            if param.required {
                required.push(param.name.clone());
            }
        }

        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required
                }
            }
        })
    }
}

// ============================================================================
// TOOL CALL PARSING
// ============================================================================

/// A tool call parsed from LLM output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

/// Result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub success: bool,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            tool_call_id: id.into(),
            success: true,
            output: output.into(),
            error: None,
        }
    }

    pub fn error(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: id.into(),
            success: false,
            output: String::new(),
            error: Some(error.into()),
        }
    }
}

/// Parse tool calls from LLM output
pub fn parse_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();

    // Try JSON format first (OpenAI-style)
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(tool_calls) = parsed.get("tool_calls").and_then(|v| v.as_array()) {
            for tc in tool_calls {
                if let (Some(id), Some(function)) =
                    (tc.get("id").and_then(|v| v.as_str()), tc.get("function"))
                {
                    if let (Some(name), Some(args)) = (
                        function.get("name").and_then(|v| v.as_str()),
                        function.get("arguments"),
                    ) {
                        let arguments: HashMap<String, serde_json::Value> =
                            if let Some(s) = args.as_str() {
                                serde_json::from_str(s).unwrap_or_default()
                            } else if let Some(obj) = args.as_object() {
                                obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                            } else {
                                HashMap::new()
                            };

                        calls.push(ToolCall {
                            id: id.to_string(),
                            name: name.to_string(),
                            arguments,
                        });
                    }
                }
            }
        }
    }

    // Try XML-style tags (common in open-source models)
    // <tool_call>{"name": "tool_name", "arguments": {...}}</tool_call>
    let re = regex::Regex::new(r"<tool_call>(.*?)</tool_call>").ok();
    if let Some(re) = re {
        for cap in re.captures_iter(text) {
            if let Some(json_str) = cap.get(1) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str.as_str()) {
                    if let Some(name) = parsed.get("name").and_then(|v| v.as_str()) {
                        let arguments = parsed
                            .get("arguments")
                            .and_then(|v| v.as_object())
                            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                            .unwrap_or_default();

                        calls.push(ToolCall {
                            id: uuid::Uuid::new_v4().to_string(),
                            name: name.to_string(),
                            arguments,
                        });
                    }
                }
            }
        }
    }

    calls
}

// ============================================================================
// TOOL REGISTRY
// ============================================================================

/// Registry for storing and managing tools
pub struct ToolRegistry {
    /// Tools by name
    tools: HashMap<String, Tool>,
    /// Tools by category
    by_category: HashMap<String, Vec<String>>,
    /// Storage path for persisting tools
    storage_path: Option<PathBuf>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            by_category: HashMap::new(),
            storage_path: None,
        }
    }

    /// Create registry with persistence path
    pub fn with_storage(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let mut registry = Self::new();
        registry.storage_path = Some(path.clone());

        // Load existing tools if directory exists
        if path.exists() {
            registry.load_from_directory(&path)?;
        }

        Ok(registry)
    }

    /// Register a tool
    pub fn register(&mut self, tool: Tool) {
        let category = format!("{:?}", tool.category);

        self.by_category
            .entry(category)
            .or_default()
            .push(tool.name.clone());

        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// List all tools
    pub fn list(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// List tools by category
    pub fn list_by_category(&self, category: &str) -> Vec<&Tool> {
        self.by_category
            .get(category)
            .map(|names| names.iter().filter_map(|n| self.tools.get(n)).collect())
            .unwrap_or_default()
    }

    /// Remove a tool
    pub fn unregister(&mut self, name: &str) -> Option<Tool> {
        if let Some(tool) = self.tools.remove(name) {
            let category = format!("{:?}", tool.category);
            if let Some(names) = self.by_category.get_mut(&category) {
                names.retain(|n| n != name);
            }
            Some(tool)
        } else {
            None
        }
    }

    /// Generate all tools as OpenAI-compatible schema
    pub fn to_openai_tools(&self) -> Vec<serde_json::Value> {
        self.tools.values().map(|t| t.to_openai_schema()).collect()
    }

    /// Load tools from a directory of YAML files
    pub fn load_from_directory(&mut self, path: &Path) -> Result<usize> {
        let mut count = 0;

        if !path.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path
                .extension()
                .is_some_and(|e| e == "yaml" || e == "yml")
            {
                match self.load_tool_from_file(&file_path) {
                    Ok(_) => count += 1,
                    Err(e) => log::warn!("Failed to load tool from {:?}: {}", file_path, e),
                }
            }
        }

        log::info!("Loaded {} tools from {:?}", count, path);
        Ok(count)
    }

    /// Load a single tool from a YAML file
    pub fn load_tool_from_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read tool file: {:?}", path))?;

        let tool: Tool = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse tool YAML: {:?}", path))?;

        self.register(tool);
        Ok(())
    }

    /// Save a tool to persistent storage
    pub fn save_tool(&self, tool: &Tool) -> Result<()> {
        let storage = self
            .storage_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No storage path configured"))?;

        fs::create_dir_all(storage)?;

        let file_path = storage.join(format!("{}.yaml", tool.name));
        let yaml = serde_yaml::to_string(tool)?;
        fs::write(&file_path, yaml)?;

        log::info!("Saved tool {} to {:?}", tool.name, file_path);
        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// BUILT-IN TOOLS
// ============================================================================

/// Create default built-in tools
pub fn create_builtin_tools() -> Vec<Tool> {
    vec![
        // File System Tools
        Tool::new("read_file", "Read the contents of a file")
            .with_param(ToolParameter {
                name: "path".to_string(),
                description: "Path to the file to read".to_string(),
                param_type: ParamType::File,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_category(ToolCategory::FileSystem)
            .with_implementation(ToolImplementation::Builtin {
                handler: "read_file".to_string(),
            }),
        Tool::new("write_file", "Write content to a file")
            .with_param(ToolParameter {
                name: "path".to_string(),
                description: "Path to the file to write".to_string(),
                param_type: ParamType::File,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_param(ToolParameter {
                name: "content".to_string(),
                description: "Content to write to the file".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_category(ToolCategory::FileSystem)
            .require_confirmation()
            .with_implementation(ToolImplementation::Builtin {
                handler: "write_file".to_string(),
            }),
        Tool::new("list_directory", "List files and directories in a path")
            .with_param(ToolParameter {
                name: "path".to_string(),
                description: "Directory path to list".to_string(),
                param_type: ParamType::File,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_category(ToolCategory::FileSystem)
            .with_implementation(ToolImplementation::Builtin {
                handler: "list_directory".to_string(),
            }),
        // Code Tools
        Tool::new("run_python", "Execute Python code")
            .with_param(ToolParameter {
                name: "code".to_string(),
                description: "Python code to execute".to_string(),
                param_type: ParamType::Code,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_category(ToolCategory::Code)
            .require_confirmation()
            .with_implementation(ToolImplementation::Python {
                script: "exec(code)".to_string(),
            }),
        Tool::new("run_shell", "Execute a shell command")
            .with_param(ToolParameter {
                name: "command".to_string(),
                description: "Shell command to execute".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_category(ToolCategory::System)
            .require_confirmation()
            .with_implementation(ToolImplementation::Shell {
                command: "$command".to_string(),
            }),
        // Research Tools
        Tool::new("web_search", "Search the web for information")
            .with_param(ToolParameter {
                name: "query".to_string(),
                description: "Search query".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_param(ToolParameter {
                name: "num_results".to_string(),
                description: "Number of results to return".to_string(),
                param_type: ParamType::Integer,
                required: false,
                default: Some("5".to_string()),
                enum_values: None,
            })
            .with_category(ToolCategory::Research)
            .with_implementation(ToolImplementation::Builtin {
                handler: "web_search".to_string(),
            }),
        Tool::new("calculate", "Perform mathematical calculations")
            .with_param(ToolParameter {
                name: "expression".to_string(),
                description: "Mathematical expression to evaluate".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_category(ToolCategory::General)
            .with_implementation(ToolImplementation::Builtin {
                handler: "calculate".to_string(),
            }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new("test_tool", "A test tool").with_param(ToolParameter {
            name: "input".to_string(),
            description: "Test input".to_string(),
            param_type: ParamType::String,
            required: true,
            default: None,
            enum_values: None,
        });

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.parameters.len(), 1);
    }

    #[test]
    fn test_openai_schema() {
        let tool = Tool::new("greet", "Say hello").with_param(ToolParameter {
            name: "name".to_string(),
            description: "Name to greet".to_string(),
            param_type: ParamType::String,
            required: true,
            default: None,
            enum_values: None,
        });

        let schema = tool.to_openai_schema();
        assert_eq!(schema["function"]["name"], "greet");
    }

    #[test]
    fn test_registry() {
        let mut registry = ToolRegistry::new();

        registry.register(Tool::new("tool1", "Test 1").with_category(ToolCategory::General));
        registry.register(Tool::new("tool2", "Test 2").with_category(ToolCategory::Code));

        assert_eq!(registry.list().len(), 2);
        assert!(registry.get("tool1").is_some());
    }

    #[test]
    fn test_builtin_tools() {
        let tools = create_builtin_tools();
        assert!(tools.len() >= 5);

        let has_read_file = tools.iter().any(|t| t.name == "read_file");
        assert!(has_read_file);
    }
}
