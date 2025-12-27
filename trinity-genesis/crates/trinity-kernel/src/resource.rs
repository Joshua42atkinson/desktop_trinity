//! Resource Manager - Hardware-Aware Resource Economy
//!
//! Intelligent management of memory, GPU, and CPU resources for always-on operation.
//! Ensures Trinity stays healthy and responsive even under load.

use crate::device::DeviceCapabilities;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// Resource Budget
// ============================================================================

/// Resource budget allocation for different components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBudget {
    /// Maximum memory for LLM inference (bytes)
    pub llm_memory_bytes: u64,
    /// Maximum memory for TTS (bytes)
    pub tts_memory_bytes: u64,
    /// Maximum memory for image generation (bytes)
    pub image_gen_memory_bytes: u64,
    /// Reserved system memory (bytes) - don't touch this
    pub system_reserved_bytes: u64,
    /// Maximum concurrent agents
    pub max_agents: usize,
    /// GPU layers to offload (0 = CPU only)
    pub gpu_layers: u32,
}

impl ResourceBudget {
    /// Create budget for 128GB unified memory (Strix Halo)
    pub fn strix_halo_128gb() -> Self {
        Self {
            llm_memory_bytes: 80 * 1024 * 1024 * 1024,     // 80GB for LLM
            tts_memory_bytes: 2 * 1024 * 1024 * 1024,      // 2GB for TTS
            image_gen_memory_bytes: 16 * 1024 * 1024 * 1024, // 16GB for SDXL
            system_reserved_bytes: 4 * 1024 * 1024 * 1024, // 4GB for OS/apps (Lowered from 16GB)
            max_agents: 4,
            gpu_layers: 999, // Offload everything
        }
    }

    /// Create conservative budget for lower memory systems
    pub fn conservative(total_ram_gb: u64) -> Self {
        let usable = total_ram_gb.saturating_sub(8); // Reserve 8GB for system
        Self {
            llm_memory_bytes: (usable * 60 / 100) * 1024 * 1024 * 1024, // 60% for LLM
            tts_memory_bytes: 1 * 1024 * 1024 * 1024,
            image_gen_memory_bytes: (usable * 20 / 100) * 1024 * 1024 * 1024, // 20%
            system_reserved_bytes: 8 * 1024 * 1024 * 1024,
            max_agents: 2,
            gpu_layers: 50,
        }
    }
}

// ============================================================================
// Live Resource Stats
// ============================================================================

/// Real-time resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    /// Current memory usage in bytes
    pub memory_used_bytes: u64,
    /// Current memory available in bytes
    pub memory_available_bytes: u64,
    /// Memory usage percentage
    pub memory_percent: f32,
    /// GPU memory used (if applicable)
    pub gpu_memory_used_bytes: u64,
    /// CPU usage percentage (0-100 per core, so max = cores * 100)
    pub cpu_percent: f32,
    /// Number of active agents
    pub active_agents: usize,
    /// System load average (1 min)
    pub load_avg_1m: f32,
    /// Whether system is under memory pressure
    pub memory_pressure: bool,
    /// Whether GPU is available
    pub gpu_available: bool,
}

impl ResourceStats {
    /// Read current stats from system
    pub fn read() -> Self {
        let (mem_used, mem_available, mem_total) = Self::read_memory();
        let cpu_percent = Self::read_cpu();
        let load_avg = Self::read_load_avg();
        let gpu_memory = Self::read_gpu_memory();

        let memory_percent = if mem_total > 0 {
            (mem_used as f32 / mem_total as f32) * 100.0
        } else {
            0.0
        };

        Self {
            memory_used_bytes: mem_used,
            memory_available_bytes: mem_available,
            memory_percent,
            gpu_memory_used_bytes: gpu_memory,
            cpu_percent,
            active_agents: 0, // Set by orchestrator
            load_avg_1m: load_avg,
            memory_pressure: memory_percent > 85.0,
            gpu_available: std::path::Path::new("/opt/rocm").exists(),
        }
    }

    fn read_memory() -> (u64, u64, u64) {
        if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
            let mut total: u64 = 0;
            let mut available: u64 = 0;

            for line in contents.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        total = kb.parse::<u64>().unwrap_or(0) * 1024;
                    }
                } else if line.starts_with("MemAvailable:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        available = kb.parse::<u64>().unwrap_or(0) * 1024;
                    }
                }
            }

            let used = total.saturating_sub(available);
            return (used, available, total);
        }
        (0, 0, 0)
    }

    fn read_cpu() -> f32 {
        // Simple load-based CPU estimate
        Self::read_load_avg() * 100.0 / num_cpus::get() as f32
    }

    fn read_load_avg() -> f32 {
        if let Ok(contents) = std::fs::read_to_string("/proc/loadavg") {
            if let Some(first) = contents.split_whitespace().next() {
                return first.parse().unwrap_or(0.0);
            }
        }
        0.0
    }

    /// Read GPU memory usage from AMD ROCm sysfs
    /// Checks /sys/class/drm/card*/device/mem_info_vram_used
    fn read_gpu_memory() -> u64 {
        // Try card1 first (common on Strix Halo), then card0
        for card in &["card1", "card0", "card2"] {
            let path = format!("/sys/class/drm/{}/device/mem_info_vram_used", card);
            if let Ok(contents) = std::fs::read_to_string(&path) {
                if let Ok(bytes) = contents.trim().parse::<u64>() {
                    return bytes;
                }
            }
        }
        0
    }
}

