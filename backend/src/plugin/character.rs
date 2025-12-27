//! Character System - YAML-based agent personality definitions
//!
//! Inspired by Eliza OS character files.
//! Defines agent personality, knowledge, and behavior patterns.

use crate::agent::AgentRole;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Complete character definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDef {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Agent role in the swarm
    #[serde(default)]
    pub role: CharacterRole,
    /// Personality description (used in system prompt)
    pub personality: String,
    /// Background/bio for the character
    #[serde(default)]
    pub bio: Vec<String>,
    /// Knowledge domains
    #[serde(default)]
    pub knowledge: Vec<String>,
    /// Tools this character can use
    #[serde(default)]
    pub tools: Vec<String>,
    /// Example conversations for few-shot learning
    #[serde(default)]
    pub examples: Vec<ConversationExample>,
    /// Style guidelines
    #[serde(default)]
    pub style: StyleConfig,
    /// Model configuration overrides
    #[serde(default)]
    pub model: ModelOverrides,
    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Character role mapping
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CharacterRole {
    Router,
    #[default]
    Core,
    Research,
    Developer,
    Writer,
    #[serde(other)]
    Custom,
}

impl From<CharacterRole> for AgentRole {
    fn from(role: CharacterRole) -> Self {
        match role {
            CharacterRole::Router => AgentRole::Router,
            CharacterRole::Core => AgentRole::Core,
            CharacterRole::Research => AgentRole::Research,
            CharacterRole::Developer => AgentRole::Developer,
            CharacterRole::Writer => AgentRole::Writer,
            CharacterRole::Custom => AgentRole::Custom("custom".to_string()),
        }
    }
}

/// Example conversation for few-shot prompting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationExample {
    pub user: String,
    pub assistant: String,
}

/// Style configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleConfig {
    /// Tone (formal, casual, academic, playful)
    #[serde(default = "default_tone")]
    pub tone: String,
    /// Communication style guidelines
    #[serde(default)]
    pub guidelines: Vec<String>,
    /// Words/phrases to avoid
    #[serde(default)]
    pub avoid: Vec<String>,
    /// Preferred vocabulary
    #[serde(default)]
    pub prefer: Vec<String>,
}

fn default_tone() -> String {
    "professional".to_string()
}

/// Model configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOverrides {
    /// Temperature override
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Max tokens override
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Top-p override
    #[serde(default = "default_top_p")]
    pub top_p: f32,
}

impl Default for ModelOverrides {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            top_p: default_top_p(),
        }
    }
}

fn default_temperature() -> f32 {
    0.7
}
fn default_max_tokens() -> u32 {
    1024
}
fn default_top_p() -> f32 {
    0.9
}

impl CharacterDef {
    /// Load character from YAML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read character file: {:?}", path))?;

        Self::from_yaml(&content)
    }

    /// Parse character from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse character YAML")
    }

    /// Generate system prompt for this character
    pub fn to_system_prompt(&self) -> String {
        let mut prompt = String::new();

        // Identity
        prompt.push_str(&format!("You are {}, ", self.name));
        prompt.push_str(&self.personality);
        prompt.push('\n');

        // Bio
        if !self.bio.is_empty() {
            prompt.push_str("\nBackground:\n");
            for line in &self.bio {
                prompt.push_str(&format!("- {}\n", line));
            }
        }

        // Knowledge
        if !self.knowledge.is_empty() {
            prompt.push_str("\nAreas of expertise:\n");
            for k in &self.knowledge {
                prompt.push_str(&format!("- {}\n", k));
            }
        }

        // Style
        prompt.push_str(&format!("\nCommunication style: {}\n", self.style.tone));
        for guideline in &self.style.guidelines {
            prompt.push_str(&format!("- {}\n", guideline));
        }

        if !self.style.avoid.is_empty() {
            prompt.push_str("\nAvoid:\n");
            for word in &self.style.avoid {
                prompt.push_str(&format!("- {}\n", word));
            }
        }

        prompt
    }

    /// Generate few-shot examples for this character
    pub fn to_few_shot_examples(&self) -> Vec<(String, String)> {
        self.examples
            .iter()
            .map(|e| (e.user.clone(), e.assistant.clone()))
            .collect()
    }
}

/// Load all characters from a directory
pub fn load_characters_from_dir(dir: impl AsRef<Path>) -> Result<Vec<CharacterDef>> {
    let dir = dir.as_ref();
    let mut characters = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            match CharacterDef::from_file(&path) {
                Ok(char) => {
                    log::info!("Loaded character: {} from {:?}", char.name, path);
                    characters.push(char);
                }
                Err(e) => {
                    log::warn!("Failed to load character from {:?}: {}", path, e);
                }
            }
        }
    }

    Ok(characters)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_CHARACTER: &str = r#"
id: trinity_agent
name: Trinity Agent
role: research
personality: >
  A versatile research agent for the Trinity AI OS.
  Provides comprehensive answers with source citations.
bio:
  - Core research agent in the Trinity swarm
  - Specializes in information retrieval and synthesis
knowledge:
  - Research methodologies
  - Information synthesis
  - Source verification
tools:
  - web_search
  - document_retrieval
examples:
  - user: What is cognitive load theory?
    assistant: Cognitive load theory describes how our working memory has limited capacity. Let me explain the three types and provide sources.
style:
  tone: professional
  guidelines:
    - Provide comprehensive answers
    - Cite sources when available
  avoid:
    - Vague responses
    - Unsourced claims
model:
  temperature: 0.6
"#;

    #[test]
    fn test_parse_character() {
        let char = CharacterDef::from_yaml(EXAMPLE_CHARACTER).unwrap();

        assert_eq!(char.id, "trinity_agent");
        assert_eq!(char.name, "Trinity Agent");
        assert_eq!(char.knowledge.len(), 3);
        assert_eq!(char.examples.len(), 1);
    }

    #[test]
    fn test_system_prompt() {
        let char = CharacterDef::from_yaml(EXAMPLE_CHARACTER).unwrap();
        let prompt = char.to_system_prompt();

        assert!(prompt.contains("Trinity Agent"));
        assert!(prompt.contains("research"));
        assert!(prompt.contains("professional"));
    }

    #[test]
    fn test_role_conversion() {
        let char = CharacterDef::from_yaml(EXAMPLE_CHARACTER).unwrap();
        let role: AgentRole = char.role.into();

        assert!(matches!(role, AgentRole::Research));
    }
}
