#![allow(unused)]
//! Autonomous Runtime - 24-hour self-operating execution loop
//!
//! This module provides the autonomous execution capability for Trinity,
//! allowing it to process task queues, perform memory consolidation,
//! and operate continuously without human intervention.

use crate::agent::self_coder::SelfCodingAgent;
use crate::agent::task_store::TaskStore;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Task priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TaskPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task status in the queue
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

/// A task in the autonomous queue
#[derive(Debug, Clone)]
pub struct AutonomousTask {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// The actual work (serialized as task type + payload)
    pub task_type: TaskType,
}

/// Types of autonomous tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskType {
    /// Generate code from a prompt
    GenerateCode {
        prompt: String,
        language: String,
        output_path: Option<String>,
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
    /// Custom task with arbitrary payload
    Custom { handler: String, payload: String },
}

impl AutonomousTask {
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
    
    /// Generate a hash for deduplication based on task name and type
    /// Two tasks with the same name and type are considered duplicates
    pub fn task_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        // Hash the task type by its JSON representation for consistency
        if let Ok(type_json) = serde_json::to_string(&self.task_type) {
            type_json.hash(&mut hasher);
        }
        hasher.finish()
    }
}

/// Configuration for the autonomous runtime
#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    /// How often to check for new tasks (seconds)
    pub poll_interval: Duration,
    /// Maximum concurrent tasks
    pub max_concurrent: usize,
    /// Enable memory consolidation cycle
    pub enable_dream_cycle: bool,
    /// Interval between dream cycles (seconds)
    pub dream_cycle_interval: Duration,
    /// Maximum runtime before auto-shutdown (0 = infinite)
    pub max_runtime: Option<Duration>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
            max_concurrent: 1, // Single-threaded for safety
            enable_dream_cycle: true,
            dream_cycle_interval: Duration::from_secs(30), // Every 30s for testing (was 3600)
            max_runtime: None,                               // Run forever
        }
    }
}

/// Task execution result
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration: Duration,
}

/// The autonomous runtime engine
#[derive(Clone)]
pub struct AutonomousRuntime {
    config: RuntimeConfig,
    /// Task queue (priority queue)
    task_queue: Arc<Mutex<VecDeque<AutonomousTask>>>,
    /// Completed tasks log
    completed_tasks: Arc<Mutex<Vec<TaskResult>>>,
    /// Runtime state
    is_running: Arc<Mutex<bool>>,
    /// Start time
    start_time: Option<Instant>,
    /// Recent task hashes for deduplication (prevents infinite rescheduling)
    recent_task_hashes: Arc<Mutex<HashSet<u64>>>,
    /// Persistent Task Store
    task_store: Arc<TaskStore>,
}

impl AutonomousRuntime {
    /// Create a new autonomous runtime
    pub fn new(config: RuntimeConfig, task_store: Arc<TaskStore>) -> Self {
        // Load pending tasks from store
        let pending = task_store.load_pending_tasks().unwrap_or_else(|e| {
            log::error!("Failed to load pending tasks: {}", e);
            Vec::new()
        });

        log::info!("Loaded {} pending tasks from TaskStore", pending.len());

        Self {
            config,
            task_queue: Arc::new(Mutex::new(VecDeque::from(pending))),
            completed_tasks: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            start_time: None,
            recent_task_hashes: Arc::new(Mutex::new(HashSet::new())),
            task_store,
        }
    }

