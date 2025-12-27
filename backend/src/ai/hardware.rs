#![allow(unused)]
//! AMD Strix Halo Hardware Optimization
//!
//! Optimized compute settings for AMD Ryzen AI Max+ 395:
//! - Radeon 8060S GPU (RDNA 3.5, 40 CUs)
//! - 128GB Unified Memory (shared CPU/GPU)
//! - XDNA 2 NPU (50 TOPS)
//! - ROCm acceleration

use anyhow::{Context, Result};
use serde::Serialize;
use std::env;
use std::process::Command;

/// Hardware detection result
#[derive(Debug, Clone, Serialize)]
pub struct HardwareInfo {
    /// Total system memory in GB
    pub total_memory_gb: u64,
    /// GPU name
    pub gpu_name: String,
    /// GPU memory in GB (or unified memory available)
    pub gpu_memory_gb: u64,
    /// Number of compute units
    pub compute_units: u32,
    /// ROCm version if available
    pub rocm_version: Option<String>,
    /// Whether this is a Strix Halo system
    pub is_strix_halo: bool,
    /// Whether unified memory is detected
    pub has_unified_memory: bool,
}

impl Default for HardwareInfo {
    fn default() -> Self {
        Self {
            total_memory_gb: 16,
            gpu_name: "Unknown".to_string(),
            gpu_memory_gb: 8,
            compute_units: 0,
            rocm_version: None,
            is_strix_halo: false,
            has_unified_memory: false,
        }
    }
}

/// Detect AMD hardware and capabilities
pub fn detect_hardware() -> HardwareInfo {
    let mut info = HardwareInfo::default();

    // Get total system memory
    if let Ok(mem) = get_total_memory() {
        info.total_memory_gb = mem;
    }

    // Detect ROCm
    if let Ok(version) = detect_rocm_version() {
        info.rocm_version = Some(version);
    }

    // Detect GPU info
    if let Ok((name, memory, cus)) = detect_amd_gpu() {
        info.gpu_name = name.clone();
        info.gpu_memory_gb = memory;
        info.compute_units = cus;

        // Check for Strix Halo identifiers
        let name_lower = name.to_lowercase();
        if name_lower.contains("8060")
            || name_lower.contains("strix")
            || name_lower.contains("radeon 890m")
            || name_lower.contains("gfx1151")
        {
            info.is_strix_halo = true;
            info.has_unified_memory = true;
            // Strix Halo shares system memory
            info.gpu_memory_gb = info.total_memory_gb;
        }
    }

    // Check for unified memory via environment
    if env::var("HSA_XNACK").map(|v| v == "1").unwrap_or(false) {
        info.has_unified_memory = true;
    }

    info
}

/// Get total system memory in GB
fn get_total_memory() -> Result<u64> {
    let output = Command::new("grep")
        .args(["MemTotal", "/proc/meminfo"])
        .output()
        .context("Failed to read memory info")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let kb: u64 = stdout
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    Ok(kb / 1024 / 1024) // Convert KB to GB
}

/// Detect ROCm version
fn detect_rocm_version() -> Result<String> {
    // Try rocminfo
    let output = Command::new("rocminfo").output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Parse version from rocminfo output
            for line in stdout.lines() {
                if line.contains("ROCm") || line.contains("HSA") {
                    return Ok(line.trim().to_string());
                }
            }
        }
    }

    // Try hip-info
    let output = Command::new("hipconfig").arg("--version").output();

    if let Ok(out) = output {
        if out.status.success() {
            return Ok(String::from_utf8_lossy(&out.stdout).trim().to_string());
        }
    }

    anyhow::bail!("ROCm not detected")
}

/// Detect AMD GPU info
fn detect_amd_gpu() -> Result<(String, u64, u32)> {
    // Try rocm-smi
    let output = Command::new("rocm-smi")
        .args(["--showproductname"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("GPU") || line.contains("Radeon") {
                    // Parse GPU name
                    let name = line
                        .split(':')
                        .next_back()
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|| "AMD GPU".to_string());

                    // Get memory from rocm-smi
                    let mem_out = Command::new("rocm-smi")
                        .args(["--showmeminfo", "vram"])
                        .output()
                        .ok();

                    let mem_gb = mem_out
                        .and_then(|o| {
                            let s = String::from_utf8_lossy(&o.stdout);
                            // Parse memory value
                            s.lines()
                                .find(|l| l.contains("Total"))
                                .and_then(|l| l.split_whitespace().nth(2))
                                .and_then(|v| v.parse::<u64>().ok())
                                .map(|mb| mb / 1024)
                        })
                        .unwrap_or(8);

                    return Ok((name, mem_gb, 40)); // 40 CUs for Strix Halo
                }
            }
        }
    }

    // Fallback: try lspci
    let output = Command::new("lspci")
        .output()
        .context("Failed to run lspci")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("VGA") && line.contains("AMD") {
            return Ok((line.to_string(), 8, 0));
        }
    }

    anyhow::bail!("AMD GPU not detected")
}

