// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Task Queue Panel
//!
//! Displays and manages autonomous tasks in the queue.

use bevy::prelude::*;
use bevy_egui::egui;
use uuid::Uuid;

/// Task status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "⏳",
            TaskStatus::Running => "⚡",
            TaskStatus::Completed => "✅",
            TaskStatus::Failed => "❌",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            TaskStatus::Pending => egui::Color32::GRAY,
            TaskStatus::Running => egui::Color32::YELLOW,
            TaskStatus::Completed => egui::Color32::GREEN,
            TaskStatus::Failed => egui::Color32::RED,
        }
    }
}

/// A task in the queue
#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub progress: Option<f32>, // 0.0 - 1.0
    pub agent: String,         // Which agent is handling this
}

/// Task queue panel state
#[derive(Resource, Default)]
pub struct TaskPanel {
    /// All tasks in the queue
    pub tasks: Vec<QueuedTask>,
    /// Filter by status
    pub filter_status: Option<TaskStatus>,
    /// New task input
    pub new_task_prompt: String,
}

impl TaskPanel {
    /// Add a new task
    pub fn add_task(&mut self, name: String, description: String, agent: String) {
        self.tasks.push(QueuedTask {
            id: Uuid::new_v4(),
            name,
            description,
            status: TaskStatus::Pending,
            progress: None,
            agent,
        });
    }

    /// Update from Brain TaskInfo list
    pub fn update_from_tasks(&mut self, protocol_tasks: Vec<trinity_protocol::task::TaskInfo>) {
        self.tasks.clear();
        for task in protocol_tasks {
            let status = match task.status.as_str() {
                "running" => TaskStatus::Running,
                "completed" => TaskStatus::Completed,
                "failed" => TaskStatus::Failed,
                _ => TaskStatus::Pending,
            };

            self.tasks.push(QueuedTask {
                id: task.id,
                name: task.name,
                description: task.description,
                status,
                progress: if task.status == "running" {
                    Some(0.5)
                } else {
                    None
                },
                agent: task.agent.unwrap_or_else(|| "Brain".to_string()),
            });
        }
    }

    /// Render the task panel UI and return new task if submitted
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<(String, String)> {
        let mut new_task = None;

        ui.heading("📋 Task Queue");
        ui.separator();

        // Summary
        let pending = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .count();
        let running = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Running)
            .count();
        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();

        ui.horizontal(|ui| {
            ui.label(format!("⏳ {} pending", pending));
            ui.label(format!("⚡ {} running", running));
            ui.label(format!("✅ {} done", completed));
        });

        ui.separator();

        // Task list
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for task in &self.tasks {
                    self.render_task(ui, task);
                }

                if self.tasks.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No tasks in queue");
                }
            });

        ui.separator();

        // New task input
        ui.horizontal(|ui| {
            ui.label("New:");
            let _response = ui.add(
                egui::TextEdit::singleline(&mut self.new_task_prompt)
                    .hint_text("Describe a task...")
                    .desired_width(ui.available_width() - 50.0),
            );

            if ui.button("➕").clicked() && !self.new_task_prompt.is_empty() {
                // Return the task for submission
                new_task = Some((self.new_task_prompt.clone(), "Generic Task".to_string()));
                // We optimistically add it to UI?
                // Better to wait for confirmation, but let's add it as 'Pending' locally
                self.add_task(
                    self.new_task_prompt.clone(),
                    String::new(),
                    "Brain".to_string(), // It's going to the Brain
                );
                self.new_task_prompt.clear();
            }
        });

        new_task
    }

    fn render_task(&self, ui: &mut egui::Ui, task: &QueuedTask) {
        ui.horizontal(|ui| {
            ui.label(task.status.icon());
            ui.colored_label(task.status.color(), &task.name);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("@{}", task.agent));
            });
        });

        if let Some(progress) = task.progress {
            ui.add(egui::ProgressBar::new(progress).show_percentage());
        }

        ui.add_space(4.0);
    }
}