    /// Add a task to the queue (with deduplication)
    pub fn enqueue(&self, task: AutonomousTask) -> Option<Uuid> {
        let task_hash = task.task_hash();
        
        // Check for duplicate task
        {
            let hashes = self.recent_task_hashes.lock().unwrap();
            if hashes.contains(&task_hash) {
                log::debug!("Task '{}' is a duplicate, skipping", task.name);
                return None;
            }
        }
        
        // Record this task hash
        {
            let mut hashes = self.recent_task_hashes.lock().unwrap();
            hashes.insert(task_hash);
            
            // Limit hash set size to prevent unbounded growth
            if hashes.len() > 1000 {
                // Clear oldest by just resetting (simple approach)
                hashes.clear();
                hashes.insert(task_hash);
            }
        }
        
        if let Err(e) = self.task_store.save_task(&task) {
            log::error!("Failed to persist task {}: {}", task.id, e);
        }

        let id = task.id;
        let mut queue = self.task_queue.lock().unwrap();

        // Insert based on priority (higher priority at front)
        let insert_pos = queue
            .iter()
            .position(|t| t.priority < task.priority)
            .unwrap_or(queue.len());

        queue.insert(insert_pos, task);
        log::info!("Task {} enqueued at position {}", id, insert_pos);

        Some(id)
    }

    /// Get next task from queue (public for external loop)
    pub fn dequeue_task(&self) -> Option<AutonomousTask> {
        let mut queue = self.task_queue.lock().unwrap();
        queue.pop_front()
    }

    /// Check if runtime is running
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    /// Start the runtime (mark as running)
    pub fn start(&mut self) {
        let mut running = self.is_running.lock().unwrap();
        *running = true;
        self.start_time = Some(Instant::now());
        log::info!("Autonomous runtime started");
    }

    /// Stop the runtime
    pub fn stop(&self) {
        let mut running = self.is_running.lock().unwrap();
        *running = false;
        log::info!("Autonomous runtime stopping...");
    }

    /// Get queue status
    pub fn queue_status(&self) -> QueueStatus {
        let queue = self.task_queue.lock().unwrap();
        let completed = self.completed_tasks.lock().unwrap();

        QueueStatus {
            pending: queue.len(),
            completed: completed.len(),
            is_running: self.is_running(),
            uptime: self.start_time.map(|s| s.elapsed()),
        }
    }

    /// Get completed task results (for API)
    pub fn get_completed_tasks(&self) -> Vec<TaskResult> {
        let completed = self.completed_tasks.lock().unwrap();
        completed.clone()
    }

    /// Record the result of a task execution
    pub fn record_result(&mut self, task_id: Uuid, result: Result<String>) {
        let task_result = match result {
            Ok(output) => {
                log::info!("Task {} completed successfully", task_id);
                TaskResult {
                    task_id,
                    success: true,
                    output: Some(output),
                    error: None,
                    duration: Duration::from_secs(0), // TODO: track actual duration
                }
            }
            Err(e) => {
                log::error!("Task {} failed: {}", task_id, e);
                TaskResult {
                    task_id,
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    duration: Duration::from_secs(0),
                }
            }
        };

        let mut completed = self.completed_tasks.lock().unwrap();
        completed.push(task_result);
    }

