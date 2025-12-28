// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Zombie Reaper - GPU Memory Stability Service
//!
//! Strix Halo's AMDGPU driver can hold onto memory handles from previous
//! failed or crashed processes. This service ensures a clean state on boot.

use std::process;
use sysinfo::System;

pub struct ZombieReaper;

impl ZombieReaper {
    /// Scan for and kill stale inference/trinity processes
    pub fn reap() {
        let current_pid = process::id();
        // Only refresh processes, not everything (save memory/time)
        let mut sys = System::new();
        use sysinfo::ProcessesToUpdate;
        sys.refresh_processes(ProcessesToUpdate::All, true);

        tracing::info!(
            "ZombieReaper: Scanning for stale processes (Current PID: {})...",
            current_pid
        );

        // Targeted list - avoid killing the UI (trinity-body/desktop)
        let targets = ["llama"]; // Only kill llama processes, not ourselves
        let mut found_zombies = 0;

        for (pid, process) in sys.processes() {
            let pid_val = pid.as_u32();
            // Skip ourselves
            if pid_val == current_pid as u32 {
                continue;
            }

            let name = process.name().to_string_lossy().to_lowercase();
            let is_target = targets.iter().any(|t| name.contains(t));

            if is_target {
                tracing::warn!(
                    "ZombieReaper: Found stale process '{}' (PID: {}). Reaping...",
                    name,
                    pid_val
                );

                if process.kill() {
                    found_zombies += 1;
                } else {
                    tracing::error!("ZombieReaper: Failed to kill PID {}", pid_val);
                }
            }
        }

        if found_zombies > 0 {
            tracing::info!(
                "ZombieReaper: Successfully reaped {} zombie processes. Waiting for cleanup...",
                found_zombies
            );
            // Wait for OS to release GPU handles
            std::thread::sleep(std::time::Duration::from_secs(2));
        } else {
            tracing::debug!("ZombieReaper: No stale processes found.");
        }
    }
}
