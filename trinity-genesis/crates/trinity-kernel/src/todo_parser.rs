// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! # TODO Parser — Self-Improvement Task Extraction
//!
//! ## Philosophy
//! "Trinity reads its own conscience. The TODO file is the moral compass;
//!  the parser transforms intent into action."
//!
//! This module parses markdown TODO/roadmap files into structured
//! `AutonomousTask` items that can be enqueued for self-execution.

use crate::runtime::{AutonomousTask, TaskPriority, TaskType};
use std::path::Path;

/// A parsed TODO item from markdown
#[derive(Debug, Clone)]
pub struct TodoItem {
    /// The task title/description
    pub title: String,
    /// Optional file path hint (extracted from backticks)
    pub file_hint: Option<String>,
    /// Priority level based on markdown markers
    pub priority: TaskPriority,
    /// Whether this item is already complete
    pub complete: bool,
    /// Nesting depth (for subtasks)
    pub depth: usize,
}

/// Completion status from markdown checkbox
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxStatus {
    /// `[ ]` - Not started
    Unchecked,
    /// `[/]` - In progress
    InProgress,
    /// `[x]` - Complete
    Checked,
}

impl TodoItem {
    /// Parse a markdown file into TODO items
    pub fn parse_markdown(content: &str) -> Vec<TodoItem> {
        let mut items = Vec::new();

        for line in content.lines() {
            // Skip empty lines and headers
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Look for checkbox patterns: - [ ], - [x], - [/]
            if let Some(item) = Self::parse_checkbox_line(line) {
                items.push(item);
            }
        }

        items
    }

    /// Parse a single checkbox line
    fn parse_checkbox_line(line: &str) -> Option<TodoItem> {
        // Calculate depth from leading whitespace
        let depth = line.len() - line.trim_start().len();
        let depth = depth / 2; // Assume 2-space indent

        let trimmed = line.trim();

        // Must start with - [ or * [
        if !trimmed.starts_with("- [") && !trimmed.starts_with("* [") {
            return None;
        }

        // Extract checkbox status
        let (status, rest) = if trimmed.contains("- [ ]") || trimmed.contains("* [ ]") {
            (CheckboxStatus::Unchecked, trimmed.split("] ").nth(1)?)
        } else if trimmed.contains("- [x]") || trimmed.contains("* [x]") {
            (CheckboxStatus::Checked, trimmed.split("] ").nth(1)?)
        } else if trimmed.contains("- [/]") || trimmed.contains("* [/]") {
            (CheckboxStatus::InProgress, trimmed.split("] ").nth(1)?)
        } else {
            return None;
        };

        // Extract file hint from backticks
        let file_hint = Self::extract_file_hint(rest);

        // Determine priority based on keywords and status
        let priority = Self::determine_priority(rest, status);

        Some(TodoItem {
            title: rest.to_string(),
            file_hint,
            priority,
            complete: status == CheckboxStatus::Checked,
            depth,
        })
    }

    /// Extract file path from backticks like `path/to/file.rs`
    fn extract_file_hint(text: &str) -> Option<String> {
        // Look for patterns like `file.rs` or `path/to/file.rs`
        let mut in_backtick = false;
        let mut current = String::new();

        for ch in text.chars() {
            if ch == '`' {
                if in_backtick {
                    // Check if this looks like a file path
                    if current.contains('.') || current.contains('/') {
                        return Some(current);
                    }
                    current.clear();
                }
                in_backtick = !in_backtick;
            } else if in_backtick {
                current.push(ch);
            }
        }

        None
    }

    /// Determine priority from keywords
    fn determine_priority(text: &str, status: CheckboxStatus) -> TaskPriority {
        let lower = text.to_lowercase();

        // In-progress items get higher priority
        if status == CheckboxStatus::InProgress {
            return TaskPriority::High;
        }

        // Critical keywords
        if lower.contains("critical") || lower.contains("blocker") || lower.contains("p0") {
            return TaskPriority::Critical;
        }

        // High priority keywords
        if lower.contains("wire")
            || lower.contains("fix")
            || lower.contains("important")
            || lower.contains("p1")
        {
            return TaskPriority::High;
        }

        // Low priority keywords
        if lower.contains("polish") || lower.contains("later") || lower.contains("p2") {
            return TaskPriority::Low;
        }

        TaskPriority::Normal
    }

    /// Convert to an AutonomousTask
    pub fn to_autonomous_task(&self) -> AutonomousTask {
        // Determine task type from content analysis
        let task_type = self.infer_task_type();

        AutonomousTask::new(&self.title, task_type)
            .with_priority(self.priority)
            .with_description(format!("Auto-generated from TODO: {}", self.title))
    }