    /// Run the autonomous loop (blocking)
    pub async fn run(&mut self, agent: Arc<tokio::sync::Mutex<SelfCodingAgent>>) -> Result<()> {
        log::info!("Starting autonomous runtime");

        {
            let mut running = self.is_running.lock().unwrap();
            *running = true;
        }

        self.start_time = Some(Instant::now());
        let mut last_dream_cycle = Instant::now();

        loop {
            // Check if we should stop
            if !self.is_running() {
                log::info!("Runtime stopped");
                break;
            }

            // Check max runtime
            if let Some(max) = self.config.max_runtime {
                if self.start_time.unwrap().elapsed() > max {
                    log::info!("Max runtime reached, stopping");
                    self.stop();
                    break;
                }
            }

            // Check for dream cycle
            if self.config.enable_dream_cycle
                && last_dream_cycle.elapsed() > self.config.dream_cycle_interval
            {
                log::info!("Starting dream cycle (memory consolidation)");
                self.run_dream_cycle().await?;
                last_dream_cycle = Instant::now();
            }

            // Process next task
            if let Some(mut task) = self.dequeue_task() {
                log::info!("Processing task: {} ({})", task.name, task.id);

                task.status = TaskStatus::Running;
                task.started_at = Some(chrono::Utc::now());
                if let Err(e) = self.task_store.update_status(&task.id, &TaskStatus::Running) {
                    log::warn!("Failed to update task status to Running: {}", e);
                }

                let start = Instant::now();
                let result = self.execute_task(&task, agent.clone()).await;
                let duration = start.elapsed();

                let task_result = match result {
                    Ok(output) => {
                        task.status = TaskStatus::Completed;
                        task.completed_at = Some(chrono::Utc::now());
                        log::info!("Task {} completed in {:?}", task.id, duration);
                        if let Err(e) = self.task_store.update_status(&task.id, &TaskStatus::Completed) {
                            log::warn!("Failed to update task status to Completed: {}", e);
                        }

                        TaskResult {
                            task_id: task.id,
                            success: true,
                            output: Some(output),
                            error: None,
                            duration,
                        }
                    }
                    Err(e) => {
                        task.status = TaskStatus::Failed(e.to_string());
                        task.completed_at = Some(chrono::Utc::now());
                        log::error!("Task {} failed: {}", task.id, e);
                        if let Err(e) = self.task_store.update_status(&task.id, &task.status) {
                            log::warn!("Failed to update task status to Failed: {}", e);
                        }

                        TaskResult {
                            task_id: task.id,
                            success: false,
                            output: None,
                            error: Some(e.to_string()),
                            duration,
                        }
                    }
                };

                // Log result
                let mut completed = self.completed_tasks.lock().unwrap();
                completed.push(task_result);
            } else {
                // No tasks, sleep
                tokio::time::sleep(self.config.poll_interval).await;
            }
        }

        Ok(())
    }

    /// Execute a single task
    async fn execute_task(
        &self,
        task: &AutonomousTask,
        agent: Arc<tokio::sync::Mutex<SelfCodingAgent>>,
    ) -> Result<String> {
        match &task.task_type {
            TaskType::GenerateCode {
                prompt,
                language,
                output_path,
            } => {
                let mut guard = agent.lock().await;
                let result = guard.generate_code(prompt, language).await?;

                if let (Some(path), Some(content)) = (output_path, &result.content) {
                    guard.write_file(path, content).await?;
                }

                Ok(result.content.unwrap_or_default())
            }

            TaskType::EditFile { path, instructions } => {
                let mut guard = agent.lock().await;
                let result = guard.edit_code(path, instructions).await?;
                Ok(result.content.unwrap_or_default())
            }

            TaskType::RunCommand {
                command,
                working_dir,
            } => {
                let mut guard = agent.lock().await;
                // If working_dir is specified, we might want to change it temporarily,
                // but run_shell_command uses config.workspace_root.
                // For now, we'll let run_shell_command handle execution.
                // Note: working_dir in TaskType is currently ignored by run_shell_command 
                // unless we enhance run_shell_command to take an optional dir.
                // For safety/simplicity, we stick to workspace root or current dir.
                
                log::info!("Executing shell command: {}", command);
                let result = guard.run_shell_command(command).await?;
                Ok(result.content.unwrap_or_default())
            }

            TaskType::MemoryConsolidation => {
                self.run_dream_cycle().await?;
                Ok("Memory consolidation complete".to_string())
            }

            TaskType::WorkspaceScan { path } => {
                let mut guard = agent.lock().await;
                let files = guard.list_files(path).await?;
                Ok(format!("Scanned {} files", files.len()))
            }

            TaskType::Custom { handler, payload } => {
                if handler == "scan_todos" {
                    let todos = scan_for_todos(agent.clone(), payload).await;
                    let count = todos.len();
                    
                    if count > 0 {
                        log::info!("Found {} TODOs via self-grooming. scheduling fixes for top 3.", count);
                        for file_path in todos.into_iter().take(3) {
                            // Schedule a fix task
                            let fix_task = AutonomousTask::new(
                                format!("Auto-Fix TODO in {}", file_path),
                                TaskType::EditFile { 
                                    path: file_path.clone(), 
                                    instructions: "Analyze this file, locate the TODO or FIXME comments, and attempt to implement the missing functionality. If the task is too complex, add a detailed explanation comment instead.".to_string() 
                                }
                            ).with_priority(TaskPriority::Normal);
                            
                            self.enqueue(fix_task);
                        }
                    }
                    
                    Ok(format!("Scanned for TODOs in {}. Found {}, scheduled 3 fixes.", payload, count))
                } else {
                    log::info!(
                        "Custom task: {} with payload size {}",
                        handler,
                        payload.len()
                    );
                    Ok(format!("Custom task {} processed", handler))
                }
            }
        }
    }

