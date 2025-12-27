//! # Antigravity Window (The Inner Eye)
//!
//! ## Philosophy
//! "To trust the machine, we must see it think. The Antigravity Window acts as
//!  a glass-bottom boat over the ocean of the AI's cognition. It reveals the
//!  hidden currents of thought, code generation, and task scheduling."
//!
//! ## Self-Employment
//! This panel enables Trinity's autonomous work capability. You can:
//! - Submit tasks for Trinity to work on
//! - Watch agents think and code in real-time
//! - Walk away while Trinity keeps working

use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::VecDeque;
use trinity_protocol::artifact::{AgentMode, Artifact};
use trinity_protocol::stream::{AgentStatus, ModelTier, StreamEvent};

/// Maximum events to keep in the log
const MAX_EVENTS: usize = 100;

/// Antigravity Window panel resource
#[derive(Resource)]
pub struct AntigravityPanel {
    /// Whether the panel is visible
    pub visible: bool,
    /// Current thought stream buffer
    pub thought_buffer: String,
    /// Current code being generated
    pub code_buffer: String,
    /// File being edited
    pub current_file: Option<String>,
    /// Active agents status
    pub agents: Vec<AgentStatus>,
    /// Event log
    events: VecDeque<EventLogEntry>,
    /// Build output
    pub build_output: String,

    // === Self-Employment: Task Submission ===
    /// New task name input
    pub new_task_name: String,
    /// New task description input
    pub new_task_description: String,
    /// Selected task type for new task
    pub new_task_type: TaskTypeSelection,
    /// Flag to signal task submission request
    pub submit_task_requested: bool,
    /// Queue status from brain
    pub queue_pending: usize,
    pub queue_running: usize,
    pub queue_completed: usize,

    // === Artifact System ===
    /// Current agent mode (Planning vs Fast)
    pub agent_mode: AgentMode,
    /// Accumulated artifacts from the current session
    pub artifacts: Vec<ArtifactEntry>,
    /// Index of currently selected artifact
    pub selected_artifact: Option<usize>,
}

/// An artifact entry for display
#[derive(Clone)]
pub struct ArtifactEntry {
    pub timestamp: String,
    pub agent_id: String,
    pub artifact: Artifact,
}

/// Task type selection for UI
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TaskTypeSelection {
    #[default]
    GenerateCode,
    WriteDocument,
    Research,
    ReviewCode,
    Custom,
}

/// A logged event for display
#[derive(Clone)]
struct EventLogEntry {
    timestamp: String,
    agent_id: String,
    message: String,
    event_type: EventType,
}

#[derive(Clone, Copy)]
enum EventType {
    Info,
    Thinking,
    Code,
    Command,
    Success,
    Error,
    Artifact,
    ModeChange,
}

impl Default for AntigravityPanel {
    fn default() -> Self {
        Self {
            visible: false,
            thought_buffer: String::new(),
            code_buffer: String::new(),
            current_file: None,
            agents: vec![
                AgentStatus {
                    id: "agent-0".to_string(),
                    name: "Coder 1".to_string(),
                    model_tier: ModelTier::Standard,
                    is_busy: false,
                    current_task: None,
                },
                AgentStatus {
                    id: "agent-1".to_string(),
                    name: "Coder 2".to_string(),
                    model_tier: ModelTier::Standard,
                    is_busy: false,
                    current_task: None,
                },
            ],
            events: VecDeque::new(),
            build_output: String::new(),
            // Self-Employment defaults
            new_task_name: String::new(),
            new_task_description: String::new(),
            new_task_type: TaskTypeSelection::default(),
            submit_task_requested: false,
            queue_pending: 0,
            queue_running: 0,
            queue_completed: 0,
            // Artifact System defaults
            agent_mode: AgentMode::default(),
            artifacts: Vec::new(),
            selected_artifact: None,
        }
    }
}

