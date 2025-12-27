#![allow(unused)]
//! Tool Factory - Dynamic tool creation and LLM-assisted generation
//!
//! Allows Trinity to create new tools at runtime based on natural language
//! descriptions, storing them for future use.

use super::tools::{
    ParamType, Tool, ToolCategory, ToolImplementation, ToolParameter, ToolRegistry,
};
use crate::ai::llm::{GgufGenerateConfig, GgufModel};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Factory for creating tools dynamically
pub struct ToolFactory {
    /// Storage path for generated tools
    storage_path: PathBuf,
    /// Template for generating tool definitions
    generation_prompt: String,
}

impl ToolFactory {
    /// Create a new tool factory
    pub fn new(storage_path: impl Into<PathBuf>) -> Self {
        Self {
            storage_path: storage_path.into(),
            generation_prompt: DEFAULT_GENERATION_PROMPT.to_string(),
        }
    }

    /// Create a tool from a natural language description
    pub fn create_from_description(
        &self,
        name: &str,
        description: &str,
        category: ToolCategory,
    ) -> Result<Tool> {
        // Parse the description to extract parameters
        let params = self.infer_parameters(description);

        let tool = Tool::new(name, description)
            .with_category(category)
            .with_implementation(ToolImplementation::Generated {
                prompt: description.to_string(),
                examples: Vec::new(),
            });

        // Add inferred parameters
        let mut tool = tool;
        for param in params {
            tool = tool.with_param(param);
        }

        Ok(tool)
    }

    /// Create a tool with LLM assistance
    pub fn create_with_llm(&self, description: &str, model: &mut GgufModel) -> Result<Tool> {
        let prompt = format!(
            "{}\n\nUser request: {}\n\nGenerate a tool definition in YAML format:",
            self.generation_prompt, description
        );

        let config = GgufGenerateConfig {
            temperature: 0.3,
            max_tokens: 1024,
            ..Default::default()
        };

        let response = model.generate(&prompt, &config)?;

        // Parse the YAML response
        self.parse_tool_yaml(&response)
    }

    /// Infer parameters from a description (basic heuristics)
    fn infer_parameters(&self, description: &str) -> Vec<ToolParameter> {
        let mut params = Vec::new();
        let desc_lower = description.to_lowercase();

        // File operations
        if desc_lower.contains("file") || desc_lower.contains("path") {
            params.push(ToolParameter {
                name: "path".to_string(),
                description: "File or directory path".to_string(),
                param_type: ParamType::File,
                required: true,
                default: None,
                enum_values: None,
            });
        }

        // Content/text operations
        if desc_lower.contains("content")
            || desc_lower.contains("text")
            || desc_lower.contains("write")
        {
            params.push(ToolParameter {
                name: "content".to_string(),
                description: "Text content".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            });
        }

        // Search/query operations
        if desc_lower.contains("search")
            || desc_lower.contains("query")
            || desc_lower.contains("find")
        {
            params.push(ToolParameter {
                name: "query".to_string(),
                description: "Search query".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            });
        }

        // URL operations
        if desc_lower.contains("url") || desc_lower.contains("http") || desc_lower.contains("api") {
            params.push(ToolParameter {
                name: "url".to_string(),
                description: "URL to access".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            });
        }

        // Code operations
        if desc_lower.contains("code")
            || desc_lower.contains("script")
            || desc_lower.contains("execute")
        {
            params.push(ToolParameter {
                name: "code".to_string(),
                description: "Code to execute".to_string(),
                param_type: ParamType::Code,
                required: true,
                default: None,
                enum_values: None,
            });
        }

        params
    }

    /// Parse a YAML tool definition
    fn parse_tool_yaml(&self, yaml: &str) -> Result<Tool> {
        // Extract YAML block if wrapped in code fence
        let yaml_content = if yaml.contains("```") {
            yaml.split("```")
                .nth(1)
                .map(|s| s.trim_start_matches("yaml").trim())
                .unwrap_or(yaml)
        } else {
            yaml
        };

        serde_yaml::from_str(yaml_content).context("Failed to parse tool YAML")
    }

    /// Create a shell tool
    pub fn create_shell_tool(&self, name: &str, description: &str, command: &str) -> Tool {
        Tool::new(name, description)
            .with_category(ToolCategory::System)
            .with_implementation(ToolImplementation::Shell {
                command: command.to_string(),
            })
            .require_confirmation()
    }

