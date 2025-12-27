//! Task Service - Autonomous Task Queue RPC Interface
//!
//! Defines the Tarpc service for remote task management.

use crate::types::{ProtocolError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export task types for convenience
pub use trinity_kernel::runtime::{
    AutonomousTask, TaskType, TaskPriority, TaskStatus, QueueStatus, TaskResult,
};

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
