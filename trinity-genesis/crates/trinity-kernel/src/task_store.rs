//! # Task Store (Persistent Task Queue)
//!
//! ## Philosophy
//! "A mind that forgets its commitments is unreliable. The TaskStore ensures
//!  Trinity remembers what it promised to do, even across restarts and failures."
//!
//! ## Purpose
//! SQLite-backed persistence for autonomous tasks. Survives restarts so Trinity
//! can resume work after reboots, crashes, or power outages.
//!
//! Migrated from day_dream/backend/src/agent/task_store.rs

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

use crate::runtime::{AutonomousTask, TaskPriority, TaskStatus, TaskType};

/// Persistent store for autonomous tasks
pub struct TaskStore {
    conn: Mutex<Connection>,
}

impl TaskStore {
    /// Create a new task store, initializing the database
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn =
            Connection::open(db_path.as_ref()).context("Failed to open task store database")?;

        // Create tables
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS autonomous_tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                priority INTEGER NOT NULL DEFAULT 1,
                status TEXT NOT NULL DEFAULT 'pending',
                task_type_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                assigned_agent TEXT
            );

            CREATE TABLE IF NOT EXISTS checkpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                generated_at TEXT NOT NULL,
                content TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_status ON autonomous_tasks(status);
            "#,
        )
        .context("Failed to create task store tables")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// In-memory store for testing
    pub fn in_memory() -> Result<Self> {
        Self::new(":memory:")
    }

    /// Save a task to the database
    pub fn save_task(&self, task: &AutonomousTask) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let priority: i32 = match task.priority {
            TaskPriority::Low => 0,
            TaskPriority::Normal => 1,
            TaskPriority::High => 2,
            TaskPriority::Critical => 3,
        };

        let status_str = match &task.status {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed(_) => "failed",
            TaskStatus::Cancelled => "cancelled",
        };

        let task_type_json =
            serde_json::to_string(&task.task_type).context("Failed to serialize task type")?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO autonomous_tasks 
            (id, name, description, priority, status, task_type_json, created_at, started_at, completed_at, assigned_agent)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                task.id.to_string(),
                task.name,
                task.description,
                priority,
                status_str,
                task_type_json,
                task.created_at.to_rfc3339(),
                task.started_at.map(|t| t.to_rfc3339()),
                task.completed_at.map(|t| t.to_rfc3339()),
                task.assigned_agent,
            ],
        )
        .context("Failed to save task")?;

        Ok(())
    }

    /// Load all pending tasks (for restart recovery)
    pub fn load_pending_tasks(&self) -> Result<Vec<AutonomousTask>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, priority, task_type_json, created_at, assigned_agent
            FROM autonomous_tasks
            WHERE status = 'pending' OR status = 'running'
            ORDER BY priority DESC, created_at ASC
            "#,
        )?;

        let tasks = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let description: String = row.get(2)?;
                let priority_int: i32 = row.get(3)?;
                let task_type_json: String = row.get(4)?;
                let created_str: String = row.get(5)?;
                let assigned_agent: Option<String> = row.get(6)?;

                Ok((
                    id_str,
                    name,
                    description,
                    priority_int,
                    task_type_json,
                    created_str,
                    assigned_agent,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(
                |(id_str, name, desc, priority_int, type_json, created_str, assigned_agent)| {
                    let id = uuid::Uuid::parse_str(&id_str).ok()?;
                    let priority = match priority_int {
                        0 => TaskPriority::Low,
                        2 => TaskPriority::High,
                        3 => TaskPriority::Critical,
                        _ => TaskPriority::Normal,
                    };
                    let task_type: TaskType = serde_json::from_str(&type_json).ok()?;
                    let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                        .ok()?
                        .with_timezone(&chrono::Utc);

                    Some(AutonomousTask {
                        id,
                        name,
                        description: desc,
                        priority,
                        status: TaskStatus::Pending,
                        created_at,
                        started_at: None,
                        completed_at: None,
                        task_type,
                        assigned_agent,
                    })
                },
            )
            .collect();

        Ok(tasks)
    }

    /// Update task status
    pub fn update_status(&self, task_id: &uuid::Uuid, status: &TaskStatus) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let status_str = match status {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed(_) => "failed",
            TaskStatus::Cancelled => "cancelled",
        };

        conn.execute(
            "UPDATE autonomous_tasks SET status = ?1, completed_at = ?2 WHERE id = ?3",
            params![
                status_str,
                chrono::Utc::now().to_rfc3339(),
                task_id.to_string()
            ],
        )?;

        Ok(())
    }

    /// Save a checkpoint report (for dream cycle status)
    pub fn save_checkpoint(&self, content: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO checkpoints (generated_at, content) VALUES (?1, ?2)",
            params![chrono::Utc::now().to_rfc3339(), content],
        )?;

        // Keep only last 50 checkpoints
        conn.execute(
            r#"
            DELETE FROM checkpoints WHERE id NOT IN (
                SELECT id FROM checkpoints ORDER BY generated_at DESC LIMIT 50
            )
            "#,
            [],
        )?;

        Ok(())
    }

    /// Get recent checkpoints
    pub fn get_recent_checkpoints(&self, limit: usize) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT generated_at, content FROM checkpoints ORDER BY generated_at DESC LIMIT ?1",
        )?;

        let checkpoints = stmt
            .query_map([limit], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(checkpoints)
    }

    /// Count pending tasks
    pub fn pending_count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM autonomous_tasks WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Clear all tasks (for testing)
    pub fn clear(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM autonomous_tasks", [])?;
        conn.execute("DELETE FROM checkpoints", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_store_roundtrip() {
        let store = TaskStore::in_memory().unwrap();

        let task = AutonomousTask::new("Test Task", TaskType::MemoryConsolidation);

        store.save_task(&task).unwrap();

        let loaded = store.load_pending_tasks().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Test Task");
    }

    #[test]
    fn test_status_update() {
        let store = TaskStore::in_memory().unwrap();

        let task = AutonomousTask::new("Test Task", TaskType::MemoryConsolidation);
        store.save_task(&task).unwrap();

        store.update_status(&task.id, &TaskStatus::Running).unwrap();
        store
            .update_status(&task.id, &TaskStatus::Completed)
            .unwrap();

        // Completed tasks shouldn't be loaded as pending
        let loaded = store.load_pending_tasks().unwrap();
        assert_eq!(loaded.len(), 0);
    }
}
