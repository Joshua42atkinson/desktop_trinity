//! Terminal Execution API
//!
//! Provides shell command execution capabilities for Trinity.
//! This is a core IDE feature enabling build, run, and debug operations.

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::AppState;

/// Request to execute a terminal command
#[derive(Debug, Deserialize)]
pub struct TerminalRequest {
    /// Command to execute (e.g., "cargo build")
    pub command: String,
    /// Working directory (defaults to project root)
    pub cwd: Option<String>,
    /// Environment variables to set
    pub env: Option<std::collections::HashMap<String, String>>,
    /// Timeout in seconds (default: 300)
    pub timeout_secs: Option<u64>,
}

/// Response from terminal execution
#[derive(Debug, Serialize)]
pub struct TerminalResponse {
    /// Command exit code (0 = success)
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Whether the command timed out
    pub timed_out: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

/// Execute a command and return the result
async fn execute_command(
    State(_state): State<AppState>,
    Json(request): Json<TerminalRequest>,
) -> Json<TerminalResponse> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(request.timeout_secs.unwrap_or(300));

    // Build command using sh -c to handle pipes, redirects, etc.

    // Build command
    let mut command = Command::new("sh");
    command.arg("-c").arg(&request.command);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    // Set working directory
    if let Some(cwd) = &request.cwd {
        command.current_dir(cwd);
    }

    // Set environment variables
    if let Some(env) = &request.env {
        for (key, value) in env {
            command.env(key, value);
        }
    }

    // Execute with timeout
    let result = tokio::time::timeout(timeout, async {
        match command.spawn() {
            Ok(mut child) => {
                let mut stdout = String::new();
                let mut stderr = String::new();

                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_string(&mut stdout).await;
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_string(&mut stderr).await;
                }

                match child.wait().await {
                    Ok(status) => (status.code().unwrap_or(-1), stdout, stderr),
                    Err(e) => (-1, String::new(), format!("Wait error: {}", e)),
                }
            }
            Err(e) => (-1, String::new(), format!("Spawn error: {}", e)),
        }
    })
    .await;

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((exit_code, stdout, stderr)) => Json(TerminalResponse {
            exit_code,
            stdout,
            stderr,
            timed_out: false,
            duration_ms,
        }),
        Err(_) => Json(TerminalResponse {
            exit_code: -1,
            stdout: String::new(),
            stderr: "Command timed out".to_string(),
            timed_out: true,
            duration_ms,
        }),
    }
}

/// Quick command for simple operations (ls, pwd, etc.)
#[derive(Debug, Deserialize)]
pub struct QuickCommand {
    pub command: String,
}

async fn quick_exec(
    State(state): State<AppState>,
    Json(request): Json<QuickCommand>,
) -> Json<TerminalResponse> {
    execute_command(
        State(state),
        Json(TerminalRequest {
            command: request.command,
            cwd: None,
            env: None,
            timeout_secs: Some(30), // Quick commands have 30s timeout
        }),
    )
    .await
}

pub fn terminal_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/api/terminal/execute", post(execute_command))
        .route("/api/terminal/quick", post(quick_exec))
        .with_state(state.clone())
}
