//! Device abstraction for AMD Strix Halo hardware
//!
//! Provides unified access to GPU (Radeon 8060S) and future NPU (XDNA 2) capabilities.
//! Integrates with TrinityConfig for runtime configuration.

use anyhow::Result;
use bevy::prelude::Resource;
use sysinfo::System;

use crate::config::{HardwareConfig, TrinityConfig};

// ============================================================================
// Device Types
// ============================================================================

/// Types of compute devices available on Strix Halo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// AMD Radeon 8060S iGPU (RDNA 3.5, 40 CUs)
    Gpu,
    /// AMD XDNA 2 NPU (50 TOPS) - future support
    Npu,
    /// CPU fallback (Zen 5, 16 cores)
    Cpu,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Gpu => write!(f, "AMD Radeon 8060S (RDNA 3.5)"),
            DeviceType::Npu => write!(f, "AMD XDNA 2 NPU"),
            DeviceType::Cpu => write!(f, "AMD Ryzen AI Max+ 395 CPU (Zen 5)"),
        }
    }
}

impl DeviceType {
    /// Parse device type from config string
    pub fn from_config(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gpu" => DeviceType::Gpu,
            "npu" => DeviceType::Npu,
            "cpu" => DeviceType::Cpu,
            "auto" => DeviceType::Gpu, // Auto defaults to GPU, falls back as needed
            _ => {
                tracing::warn!("Unknown device type '{}', defaulting to auto (GPU)", s);
                DeviceType::Gpu
            }
        }
    }
}

// ============================================================================
// Device Capabilities
// ============================================================================

/// Runtime-detected device capabilities
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    /// Type of device
    pub device_type: DeviceType,
    /// Total system memory in GB
    pub total_memory_gb: f64,
    /// Available system memory in GB
    pub available_memory_gb: f64,
    /// Configured VRAM limit in GB (from config or detected)
    pub vram_limit_gb: f64,
    /// Number of compute units (GPU CUs or CPU cores)
    pub compute_units: u32,
    /// Whether FP16 is supported
    pub supports_fp16: bool,
    /// GPU driver version (if applicable)
    pub driver_info: Option<String>,
}

impl DeviceCapabilities {
    /// Detect system capabilities
    pub fn detect(config: &HardwareConfig) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_memory_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
        let available_memory_gb = sys.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0);

        // Use configured VRAM limit or default to 96GB for Strix Halo
        let vram_limit_gb = config.max_vram_gb.unwrap_or(96.0);

        // Detect CPU cores
        let compute_units = sys.cpus().len() as u32;

        Self {
            device_type: DeviceType::from_config(&config.preferred_device),
            total_memory_gb,
            available_memory_gb,
            vram_limit_gb,
            compute_units,
            supports_fp16: true, // Strix Halo supports FP16
            driver_info: Self::detect_driver(),
        }
    }

    fn detect_driver() -> Option<String> {
        // Try to read ROCm version
        std::fs::read_to_string("/opt/rocm/.info/version")
            .ok()
            .map(|s| format!("ROCm {}", s.trim()))
    }
}

// ============================================================================
// Trinity Device
// ============================================================================

/// Trinity device abstraction for AMD Strix Halo
#[derive(Debug, Clone, Resource)]
pub struct TrinityDevice {
    /// Device type
    pub device_type: DeviceType,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Device capabilities (detected at init)
    pub capabilities: DeviceCapabilities,
}

impl TrinityDevice {
    /// Create a new Trinity device using default config
    pub fn new() -> Result<Self> {
        let config = TrinityConfig::load()
            .unwrap_or_default()
            .with_env_overrides();
        Self::with_config(&config.hardware)
    }

    /// Create a Trinity device with specific hardware config
    pub fn with_config(config: &HardwareConfig) -> Result<Self> {
        // Set HSA override for gfx1151 Strix Halo
        std::env::set_var("HSA_OVERRIDE_GFX_VERSION", &config.hsa_override_version);
        tracing::debug!(
            "Set HSA_OVERRIDE_GFX_VERSION={}",
            config.hsa_override_version
        );

        // Detect capabilities
        let capabilities = DeviceCapabilities::detect(config);
        tracing::info!(
            "Detected: {:.1} GB total memory, {:.1} GB available, {} CUs",
            capabilities.total_memory_gb,
            capabilities.available_memory_gb,
            capabilities.compute_units
        );

        if let Some(ref driver) = capabilities.driver_info {
            tracing::info!("Driver: {}", driver);
        }

        // Initialize device based on preference
        let preferred = DeviceType::from_config(&config.preferred_device);

        let available_memory = match preferred {
            DeviceType::Gpu => {
                (config.max_vram_gb.unwrap_or(96.0) * 1024.0 * 1024.0 * 1024.0) as u64
            }
            _ => (capabilities.available_memory_gb * 1024.0 * 1024.0 * 1024.0) as u64,
        };

        Ok(Self {
            device_type: preferred,
            available_memory,
            capabilities,
        })
    }

    /// Check if this is a GPU device
    pub fn is_gpu(&self) -> bool {
        self.device_type == DeviceType::Gpu
    }

    /// Get available memory in GB
    pub fn available_memory_gb(&self) -> f64 {
        self.available_memory as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Get device capabilities
    pub fn capabilities(&self) -> &DeviceCapabilities {
        &self.capabilities
    }

    /// Get a summary string for display
    pub fn summary(&self) -> String {
        format!(
            "{} ({:.1} GB available)",
            self.device_type,
            self.available_memory_gb()
        )
    }
}

impl Default for TrinityDevice {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Trinity device")
    }
}
