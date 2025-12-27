#![allow(unused)]
//! GGUF Agent - Integrates GGUF model with tool calling
//!
//! This is the main integration point between the native GGUF model
//! and the tool system, enabling GPT-OSS 120B to call tools.

use crate::ai::llm::gguf_loader::ModelType;
use crate::ai::llm::{GgufConfig, GgufGenerateConfig, GgufModel};
use crate::ai::model_manager::ModelManager;
use crate::ai::tool_executor::{ExecutorConfig, ToolExecutor};
use crate::ai::tool_factory::create_research_tools;
use crate::ai::tools::{
    create_builtin_tools, parse_tool_calls, Tool, ToolCall, ToolRegistry, ToolResult,
};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Configuration for the GGUF Agent
#[derive(Clone, Debug)]
pub struct GgufAgentConfig {
    /// Type of model to use (Smart or Fast)
    pub model_type: ModelType,
    /// Tool executor configuration
    pub executor_config: ExecutorConfig,
    /// System prompt for the agent
    pub system_prompt: String,
    /// Maximum tool calls per turn
    pub max_tool_calls: usize,
    /// Whether to automatically execute tools
    pub auto_execute_tools: bool,
}

impl Default for GgufAgentConfig {
    fn default() -> Self {
        Self {
            model_type: ModelType::Smart,
            executor_config: ExecutorConfig::default(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            max_tool_calls: 5,
            auto_execute_tools: true,
        }
    }
}

/// Main GGUF-powered agent with tool calling
pub struct GgufAgent {
    config: GgufAgentConfig,
    model: Option<Arc<Mutex<GgufModel>>>,
    model_manager: Option<ModelManager>,
    tool_registry: ToolRegistry,
    executor: ToolExecutor,
    /// Conversation history
    history: Vec<Message>,
}

/// A message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl GgufAgent {
    /// Create a new agent without loading the model
    pub fn new(config: GgufAgentConfig) -> Self {
        let mut tool_registry = ToolRegistry::new();

        // Register built-in tools
        for tool in create_builtin_tools() {
            tool_registry.register(tool);
        }

        // Register R&D tools
        for tool in create_research_tools() {
            tool_registry.register(tool);
        }

        Self {
            config: config.clone(),
            model: None,
            model_manager: Some(ModelManager::default()),
            tool_registry,
            executor: ToolExecutor::with_config(config.executor_config),
            history: vec![Message {
                role: MessageRole::System,
                content: config.system_prompt,
                tool_calls: None,
                tool_results: None,
            }],
        }
    }

    /// Load the GGUF model using the manager
    pub fn load_model(&mut self) -> Result<()> {
        log::info!(
            "Loading GGUF model for agent ({:?})",
            self.config.model_type
        );
        if let Some(manager) = &self.model_manager {
            let model = manager.get_model(self.config.model_type)?;
            self.model = Some(model);
        }
        Ok(())
    }

    /// Check if model is loaded
    pub fn is_ready(&self) -> bool {
        self.model.is_some()
    }

    /// Register a custom tool
    pub fn register_tool(&mut self, tool: Tool) {
        self.tool_registry.register(tool);
    }

    /// Get all available tools
    pub fn list_tools(&self) -> Vec<&Tool> {
        self.tool_registry.list()
    }

    /// Chat with the agent (main entry point)
    pub async fn chat(&mut self, user_message: &str) -> Result<String> {
        // Add user message to history
        self.history.push(Message {
            role: MessageRole::User,
            content: user_message.to_string(),
            tool_calls: None,
            tool_results: None,
        });

        // Build the prompt
        let prompt = self.build_prompt();

        // Generate response
        let model_arc = self
            .model
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Model not loaded"))?;

        let gen_config = GgufGenerateConfig::default();
        let response = {
            let mut model = model_arc.lock().unwrap();
            model.generate(&prompt, &gen_config)?
        };

        // Check for tool calls
        let tool_calls = parse_tool_calls(&response);

        if !tool_calls.is_empty() && self.config.auto_execute_tools {
            // Execute tools and continue the conversation
            self.handle_tool_calls(&tool_calls, &response).await
        } else {
            // No tool calls, just return the response
            self.history.push(Message {
                role: MessageRole::Assistant,
                content: response.clone(),
                tool_calls: None,
                tool_results: None,
            });

            Ok(response)
        }
    }

