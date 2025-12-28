// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Autonomous Runtime - Task Queue and Self-Operating Execution
//!
//! This module provides the autonomous execution capability for Trinity Genesis,
//! allowing it to process task queues, perform memory consolidation ("dream cycles"),
//! and operate continuously without human intervention.

use anyhow::Result;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

// ============================================================================
// Task Types (Moved to trinity-protocol)
// ============================================================================

pub use trinity_protocol::task::{
    AutonomousTask, QueueStatus, RuntimeConfig, TaskPriority, TaskResult, TaskStatus, TaskType,
};

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
