// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Task Service - Autonomous Task Queue RPC Interface
//!
//! Defines the Tarpc service for remote task management.

use crate::types::{AssessmentType, Difficulty, ProtocolError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::Duration;

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
        assessment_type: AssessmentType, // Quiz, Lab, Challenge
        difficulty: Difficulty,          // Beginner, Intermediate, etc.
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

/// Simplified task representation for RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub priority: u8,    // 0=Low, 1=Normal, 2=High, 3=Critical
    pub status: String,  // "pending", "running", "completed", "failed"
    pub agent: Option<String>,
    pub created_at: String,
}

impl From<AutonomousTask> for TaskInfo {
    fn from(task: AutonomousTask) -> Self {
        Self {
            id: task.id,
            name: task.name,
            description: task.description,
            priority: task.priority as u8,
            status: match task.status {
                TaskStatus::Pending => "pending".to_string(),
                TaskStatus::Running => "running".to_string(),
                TaskStatus::Completed => "completed".to_string(),
                TaskStatus::Failed(_) => "failed".to_string(),
                TaskStatus::Cancelled => "cancelled".to_string(),
            },
            agent: task.assigned_agent,
            created_at: task.created_at.to_rfc3339(),
        }
    }
}

/// Task service provides task queue management over RPC.
///
/// This service runs on the Brain node and allows the Body node
/// to submit, monitor, and cancel tasks.
#[tarpc::service]
pub trait TaskService {
    /// Get current queue status
    async fn status() -> QueueStatus;

    /// List all pending tasks
    async fn list_pending() -> Vec<TaskInfo>;

    /// List recent completed results
    async fn list_completed(limit: usize) -> Vec<TaskResult>;

    /// Submit a new task
    async fn submit(name: String, task_type: TaskType, priority: u8) -> Result<Uuid, ProtocolError>;

    /// Cancel a pending task
    async fn cancel(task_id: Uuid) -> Result<bool, ProtocolError>;

    /// Start the runtime
    async fn start() -> Result<(), ProtocolError>;

    /// Stop the runtime
    async fn stop() -> Result<(), ProtocolError>;
}
