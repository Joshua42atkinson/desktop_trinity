#![allow(unused)]
//! Trinity Plugin System
//!
//! Extensible plugin architecture inspired by Eliza OS.
//! Plugins can add new capabilities, connectors, and tools.

pub mod character;
pub mod registry;

use crate::agent::{AgentRole, AgentState};
use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Message passed to/from agents
#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub id: uuid::Uuid,
    pub source: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AgentMessage {
    pub fn new(source: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            source: source.into(),
            content: content.into(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Response from an agent or plugin
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub actions: Vec<PluginAction>,
    pub metadata: HashMap<String, String>,
}

impl AgentResponse {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            actions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_action(mut self, action: PluginAction) -> Self {
        self.actions.push(action);
        self
    }
}

/// Actions a plugin can request
#[derive(Debug, Clone)]
pub enum PluginAction {
    /// Send a message to another agent
    SendMessage { target: String, message: String },
    /// Execute a tool
    ExecuteTool {
        name: String,
        args: HashMap<String, String>,
    },
    /// Store data in memory
    StoreMemory { key: String, value: String },
    /// Retrieve data from memory
    RetrieveMemory { key: String },
    /// Schedule a task
    ScheduleTask { delay_ms: u64, task: String },
}

/// Context provided to plugins during lifecycle
#[derive(Default)]
pub struct PluginContext {
    /// Plugin configuration
    pub config: HashMap<String, String>,
    /// Shared state (type-erased)
    pub state: HashMap<String, Arc<dyn Any + Send + Sync>>,
    /// Available tools
    pub tools: Vec<String>,
}

impl PluginContext {
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    pub fn set_state<T: Any + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.state.insert(key.into(), Arc::new(value));
    }

    pub fn get_state<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.state.get(key)?.downcast_ref::<T>()
    }
}

/// Core plugin trait - implement this to extend Trinity
pub trait TrinityPlugin: Send + Sync {
    /// Unique plugin identifier
    fn id(&self) -> &str;

    /// Human-readable plugin name
    fn name(&self) -> &str;

    /// Plugin version (semver)
    fn version(&self) -> &str;

    /// Called when plugin is loaded
    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()>;

    /// Called when plugin is unloaded
    fn on_unload(&mut self) -> Result<()>;

    /// Handle an incoming message
    fn on_message(&self, msg: &AgentMessage, ctx: &PluginContext) -> Option<AgentResponse>;

    /// List tools this plugin provides
    fn tools(&self) -> Vec<ToolDefinition> {
        Vec::new()
    }

    /// Execute a tool by name
    fn execute_tool(&self, _name: &str, _args: &HashMap<String, String>) -> Result<String> {
        anyhow::bail!("Tool not found")
    }
}

/// Definition of a tool provided by a plugin
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

/// Parameter for a tool
#[derive(Debug, Clone)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
}

/// Type of a tool parameter
#[derive(Debug, Clone)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}

/// Plugin metadata for discovery
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    impl TrinityPlugin for TestPlugin {
        fn id(&self) -> &str {
            "test"
        }
        fn name(&self) -> &str {
            "Test Plugin"
        }
        fn version(&self) -> &str {
            "0.1.0"
        }

        fn on_load(&mut self, _ctx: &mut PluginContext) -> Result<()> {
            Ok(())
        }

        fn on_unload(&mut self) -> Result<()> {
            Ok(())
        }

        fn on_message(&self, msg: &AgentMessage, _ctx: &PluginContext) -> Option<AgentResponse> {
            if msg.content.contains("test") {
                Some(AgentResponse::text("Test response"))
            } else {
                None
            }
        }
    }

    #[test]
    fn test_plugin_trait() {
        let plugin = TestPlugin;
        assert_eq!(plugin.id(), "test");
        assert_eq!(plugin.name(), "Test Plugin");
    }

    #[test]
    fn test_message_creation() {
        let msg = AgentMessage::new("user", "Hello").with_metadata("channel", "discord");

        assert_eq!(msg.source, "user");
        assert_eq!(msg.metadata.get("channel"), Some(&"discord".to_string()));
    }
}
