//! Autonomous Runtime - Task Queue and Self-Operating Execution
//!
//! This module provides the autonomous execution capability for Trinity Genesis,
//! allowing it to process task queues, perform memory consolidation ("dream cycles"),
//! and operate continuously without human intervention.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

// ============================================================================
// Task Priority
// ============================================================================

/// Task priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}

// ============================================================================
// Task Status
// ============================================================================

/// Task status in the queue
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

// ============================================================================
// Task Types
// ============================================================================

/// Types of autonomous tasks that can be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskType {
    /// Interactive chat message (fast, GPU-routed)
    Chat { message: String },
    /// Generate code from a prompt
    GenerateCode {
        prompt: String,
        language: String,
        output_path: Option<String>,
    },
    /// Review code for issues (NPU-routed)
    ReviewCode { path: String, focus: Option<String> },
    /// Research a topic (NPU-routed)
    Research {
        topic: String,
        depth: Option<String>,
    },
    /// Edit an existing file
    EditFile { path: String, instructions: String },
    /// Run a shell command
    RunCommand {
        command: String,
        working_dir: Option<String>,
    },
    /// Consolidate memories ("dream" cycle)
    MemoryConsolidation,
    /// Scan workspace for improvements
    WorkspaceScan { path: String },
    /// Think about a topic (pure LLM)
    Think { prompt: String },
    /// Web Browse
    WebBrowse { url: String },
    /// Google Drive Operation
    GoogleDrive { operation: String, path: String },
    /// Read a file
    ReadFile { path: String },
    /// Delete a file or directory
    DeletePath { path: String, recursive: bool },
    /// Create a directory
    CreateDirectory { path: String },
    /// Move/rename a file or directory
    MovePath { from: String, to: String },
    /// Copy a file
    CopyFile { from: String, to: String },
    /// List directory contents
    ListDirectory { path: String },
    /// Generate a written document
    WriteDocument {
        topic: String,
        style: String, // Technical, BlogPost, etc.
        target_words: Option<u32>,
        output_path: Option<String>,
    },
    /// Generate an educational assessment
    GenerateAssessment {
        topic: String,
        assessment_type: String, // Quiz, Lab, Challenge
        difficulty: String,      // Beginner, Intermediate, etc.
    },
    /// Custom task with arbitrary payload
    Custom { handler: String, payload: String },
}

// ============================================================================
// Autonomous Task
// ============================================================================

/// A task in the autonomous queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousTask {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub task_type: TaskType,
    /// Which agent/persona should handle this
    pub assigned_agent: Option<String>,
    /// Maximum tokens allowed for this task
    pub token_limit: Option<u32>,
    /// Tokens consumed so far
    pub token_usage: u32,
}

impl AutonomousTask {
    /// Create a new task
    pub fn new(name: impl Into<String>, task_type: TaskType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            priority: TaskPriority::default(),
            status: TaskStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            task_type,
            assigned_agent: None,
            token_limit: None,
            token_usage: 0,
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.assigned_agent = Some(agent.into());
        self
    }

    /// Mark task as running
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(chrono::Utc::now());
    }

    /// Mark task as completed
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Mark task as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = TaskStatus::Failed(error.into());
        self.completed_at = Some(chrono::Utc::now());
    }
}

// ============================================================================
// Task Result
// ============================================================================

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub task_name: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub tokens_consumed: u32,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Runtime Configuration
// ============================================================================

/// Configuration for the autonomous runtime
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// How often to check for new tasks (milliseconds)
    pub poll_interval_ms: u64,
    /// Maximum concurrent tasks
    pub max_concurrent: usize,
    /// Enable memory consolidation cycle
    pub enable_dream_cycle: bool,
    /// Interval between dream cycles (seconds)
    pub dream_cycle_interval_secs: u64,
    /// Maximum runtime before auto-shutdown (0 = infinite)
    pub max_runtime_secs: Option<u64>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 5000, // 5 seconds
            max_concurrent: 1,      // Single-threaded for safety
            enable_dream_cycle: true,
            dream_cycle_interval_secs: 3600, // Every hour
            max_runtime_secs: None,          // Run forever
        }
    }
}

// ============================================================================
// Queue Status
// ============================================================================

/// Queue status information for API/UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub is_running: bool,
    pub uptime_secs: Option<u64>,
    pub total_tokens_consumed: u64,
}

// ============================================================================
// Autonomous Runtime
// ============================================================================

