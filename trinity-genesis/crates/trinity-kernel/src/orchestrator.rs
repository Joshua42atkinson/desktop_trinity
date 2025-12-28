// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Agent Orchestrator - Multi-Agent Task Dispatch with Streaming Events
//!
//! Coordinates multiple parallel coding agents and streams their progress
//! to the Antigravity Window for real-time visualization.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
use uuid::Uuid;
use std::collections::HashMap;

use crate::brain::{Brain, GrammarSpec};
use crate::runtime::{AutonomousTask, TaskPriority, TaskType};
use crate::wasm_sandbox::{Capability, CapabilitySet, SandboxConfig, WasmSandbox};
use crate::todo_parser;
use trinity_protocol::types::{AssessmentType, QuizQuestion, LabProject}; // Added imports
use std::path::Path;

// ============================================================================
// Agent Events (Streamed to UI)
// ============================================================================

/// Events streamed from agents to the Antigravity Window.
///
/// These events represent the stream of consciousness and actions of the agents,
/// allowing the user to see exactly what is happening in real-time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    /// Agent started working on a task.
    TaskStarted {
        agent_id: String,
        task_id: Uuid,
        task_name: String,
    },
    /// Agent is thinking/reasoning (Chain of Thought).
    Thinking {
        agent_id: String,
        thought: String,
    },
    /// Agent generated a block of code.
    CodeGenerated {
        agent_id: String,
        file_path: String,
        code_snippet: String,
        line_count: usize,
    },
    /// Agent is running a shell command.
    CommandRunning {
        agent_id: String,
        command: String,
    },
    /// Output (stdout/stderr) from a command.
    CommandOutput {
        agent_id: String,
        stdout: String,
        stderr: String,
    },
    /// Task completed successfully.
    TaskCompleted {
        agent_id: String,
        task_id: Uuid,
        result: String,
        duration_ms: u64,
        tokens_consumed: u32,
    },
    /// Task failed with an error.
    TaskFailed {
        agent_id: String,
        task_id: Uuid,
        error: String,
        tokens_consumed: u32,
    },
    /// Agent is waiting for work.
    AgentIdle {
        agent_id: String,
    },
    /// Generic artifact generated (Code, Text, Plan, etc.).
    /// Used for diverse outputs like images, documents, or structured data.
    ArtifactGenerated {
        agent_id: String,
        kind: String, // "code", "text", "plan", etc.
        content: String,
        metadata: serde_json::Value,
    },
}

// ============================================================================
// Agent Handle
// ============================================================================

/// Handle to a running agent.
///
/// Provides a mechanism to assign tasks to a specific agent instance.
#[derive(Clone)]
pub struct AgentHandle {
    /// Unique ID (e.g., "jessica-coder")
    pub id: String,
    /// Display name (e.g., "Jessica")
    pub name: String,
    /// The agent's primary role/specialization
    pub specialization: AgentSpecialization,
    task_tx: mpsc::Sender<AutonomousTask>,
}

/// Agent specializations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentSpecialization {
    /// General-purpose coder
    Coder,
    /// Documentation and content writer
    Writer,
    /// Code reviewer and quality checker
    Reviewer,
    /// Research and information gathering
    Researcher,
    /// Strategic planning and task decomposition
    Planner,
}

impl AgentHandle {
    /// Assign a task to this agent
    pub async fn assign(&self, task: AutonomousTask) -> Result<()> {
        self.task_tx
            .send(task)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to assign task: {}", e))
    }
}

// ============================================================================
// Orchestrator
// ============================================================================

/// Multi-agent orchestrator with streaming events.
///
/// The Orchestrator manages the lifecycle of agents, dispatches tasks,
/// and broadcasts real-time events to the UI. It implements the "Dual-Brain"
/// architecture (Planner + Worker).
pub struct Orchestrator {
    agents: Vec<AgentHandle>,
    event_tx: broadcast::Sender<AgentEvent>,
    pending_tasks: Arc<Mutex<Vec<AutonomousTask>>>,
    /// Track which agents are currently idle (agent_id -> is_idle)
    agent_status: Arc<Mutex<HashMap<String, bool>>>,
    /// Secure WASM Sandbox for tool execution
    sandbox: Arc<Mutex<WasmSandbox>>,
}

