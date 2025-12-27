use bevy_egui::egui;

pub struct LogConsolePanel;

impl LogConsolePanel {
    pub fn show(ui: &mut egui::Ui, logs: &[String]) {
        ui.heading("System Logs");
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2]) // Expand to fill available space
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for log in logs {
                    // Colorize logs slightly based on content
                    let text = if log.starts_with("Kernel Panic") {
                        egui::RichText::new(log).color(egui::Color32::RED)
                    } else if log.starts_with("DONE") {
                        egui::RichText::new(log).color(egui::Color32::GREEN)
                    } else {
                        egui::RichText::new(log)
                    };

                    ui.label(text);
                }
            });
    }
}