/// The autonomous runtime engine
///
/// This manages a priority queue of tasks and executes them sequentially.
/// It runs on the Brain node and reports status to the Body node via RPC.
#[derive(Clone)]
pub struct AutonomousRuntime {
    config: RuntimeConfig,
    /// Task queue (priority ordered)
    task_queue: Arc<Mutex<VecDeque<AutonomousTask>>>,
    /// Completed task results
    completed_tasks: Arc<Mutex<Vec<TaskResult>>>,
    /// Failed task count
    failed_count: Arc<Mutex<usize>>,
    /// Runtime state
    is_running: Arc<Mutex<bool>>,
    /// Start time
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl AutonomousRuntime {
    /// Create a new autonomous runtime
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            completed_tasks: Arc::new(Mutex::new(Vec::new())),
            failed_count: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Add a task to the queue (priority ordered)
    pub fn enqueue(&self, task: AutonomousTask) -> Uuid {
        let id = task.id;
        let mut queue = self.task_queue.lock().unwrap();

        // Insert based on priority (higher priority at front)
        let insert_pos = queue
            .iter()
            .position(|t| t.priority < task.priority)
            .unwrap_or(queue.len());

        queue.insert(insert_pos, task);
        tracing::info!("Task {} enqueued at position {}", id, insert_pos);

        id
    }

    /// Add a simple task by name and type
    pub fn add_task(&self, name: &str, task_type: TaskType) -> Uuid {
        let task = AutonomousTask::new(name, task_type);
        self.enqueue(task)
    }

    /// Get next task from queue
    pub fn dequeue(&self) -> Option<AutonomousTask> {
        let mut queue = self.task_queue.lock().unwrap();
        queue.pop_front()
    }

    /// Peek at the next task without removing it
    pub fn peek(&self) -> Option<AutonomousTask> {
        let queue = self.task_queue.lock().unwrap();
        queue.front().cloned()
    }

    /// Check if runtime is running
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    /// Start the runtime (mark as running)
    pub fn start(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = true;
        let mut start = self.start_time.lock().unwrap();
        *start = Some(Instant::now());
        tracing::info!("Autonomous runtime started");
    }

    /// Stop the runtime
    pub fn stop(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = false;
        tracing::info!("Autonomous runtime stopping...");
    }

    /// Get queue status
    pub fn status(&self) -> QueueStatus {
        let queue = self.task_queue.lock().unwrap();
        let completed = self.completed_tasks.lock().unwrap();
        let failed = *self.failed_count.lock().unwrap();
        let start = self.start_time.lock().unwrap();

        QueueStatus {
            pending: queue.len(),
            running: if self.is_running() { 1 } else { 0 },
            completed: completed.len(),
            failed,
            is_running: self.is_running(),
            uptime_secs: start.map(|s| s.elapsed().as_secs()),
            total_tokens_consumed: completed.iter().map(|t| t.tokens_consumed as u64).sum(),
        }
    }

    /// Get all pending tasks
    pub fn pending_tasks(&self) -> Vec<AutonomousTask> {
        let queue = self.task_queue.lock().unwrap();
        queue.iter().cloned().collect()
    }

    /// Get completed task results
    pub fn completed_results(&self) -> Vec<TaskResult> {
        let completed = self.completed_tasks.lock().unwrap();
        completed.clone()
    }

    /// Record a task result
    pub fn record_result(
        &self,
        task: &AutonomousTask,
        result: Result<String>,
        duration: Duration,
        tokens: u32,
    ) {
        let task_result = match result {
            Ok(output) => {
                tracing::info!("Task {} completed successfully", task.id);
                TaskResult {
                    task_id: task.id,
                    task_name: task.name.clone(),
                    success: true,
                    output: Some(output),
                    error: None,
                    duration_ms: duration.as_millis() as u64,
                    tokens_consumed: tokens,
                    completed_at: chrono::Utc::now(),
                }
            }
            Err(e) => {
                tracing::error!("Task {} failed: {}", task.id, e);
                let mut failed = self.failed_count.lock().unwrap();
                *failed += 1;
                TaskResult {
                    task_id: task.id,
                    task_name: task.name.clone(),
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    duration_ms: duration.as_millis() as u64,
                    tokens_consumed: tokens,
                    completed_at: chrono::Utc::now(),
                }
            }
        };

        let mut completed = self.completed_tasks.lock().unwrap();
        completed.push(task_result);
    }

    /// Clear completed tasks (keep only recent N)
    pub fn prune_completed(&self, keep_count: usize) {
        let mut completed = self.completed_tasks.lock().unwrap();
        if completed.len() > keep_count {
            let drain_count = completed.len() - keep_count;
            completed.drain(0..drain_count);
        }
    }

    /// Cancel a pending task by ID
    pub fn cancel(&self, task_id: Uuid) -> bool {
        let mut queue = self.task_queue.lock().unwrap();
        if let Some(pos) = queue.iter().position(|t| t.id == task_id) {
            queue.remove(pos);
            tracing::info!("Task {} cancelled", task_id);
            true
        } else {
            false
        }
    }

    /// Get config
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
}

impl Default for AutonomousRuntime {
    fn default() -> Self {
        Self::new(RuntimeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = AutonomousTask::new(
            "Test Task",
            TaskType::Think {
                prompt: "Hello world".to_string(),
            },
        );

        assert!(task.status == TaskStatus::Pending);
        assert!(task.priority == TaskPriority::Normal);
    }

    #[test]
    fn test_priority_queue() {
        let runtime = AutonomousRuntime::default();

        // Enqueue low priority first
        runtime.enqueue(
            AutonomousTask::new("Low", TaskType::MemoryConsolidation)
                .with_priority(TaskPriority::Low),
        );

        // Enqueue high priority second
        runtime.enqueue(
            AutonomousTask::new("High", TaskType::MemoryConsolidation)
                .with_priority(TaskPriority::High),
        );

        // High priority should be dequeued first
        let first = runtime.dequeue().unwrap();
        assert_eq!(first.name, "High");
    }

    #[test]
    fn test_status() {
        let runtime = AutonomousRuntime::default();
        runtime.add_task("Test", TaskType::MemoryConsolidation);

        let status = runtime.status();
        assert_eq!(status.pending, 1);
        assert!(!status.is_running);
    }
}