    /// Infer TaskType from the TODO item content
    fn infer_task_type(&self) -> TaskType {
        let lower = self.title.to_lowercase();

        // Code generation tasks
        if lower.contains("create") || lower.contains("implement") || lower.contains("add") {
            if let Some(ref file) = self.file_hint {
                // Determine language from file extension
                let lang = Path::new(file)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("rust")
                    .to_string();

                return TaskType::GenerateCode {
                    prompt: self.title.clone(),
                    language: lang,
                    output_path: Some(file.clone()),
                };
            }
        }

        // Edit file tasks
        if lower.contains("modify") || lower.contains("update") || lower.contains("wire") {
            if let Some(ref file) = self.file_hint {
                return TaskType::EditFile {
                    path: file.clone(),
                    instructions: self.title.clone(),
                };
            }
        }

        // Research tasks
        if lower.contains("research") || lower.contains("investigate") || lower.contains("analyze")
        {
            return TaskType::Research {
                topic: self.title.clone(),
                depth: Some("thorough".to_string()),
            };
        }

        // Default to Think (pure reasoning)
        TaskType::Think {
            prompt: format!(
                "You are a Rust expert. Complete this TODO item:\n\n{}\n\nProvide the implementation.",
                self.title
            ),
        }
    }

    /// Check if this item needs work
    pub fn needs_work(&self) -> bool {
        !self.complete
    }
}

/// Parse a TODO file and return actionable tasks
pub fn parse_todo_file(path: &str) -> anyhow::Result<Vec<TodoItem>> {
    let content = std::fs::read_to_string(path)?;
    Ok(TodoItem::parse_markdown(&content))
}

/// Scan a workspace directory for TODO comments in code files
pub fn scan_workspace_for_todos(workspace_root: &Path) -> anyhow::Result<Vec<TodoItem>> {
    let mut items = Vec::new();

    // Recursive walker
    let mut stack = vec![workspace_root.to_path_buf()];

    while let Some(path) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Skip hidden and build dirs
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }

                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    // Check extension
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        match ext {
                            "rs" | "sh" | "md" | "toml" | "js" | "ts" | "py" | "html" | "css" => {
                                if let Ok(mut file_items) =
                                    scan_file_for_todos(&path, workspace_root)
                                {
                                    items.append(&mut file_items);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(items)
}

/// Scan a single file for TODO patterns
fn scan_file_for_todos(path: &Path, root: &Path) -> anyhow::Result<Vec<TodoItem>> {
    let content = std::fs::read_to_string(path)?;
    let rel_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    let mut items = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if let Some(todo_content) = parse_todo_comment(line) {
            // Determine priority
            let priority = if todo_content.to_lowercase().contains("critical")
                || todo_content.to_lowercase().contains("fixme")
            {
                TaskPriority::Critical
            } else if todo_content.to_lowercase().contains("important") {
                TaskPriority::High
            } else {
                TaskPriority::Normal
            };

            items.push(TodoItem {
                title: format!("{} (L{})", todo_content, line_num + 1),
                file_hint: Some(rel_path.clone()),
                priority,
                complete: false,
                depth: 0,
            });
        }
    }

    Ok(items)
}

/// Extract TODO content from a line comment
fn parse_todo_comment(line: &str) -> Option<String> {
    let line = line.trim();

    // Support various styles:
    // // TODO: message
    // // FIXME: message
    // /// TODO: message
    // # TODO: message
    // <!-- TODO: message -->

    let markers = ["//", "///", "#", "<!--"];
    let keywords = ["TODO", "FIXME", "XXX", "HACK"];

    for marker in markers {
        if line.starts_with(marker) {
            let content_start = line[marker.len()..].trim();
            for keyword in keywords {
                if content_start.starts_with(keyword) {
                    let msg = content_start[keyword.len()..]
                        .trim_start_matches(':')
                        .trim();
                    // Clear trailing comment closers like -->
                    let msg = msg.trim_end_matches("-->").trim();
                    if !msg.is_empty() {
                        return Some(msg.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Get only incomplete items from a TODO file
pub fn get_pending_items(path: &str) -> anyhow::Result<Vec<TodoItem>> {
    let items = parse_todo_file(path)?;
    Ok(items.into_iter().filter(|i| i.needs_work()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_checkbox() {
        let content = r#"
# Test TODO

- [ ] Implement `foo.rs` module
- [x] Already done
- [/] In progress task
  - [ ] Nested subtask
"#;

        let items = TodoItem::parse_markdown(content);
        assert_eq!(items.len(), 4);

        // First item
        assert_eq!(items[0].title, "Implement `foo.rs` module");
        assert_eq!(items[0].file_hint, Some("foo.rs".to_string()));
        assert!(!items[0].complete);

        // Second item (complete)
        assert!(items[1].complete);

        // Third item (in progress = high priority)
        assert_eq!(items[2].priority, TaskPriority::High);

        // Fourth item (nested)
        assert_eq!(items[3].depth, 1);
    }

    #[test]
    fn test_task_type_inference() {
        let item = TodoItem {
            title: "Wire `coder.rs` to Brain RPC".to_string(),
            file_hint: Some("coder.rs".to_string()),
            priority: TaskPriority::Normal,
            complete: false,
            depth: 0,
        };

        let task = item.to_autonomous_task();
        matches!(task.task_type, TaskType::EditFile { .. });
    }
}
