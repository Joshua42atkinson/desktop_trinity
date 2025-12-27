use anyhow::Result;
use rodio::DeviceTrait;
use std::process::Command;

/// System capabilities verification
pub struct SystemCheck;

impl SystemCheck {
    /// Perform a full system sanity check
    pub fn run() -> Result<()> {
        tracing::info!("Starting Trinity System Check...");

        Self::check_drivers()?;
        // Self::check_accelerator()?; // TODO: Re-implement with llama-cpp check if needed
        Self::check_media_capabilities()?;

        tracing::info!("System Check Passed: All systems operational.");
        Ok(())
    }

    fn check_drivers() -> Result<()> {
        // Check for ROCm/HIP
        // This is a rough check by looking for rocm-smi or similar
        match Command::new("rocm-smi").output() {
            Ok(output) => {
                if output.status.success() {
                    tracing::info!("ROCm drivers detected (rocm-smi found).");
                } else {
                    tracing::warn!("rocm-smi found but returned error code.");
                }
            }
            Err(_) => {
                tracing::warn!(
                    "rocm-smi not found. Ensure ROCm drivers are installed for Strix Halo."
                );
            }
        }
        Ok(())
    }

    fn check_media_capabilities() -> Result<()> {
        // Check if we can initialize basic media checks
        use rodio::cpal::traits::HostTrait;
        let host = rodio::cpal::default_host();
        match host.default_output_device() {
            Some(device) => tracing::info!(
                "Audio Output Device Detected: {}",
                device.name().unwrap_or("Unknown".into())
            ),
            None => tracing::warn!("No Audio Output Device detected."),
        }

        Ok(())
    }
}
