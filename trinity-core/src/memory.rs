//! Unified Memory Manager for AMD Strix Halo
//!
//! Manages the 128GB unified memory, with up to 96GB allocatable as VRAM.
//! Integrates with sysinfo for real-time system memory tracking.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use sysinfo::System;

use crate::config::HardwareConfig;

// ============================================================================
// Memory Stats (for UI and monitoring)
// ============================================================================

/// Memory statistics for display and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total system RAM in GB
    pub total_gb: f64,
    /// VRAM allocation limit in GB (from BIOS/config)
    pub vram_limit_gb: f64,
    /// Memory allocated by Trinity in GB (tracked internally)
    pub allocated_gb: f64,
    /// Available VRAM for new allocations in GB
    pub available_gb: f64,
    /// Actual system memory used in GB (from sysinfo)
    pub system_used_gb: f64,
    /// Actual system memory free in GB (from sysinfo)
    pub system_free_gb: f64,
    /// Memory pressure percentage (0-100)
    pub pressure_percent: f64,
}

impl MemoryStats {
    /// Check if memory is under pressure (>80% used)
    pub fn is_under_pressure(&self) -> bool {
        self.pressure_percent > 80.0
    }

    /// Check if we're critically low (<10% free)
    pub fn is_critical(&self) -> bool {
        self.pressure_percent > 90.0
    }
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory: {:.1}/{:.1} GB allocated ({:.1} GB available, {:.1} GB VRAM limit)",
            self.allocated_gb, self.total_gb, self.available_gb, self.vram_limit_gb
        )
    }
}

// ============================================================================
// Unified Memory Manager
// ============================================================================

/// Unified Memory Manager for Strix Halo's 128GB unified memory
#[derive(Resource)]
pub struct UnifiedMemoryManager {
    /// Total system memory in bytes
    total_memory: u64,
    /// Currently allocated memory (Trinity's internal tracking)
    allocated: AtomicU64,
    /// VRAM allocation limit (configured in BIOS)
    vram_limit: u64,
    /// System info handle for real-time queries
    sysinfo: std::sync::Mutex<System>,
}

impl UnifiedMemoryManager {
    /// Create a new memory manager
    ///
    /// # Arguments
    /// * `total_gb` - Total system RAM in GB (typically 128)
    /// * `vram_limit_gb` - VRAM allocation limit from BIOS (up to 96GB)
    pub fn new(total_gb: u64, vram_limit_gb: u64) -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();

        Self {
            total_memory: total_gb * 1024 * 1024 * 1024,
            allocated: AtomicU64::new(0),
            vram_limit: vram_limit_gb * 1024 * 1024 * 1024,
            sysinfo: std::sync::Mutex::new(sys),
        }
    }

    /// Create from hardware config
    pub fn from_config(config: &HardwareConfig) -> Self {
        let vram_gb = config.max_vram_gb.unwrap_or(96.0) as u64;

        // Detect total system memory
        let mut sys = System::new_all();
        sys.refresh_memory();
        let total_gb = sys.total_memory() / (1024 * 1024 * 1024);

        tracing::info!(
            "Memory Manager: {} GB total, {} GB VRAM limit",
            total_gb,
            vram_gb
        );

        Self {
            total_memory: sys.total_memory(),
            allocated: AtomicU64::new(0),
            vram_limit: vram_gb * 1024 * 1024 * 1024,
            sysinfo: std::sync::Mutex::new(sys),
        }
    }

    /// Default configuration for Strix Halo (128GB total, 96GB VRAM)
    pub fn strix_halo_default() -> Self {
        Self::new(128, 96)
    }

    /// Check if we can allocate the requested amount
    pub fn can_allocate(&self, bytes: u64) -> bool {
        let current = self.allocated.load(Ordering::Relaxed);
        current + bytes <= self.vram_limit
    }

    /// Try to allocate memory, returns true if successful
    pub fn try_allocate(&self, bytes: u64) -> bool {
        let current = self.allocated.load(Ordering::Relaxed);
        if current + bytes > self.vram_limit {
            return false;
        }
        self.allocated.fetch_add(bytes, Ordering::Relaxed);
        true
    }

    /// Free allocated memory
    pub fn free(&self, bytes: u64) {
        self.allocated.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Get current allocation in bytes
    pub fn allocated_bytes(&self) -> u64 {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Get available memory in bytes
    pub fn available_bytes(&self) -> u64 {
        self.vram_limit - self.allocated.load(Ordering::Relaxed)
    }

    /// Refresh system memory stats (call periodically for real-time data)
    pub fn refresh(&self) {
        if let Ok(mut sys) = self.sysinfo.try_lock() {
            sys.refresh_memory();
        }
    }

    /// Get real-time memory statistics
    pub fn stats(&self) -> MemoryStats {
        let allocated = self.allocated.load(Ordering::Relaxed);

        // Get real system stats
        let (system_used, system_free, total) = if let Ok(sys) = self.sysinfo.try_lock() {
            (
                sys.used_memory(),
                sys.available_memory(),
                sys.total_memory(),
            )
        } else {
            // Fallback if lock fails
            (0, self.total_memory, self.total_memory)
        };

        let total_gb = total as f64 / (1024.0 * 1024.0 * 1024.0);
        let system_used_gb = system_used as f64 / (1024.0 * 1024.0 * 1024.0);
        let system_free_gb = system_free as f64 / (1024.0 * 1024.0 * 1024.0);
        let pressure = if total > 0 {
            (system_used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        MemoryStats {
            total_gb,
            vram_limit_gb: self.vram_limit as f64 / (1024.0 * 1024.0 * 1024.0),
            allocated_gb: allocated as f64 / (1024.0 * 1024.0 * 1024.0),
            available_gb: (self.vram_limit - allocated) as f64 / (1024.0 * 1024.0 * 1024.0),
            system_used_gb,
            system_free_gb,
            pressure_percent: pressure,
        }
    }

    /// Get real-time stats with a refresh
    pub fn stats_live(&self) -> MemoryStats {
        self.refresh();
        self.stats()
    }

    /// Get VRAM limit in GB
    pub fn vram_limit_gb(&self) -> f64 {
        self.vram_limit as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Get total system memory in GB
    pub fn total_memory_gb(&self) -> f64 {
        self.total_memory as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

impl Default for UnifiedMemoryManager {
    fn default() -> Self {
        Self::strix_halo_default()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_allocation() {
        let mgr = UnifiedMemoryManager::new(128, 96);

        // Allocate 32GB
        assert!(mgr.try_allocate(32 * 1024 * 1024 * 1024));

        // Should have ~64GB available
        let stats = mgr.stats();
        assert!(stats.available_gb > 60.0);
    }

    #[test]
    fn test_over_allocation() {
        let mgr = UnifiedMemoryManager::new(128, 96);

        // Try to allocate more than VRAM limit
        assert!(!mgr.can_allocate(100 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_real_time_stats() {
        let mgr = UnifiedMemoryManager::new(128, 96);
        let stats = mgr.stats_live();

        // Real stats should have non-zero values
        assert!(stats.total_gb > 0.0);
        assert!(stats.system_used_gb >= 0.0);
    }

    #[test]
    fn test_pressure_detection() {
        let stats = MemoryStats {
            total_gb: 128.0,
            vram_limit_gb: 96.0,
            allocated_gb: 0.0,
            available_gb: 96.0,
            system_used_gb: 110.0,
            system_free_gb: 18.0,
            pressure_percent: 85.0,
        };
        assert!(stats.is_under_pressure());
        assert!(!stats.is_critical());
    }
}
