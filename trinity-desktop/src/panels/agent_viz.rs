use bevy_egui::egui;

pub struct AgentVizPanel;

impl AgentVizPanel {
    pub fn show(ui: &mut egui::Ui, agent_count_str: &str) {
        ui.heading("Active Agents");

        // Parse agent count from status string (hacky, but works for bridge v1)
        // Format: "Active Agents: N"
        let count = agent_count_str
            .split_whitespace()
            .last()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(200.0) // Limit height so it doesn't take over
            .show(ui, |ui| {
                if count == 0 {
                    ui.label(egui::RichText::new("No active agents (Idle)").italics());
                } else {
                    for i in 0..count {
                        ui.horizontal(|ui| {
                            ui.label(format!("🤖 Agent #{}", i + 1));
                            ui.label(
                                egui::RichText::new("Thinking...").color(egui::Color32::YELLOW),
                            );
                        });
                    }
                }
            });
    }
}
