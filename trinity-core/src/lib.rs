//! Trinity Core - Rust-native AI Agent Runtime for AMD Strix Halo
//!
//! This crate provides the core runtime for Trinity, a "Digital Familiar" AI OS
//! optimized for AMD Ryzen AI Max+ 395 (Strix Halo) with 128GB unified memory.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Trinity AI OS                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
//! │  │   Agent     │  │  Inference  │  │   Memory    │         │
//! │  │   Swarm     │  │   Engine    │  │   Manager   │         │
//! │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
//! │         │                │                │                 │
//! │  ┌──────┴────────────────┴────────────────┴──────┐         │
//! │  │              Bevy ECS Runtime                  │         │
//! │  └──────────────────────┬────────────────────────┘         │
//! │                         │                                   │
//! │  ┌──────────────────────┴────────────────────────┐         │
//! │  │         Candle + HIP (AMD Strix Halo)         │         │
//! │  │              Native LLM Inference              │         │
//! │  └───────────────────────────────────────────────┘         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - **memory** (default): Memory/learning system with PostgreSQL + sled
//! - **inference**: Candle-based LLM inference (may have version conflicts)
//! - **media**: Audio/image/physics support
//! - **hip**: AMD HIP/ROCm backend for GPU acceleration
//! - **cpu**: CPU-only fallback for development/testing

pub mod agent;
pub mod brain;
pub mod chat;
pub mod config;
pub mod creative;
pub mod system;
pub mod visuals;
pub mod voice;

// Re-export config for convenience
pub use config::TrinityConfig;

#[cfg(any(feature = "desktop", feature = "memory"))]
pub mod learning;
#[cfg(feature = "desktop")]
pub mod llm_server;
#[cfg(any(feature = "desktop", feature = "memory"))]
pub mod memory;
#[cfg(any(feature = "desktop", feature = "memory"))]
pub mod notebook;
#[cfg(any(feature = "desktop", feature = "memory"))]
pub mod workflow;

// Optional modules that require candle/inference
// Optional modules that require candle/inference
// #[cfg(feature = "inference")]
// pub mod inference; // Disabled for now to remove candle dependency

pub mod device;
pub mod kernel;
pub mod system_check;

// Re-exports for convenience
// Re-exports for convenience
// #[cfg(feature = "inference")]
// pub use device::TrinityDevice;
// #[cfg(feature = "inference")]
// pub use inference::{GgufModel, InferenceConfig};
pub use kernel::Kernel;
pub use system_check::SystemCheck;

#[cfg(feature = "desktop")]
pub use learning::{
    MemoryConsolidator, MemoryFragment, RelationalStore, TrinityMemory, VectorStore,
};
#[cfg(feature = "desktop")]
pub use llm_server::{setup_strix_halo_env, LlmServer, LlmServerConfig};
#[cfg(feature = "llama-cpp")]
pub use llm_server::{LlamaNative, LlamaNativeConfig};
#[cfg(feature = "desktop")]
pub use memory::UnifiedMemoryManager;

/// Trinity version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Target hardware description
pub const HARDWARE_TARGET: &str = "AMD Ryzen AI Max+ 395 (Strix Halo, gfx1103)";

/// Maximum VRAM allocation (96GB dedicated from 128GB unified memory via BIOS)
pub const MAX_VRAM_GB: usize = 96;

/// Initialize Trinity runtime with default configuration
pub fn init() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("trinity_core=debug".parse()?),
        )
        .init();

    tracing::info!("Trinity Core v{} initializing...", VERSION);
    tracing::info!("Target hardware: {}", HARDWARE_TARGET);

    // Set HSA override for gfx1151 compatibility
    #[cfg(feature = "hip")]
    {
        std::env::set_var("HSA_OVERRIDE_GFX_VERSION", "11.5.1");
        tracing::debug!("Set HSA_OVERRIDE_GFX_VERSION=11.5.1 for Strix Halo");
    }

    // Run system checks
    if let Err(e) = system_check::SystemCheck::run() {
        tracing::error!("System check failed: {}", e);
    }

    Ok(())
}