    /// Create a Python tool
    pub fn create_python_tool(&self, name: &str, description: &str, script: &str) -> Tool {
        Tool::new(name, description)
            .with_category(ToolCategory::Code)
            .with_implementation(ToolImplementation::Python {
                script: script.to_string(),
            })
    }

    /// Create an HTTP API tool
    pub fn create_http_tool(&self, name: &str, description: &str, url: &str, method: &str) -> Tool {
        Tool::new(name, description)
            .with_category(ToolCategory::Web)
            .with_implementation(ToolImplementation::Http {
                url: url.to_string(),
                method: method.to_string(),
                headers: std::collections::HashMap::new(),
            })
    }

    /// Save a tool to storage
    pub fn save_tool(&self, tool: &Tool) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.storage_path)?;

        let file_path = self.storage_path.join(format!("{}.yaml", tool.name));
        let yaml = serde_yaml::to_string(tool)?;
        std::fs::write(&file_path, yaml)?;

        log::info!("Saved generated tool: {} to {:?}", tool.name, file_path);
        Ok(file_path)
    }
}

/// Default prompt for LLM-assisted tool generation
const DEFAULT_GENERATION_PROMPT: &str = r#"
You are a tool definition generator. Given a natural language description,
generate a tool definition in YAML format.

The tool definition should include:
- name: A short, snake_case identifier
- description: A clear description of what the tool does
- parameters: List of parameters with name, description, type, and required
- category: One of: general, file_system, code, research, web, database, system, custom
- implementation: How the tool is implemented (shell, python, http, etc.)

Example output:
```yaml
name: count_words
description: Count the number of words in a text file
parameters:
  - name: path
    description: Path to the text file
    type: file
    required: true
category: file_system
implementation:
  type: shell
  command: wc -w $path
```
"#;

// ============================================================================
// RESEARCH & DEVELOPMENT TOOLS
// ============================================================================