// ============================================================================
// Resource Manager
// ============================================================================

/// Hardware-aware resource manager for Trinity
pub struct ResourceManager {
    /// Device capabilities (static)
    pub capabilities: DeviceCapabilities,
    /// Resource budget allocation
    pub budget: ResourceBudget,
    /// Allocated memory tracker
    allocated_bytes: AtomicU64,
}

impl ResourceManager {
    /// Create a new resource manager with auto-detection
    pub fn new() -> Self {
        let capabilities = DeviceCapabilities::detect();
        let budget = if capabilities.system_ram_gb >= 96 {
            ResourceBudget::strix_halo_128gb()
        } else {
            ResourceBudget::conservative(capabilities.system_ram_gb)
        };

        tracing::info!(
            "ResourceManager initialized: {}GB RAM, {} cores, budget: {}GB LLM",
            capabilities.system_ram_gb,
            capabilities.cpu_cores,
            budget.llm_memory_bytes / (1024 * 1024 * 1024)
        );

        Self {
            capabilities,
            budget,
            allocated_bytes: AtomicU64::new(0),
        }
    }

    /// Create with Strix Halo preset
    pub fn strix_halo() -> Self {
        let capabilities = DeviceCapabilities::strix_halo_preset();
        let budget = ResourceBudget::strix_halo_128gb();

        Self {
            capabilities,
            budget,
            allocated_bytes: AtomicU64::new(0),
        }
    }

    /// Get current resource stats
    pub fn stats(&self) -> ResourceStats {
        ResourceStats::read()
    }

    /// Check if we can allocate the requested bytes
    pub fn can_allocate(&self, bytes: u64) -> bool {
        let stats = self.stats();
        stats.memory_available_bytes > bytes + self.budget.system_reserved_bytes
    }

    /// Request allocation (returns true if granted)
    pub fn allocate(&self, bytes: u64) -> bool {
        if self.can_allocate(bytes) {
            self.allocated_bytes.fetch_add(bytes, Ordering::SeqCst);
            true
        } else {
            tracing::warn!("Allocation denied: requested {}MB, insufficient memory", bytes / (1024 * 1024));
            false
        }
    }

    /// Release allocation
    pub fn release(&self, bytes: u64) {
        self.allocated_bytes.fetch_sub(bytes, Ordering::SeqCst);
    }

    /// Get current allocation
    pub fn current_allocation(&self) -> u64 {
        self.allocated_bytes.load(Ordering::SeqCst)
    }

    /// Check if system is healthy for operation
    pub fn is_healthy(&self) -> bool {
        let stats = self.stats();
        !stats.memory_pressure && stats.load_avg_1m < (self.capabilities.cpu_cores as f32 * 2.0)
    }

    /// Get recommended GPU layers based on available memory
    pub fn recommended_gpu_layers(&self) -> u32 {
        let stats = self.stats();
        if stats.memory_available_bytes > 80 * 1024 * 1024 * 1024 {
            999 // Full offload
        } else if stats.memory_available_bytes > 40 * 1024 * 1024 * 1024 {
            60
        } else if stats.memory_available_bytes > 20 * 1024 * 1024 * 1024 {
            30
        } else {
            0 // CPU only
        }
    }

    /// Wait for memory to become available (with timeout)
    pub async fn wait_for_memory(&self, bytes: u64, timeout_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            if self.can_allocate(bytes) {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        false
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

// Helper module
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_stats_read() {
        let stats = ResourceStats::read();
        assert!(stats.memory_used_bytes > 0 || cfg!(not(target_os = "linux")));
    }

    #[test]
    fn test_resource_manager_creation() {
        let rm = ResourceManager::strix_halo();
        assert_eq!(rm.capabilities.system_ram_gb, 128);
        assert!(rm.budget.llm_memory_bytes > 0);
    }

    #[test]
    fn test_allocation() {
        let rm = ResourceManager::strix_halo();
        // Test allocation tracking (not system-dependent)
        rm.allocated_bytes.store(0, Ordering::SeqCst);
        rm.allocated_bytes.fetch_add(1024, Ordering::SeqCst);
        assert_eq!(rm.current_allocation(), 1024);
        rm.release(1024);
        assert_eq!(rm.current_allocation(), 0);
    }
}