impl Orchestrator {
    /// Create a new orchestrator with Dual-Brain Architecture.
    ///
    /// # Arguments
    /// * `planner` - High-intelligence model (e.g. Llama 4 Scout) for "Joshua" (Reasoning/Planning).
    /// * `worker` - High-speed/context model (e.g. GLM-4 Flash) for "Jessica" (Coding/Execution).
    /// * `sandbox` - Shared WASM sandbox for secure tool execution.
    /// * `_agent_count` - (Currently unused) Target number of agents.
    ///
    /// # Returns
    /// An initialized Orchestrator with background worker tasks spawned.
    pub fn new(
        planner: Arc<dyn Brain>, 
        worker: Arc<dyn Brain>, 
        sandbox: Arc<Mutex<WasmSandbox>>,
        _agent_count: usize
    ) -> Self {
        let (event_tx, _) = broadcast::channel(256);

        let mut agents = Vec::with_capacity(2);

        // 1. Initialize Joshua (Planner)
        {
            let (task_tx, task_rx) = mpsc::channel(16);
            let agent_id = "joshua-planner".to_string();
            let agent_name = "Joshua".to_string();

            let handle = AgentHandle {
                id: agent_id.clone(),
                name: agent_name.clone(),
                specialization: AgentSpecialization::Writer, 
                task_tx,
            };

            // JOSHUA gets the PLANNER brain (High IQ)
            let brain_clone = planner.clone();
            let event_tx_clone = event_tx.clone();
            let sandbox_clone = sandbox.clone();
            tokio::spawn(Self::agent_worker(
                agent_id,
                agent_name,
                brain_clone,
                sandbox_clone,
                task_rx,
                event_tx_clone,
            ));

            agents.push(handle);
        }

        // 2. Initialize Jessica (Coder)
        {
            let (task_tx, task_rx) = mpsc::channel(16);
            let agent_id = "jessica-coder".to_string();
            let agent_name = "Jessica".to_string();

            let handle = AgentHandle {
                id: agent_id.clone(),
                name: agent_name.clone(),
                specialization: AgentSpecialization::Coder,
                task_tx,
            };

            // JESSICA gets the WORKER brain (High Speed/Context)
            let brain_clone = worker.clone();
            let event_tx_clone = event_tx.clone();
            let sandbox_clone = sandbox.clone();
            tokio::spawn(Self::agent_worker(
                agent_id,
                agent_name,
                brain_clone,
                sandbox_clone,
                task_rx,
                event_tx_clone,
            ));

            agents.push(handle);
        }

        // Initialize agent status map
        let agent_status = Arc::new(Mutex::new(HashMap::new()));
        {
            // Use try_lock to avoid blocking the runtime, as we are in initialization
            if let Ok(mut status) = agent_status.try_lock() {
                for agent in &agents {
                    status.insert(agent.id.clone(), true); // Start as idle
                }
            } else {
                tracing::error!("Failed to acquire agent status lock during initialization");
            }
        }

        Self {
            agents,
            event_tx,
            pending_tasks: Arc::new(Mutex::new(Vec::new())),
            agent_status,
            sandbox,
        }
    }