/// Create research and development focused tools
pub fn create_research_tools() -> Vec<Tool> {
    vec![
        // Code Analysis
        Tool::new(
            "analyze_code",
            "Analyze code for patterns, complexity, and quality",
        )
        .with_param(ToolParameter {
            name: "path".to_string(),
            description: "Path to file or directory to analyze".to_string(),
            param_type: ParamType::File,
            required: true,
            default: None,
            enum_values: None,
        })
        .with_param(ToolParameter {
            name: "analysis_type".to_string(),
            description: "Type of analysis to perform".to_string(),
            param_type: ParamType::String,
            required: false,
            default: Some("all".to_string()),
            enum_values: Some(vec![
                "complexity".to_string(),
                "dependencies".to_string(),
                "patterns".to_string(),
                "security".to_string(),
                "all".to_string(),
            ]),
        })
        .with_category(ToolCategory::Research)
        .with_implementation(ToolImplementation::Generated {
            prompt: "Analyze code quality and patterns".to_string(),
            examples: Vec::new(),
        }),
        // Documentation Generator
        Tool::new("generate_docs", "Generate documentation from code")
            .with_param(ToolParameter {
                name: "path".to_string(),
                description: "Path to source code".to_string(),
                param_type: ParamType::File,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_param(ToolParameter {
                name: "format".to_string(),
                description: "Output documentation format".to_string(),
                param_type: ParamType::String,
                required: false,
                default: Some("markdown".to_string()),
                enum_values: Some(vec![
                    "markdown".to_string(),
                    "html".to_string(),
                    "rst".to_string(),
                ]),
            })
            .with_category(ToolCategory::Code)
            .with_implementation(ToolImplementation::Generated {
                prompt: "Generate documentation from source code".to_string(),
                examples: Vec::new(),
            }),
        // Test Generator
        Tool::new("generate_tests", "Generate unit tests for code")
            .with_param(ToolParameter {
                name: "path".to_string(),
                description: "Path to source file".to_string(),
                param_type: ParamType::File,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_param(ToolParameter {
                name: "framework".to_string(),
                description: "Test framework to use".to_string(),
                param_type: ParamType::String,
                required: false,
                default: None,
                enum_values: Some(vec![
                    "rust".to_string(),
                    "pytest".to_string(),
                    "jest".to_string(),
                ]),
            })
            .with_category(ToolCategory::Code)
            .with_implementation(ToolImplementation::Generated {
                prompt: "Generate unit tests for the given code".to_string(),
                examples: Vec::new(),
            }),
        // Refactoring Suggestions
        Tool::new(
            "suggest_refactoring",
            "Suggest code refactoring improvements",
        )
        .with_param(ToolParameter {
            name: "path".to_string(),
            description: "Path to code to refactor".to_string(),
            param_type: ParamType::File,
            required: true,
            default: None,
            enum_values: None,
        })
        .with_category(ToolCategory::Code)
        .with_implementation(ToolImplementation::Generated {
            prompt: "Analyze code and suggest refactoring improvements".to_string(),
            examples: Vec::new(),
        }),
        // Dependency Analysis
        Tool::new(
            "analyze_dependencies",
            "Analyze project dependencies and suggest updates",
        )
        .with_param(ToolParameter {
            name: "manifest".to_string(),
            description: "Path to package manifest (Cargo.toml, package.json, etc.)".to_string(),
            param_type: ParamType::File,
            required: true,
            default: None,
            enum_values: None,
        })
        .with_category(ToolCategory::Research)
        .with_implementation(ToolImplementation::Builtin {
            handler: "analyze_dependencies".to_string(),
        }),
        // Knowledge Base Query
        Tool::new("query_knowledge", "Query the Trinity knowledge base")
            .with_param(ToolParameter {
                name: "query".to_string(),
                description: "Natural language query".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_param(ToolParameter {
                name: "limit".to_string(),
                description: "Maximum number of results".to_string(),
                param_type: ParamType::Integer,
                required: false,
                default: Some("5".to_string()),
                enum_values: None,
            })
            .with_category(ToolCategory::Research)
            .with_implementation(ToolImplementation::Builtin {
                handler: "query_knowledge".to_string(),
            }),
        // Store Knowledge
        Tool::new("store_knowledge", "Store information in the knowledge base")
            .with_param(ToolParameter {
                name: "content".to_string(),
                description: "Content to store".to_string(),
                param_type: ParamType::String,
                required: true,
                default: None,
                enum_values: None,
            })
            .with_param(ToolParameter {
                name: "category".to_string(),
                description: "Category for organization".to_string(),
                param_type: ParamType::String,
                required: false,
                default: None,
                enum_values: None,
            })
            .with_param(ToolParameter {
                name: "tags".to_string(),
                description: "Comma-separated tags".to_string(),
                param_type: ParamType::String,
                required: false,
                default: None,
                enum_values: None,
            })
            .with_category(ToolCategory::Database)
            .with_implementation(ToolImplementation::Builtin {
                handler: "store_knowledge".to_string(),
            }),
        // Git Operations
        Tool::new("git_status", "Get the current git status")
            .with_param(ToolParameter {
                name: "path".to_string(),
                description: "Repository path".to_string(),
                param_type: ParamType::File,
                required: false,
                default: Some(".".to_string()),
                enum_values: None,
            })
            .with_category(ToolCategory::System)
            .with_implementation(ToolImplementation::Shell {
                command: "cd $path && git status --porcelain".to_string(),
            }),
        Tool::new("git_diff", "Show git diff of changes")
            .with_param(ToolParameter {
                name: "path".to_string(),
                description: "Repository or file path".to_string(),
                param_type: ParamType::File,
                required: false,
                default: Some(".".to_string()),
                enum_values: None,
            })
            .with_category(ToolCategory::System)
            .with_implementation(ToolImplementation::Shell {
                command: "git diff $path".to_string(),
            }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let factory = ToolFactory::new("/tmp/tools");
        let tool = factory
            .create_from_description(
                "test_tool",
                "A tool that reads a file",
                ToolCategory::FileSystem,
            )
            .unwrap();

        assert_eq!(tool.name, "test_tool");
        // Should infer 'path' parameter from "reads a file"
        assert!(tool.parameters.iter().any(|p| p.name == "path"));
    }

    #[test]
    fn test_shell_tool_creation() {
        let factory = ToolFactory::new("/tmp/tools");
        let tool =
            factory.create_shell_tool("list_files", "List files in current directory", "ls -la");

        assert!(matches!(
            tool.implementation,
            ToolImplementation::Shell { .. }
        ));
    }

    #[test]
    fn test_research_tools() {
        let tools = create_research_tools();
        assert!(tools.len() >= 5);

        let has_git = tools.iter().any(|t| t.name == "git_status");
        assert!(has_git);
    }
}