    /// Run the dream cycle (memory consolidation)
    async fn run_dream_cycle(&self) -> Result<()> {
        log::info!("Dream cycle: Consolidating memories and scheduling self-improvement");

        // 1. Schedule a self-grooming scan (Standard maintenance)
        let task = AutonomousTask::new(
            "Self-Grooming: Scan for TODOs",
            TaskType::Custom {
                handler: "scan_todos".to_string(),
                payload: "backend/src".to_string(),
            },
        )
        .with_priority(TaskPriority::Low);

        self.enqueue(task);

        // 2. Goal Seeking: Check for high-level goals in GOALS.md
        // This allows the user to direct the autonomous agent asynchronously
        // We inject a task to read/parse GOALS.md and spawn sub-tasks
        let goal_task = AutonomousTask::new(
            "Goal Seeking: Process GOALS.md",
            TaskType::GenerateCode {
                prompt: "Read the file 'GOALS.md' in the workspace root. \
                         If it exists, parse the items marked as '[ ]' (uncompleted). \
                         For the highest priority uncompleted goal, generate a plan and \
                         create a new file called 'active_goal_plan.md' with the details. \
                         If 'active_goal_plan.md' already exists, read it and generate \
                         the next step as a code modification task.".to_string(),
                language: "markdown".to_string(),
                output_path: None, // The agent will decide where to write
            },
        ).with_priority(TaskPriority::Normal);
        
        self.enqueue(goal_task);

        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }
}

// Helper to scan for TODOs
async fn scan_for_todos(agent: Arc<tokio::sync::Mutex<SelfCodingAgent>>, path: &str) -> Vec<String> {
    let mut todos = Vec::new();
    
    // Lock briefly to get file list
    let files_result = {
        let mut guard = agent.lock().await;
        guard.list_files(path).await
    };

    if let Ok(files) = files_result {
        for file in files {
            // Only check code files
            let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !["rs", "js", "ts", "py"].contains(&ext) {
                continue;
            }

            // Using tokio::fs for async read - no lock needed here since we are reading standard fs
            if let Ok(content) = tokio::fs::read_to_string(&file).await {
                if content.contains("TODO") || content.contains("FIXME") {
                    todos.push(file.to_string_lossy().to_string());
                }
            }
        }
    }
    todos
}

/// Queue status information
#[derive(Debug, Clone)]
pub struct QueueStatus {
    pub pending: usize,
    pub completed: usize,
    pub is_running: bool,
    pub uptime: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = AutonomousTask::new(
            "Test Task",
            TaskType::GenerateCode {
                prompt: "Hello world".to_string(),
                language: "rust".to_string(),
                output_path: None,
            },
        );

        assert!(task.status == TaskStatus::Pending);
        assert!(task.priority == TaskPriority::Normal);
    }

    #[test]
    fn test_priority_queue() {
        let store = Arc::new(TaskStore::in_memory().unwrap());
        let runtime = AutonomousRuntime::new(RuntimeConfig::default(), store);

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
        let first = runtime.dequeue_task().unwrap();
        assert_eq!(first.name, "High");
    }
}
