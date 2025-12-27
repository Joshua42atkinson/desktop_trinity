use bevy_egui::egui;

pub struct DashboardPanel;

impl DashboardPanel {
    pub fn show(ui: &mut egui::Ui, status: &str) {
        egui::Grid::new("system_stats")
            .striped(true)
            .min_col_width(100.0)
            .show(ui, |ui| {
                ui.heading("System Status");
                ui.end_row();

                ui.label("Kernel Status:");
                ui.label(status);
                ui.end_row();

                // Placeholder Stats - these would ideally come from the KernelBridge resource
                ui.label("VRAM Usage:");
                ui.label("75.4 / 128.0 GB (Inference Active)"); // Mock for now
                ui.end_row();

                ui.label("Compute:");
                ui.label("AMD Ryzen AI Max+ 395 (16C/32T)");
                ui.end_row();

                ui.label("NPU:");
                ui.label("XDNA 2 - 50 TOPS (Idle)");
                ui.end_row();
            });
    }
}
