//! Safety Module - Kill Switch and Rollback Logic
//!
//! ╔═══════════════════════════════════════════════════════════════════════════╗
//! ║                          ⚠️  IMMUTABLE FILE  ⚠️                            ║
//! ║                                                                            ║
//! ║  This file is PROTECTED and cannot be modified by the autopoietic loop.  ║
//! ║  It contains the kill switch and rollback logic that prevents Trinity     ║
//! ║  from entering an unrecoverable state.                                    ║
//! ║                                                                            ║
//! ║  If you need to modify this file, do so MANUALLY with extreme caution.   ║
//! ╚═══════════════════════════════════════════════════════════════════════════╝
//!
//! ## Philosophy
//! "The safety module is the conscience of the system. It has the power to
//!  halt all self-modification and restore a known-good state. Without it,
//!  autopoiesis becomes a crash loop."

use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

// ============================================================================
// Kill Switch
// ============================================================================

/// Global kill switch state
static KILL_SWITCH_ACTIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Check if the kill switch is active
pub fn is_kill_switch_active() -> bool {
    KILL_SWITCH_ACTIVE.load(std::sync::atomic::Ordering::SeqCst)
}

/// Activate the kill switch - halts all autopoietic operations
pub fn activate_kill_switch(reason: &str) {
    error!("🛑 KILL SWITCH ACTIVATED: {}", reason);
    KILL_SWITCH_ACTIVE.store(true, std::sync::atomic::Ordering::SeqCst);

    // Write to file for persistence across restarts
    let kill_file = PathBuf::from("/home/joshua/antigravity/trinity_genesis/KILL_SWITCH");
    if let Err(e) = std::fs::write(&kill_file, reason) {
        error!("Failed to write kill switch file: {}", e);
    }
}

/// Deactivate the kill switch (manual intervention required)
pub fn deactivate_kill_switch() {
    warn!("Kill switch deactivated by manual intervention");
    KILL_SWITCH_ACTIVE.store(false, std::sync::atomic::Ordering::SeqCst);

    // Remove the kill file
    let kill_file = PathBuf::from("/home/joshua/antigravity/trinity_genesis/KILL_SWITCH");
    let _ = std::fs::remove_file(&kill_file);
}

/// Check for persistent kill switch on startup
pub fn check_persistent_kill_switch() -> bool {
    let kill_file = PathBuf::from("/home/joshua/antigravity/trinity_genesis/KILL_SWITCH");
    if kill_file.exists() {
        if let Ok(reason) = std::fs::read_to_string(&kill_file) {
            error!("Persistent kill switch found: {}", reason);
            KILL_SWITCH_ACTIVE.store(true, std::sync::atomic::Ordering::SeqCst);
            return true;
        }
    }
    false
}

// ============================================================================
// Rollback Capability
// ============================================================================

/// Emergency rollback to last known good version
pub fn emergency_rollback(backup_dir: &Path, target_dir: &Path) -> Result<()> {
    info!("🔄 EMERGENCY ROLLBACK initiated");

    // Find the latest backup
    let mut latest_version = 0u64;
    let mut latest_path: Option<PathBuf> = None;

    for entry in std::fs::read_dir(backup_dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if let Some(version_str) = name.strip_prefix("v") {
                if let Ok(version) = version_str.parse::<u64>() {
                    if version > latest_version {
                        latest_version = version;
                        latest_path = Some(entry.path());
                    }
                }
            }
        }
    }

    match latest_path {
        Some(backup_path) => {
            info!("Rolling back to v{}...", latest_version);

            // Copy backup crates to target
            let backup_crates = backup_path.join("crates");
            let target_crates = target_dir.join("crates");

            copy_dir_recursive(&backup_crates, &target_crates)?;

            info!("✓ Emergency rollback complete to v{}", latest_version);
            Ok(())
        }
        None => {
            error!("No backups found for emergency rollback!");
            anyhow::bail!("No backups available")
        }
    }
}

/// Quick health check for compilation
pub fn compilation_health_check(source_dir: &Path) -> bool {
    info!("Running compilation health check...");

    let output = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(source_dir)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                info!("✓ Compilation health check passed");
                true
            } else {
                error!("✗ Compilation health check FAILED");
                let stderr = String::from_utf8_lossy(&result.stderr);
                error!("Error: {}", stderr);
                false
            }
        }
        Err(e) => {
            error!("Failed to run health check: {}", e);
            false
        }
    }
}

// ============================================================================
// Failure Tracking
// ============================================================================

/// Track consecutive failures (persisted to file)
pub struct FailureTracker {
    file_path: PathBuf,
    max_failures: u32,
}

impl FailureTracker {
    /// Create a new failure tracker
    pub fn new(file_path: impl Into<PathBuf>, max_failures: u32) -> Self {
        Self {
            file_path: file_path.into(),
            max_failures,
        }
    }

    /// Record a failure
    pub fn record_failure(&self) -> Result<u32> {
        let current = self.get_count();
        let new_count = current + 1;
        std::fs::write(&self.file_path, new_count.to_string())?;

        if new_count >= self.max_failures {
            activate_kill_switch(&format!(
                "Too many consecutive failures: {} >= {}",
                new_count, self.max_failures
            ));
        }

        Ok(new_count)
    }

    /// Record a success (resets counter)
    pub fn record_success(&self) -> Result<()> {
        std::fs::write(&self.file_path, "0")?;
        Ok(())
    }

    /// Get current failure count
    pub fn get_count(&self) -> u32 {
        std::fs::read_to_string(&self.file_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    /// Check if we've exceeded the limit
    pub fn is_exceeded(&self) -> bool {
        self.get_count() >= self.max_failures
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

// ============================================================================
// Integrity Validation
// ============================================================================

/// Validate critical files haven't been corrupted
pub fn validate_critical_integrity() -> Result<()> {
    // Check that key files exist
    let critical_files = [
        "/home/joshua/antigravity/trinity-genesis/Cargo.toml",
        "/home/joshua/antigravity/trinity-genesis/crates/trinity-kernel/Cargo.toml",
        "/home/joshua/antigravity/trinity-genesis/crates/trinity-brain/Cargo.toml",
    ];

    for file in critical_files {
        if !Path::new(file).exists() {
            activate_kill_switch(&format!("Critical file missing: {}", file));
            anyhow::bail!("Critical file missing: {}", file);
        }
    }

    info!("✓ Critical file integrity validated");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_switch() {
        assert!(!is_kill_switch_active());

        // Note: Don't actually activate in tests as it writes to filesystem
        // Just test the atomic operations work
    }

    #[test]
    fn test_failure_tracker() {
        use tempfile::NamedTempFile;

        let file = NamedTempFile::new().unwrap();
        let tracker = FailureTracker::new(file.path(), 3);

        assert_eq!(tracker.get_count(), 0);
        assert!(!tracker.is_exceeded());
    }
}