impl AntigravityPanel {
    /// Process a stream event
    pub fn process_event(&mut self, event: StreamEvent) {
        let now = chrono::Local::now().format("%H:%M:%S").to_string();

        match event {
            StreamEvent::TaskStarted {
                agent_id,
                task_name,
                ..
            } => {
                self.log_event(
                    &now,
                    &agent_id,
                    format!("Started: {}", task_name),
                    EventType::Info,
                );
                // Update agent status
                if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
                    agent.is_busy = true;
                    agent.current_task = Some(task_name);
                }
            }

            StreamEvent::Thinking { agent_id, thought } => {
                self.thought_buffer = thought.clone();
                self.log_event(
                    &now,
                    &agent_id,
                    format!("💭 {}", thought),
                    EventType::Thinking,
                );
            }

            StreamEvent::CodeGenerated {
                agent_id,
                file_path,
                code_snippet,
                line_count,
            } => {
                self.current_file = Some(file_path.clone());
                self.code_buffer = code_snippet;
                self.log_event(
                    &now,
                    &agent_id,
                    format!("📝 {} ({} lines)", file_path, line_count),
                    EventType::Code,
                );
            }

            StreamEvent::CommandRunning { agent_id, command } => {
                self.log_event(
                    &now,
                    &agent_id,
                    format!("▶ {}", command),
                    EventType::Command,
                );
            }

            StreamEvent::CommandOutput {
                agent_id,
                stdout,
                stderr,
            } => {
                self.build_output = if stderr.is_empty() { stdout } else { stderr };
                self.log_event(
                    &now,
                    &agent_id,
                    "Command output received".to_string(),
                    EventType::Info,
                );
            }

            StreamEvent::TaskCompleted {
                agent_id,
                duration_ms,
                ..
            } => {
                self.log_event(
                    &now,
                    &agent_id,
                    format!("✅ Completed in {}ms", duration_ms),
                    EventType::Success,
                );
                // Update agent status
                if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
                    agent.is_busy = false;
                    agent.current_task = None;
                }
                // Clear buffers
                self.thought_buffer.clear();
            }

            StreamEvent::TaskFailed {
                agent_id, error, ..
            } => {
                self.log_event(
                    &now,
                    &agent_id,
                    format!("❌ Failed: {}", error),
                    EventType::Error,
                );
                if let Some(agent) = self.agents.iter_mut().find(|a| a.id == agent_id) {
                    agent.is_busy = false;
                    agent.current_task = None;
                }
            }

            StreamEvent::AgentStatusUpdate { agents } => {
                self.agents = agents;
            }

            StreamEvent::ArtifactGenerated { agent_id, artifact } => {
                self.artifacts.push(ArtifactEntry {
                    timestamp: now.clone(),
                    agent_id: agent_id.clone(),
                    artifact: artifact.clone(),
                });
                self.log_event(
                    &now,
                    &agent_id,
                    format!("📦 Artifact: {}", artifact.kind_name()),
                    EventType::Artifact,
                );
            }