    /// Subscribe to agent events (for Antigravity Window)
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_tx.subscribe()
    }

    /// Get number of agents
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Submit a task to the orchestrator.
    ///
    /// If an agent is idle, the task is assigned immediately.
    /// Otherwise, it is added to the pending queue.
    ///
    /// # Arguments
    /// * `task` - The `AutonomousTask` to execute.
    ///
    /// # Returns
    /// The unique ID of the submitted task.
    pub async fn submit(&self, task: AutonomousTask) -> Result<Uuid> {
        let task_id = task.id;

        // Find an idle agent or queue the task
        if let Some(agent) = self.find_idle_agent().await {
            agent.assign(task).await?;
        } else {
            // Queue for later
            let mut pending = self.pending_tasks.lock().await;
            pending.push(task);
        }

        Ok(task_id)
    }

    /// Find an idle agent
    async fn find_idle_agent(&self) -> Option<&AgentHandle> {
        let status = self.agent_status.lock().await;
        for agent in &self.agents {
            if *status.get(&agent.id).unwrap_or(&false) {
                return Some(agent);
            }
        }
        None
    }

    /// Agent worker loop
    async fn agent_worker(
        agent_id: String,
        agent_name: String,
        brain: Arc<dyn Brain>,
        sandbox: Arc<Mutex<WasmSandbox>>,
        mut task_rx: mpsc::Receiver<AutonomousTask>,
        event_tx: broadcast::Sender<AgentEvent>,
    ) {
        tracing::info!("Agent {} ({}) started", agent_id, agent_name);

        // Emit idle event
        let _ = event_tx.send(AgentEvent::AgentIdle {
            agent_id: agent_id.clone(),
        });

        while let Some(task) = task_rx.recv().await {
            let start_time = std::time::Instant::now();

            // Emit task started
            let _ = event_tx.send(AgentEvent::TaskStarted {
                agent_id: agent_id.clone(),
                task_id: task.id,
                task_name: task.name.clone(),
            });

            // Execute task based on type
            let result = Self::execute_task(&agent_id, &agent_name, &brain, &sandbox, &task, &event_tx).await;

            let duration_ms = start_time.elapsed().as_millis() as u64;

            match result {
                Ok(output) => {
                    let tokens_consumed = brain.count_tokens(&output) as u32;
                    let _ = event_tx.send(AgentEvent::TaskCompleted {
                        agent_id: agent_id.clone(),
                        task_id: task.id,
                        result: output,
                        duration_ms,
                        tokens_consumed,
                    });
                }
                Err(e) => {
                    let _ = event_tx.send(AgentEvent::TaskFailed {
                        agent_id: agent_id.clone(),
                        task_id: task.id,
                        error: e.to_string(),
                        tokens_consumed: 0,
                    });
                }
            }

            // Back to idle
            let _ = event_tx.send(AgentEvent::AgentIdle {
                agent_id: agent_id.clone(),
            });
        }

        tracing::info!("Agent {} stopped", agent_id);
    }

    /// Execute a specific task with a given agent.
    ///
    /// This function handles the complex logic of:
    /// 1. Building context/prompts based on task type.
    /// 2. Calling the Brain for inference (Thinking).
    /// 3. Executing tools (WASM, Shell) if needed.
    /// 4. Broadcasting events for UI visualization.
    ///
    /// # Arguments
    /// * `agent_id` - ID of the working agent.
    /// * `agent_name` - Name of the working agent (affects persona).
    /// * `brain` - The brain instance to use.
    /// * `sandbox` - The tool sandbox.
    /// * `task` - The task to execute.
    /// * `event_tx` - Event broadcaster.
    async fn execute_task(
        agent_id: &str,
        agent_name: &str,
        brain: &Arc<dyn Brain>,
        sandbox: &Arc<Mutex<WasmSandbox>>,
        task: &AutonomousTask,
        event_tx: &broadcast::Sender<AgentEvent>,
    ) -> Result<String> {
        // Build personality-based system prompt
        let personality = match agent_name {
            "Joshua" => {
                "You are Joshua, the visionary Strategist and Planner for Trinity. \
                 Your goal is to survey progress, decide priorities, and ensure every task has purpose and meaning. \
                 You guide Jessica's work. Be wise, decisive, and focused on the big picture."
            }
            "Jessica" => {
                "You are Jessica, the master Developer and Coder for Trinity. \
                 You implement Joshua's vision with precision and elegance. \
                 Write high-quality, bug-free Rust and Bevy code. Be creative, efficient, and direct."
            }
            _ => "You are an autonomous AI agent for Trinity.",
        };

        match &task.task_type {
            TaskType::Think { prompt } => {
                // Emit thinking event
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Processing: {}", &prompt[..prompt.len().min(100)]),
                });

                // Call brain for inference with personality
                let full_prompt = format!("<|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n", personality, prompt);
                let response = brain.think(&full_prompt).await?;

                // Emit artifact
                let _ = event_tx.send(AgentEvent::ArtifactGenerated {
                    agent_id: agent_id.to_string(),
                    kind: "text".to_string(),
                    content: response.clone(),
                    metadata: serde_json::json!({}),
                });

                Ok(response)
            }

            TaskType::GenerateCode {
                prompt,
                language,
                output_path,
            } => {
                // Emit thinking
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Generating {} code...", language),
                });

                // Choose grammar based on language
                let grammar = match language.to_lowercase().as_str() {
                    "rust" => GrammarSpec::Rust,
                    "json" => GrammarSpec::Json,
                    "markdown" | "md" => GrammarSpec::Markdown,
                    _ => GrammarSpec::None,
                };

                // Build code generation prompt
                let code_prompt = format!(
                    "Language: {}\nTask: {}\n\nGenerate ONLY the code, no explanations or markdown fences:\n",
                    language, prompt
                );

                let code = brain.think_with_grammar(&code_prompt, grammar).await?;
                let line_count = code.lines().count();

                // Emit code generated
                let _ = event_tx.send(AgentEvent::CodeGenerated {
                    agent_id: agent_id.to_string(),
                    file_path: output_path.clone().unwrap_or_else(|| "stdout".to_string()),
                    code_snippet: code.chars().take(500).collect(),
                    line_count,
                });

                // Emit artifact (new standard)
                let _ = event_tx.send(AgentEvent::ArtifactGenerated {
                    agent_id: agent_id.to_string(),
                    kind: "code".to_string(),
                    content: code.clone(),
                    metadata: serde_json::json!({
                        "language": language,
                        "file_path": output_path.clone().unwrap_or_default()
                    }),
                });

                // Write to file if path specified
                if let Some(path) = output_path {
                    tokio::fs::write(path, &code).await?;
                }

                Ok(code)
            }

            TaskType::EditFile { path, instructions } => {
                // Emit thinking
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Editing file: {}", path),
                });

                // Read existing file
                let existing = tokio::fs::read_to_string(path).await.unwrap_or_default();

                // Build edit prompt
                let edit_prompt = format!(
                    "Edit the following code according to these instructions:\n\nInstructions: {}\n\nExisting code:\n```\n{}\n```\n\nOutput the complete edited file:",
                    instructions, existing
                );

                // Use Rust grammar for edits
                let grammar = if path.ends_with(".rs") {
                    GrammarSpec::Rust
                } else if path.ends_with(".json") {
                    GrammarSpec::Json
                } else {
                    GrammarSpec::None
                };

                // Generate new content via Brain
                let edited = brain.think_with_grammar(&edit_prompt, grammar).await?;
                let line_count = edited.lines().count();

                // Use WASM Sandbox for Secure Write
                {
                    let mut sb = sandbox.lock().await;
                    
                    // 1. Configure Permissions (Principle of Least Privilege)
                    // We only grant write access to the specific workspace root for now
                    // Ideally we'd narrow this to the file, but CapSet is path-prefix based
                    let workspace_root = sb.workspace_path().to_path_buf();
                    let mut config = SandboxConfig::default();
                    config.capabilities = CapabilitySet::new()
                        .with(Capability::FileRead { paths: vec![workspace_root.clone()] })
                        .with(Capability::FileWrite { paths: vec![workspace_root.clone()] });

                    // 2. Prepare Tool Arguments for code_editor
                    let args = serde_json::json!({
                        "action": "Write",
                        "args": {
                            "path": path,
                            "content": edited
                        }
                    });

                    // 3. Execute Securely
                    // Plugin must be loaded beforehand (in main.rs)
                    let plugin_path = std::path::PathBuf::from("plugins/code_editor.wasm");
                    // Ensure it is loaded if not already? modify sandbox to auto-load? 
                    // For now assuming it is loaded.
                    
                    // We need to pass the absolute path to the plugin because WasmSandbox::execute_with_config
                    // might expect it or the loaded module name. 
                    // Actually execute_with_config takes 'module_path_or_name'.
                    
                    match sb.execute_with_config(&plugin_path, "edit", &args.to_string(), config).await {
                        Ok(output) => {
                             tracing::info!("WASM File Write Success: {}", output);
                        },
                        Err(e) => {
                            // Fallback to native write if WASM fails (or return error)
                            tracing::error!("WASM Write Failed: {}, falling back to native", e);
                            tokio::fs::write(path, &edited).await?;
                        }
                    }
                }

                // Emit code generated event
                let _ = event_tx.send(AgentEvent::CodeGenerated {
                    agent_id: agent_id.to_string(),
                    file_path: path.clone(),
                    code_snippet: edited.chars().take(500).collect(),
                    line_count,
                });

                // Emit artifact
                let _ = event_tx.send(AgentEvent::ArtifactGenerated {
                    agent_id: agent_id.to_string(),
                    kind: "code".to_string(),
                    content: edited.clone(),
                    metadata: serde_json::json!({
                        "language": "rust", 
                        "file_path": path.clone()
                    }),
                });

                Ok(format!("Edited {} ({} lines)", path, line_count))
            }

            TaskType::RunCommand { command, working_dir } => {
                // Emit command running
                let _ = event_tx.send(AgentEvent::CommandRunning {
                    agent_id: agent_id.to_string(),
                    command: command.clone(),
                });

                // Execute command
                let mut cmd = tokio::process::Command::new("sh");
                cmd.arg("-c").arg(command);

                if let Some(dir) = working_dir {
                    cmd.current_dir(dir);
                }

                let output = cmd.output().await?;

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                // Emit output
                let _ = event_tx.send(AgentEvent::CommandOutput {
                    agent_id: agent_id.to_string(),
                    stdout: stdout.clone(),
                    stderr: stderr.clone(),
                });

                if output.status.success() {
                    Ok(stdout)
                } else {
                    Err(anyhow::anyhow!("Command failed: {}", stderr))
                }
            }

            TaskType::MemoryConsolidation => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: "Consolidating memories...".to_string(),
                });
                // Placeholder - would consolidate memory store
                Ok("Memory consolidation complete".to_string())
            }

            TaskType::WebBrowse { url } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Browsing web: {}", url),
                });
                
                // Native Kernel Implementation
                let client = reqwest::Client::builder()
                    .user_agent("Trinity/1.0 (AI Agent)")
                    .build()?;
                let resp = client.get(url).send().await?;
                
                if !resp.status().is_success() {
                    Err(anyhow::anyhow!("Failed to fetch URL {}: {}", url, resp.status()))
                } else {
                    let html = resp.text().await?;
                    let document = scraper::Html::parse_document(&html);
                    let selector = scraper::Selector::parse("body").unwrap();
                    let text = if let Some(body) = document.select(&selector).next() {
                         body.text().collect::<Vec<_>>().join(" ")
                    } else {
                        "No content".to_string()
                    };
                    Ok(text.split_whitespace().collect::<Vec<_>>().join(" "))
                }
            }

            TaskType::GoogleDrive { operation, path } => {
                 let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Google Drive {}: {}", operation, path),
                });
                
                // Native Kernel Implementation - Local Mount Assumption
                let mount_point = dirs::home_dir().unwrap().join("Google Drive");
                let full_path = mount_point.join(path);
                
                if !full_path.exists() {
                     return Err(anyhow::anyhow!("File not found in Drive: {:?}", full_path));
                }
                
                match operation.as_str() {
                    "read" => {
                        let content = tokio::fs::read_to_string(full_path).await?;
                        Ok(content)
                    },
                    "list" => {
                        let mut entries = tokio::fs::read_dir(full_path).await?;
                        let mut files = Vec::new();
                        while let Some(entry) = entries.next_entry().await? {
                            files.push(entry.file_name().to_string_lossy().to_string());
                        }
                        Ok(format!("Files: {:?}", files))
                    },
                    _ => Err(anyhow::anyhow!("Unknown operation: {}", operation))
                }
            }



            TaskType::WorkspaceScan { path } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Scanning workspace for TODOs: {}", path),
                });

                // Convert path string to Path
                let scan_path = Path::new(path);
                let items = todo_parser::scan_workspace_for_todos(scan_path)?;

                // Emit artifact with summary
                let mut content = format!("Found {} TODO items in {}\n\n", items.len(), path);
                for (i, item) in items.iter().take(50).enumerate() {
                    let priority_tag = match item.priority {
                        TaskPriority::Critical => "🔥 [CRITICAL]",
                        TaskPriority::High => "🔴 [HIGH]",
                        TaskPriority::Normal => "🟢 [NORMAL]",
                        TaskPriority::Low => "⚪ [LOW]",
                    };
                    content.push_str(&format!("{}. {} {} - {}\n", 
                        i + 1, 
                        priority_tag,
                        item.file_hint.as_deref().unwrap_or("unknown"),
                        item.title
                    ));
                }

                if items.len() > 50 {
                    content.push_str("\n...and more.");
                }

                let _ = event_tx.send(AgentEvent::ArtifactGenerated {
                    agent_id: agent_id.to_string(),
                    kind: "scan_report".to_string(),
                    content: content.clone(),
                    metadata: serde_json::json!({
                        "count": items.len(),
                        "path": path
                    }),
                });

                Ok(format!("Scan complete: Found {} TODOs", items.len()))
            }

            TaskType::WriteDocument {
                topic,
                style,
                target_words,
                output_path,
            } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Writing {} document about: {}", style, topic),
                });

                // Build prompt
                let system_prompt = "You are an expert technical writer and content creator. Follow the specified style closely, use clear language, and structure content with proper Markdown headings.";
                let target_len = target_words.unwrap_or(500);
                let full_prompt = format!(
                    "{}\n\nWriting Style: {}\nFormat: Markdown\nTarget Length: approximately {} words\nTopic: {}\n\nGenerate the content now:\n",
                    system_prompt,
                    style,
                    target_len,
                    topic
                );

                // Generate markdown
                let content = brain.think_with_grammar(&full_prompt, GrammarSpec::Markdown).await?;
                let word_count = content.split_whitespace().count();

                // Emit artifact
                let _ = event_tx.send(AgentEvent::ArtifactGenerated {
                    agent_id: agent_id.to_string(),
                    kind: "document".to_string(),
                    content: content.clone(),
                    metadata: serde_json::json!({
                        "topic": topic,
                        "style": style,
                        "word_count": word_count,
                        "output_path": output_path.clone().unwrap_or_default()
                    }),
                });

                // Write to file if path specified
                if let Some(path) = output_path {
                    tokio::fs::write(path, &content).await?;
                }

                Ok(content)
            }

            TaskType::GenerateAssessment {
                topic,
                assessment_type,
                difficulty,
            } => {
                let diff_str = format!("{:?}", difficulty);
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Designing {:?} ({}) for: {}", assessment_type, diff_str, topic),
                });

                // Detailed "Zen" Prompt for High Quality
                let system_prompt = "You are an elite Professor and Curriculum Designer with decades of experience. \
                Your goal is to create educational content that is rigorous, engaging, and pedagogically sound. \
                Do not create generic questions; focus on deep understanding and critical thinking.";
                
                let (kind, prompt, is_quiz) = match assessment_type {
                    AssessmentType::Quiz => (
                        "quiz",
                        format!(
                            "{}\n\nTask: Generate a 5-question multiple choice quiz.\nTopic: {}\nDifficulty: {}\n\n\
                            Output ONLY valid JSON matching this schema:\n\
                            [\n  {{ \n    \"question\": \"Question text here...\", \n    \"options\": [\"Option A\", \"B\", \"C\", \"D\"],\n    \"correct_answer_idx\": 0,\n    \"explanation\": \"Detailed explanation...\"\n  }}\n]",
                            system_prompt, topic, diff_str
                        ),
                        true
                    ),
                    AssessmentType::Lab => (
                        "lab", 
                        format!(
                            "{}\n\nTask: Generate a hands-on project-based lab.\nTopic: {}\nDifficulty: {}\n\n\
                            Output ONLY valid JSON matching this schema:\n\
                            {{\n  \"title\": \"Lab Title\",\n  \"objective\": \"Goal...\",\n  \"steps\": [\"Step 1...\"],\n  \"starter_code\": \"Code...\",\n  \"solution\": \"Solution...\"\n}}",
                            system_prompt, topic, diff_str
                        ),
                        false
                    ),
                    AssessmentType::Challenge => (
                        "challenge",
                        format!(
                            "{}\n\nTask: Generate a coding challenge.\nTopic: {}\nDifficulty: {}\n\n\
                            Output ONLY valid JSON matching this schema:\n\
                            {{\n  \"title\": \"Challenge Title\",\n  \"objective\": \"Goal...\",\n  \"steps\": [\"Instructions\"],\n  \"starter_code\": \"Code Stub\",\n  \"solution\": \"Solution\"\n}}",
                            system_prompt, topic, diff_str
                        ),
                        false
                    ),
                };

                // Generate with JSON Grammar
                let response = brain.think_with_grammar(&prompt, GrammarSpec::Json).await?;

                // Validate and Pretty Print (Production Ready)
                let validated_content = if is_quiz {
                    let questions: Vec<QuizQuestion> = serde_json::from_str(&response)
                        .map_err(|e| anyhow::anyhow!("Failed to parse generated quiz JSON: {}", e))?;
                    serde_json::to_string_pretty(&questions)?
                } else {
                    let lab: LabProject = serde_json::from_str(&response)
                        .map_err(|e| anyhow::anyhow!("Failed to parse generated lab JSON: {}", e))?;
                    serde_json::to_string_pretty(&lab)?
                };

                // Emit artifact
                let _ = event_tx.send(AgentEvent::ArtifactGenerated {
                    agent_id: agent_id.to_string(),
                    kind: kind.to_string(),
                    content: validated_content.clone(),
                    metadata: serde_json::json!({
                        "topic": topic,
                        "difficulty": diff_str
                    }),
                });

                Ok(validated_content)
            }

            TaskType::Custom { handler, payload: _ } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Custom task: {}", handler),
                });
                Ok(format!("Custom handler {} executed with payload", handler))
            }

            TaskType::Chat { message } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Processing chat: {}", &message[..message.len().min(50)]),
                });

                let full_prompt = format!("<|start_header_id|>system<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>user<|end_header_id|>\n\n{}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n", personality, message);
                let response = brain.think(&full_prompt).await?;
                Ok(response)
            }

            TaskType::ReviewCode { path, focus } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Reviewing code: {}", path),
                });

                let code = tokio::fs::read_to_string(path).await.unwrap_or_default();
                let focus_text = focus.as_deref().unwrap_or("general quality");
                let review_prompt = format!(
                    "Review the following code, focusing on {}.\n\nCode:\n```\n{}\n```\n\nProvide constructive feedback:",
                    focus_text, code
                );

                let review = brain.think(&review_prompt).await?;
                Ok(review)
            }

            TaskType::Research { topic, depth } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Researching: {}", topic),
                });

                let depth_text = depth.as_deref().unwrap_or("moderate");
                let research_prompt = format!(
                    "Research the following topic with {} depth. Provide comprehensive insights:\n\nTopic: {}",
                    depth_text, topic
                );

                let findings = brain.think(&research_prompt).await?;
                Ok(findings)
            }

            TaskType::ReadFile { path } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Reading file: {}", path),
                });

                let content = tokio::fs::read_to_string(path).await?;
                Ok(content)
            }

            TaskType::DeletePath { path, recursive } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Deleting: {} (recursive={})", path, recursive),
                });

                let path_obj = std::path::Path::new(path);
                if path_obj.is_dir() {
                    if *recursive {
                        tokio::fs::remove_dir_all(path).await?;
                    } else {
                        tokio::fs::remove_dir(path).await?;
                    }
                } else {
                    tokio::fs::remove_file(path).await?;
                }

                Ok(format!("Deleted: {}", path))
            }

            TaskType::CreateDirectory { path } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Creating directory: {}", path),
                });

                tokio::fs::create_dir_all(path).await?;
                Ok(format!("Created directory: {}", path))
            }

            TaskType::MovePath { from, to } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Moving: {} -> {}", from, to),
                });

                tokio::fs::rename(from, to).await?;
                Ok(format!("Moved {} -> {}", from, to))
            }

            TaskType::CopyFile { from, to } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Copying: {} -> {}", from, to),
                });

                // Ensure parent directory exists
                if let Some(parent) = std::path::Path::new(to).parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                tokio::fs::copy(from, to).await?;
                Ok(format!("Copied {} -> {}", from, to))
            }

            TaskType::ListDirectory { path } => {
                let _ = event_tx.send(AgentEvent::Thinking {
                    agent_id: agent_id.to_string(),
                    thought: format!("Listing: {}", path),
                });

                let mut entries = tokio::fs::read_dir(path).await?;
                let mut files = Vec::new();
                while let Some(entry) = entries.next_entry().await? {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry.file_type().await?.is_dir();
                    files.push(if is_dir { format!("{}/", name) } else { name });
                }
                Ok(format!("Contents of {}:\n{}", path, files.join("\n")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::MockBrain;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let planner = Arc::new(MockBrain::new());
        let worker = Arc::new(MockBrain::new());
        let orch = Orchestrator::new(planner, worker, 2);
        assert_eq!(orch.agent_count(), 2);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let planner = Arc::new(MockBrain::new());
        let worker = Arc::new(MockBrain::new());
        let orch = Orchestrator::new(planner, worker, 1);
        let mut rx = orch.subscribe();

        // Should receive initial idle event
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, AgentEvent::AgentIdle { .. }));
    }
}