/// Compute configuration optimized for hardware
#[derive(Debug, Clone)]
pub struct ComputeConfig {
    /// GPU layers to offload (-1 = all)
    pub gpu_layers: i32,
    /// Number of threads for CPU compute
    pub threads: u32,
    /// Batch size for inference
    pub batch_size: usize,
    /// Context size
    pub context_size: usize,
    /// Use flash attention
    pub use_flash_attention: bool,
    /// Use tensor cores / matrix cores
    pub use_tensor_cores: bool,
    /// Memory mapping mode
    pub mmap: bool,
    /// Lock model in memory
    pub mlock: bool,
}

impl ComputeConfig {
    /// Optimal config for Strix Halo with 128GB unified memory
    pub fn strix_halo_optimal(model_size_gb: f64) -> Self {
        Self {
            gpu_layers: -1, // All layers on GPU
            threads: 24,    // Zen 5 has 24 threads (12 cores)
            batch_size: 512,
            context_size: 32768,
            use_flash_attention: true,
            use_tensor_cores: true, // RDNA 3.5 has AI accelerators
            mmap: true,             // Memory map model file
            mlock: false,           // Don't lock (unified memory)
        }
    }

    /// Config for Qwen3-235B on Strix Halo
    pub fn qwen3_235b_strix_halo() -> Self {
        Self {
            gpu_layers: -1,
            threads: 16,     // Leave some for system
            batch_size: 256, // Lower for 235B model
            context_size: 32768,
            use_flash_attention: true,
            use_tensor_cores: true,
            mmap: true,
            mlock: false,
        }
    }

    /// Conservative config for memory-constrained systems
    pub fn conservative(available_memory_gb: u64) -> Self {
        let batch = if available_memory_gb > 64 { 256 } else { 128 };
        let ctx = if available_memory_gb > 64 {
            16384
        } else {
            8192
        };

        Self {
            gpu_layers: 0, // CPU only
            threads: 8,
            batch_size: batch,
            context_size: ctx,
            use_flash_attention: false,
            use_tensor_cores: false,
            mmap: true,
            mlock: false,
        }
    }

    /// Auto-detect best config
    pub fn auto() -> Self {
        let hw = detect_hardware();

        if hw.is_strix_halo {
            log::info!("Detected Strix Halo - using optimal config");
            Self::strix_halo_optimal(65.0)
        } else if hw.gpu_memory_gb >= 24 {
            log::info!("Detected high-memory GPU - using full offload");
            Self {
                gpu_layers: -1,
                threads: 8,
                batch_size: 256,
                context_size: 16384,
                use_flash_attention: true,
                use_tensor_cores: true,
                mmap: true,
                mlock: false,
            }
        } else {
            log::info!("Using conservative config");
            Self::conservative(hw.total_memory_gb)
        }
    }

    /// Convert to environment variables for llama.cpp / candle
    pub fn to_env_vars(&self) -> Vec<(String, String)> {
        let mut vars = vec![
            ("CUDA_VISIBLE_DEVICES".to_string(), "0".to_string()),
            ("OMP_NUM_THREADS".to_string(), self.threads.to_string()),
        ];

        // ROCm / HIP specific
        vars.push(("HIP_VISIBLE_DEVICES".to_string(), "0".to_string()));

        // For unified memory (Strix Halo)
        vars.push(("HSA_XNACK".to_string(), "1".to_string()));

        // Optimize memory allocation
        vars.push(("GPU_MAX_ALLOC_PERCENT".to_string(), "95".to_string()));
        vars.push(("GPU_SINGLE_ALLOC_PERCENT".to_string(), "95".to_string()));

        vars
    }
}

/// Set optimal environment variables for inference
pub fn configure_environment(config: &ComputeConfig) {
    for (key, value) in config.to_env_vars() {
        env::set_var(&key, &value);
        log::debug!("Set {}={}", key, value);
    }
}

/// Print hardware summary
pub fn print_hardware_summary() {
    let hw = detect_hardware();

    println!("╔══════════════════════════════════════════╗");
    println!("║      Trinity Hardware Configuration      ║");
    println!("╠══════════════════════════════════════════╣");
    println!(
        "║ GPU: {:<35} ║",
        hw.gpu_name.chars().take(35).collect::<String>()
    );
    println!(
        "║ Memory: {:>3} GB ({})           ║",
        hw.total_memory_gb,
        if hw.has_unified_memory {
            "unified"
        } else {
            "discrete"
        }
    );
    println!(
        "║ Compute Units: {:>3}                       ║",
        hw.compute_units
    );
    println!(
        "║ ROCm: {:<35} ║",
        hw.rocm_version.as_deref().unwrap_or("Not detected")
    );
    println!(
        "║ Strix Halo: {:<29} ║",
        if hw.is_strix_halo {
            "✓ Yes"
        } else {
            "✗ No"
        }
    );
    println!("╚══════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strix_halo_config() {
        let config = ComputeConfig::strix_halo_optimal(65.0);
        assert_eq!(config.gpu_layers, -1);
        assert!(config.use_flash_attention);
    }

    #[test]
    fn test_env_vars() {
        let config = ComputeConfig::strix_halo_optimal(65.0);
        let vars = config.to_env_vars();
        assert!(vars.iter().any(|(k, _)| k == "HSA_XNACK"));
    }

    #[test]
    fn test_qwen3_config() {
        let config = ComputeConfig::qwen3_235b_strix_halo();
        assert_eq!(config.context_size, 32768);
    }
}
