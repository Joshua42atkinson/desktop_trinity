//! Systemd Control via D-Bus
//!
//! Provides integration with systemd for service management.
//! Allows Trinity to manage its own lifecycle and other services.
//!
//! ## Philosophy
//! "The Will must control itself. An agent that cannot restart
//!  its own service is not truly autonomous."
//!
//! ## Usage
//! ```rust,ignore
//! let controller = SystemdController::connect().await?;
//! controller.restart_unit("trinity-brain.service").await?;
//! ```

use anyhow::{Context, Result};
use std::process::Command;
use tracing::{debug, info, warn};

/// Controller for systemd service management
pub struct SystemdController {
    /// Whether running as user or system service
    user_mode: bool,
}

impl SystemdController {
    /// Create a new systemd controller
    ///
    /// Defaults to user mode (systemctl --user) which is appropriate
    /// for Trinity running under a user session.
    pub fn new() -> Self {
        Self { user_mode: true }
    }

    /// Create controller for system-level services (requires root)
    pub fn system() -> Self {
        Self { user_mode: false }
    }

    /// Build the base systemctl command
    fn systemctl(&self) -> Command {
        let mut cmd = Command::new("systemctl");
        if self.user_mode {
            cmd.arg("--user");
        }
        cmd
    }

    /// Start a systemd unit
    pub fn start_unit(&self, unit_name: &str) -> Result<()> {
        info!("Starting systemd unit: {}", unit_name);

        let output = self
            .systemctl()
            .arg("start")
            .arg(unit_name)
            .output()
            .context("Failed to execute systemctl start")?;

        if output.status.success() {
            info!("✓ Started: {}", unit_name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to start {}: {}", unit_name, stderr)
        }
    }

    /// Stop a systemd unit
    pub fn stop_unit(&self, unit_name: &str) -> Result<()> {
        info!("Stopping systemd unit: {}", unit_name);

        let output = self
            .systemctl()
            .arg("stop")
            .arg(unit_name)
            .output()
            .context("Failed to execute systemctl stop")?;

        if output.status.success() {
            info!("✓ Stopped: {}", unit_name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to stop {}: {}", unit_name, stderr)
        }
    }

    /// Restart a systemd unit
    pub fn restart_unit(&self, unit_name: &str) -> Result<()> {
        info!("Restarting systemd unit: {}", unit_name);

        let output = self
            .systemctl()
            .arg("restart")
            .arg(unit_name)
            .output()
            .context("Failed to execute systemctl restart")?;

        if output.status.success() {
            info!("✓ Restarted: {}", unit_name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to restart {}: {}", unit_name, stderr)
        }
    }

    /// Get the status of a systemd unit
    pub fn get_status(&self, unit_name: &str) -> Result<UnitStatus> {
        debug!("Getting status for: {}", unit_name);

        let output = self
            .systemctl()
            .arg("is-active")
            .arg(unit_name)
            .output()
            .context("Failed to execute systemctl is-active")?;

        let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let status = match status_str.as_str() {
            "active" => UnitStatus::Active,
            "inactive" => UnitStatus::Inactive,
            "failed" => UnitStatus::Failed,
            "activating" => UnitStatus::Activating,
            "deactivating" => UnitStatus::Deactivating,
            _ => UnitStatus::Unknown(status_str),
        };

        debug!("Unit {} status: {:?}", unit_name, status);
        Ok(status)
    }

    /// Check if a unit is active
    pub fn is_active(&self, unit_name: &str) -> bool {
        matches!(self.get_status(unit_name), Ok(UnitStatus::Active))
    }

    /// Restart Trinity's own brain service
    ///
    /// This is the "self-healing" capability that allows Trinity
    /// to recover from certain failure states.
    pub fn restart_self(&self) -> Result<()> {
        warn!("Trinity is restarting itself...");
        self.restart_unit("trinity-brain.service")
    }

    /// Schedule a restart after a delay (useful for graceful shutdown)
    pub fn schedule_restart(&self, delay_secs: u32) -> Result<()> {
        info!("Scheduling restart in {} seconds", delay_secs);

        // Use systemd-run to schedule a restart
        let output = Command::new("systemd-run")
            .args(if self.user_mode {
                vec!["--user"]
            } else {
                vec![]
            })
            .arg("--on-active")
            .arg(format!("{}s", delay_secs))
            .arg("--unit=trinity-restart-scheduled")
            .arg("systemctl")
            .args(if self.user_mode {
                vec!["--user"]
            } else {
                vec![]
            })
            .arg("restart")
            .arg("trinity-brain.service")
            .output()
            .context("Failed to schedule restart")?;

        if output.status.success() {
            info!("✓ Restart scheduled for {} seconds from now", delay_secs);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to schedule restart: {}", stderr)
        }
    }

    /// Enable a unit to start at boot/login
    pub fn enable_unit(&self, unit_name: &str) -> Result<()> {
        info!("Enabling unit: {}", unit_name);

        let output = self
            .systemctl()
            .arg("enable")
            .arg(unit_name)
            .output()
            .context("Failed to execute systemctl enable")?;

        if output.status.success() {
            info!("✓ Enabled: {}", unit_name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to enable {}: {}", unit_name, stderr)
        }
    }

    /// Disable a unit from starting at boot/login
    pub fn disable_unit(&self, unit_name: &str) -> Result<()> {
        info!("Disabling unit: {}", unit_name);

        let output = self
            .systemctl()
            .arg("disable")
            .arg(unit_name)
            .output()
            .context("Failed to execute systemctl disable")?;

        if output.status.success() {
            info!("✓ Disabled: {}", unit_name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to disable {}: {}", unit_name, stderr)
        }
    }

    /// Reload systemd daemon (needed after unit file changes)
    pub fn daemon_reload(&self) -> Result<()> {
        info!("Reloading systemd daemon");

        let output = self
            .systemctl()
            .arg("daemon-reload")
            .output()
            .context("Failed to execute systemctl daemon-reload")?;

        if output.status.success() {
            info!("✓ Daemon reloaded");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to reload daemon: {}", stderr)
        }
    }
}

impl Default for SystemdController {
    fn default() -> Self {
        Self::new()
    }
}

/// Status of a systemd unit
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitStatus {
    /// Unit is running
    Active,
    /// Unit is stopped
    Inactive,
    /// Unit failed to start or crashed
    Failed,
    /// Unit is starting up
    Activating,
    /// Unit is shutting down
    Deactivating,
    /// Unknown status
    Unknown(String),
}

impl UnitStatus {
    /// Check if the unit is in a healthy state
    pub fn is_healthy(&self) -> bool {
        matches!(self, UnitStatus::Active)
    }

    /// Check if the unit needs attention
    pub fn needs_attention(&self) -> bool {
        matches!(self, UnitStatus::Failed | UnitStatus::Unknown(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_status_health() {
        assert!(UnitStatus::Active.is_healthy());
        assert!(!UnitStatus::Inactive.is_healthy());
        assert!(!UnitStatus::Failed.is_healthy());
    }

    #[test]
    fn test_unit_status_attention() {
        assert!(UnitStatus::Failed.needs_attention());
        assert!(UnitStatus::Unknown("weird".to_string()).needs_attention());
        assert!(!UnitStatus::Active.needs_attention());
        assert!(!UnitStatus::Inactive.needs_attention());
    }
}