    /// Handle tool calls in the response
    async fn handle_tool_calls(
        &mut self,
        tool_calls: &[ToolCall],
        original_response: &str,
    ) -> Result<String> {
        let mut results = Vec::new();
        let mut calls_executed = 0;

        for call in tool_calls {
            if calls_executed >= self.config.max_tool_calls {
                log::warn!("Max tool calls reached, skipping remaining");
                break;
            }

            if let Some(tool) = self.tool_registry.get(&call.name) {
                log::info!("Executing tool: {}", call.name);
                let result = self.executor.execute(tool, call).await;
                results.push(result);
                calls_executed += 1;
            } else {
                results.push(ToolResult::error(
                    &call.id,
                    format!("Tool not found: {}", call.name),
                ));
            }
        }

        // Add assistant message with tool calls
        self.history.push(Message {
            role: MessageRole::Assistant,
            content: original_response.to_string(),
            tool_calls: Some(tool_calls.to_vec()),
            tool_results: None,
        });

        // Add tool results
        self.history.push(Message {
            role: MessageRole::Tool,
            content: String::new(),
            tool_calls: None,
            tool_results: Some(results.clone()),
        });

        // Generate follow-up response with tool results
        let follow_up_prompt = self.build_prompt();

        let model_arc = self
            .model
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Model not loaded"))?;

        let gen_config = GgufGenerateConfig::default();
        let final_response = {
            let mut model = model_arc.lock().unwrap();
            model.generate(&follow_up_prompt, &gen_config)?
        };

        // Check for more tool calls (recursive, limited by max_tool_calls)
        let new_tool_calls = parse_tool_calls(&final_response);

        if !new_tool_calls.is_empty() && calls_executed < self.config.max_tool_calls {
            // Recursive tool execution
            Box::pin(self.handle_tool_calls(&new_tool_calls, &final_response)).await
        } else {
            self.history.push(Message {
                role: MessageRole::Assistant,
                content: final_response.clone(),
                tool_calls: None,
                tool_results: None,
            });

            Ok(final_response)
        }
    }

    /// Build the full prompt from history
    fn build_prompt(&self) -> String {
        let mut prompt = String::new();

        // Add tool definitions
        let tools_json =
            serde_json::to_string_pretty(&self.tool_registry.to_openai_tools()).unwrap_or_default();

        prompt.push_str("# Available Tools\n\n");
        prompt.push_str(&tools_json);
        prompt.push_str("\n\n# Conversation\n\n");

        // Add conversation history
        for msg in &self.history {
            match msg.role {
                MessageRole::System => {
                    prompt.push_str(&format!("System: {}\n\n", msg.content));
                }
                MessageRole::User => {
                    prompt.push_str(&format!("User: {}\n\n", msg.content));
                }
                MessageRole::Assistant => {
                    prompt.push_str(&format!("Assistant: {}\n\n", msg.content));
                }
                MessageRole::Tool => {
                    if let Some(ref results) = msg.tool_results {
                        for result in results {
                            prompt.push_str(&format!(
                                "Tool Result ({}): {}\n\n",
                                if result.success { "success" } else { "error" },
                                if result.success {
                                    &result.output
                                } else {
                                    result.error.as_deref().unwrap_or("Unknown error")
                                }
                            ));
                        }
                    }
                }
            }
        }

        prompt.push_str("Assistant: ");

        prompt
    }

    /// Clear conversation history (keep system prompt)
    pub fn clear_history(&mut self) {
        let system_msg = self.history.first().cloned();
        self.history.clear();
        if let Some(msg) = system_msg {
            self.history.push(msg);
        }
    }

    /// Get conversation history
    pub fn get_history(&self) -> &[Message] {
        &self.history
    }

    /// Get tool execution log
    pub fn get_execution_log(&self) -> &[crate::ai::tool_executor::ExecutionLogEntry] {
        self.executor.get_log()
    }
}

/// Default system prompt for the agent
const DEFAULT_SYSTEM_PROMPT: &str = r#"You are Trinity, an advanced AI agent running locally on AMD Strix Halo hardware.

You have access to tools for file operations, code execution, research, and more.
When you need to perform an action, use the appropriate tool by responding with a tool call.

To call a tool, respond with:
<tool_call>{"name": "tool_name", "arguments": {"arg1": "value1"}}</tool_call>

Guidelines:
1. Use tools when appropriate to accomplish tasks
2. Always explain what you're doing before calling tools
3. After receiving tool results, interpret them for the user
4. If a tool fails, try an alternative approach
5. Be helpful, accurate, and efficient

You are sovereign - running entirely locally with no cloud dependencies."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let config = GgufAgentConfig::default();
        let agent = GgufAgent::new(config);

        // Should have built-in tools registered
        assert!(!agent.list_tools().is_empty());
    }

    #[test]
    fn test_prompt_building() {
        let config = GgufAgentConfig::default();
        let mut agent = GgufAgent::new(config);

        agent.history.push(Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            tool_calls: None,
            tool_results: None,
        });

        let prompt = agent.build_prompt();
        assert!(prompt.contains("Hello"));
        assert!(prompt.contains("Available Tools"));
    }
}
