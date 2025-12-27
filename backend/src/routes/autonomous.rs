//! Autonomous Runtime API Routes
//!
//! REST endpoints for interacting with Trinity's autonomous task queue.

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::agent::autonomous::{
    AutonomousRuntime, AutonomousTask, QueueStatus, TaskPriority, TaskResult, TaskType,
};

/// Shared runtime state
pub type SharedRuntime = Arc<Mutex<AutonomousRuntime>>;

/// Request to enqueue a new task
#[derive(Debug, Deserialize)]
pub struct EnqueueRequest {
    pub name: String,
    pub description: Option<String>,
    pub priority: Option<String>, // "low", "normal", "high", "critical"
    pub task_type: TaskTypeRequest,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskTypeRequest {
    GenerateCode {
        prompt: String,
        language: String,
        output_path: Option<String>,
    },
    EditFile {
        path: String,
        instructions: String,
    },
    RunCommand {
        command: String,
        working_dir: Option<String>,
    },
    MemoryConsolidation,
    WorkspaceScan {
        path: String,
    },
    Custom {
        handler: String,
        payload: String,
    },
}

impl From<TaskTypeRequest> for TaskType {
    fn from(req: TaskTypeRequest) -> Self {
        match req {
            TaskTypeRequest::GenerateCode {
                prompt,
                language,
                output_path,
            } => TaskType::GenerateCode {
                prompt,
                language,
                output_path,
            },
            TaskTypeRequest::EditFile { path, instructions } => {
                TaskType::EditFile { path, instructions }
            }
            TaskTypeRequest::RunCommand {
                command,
                working_dir,
            } => TaskType::RunCommand {
                command,
                working_dir,
            },
            TaskTypeRequest::MemoryConsolidation => TaskType::MemoryConsolidation,
            TaskTypeRequest::WorkspaceScan { path } => TaskType::WorkspaceScan { path },
            TaskTypeRequest::Custom { handler, payload } => TaskType::Custom { handler, payload },
        }
    }
}

/// Response for enqueue
#[derive(Debug, Serialize)]
pub struct EnqueueResponse {
    pub success: bool,
    pub task_id: Option<Uuid>,
    pub message: String,
}

/// Status response
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub is_running: bool,
    pub pending_tasks: usize,
    pub completed_tasks: usize,
    pub uptime_seconds: Option<u64>,
}

impl From<QueueStatus> for StatusResponse {
    fn from(status: QueueStatus) -> Self {
        StatusResponse {
            is_running: status.is_running,
            pending_tasks: status.pending,
            completed_tasks: status.completed,
            uptime_seconds: status.uptime.map(|d| d.as_secs()),
        }
    }
}

/// History response
#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub tasks: Vec<TaskResultDto>,
}

#[derive(Debug, Serialize)]
pub struct TaskResultDto {
    pub task_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl From<&TaskResult> for TaskResultDto {
    fn from(result: &TaskResult) -> Self {
        TaskResultDto {
            task_id: result.task_id.to_string(),
            success: result.success,
            output: result.output.clone(),
            error: result.error.clone(),
            duration_ms: result.duration.as_millis() as u64,
        }
    }
}

/// Checkpoint report response
#[derive(Debug, Serialize)]
pub struct CheckpointResponse {
    pub generated_at: String,
    pub uptime_hours: f64,
    pub tasks_completed: usize,
    pub tasks_failed: usize,
    pub recent_activity: Vec<String>,
    pub recommendations: Vec<String>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// GET /api/autonomous/status
async fn get_status(State(runtime): State<SharedRuntime>) -> Json<StatusResponse> {
    let rt = runtime.lock().unwrap();
    Json(rt.queue_status().into())
}

/// POST /api/autonomous/enqueue
async fn enqueue_task(
    State(runtime): State<SharedRuntime>,
    Json(req): Json<EnqueueRequest>,
) -> Result<Json<EnqueueResponse>, StatusCode> {
    let priority = match req.priority.as_deref() {
        Some("low") => TaskPriority::Low,
        Some("high") => TaskPriority::High,
        Some("critical") => TaskPriority::Critical,
        _ => TaskPriority::Normal,
    };

    let mut task = AutonomousTask::new(req.name, req.task_type.into()).with_priority(priority);

    if let Some(desc) = req.description {
        task = task.with_description(desc);
    }

    let rt = runtime.lock().unwrap();

    match rt.enqueue(task) {
        Some(task_id) => Ok(Json(EnqueueResponse {
            success: true,
            task_id: Some(task_id),
            message: "Task enqueued successfully".to_string(),
        })),
        None => Ok(Json(EnqueueResponse {
            success: false,
            task_id: None,
            message: "Task skipped: duplicate already in queue or recently completed".to_string(),
        })),
    }
}

/// GET /api/autonomous/history
async fn get_history(State(runtime): State<SharedRuntime>) -> Json<HistoryResponse> {
    let rt = runtime.lock().unwrap();
    let completed = rt.get_completed_tasks();

    let tasks: Vec<TaskResultDto> = completed.iter().rev().take(50).map(|r| r.into()).collect();

    Json(HistoryResponse { tasks })
}

/// POST /api/autonomous/stop
async fn stop_runtime(State(runtime): State<SharedRuntime>) -> Json<serde_json::Value> {
    let rt = runtime.lock().unwrap();
    rt.stop();
    Json(serde_json::json!({
        "success": true,
        "message": "Autonomous runtime stopping..."
    }))
}

/// GET /api/autonomous/checkpoint
async fn get_checkpoint(State(runtime): State<SharedRuntime>) -> Json<CheckpointResponse> {
    let rt = runtime.lock().unwrap();
    let status = rt.queue_status();
    let completed = rt.get_completed_tasks();

    let tasks_failed = completed.iter().filter(|t| !t.success).count();
    let recent: Vec<String> = completed
        .iter()
        .rev()
        .take(10)
        .map(|t| {
            if t.success {
                format!("✓ Task {} completed in {:?}", t.task_id, t.duration)
            } else {
                format!(
                    "✗ Task {} failed: {}",
                    t.task_id,
                    t.error.as_deref().unwrap_or("unknown")
                )
            }
        })
        .collect();

    let uptime_hours = status
        .uptime
        .map(|d| d.as_secs_f64() / 3600.0)
        .unwrap_or(0.0);

    Json(CheckpointResponse {
        generated_at: chrono::Utc::now().to_rfc3339(),
        uptime_hours,
        tasks_completed: completed.len(),
        tasks_failed,
        recent_activity: recent,
        recommendations: vec!["Consider reviewing TODO fixes from dream cycles".to_string()],
    })
}

// ============================================================================
// ROUTER
// ============================================================================

pub fn autonomous_routes(runtime: SharedRuntime) -> Router {
    Router::new()
        .route("/api/autonomous/status", get(get_status))
        .route("/api/autonomous/enqueue", post(enqueue_task))
        .route("/api/autonomous/history", get(get_history))
        .route("/api/autonomous/stop", post(stop_runtime))
        .route("/api/autonomous/checkpoint", get(get_checkpoint))
        .with_state(runtime)
}
