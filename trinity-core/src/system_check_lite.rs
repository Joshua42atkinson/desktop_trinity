//! Lightweight system check (no candle dependencies)

use anyhow::Result;

/// Basic system check without candle/inference requirements
pub struct SystemCheckLite;

impl SystemCheckLite {
    /// Run basic system checks
    pub fn run() -> Result<()> {
        tracing::info!("Running lightweight system checks (inference disabled)...");

        // Check available memory
        Self::check_memory()?;

        tracing::info!("Lightweight system checks passed");
        Ok(())
    }

    fn check_memory() -> Result<()> {
        // Basic memory check using /proc/meminfo on Linux
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        tracing::info!("System: {}", line.trim());
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
