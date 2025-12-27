use crate::{KernelBridge, KernelCommand};
use bevy_egui::egui;

pub struct TaskInputPanel;

impl TaskInputPanel {
    pub fn show(ui: &mut egui::Ui, prompt: &mut String, bridge: &KernelBridge) {
        ui.heading("Task Input");
        ui.horizontal(|ui| {
            // Text area for input
            let response = ui.add(
                egui::TextEdit::singleline(prompt)
                    .hint_text("Enter a task for Trinity...")
                    .desired_width(f32::INFINITY),
            );

            // Submit Button
            if (ui.button("Submit").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                && !prompt.trim().is_empty()
            {
                let _ = bridge
                    .command_tx
                    .blocking_send(KernelCommand::SubmitTask(prompt.clone()));
                prompt.clear();
            }
        });
    }
}
