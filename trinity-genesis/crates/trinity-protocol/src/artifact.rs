// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! # Artifact Types — Structured Agent Output
//!
//! ## Philosophy
//! "Agents don't just chat; they produce Artifacts—structured, verifiable outputs
//!  like plans, code diffs, or previews."
//!
//! Artifacts transform raw LLM output into interactive, type-safe UI elements.
//! Each artifact type has a defined schema and rendering behavior.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Agent execution mode (Planning vs Fast)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AgentMode {
    /// Reactive, quick answers (< 3 seconds, no plan visible)
    #[default]
    Fast,
    /// Deep reasoning with visible planning steps
    Planning,
    /// Background autonomous work
    Autonomous,
}

impl AgentMode {
    /// Determine mode from query complexity heuristics
    pub fn classify_query(prompt: &str) -> Self {
        let prompt_lower = prompt.to_lowercase();
        let word_count = prompt.split_whitespace().count();
        let has_task_keywords = prompt_lower.contains("create")
            || prompt_lower.contains("implement")
            || prompt_lower.contains("build")
            || prompt_lower.contains("design")
            || prompt_lower.contains("refactor")
            || prompt_lower.contains("fix");
        let has_complex_markers = prompt_lower.contains("step by step")
            || prompt_lower.contains("plan")
            || prompt_lower.contains("how would you")
            || prompt_lower.contains("break down");

        if has_complex_markers || (has_task_keywords && word_count > 20) {
            Self::Planning
        } else {
            Self::Fast
        }
    }
}

/// Structured artifact types for rich UI rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Artifact {
    /// Plain text (streaming chat messages)
    Text { content: String, streaming: bool },

    /// Structured code with language and metadata
    Code {
        language: String,
        content: String,
        file_path: Option<String>,
        /// Whether this code can be executed in sandbox
        executable: bool,
        /// Line highlights (for diffs)
        highlights: Vec<LineHighlight>,
    },

    /// Step-by-step breakdown (expandable in UI)
    Steps {
        title: String,
        items: Vec<StepItem>,
        current_step: usize,
    },

    /// Task plan with checkable items
    Plan { title: String, tasks: Vec<PlanTask> },

    /// Terminal/command output
    Terminal {
        command: String,
        output: String,
        exit_code: Option<i32>,
    },

    /// Error with structured info
    Error {
        message: String,
        suggestion: Option<String>,
        recoverable: bool,
    },

    /// Thinking/reasoning trace (collapsible)
    Thinking { content: String, collapsed: bool },

    /// Graph visualization data (for workflow DAGs)
    Graph {
        nodes: Vec<GraphNode>,
        edges: Vec<GraphEdge>,
    },
}

/// Line highlight for code diffs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineHighlight {
    pub line: usize,
    pub kind: HighlightKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightKind {
    Added,
    Removed,
    Modified,
    Error,
}

/// A step in a step-by-step breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepItem {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StepStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// A task in a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTask {
    pub id: usize,
    pub title: String,
    pub completed: bool,
    pub subtasks: Vec<PlanTask>,
}

/// Node for graph visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub label: String,
    pub node_type: String,
    pub status: NodeStatus,
    /// Position hint for layout (x, y normalized 0-1)
    pub position: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NodeStatus {
    #[default]
    Idle,
    Running,
    Completed,
    Failed,
}

/// Edge for graph visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub label: Option<String>,
}

impl Artifact {
    /// Create a streaming text artifact
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
            streaming: false,
        }
    }

    /// Create a streaming text artifact (in progress)
    pub fn streaming_text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
            streaming: true,
        }
    }

    /// Create a code artifact
    pub fn code(language: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Code {
            language: language.into(),
            content: content.into(),
            file_path: None,
            executable: false,
            highlights: vec![],
        }
    }

    /// Create a code artifact with file path
    pub fn code_file(
        language: impl Into<String>,
        content: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self::Code {
            language: language.into(),
            content: content.into(),
            file_path: Some(path.into()),
            executable: false,
            highlights: vec![],
        }
    }

    /// Create a plan artifact
    pub fn plan(title: impl Into<String>, tasks: Vec<PlanTask>) -> Self {
        Self::Plan {
            title: title.into(),
            tasks,
        }
    }

    /// Create a steps artifact
    pub fn steps(title: impl Into<String>, items: Vec<StepItem>) -> Self {
        Self::Steps {
            title: title.into(),
            items,
            current_step: 0,
        }
    }

    /// Create an error artifact
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            suggestion: None,
            recoverable: false,
        }
    }

    /// Create a thinking artifact
    pub fn thinking(content: impl Into<String>) -> Self {
        Self::Thinking {
            content: content.into(),
            collapsed: true,
        }
    }

    /// Create a terminal output artifact
    pub fn terminal(command: impl Into<String>, output: impl Into<String>) -> Self {
        Self::Terminal {
            command: command.into(),
            output: output.into(),
            exit_code: None,
        }
    }

    /// Get a short description for logging
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Text { .. } => "text",
            Self::Code { .. } => "code",
            Self::Steps { .. } => "steps",
            Self::Plan { .. } => "plan",
            Self::Terminal { .. } => "terminal",
            Self::Error { .. } => "error",
            Self::Thinking { .. } => "thinking",
            Self::Graph { .. } => "graph",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_classification() {
        // Simple questions -> Fast mode
        assert_eq!(AgentMode::classify_query("What is Rust?"), AgentMode::Fast);
        assert_eq!(AgentMode::classify_query("Hello"), AgentMode::Fast);

        // Short task requests -> Fast mode (not enough words to trigger planning)
        assert_eq!(
            AgentMode::classify_query("Create a hello world app"),
            AgentMode::Fast
        );

        // Explicit planning markers -> Planning mode regardless of length
        assert_eq!(
            AgentMode::classify_query("Step by step explain how to build a web server"),
            AgentMode::Planning
        );
        assert_eq!(
            AgentMode::classify_query("How would you design a cache?"),
            AgentMode::Planning
        );
        assert_eq!(
            AgentMode::classify_query("Break down the task into subtasks"),
            AgentMode::Planning
        );
    }

    #[test]
    fn test_artifact_constructors() {
        let text = Artifact::text("Hello world");
        assert_eq!(text.kind_name(), "text");

        let code = Artifact::code("rust", "fn main() {}");
        assert_eq!(code.kind_name(), "code");

        let error = Artifact::error("Something went wrong");
        assert_eq!(error.kind_name(), "error");
    }
}
