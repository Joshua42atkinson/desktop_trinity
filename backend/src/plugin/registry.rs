//! Plugin Registry - Manages plugin lifecycle and discovery
//!
//! Handles loading, unloading, and routing messages to plugins.

use super::{AgentMessage, AgentResponse, PluginContext, PluginMetadata, TrinityPlugin};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Plugin registry for managing all loaded plugins
pub struct PluginRegistry {
    /// Loaded plugins by ID
    plugins: HashMap<String, Box<dyn TrinityPlugin>>,
    /// Plugin metadata
    metadata: HashMap<String, PluginMetadata>,
    /// Shared context
    context: Arc<RwLock<PluginContext>>,
    /// Plugin load order (for proper unloading)
    load_order: Vec<String>,
}

impl PluginRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            metadata: HashMap::new(),
            context: Arc::new(RwLock::new(PluginContext::default())),
            load_order: Vec::new(),
        }
    }
    
    /// Register and load a plugin
    pub fn register<P: TrinityPlugin + 'static>(&mut self, mut plugin: P) -> Result<()> {
        let id = plugin.id().to_string();
        
        if self.plugins.contains_key(&id) {
            anyhow::bail!("Plugin already registered: {}", id);
        }
        
        // Create metadata
        let meta = PluginMetadata {
            id: id.clone(),
            name: plugin.name().to_string(),
            version: plugin.version().to_string(),
            author: None,
            description: None,
            dependencies: Vec::new(),
        };
        
        // Load the plugin
        {
            let mut ctx = self.context.write().unwrap();
            plugin.on_load(&mut ctx)
                .with_context(|| format!("Failed to load plugin: {}", id))?;
        }
        
        log::info!("Loaded plugin: {} v{}", plugin.name(), plugin.version());
        
        self.plugins.insert(id.clone(), Box::new(plugin));
        self.metadata.insert(id.clone(), meta);
        self.load_order.push(id);
        
        Ok(())
    }
    
    /// Unload a plugin by ID
    pub fn unload(&mut self, id: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(id) {
            plugin.on_unload()?;
            self.metadata.remove(id);
            self.load_order.retain(|x| x != id);
            log::info!("Unloaded plugin: {}", id);
        }
        Ok(())
    }
    
    /// Unload all plugins in reverse order
    pub fn unload_all(&mut self) -> Result<()> {
        let order: Vec<String> = self.load_order.iter().rev().cloned().collect();
        for id in order {
            self.unload(&id)?;
        }
        Ok(())
    }
    
    /// Get a plugin by ID
    pub fn get(&self, id: &str) -> Option<&dyn TrinityPlugin> {
        self.plugins.get(id).map(|p| p.as_ref())
    }
    
    /// List all loaded plugins
    pub fn list(&self) -> Vec<&PluginMetadata> {
        self.metadata.values().collect()
    }
    
    /// Route a message to all plugins, return first response
    pub fn route_message(&self, msg: &AgentMessage) -> Option<AgentResponse> {
        let ctx = self.context.read().unwrap();
        
        for id in &self.load_order {
            if let Some(plugin) = self.plugins.get(id) {
                if let Some(response) = plugin.on_message(msg, &ctx) {
                    log::debug!("Message handled by plugin: {}", id);
                    return Some(response);
                }
            }
        }
        
        None
    }
    
    /// Route a message to all plugins, collect all responses
    pub fn broadcast_message(&self, msg: &AgentMessage) -> Vec<(String, AgentResponse)> {
        let ctx = self.context.read().unwrap();
        let mut responses = Vec::new();
        
        for id in &self.load_order {
            if let Some(plugin) = self.plugins.get(id) {
                if let Some(response) = plugin.on_message(msg, &ctx) {
                    responses.push((id.clone(), response));
                }
            }
        }
        
        responses
    }
    
    /// Get all available tools from all plugins
    pub fn all_tools(&self) -> Vec<(String, super::ToolDefinition)> {
        let mut tools = Vec::new();
        
        for (id, plugin) in &self.plugins {
            for tool in plugin.tools() {
                tools.push((id.clone(), tool));
            }
        }
        
        tools
    }
    
    /// Execute a tool by name
    pub fn execute_tool(
        &self,
        plugin_id: &str,
        tool_name: &str,
        args: &HashMap<String, String>,
    ) -> Result<String> {
        let plugin = self.plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;
        
        plugin.execute_tool(tool_name, args)
    }
    
    /// Get shared context (for testing/debugging)
    pub fn context(&self) -> Arc<RwLock<PluginContext>> {
        Arc::clone(&self.context)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginRegistry {
    fn drop(&mut self) {
        if let Err(e) = self.unload_all() {
            log::error!("Error unloading plugins: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct EchoPlugin;
    
    impl TrinityPlugin for EchoPlugin {
        fn id(&self) -> &str { "echo" }
        fn name(&self) -> &str { "Echo" }
        fn version(&self) -> &str { "1.0.0" }
        
        fn on_load(&mut self, _ctx: &mut PluginContext) -> Result<()> { Ok(()) }
        fn on_unload(&mut self) -> Result<()> { Ok(()) }
        
        fn on_message(&self, msg: &AgentMessage, _ctx: &PluginContext) -> Option<AgentResponse> {
            Some(AgentResponse::text(format!("Echo: {}", msg.content)))
        }
    }
    
    #[test]
    fn test_registry_lifecycle() {
        let mut registry = PluginRegistry::new();
        
        // Register
        registry.register(EchoPlugin).unwrap();
        assert_eq!(registry.list().len(), 1);
        
        // Route message
        let msg = AgentMessage::new("test", "Hello");
        let response = registry.route_message(&msg).unwrap();
        assert!(response.content.contains("Echo: Hello"));
        
        // Unload
        registry.unload("echo").unwrap();
        assert_eq!(registry.list().len(), 0);
    }
    
    #[test]
    fn test_duplicate_registration() {
        let mut registry = PluginRegistry::new();
        
        registry.register(EchoPlugin).unwrap();
        assert!(registry.register(EchoPlugin).is_err());
    }
}