            StreamEvent::ModeChanged {
                agent_id,
                mode,
                reason,
            } => {
                self.agent_mode = mode;
                let mode_str = match mode {
                    AgentMode::Fast => "⚡ Fast",
                    AgentMode::Planning => "📋 Planning",
                    AgentMode::Autonomous => "🤖 Autonomous",
                };
                let msg = match reason {
                    Some(r) => format!("Mode: {} ({})", mode_str, r),
                    None => format!("Mode: {}", mode_str),
                };
                self.log_event(&now, &agent_id, msg, EventType::ModeChange);
            }
        }
    }

    fn log_event(
        &mut self,
        timestamp: &str,
        agent_id: &str,
        message: String,
        event_type: EventType,
    ) {
        self.events.push_front(EventLogEntry {
            timestamp: timestamp.to_string(),
            agent_id: agent_id.to_string(),
            message,
            event_type,
        });
        if self.events.len() > MAX_EVENTS {
            self.events.pop_back();
        }
    }

    /// Render the panel UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 Antigravity Window");
        ui.label("Submit tasks and watch Trinity work autonomously");
        ui.separator();

        // === SELF-EMPLOYMENT: Task Submission ===
        ui.collapsing("📋 Submit New Task", |ui| {
            ui.horizontal(|ui| {
                ui.label("Task Name:");
                ui.text_edit_singleline(&mut self.new_task_name);
            });

            ui.horizontal(|ui| {
                ui.label("Type:");
                egui::ComboBox::from_id_source("task_type")
                    .selected_text(format!("{:?}", self.new_task_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.new_task_type,
                            TaskTypeSelection::GenerateCode,
                            "Generate Code",
                        );
                        ui.selectable_value(
                            &mut self.new_task_type,
                            TaskTypeSelection::WriteDocument,
                            "Write Document",
                        );
                        ui.selectable_value(
                            &mut self.new_task_type,
                            TaskTypeSelection::Research,
                            "Research",
                        );
                        ui.selectable_value(
                            &mut self.new_task_type,
                            TaskTypeSelection::ReviewCode,
                            "Review Code",
                        );
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Description:");
            });
            ui.add(
                egui::TextEdit::multiline(&mut self.new_task_description)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            );

            ui.horizontal(|ui| {
                if ui.button("🚀 Submit Task").clicked() && !self.new_task_name.is_empty() {
                    self.submit_task_requested = true;
                }
                if !self.new_task_name.is_empty() {
                    ui.label(egui::RichText::new("Ready to submit").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("Enter task name").color(egui::Color32::GRAY));
                }
            });
        });

        // Queue Status
        ui.horizontal(|ui| {
            ui.label(format!(
                "📊 Queue: {} pending | {} running | {} completed",
                self.queue_pending, self.queue_running, self.queue_completed
            ));
        });

        ui.separator();

        // Agent Status Row
        ui.horizontal(|ui| {
            for agent in &self.agents {
                let (color, icon) = if agent.is_busy {
                    (egui::Color32::from_rgb(100, 255, 100), "⚡")
                } else {
                    (egui::Color32::from_rgb(150, 150, 150), "💤")
                };

                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.colored_label(color, format!("{} {}", icon, agent.name));
                        ui.label(format!("Tier: {:?}", agent.model_tier));
                        if let Some(ref task) = agent.current_task {
                            ui.label(format!("→ {}", truncate(task, 20)));
                        }
                    });
                });
            }
        });

        ui.separator();

        // Two-column layout
        ui.columns(2, |cols| {
            // Left column: Thought Stream
            cols[0].heading("💭 Thought Stream");
            egui::ScrollArea::vertical()
                .id_source("thoughts")
                .max_height(150.0)
                .show(&mut cols[0], |ui: &mut egui::Ui| {
                    if self.thought_buffer.is_empty() {
                        ui.label(egui::RichText::new("Waiting for activity...").italics());
                    } else {
                        ui.label(&self.thought_buffer);
                    }
                });

            // Right column: Code Preview
            cols[1].heading(format!(
                "📝 Code: {}",
                self.current_file.as_deref().unwrap_or("--")
            ));
            egui::ScrollArea::vertical()
                .id_source("code")
                .max_height(150.0)
                .show(&mut cols[1], |ui: &mut egui::Ui| {
                    if self.code_buffer.is_empty() {
                        ui.label(egui::RichText::new("No code generated yet").italics());
                    } else {
                        ui.code(&self.code_buffer);
                    }
                });
        });

        ui.separator();

        // Event Log
        ui.heading("📋 Event Log");
        egui::ScrollArea::vertical()
            .id_source("events")
            .max_height(120.0)
            .stick_to_bottom(false)
            .show(ui, |ui: &mut egui::Ui| {
                for entry in &self.events {
                    let color = match entry.event_type {
                        EventType::Info => egui::Color32::GRAY,
                        EventType::Thinking => egui::Color32::from_rgb(150, 150, 255),
                        EventType::Code => egui::Color32::from_rgb(100, 255, 150),
                        EventType::Command => egui::Color32::from_rgb(255, 200, 100),
                        EventType::Success => egui::Color32::from_rgb(100, 255, 100),
                        EventType::Error => egui::Color32::from_rgb(255, 100, 100),
                        EventType::Artifact => egui::Color32::from_rgb(200, 150, 255), // Purple
                        EventType::ModeChange => egui::Color32::from_rgb(100, 200, 255), // Cyan
                    };
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&entry.timestamp).small());
                        ui.label(egui::RichText::new(format!("[{}]", entry.agent_id)).small());
                        ui.colored_label(color, &entry.message);
                    });
                }
            });

        // Artifact Gallery
        ui.separator();
        ui.heading("📦 Artifact Gallery");

        if self.artifacts.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No artifacts generated yet.");
        } else {
            ui.horizontal(|ui| {
                ui.label("History:");
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    for (i, entry) in self.artifacts.iter().enumerate() {
                        let is_selected = self.selected_artifact == Some(i);
                        let label = format!("{} {}", i + 1, entry.artifact.kind_name());
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.selected_artifact = Some(i);
                        }
                    }
                });
            });

            if let Some(idx) = self.selected_artifact {
                if let Some(entry) = self.artifacts.get(idx) {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} - Produced by {}",
                                    entry.timestamp, entry.agent_id
                                ))
                                .small(),
                            );
                            if ui.button("📋 Copy").clicked() {
                                // In a real app we'd copy to clipboard
                            }
                        });

                        egui::ScrollArea::vertical()
                            .id_source("artifact_view")
                            .max_height(300.0)
                            .show(ui, |ui| {
                                match &entry.artifact {
                                    Artifact::Code {
                                        content,
                                        language,
                                        file_path,
                                        ..
                                    } => {
                                        ui.label(format!(
                                            "File: {}",
                                            file_path.as_deref().unwrap_or("snippet")
                                        ));
                                        ui.code(content);
                                    }
                                    Artifact::Text { content, .. } => {
                                        ui.label(content);
                                    }
                                    _ => {
                                        // Generic rendering for others
                                        ui.label(format!("Type: {}", entry.artifact.kind_name()));
                                        ui.code(format!("{:?}", entry.artifact));
                                    }
                                }
                            });
                    });
                }
            }
        }

        ui.add_space(8.0);
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
