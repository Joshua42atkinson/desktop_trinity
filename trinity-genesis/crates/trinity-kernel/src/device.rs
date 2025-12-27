//! Device Detection
//!
//! Hardware capability detection for Trinity Genesis.

use serde::{Deserialize, Serialize};

/// Detected device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// GPU information
    pub gpu: Option<GpuInfo>,
    /// NPU information (Strix Halo)
    pub npu: Option<NpuInfo>,
    /// System memory in GB
    pub system_ram_gb: u64,
    /// Available VRAM in GB (for unified memory this may equal system RAM)
    pub vram_gb: u64,
    /// Number of CPU cores
    pub cpu_cores: usize,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU name
    pub name: String,
    /// Vendor (AMD, NVIDIA, Intel)
    pub vendor: GpuVendor,
    /// VRAM in bytes
    pub vram_bytes: u64,
    /// Whether ROCm is available
    pub rocm_available: bool,
    /// GFX version for ROCm (e.g., "gfx1151" for Strix Halo)
    pub gfx_version: Option<String>,
}

/// NPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuInfo {
    /// NPU name
    pub name: String,
    /// TOPS (Tera Operations Per Second)
    pub tops: f32,
}

/// GPU vendor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GpuVendor {
    Amd,
    Nvidia,
    Intel,
    Unknown,
}

impl DeviceCapabilities {
    /// Detect capabilities on the current system
    pub fn detect() -> Self {
        let system_ram_gb = Self::detect_system_ram();
        let cpu_cores = num_cpus::get();
        let gpu = Self::detect_gpu();
        let npu = Self::detect_npu();
        let vram_gb = gpu
            .as_ref()
            .map(|g| g.vram_bytes / (1024 * 1024 * 1024))
            .unwrap_or(0);

        Self {
            gpu,
            npu,
            system_ram_gb,
            vram_gb,
            cpu_cores,
        }
    }

    /// Create capabilities for Strix Halo (AMD Ryzen AI Max 395+)
    pub fn strix_halo_preset() -> Self {
        Self {
            gpu: Some(GpuInfo {
                name: "AMD Radeon Graphics (Strix Halo)".to_string(),
                vendor: GpuVendor::Amd,
                vram_bytes: 96 * 1024 * 1024 * 1024, // 96GB unified
                rocm_available: true,
                gfx_version: Some("gfx1151".to_string()),
            }),
            npu: Some(NpuInfo {
                name: "AMD XDNA 2".to_string(),
                tops: 50.0,
            }),
            system_ram_gb: 128,
            vram_gb: 96,
            cpu_cores: 16,
        }
    }

    fn detect_system_ram() -> u64 {
        // Read from /proc/meminfo on Linux
        if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
            for line in contents.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb / (1024 * 1024); // Convert KB to GB
                        }
                    }
                }
            }
        }
        0
    }

    fn detect_gpu() -> Option<GpuInfo> {
        // Try to detect AMD GPU via /sys/class/drm
        let drm_path = std::path::Path::new("/sys/class/drm");
        if drm_path.exists() {
            if let Ok(entries) = std::fs::read_dir(drm_path) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("card") && !name_str.contains('-') {
                        // Check if it's AMD
                        let vendor_path = entry.path().join("device/vendor");
                        if let Ok(vendor) = std::fs::read_to_string(&vendor_path) {
                            if vendor.trim() == "0x1002" {
                                // AMD vendor ID
                                return Some(GpuInfo {
                                    name: "AMD GPU".to_string(),
                                    vendor: GpuVendor::Amd,
                                    vram_bytes: 96 * 1024 * 1024 * 1024, // Assume Strix Halo
                                    rocm_available: std::path::Path::new("/opt/rocm").exists(),
                                    gfx_version: Some("gfx1151".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_npu() -> Option<NpuInfo> {
        // Check for AMD XDNA NPU
        if std::path::Path::new("/dev/accel/accel0").exists() {
            return Some(NpuInfo {
                name: "AMD XDNA".to_string(),
                tops: 50.0,
            });
        }
        None
    }
}

// Use num_cpus crate would be ideal, but for now:
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4)
    }
}
