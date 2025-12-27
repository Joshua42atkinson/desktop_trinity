//! # Trinity Kernel (The Will)
//!
//! ## Philosophy (Architectonics)
//! "The Kernel defines the Will of the system. It handles the 'How'—executing the strategies
//!  devised by the Brain. It is the engine of Autonomy."
//!
//! ## Instructions for Developers
//! 1. **Robustness**: The Kernel must never crash. If a skill fails, the Kernel isolates it and keeps running.
//! 2. **Abstraction**: Hide the complexity of hardware and OS interaction behind clean Rust traits.
//! 3. **Resource Awareness**: Respect the physics of the machine (Strix Halo). Do not overcommit VRAM.
//!
//! ## Architecture Overview
//!
//! ```text
//!                          ┌─────────────────────────────────────┐
//!                          │         trinity-body (UI)           │
//!                          │  Bevy + egui native desktop app     │
//!                          └──────────────┬──────────────────────┘
//!                                         │ tarpc RPC
//!                          ┌──────────────▼──────────────────────┐
//!                          │        trinity-brain (Server)       │
//!                          │   RPC handlers, service routing     │
//!                          └──────────────┬──────────────────────┘
//!                                         │
//!        ┌────────────────────────────────┼────────────────────────────────┐
//!        │                    trinity-kernel (This Crate)                  │
//!        │                                                                 │
//!        │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
//!        │  │   Brain     │  │   Memory    │  │     Orchestrator        │  │
//!        │  │ (LLM Infer) │  │ (Vector DB) │  │ (Multi-Agent Dispatch)  │  │
//!        │  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
//!        │         │                │                     │                │
//!        │  ┌──────▼──────┐  ┌──────▼──────┐  ┌───────────▼─────────────┐  │
//!        │  │   TTS       │  │  Runtime    │  │   ResourceManager       │  │
//!        │  │ (Voice Out) │  │ (Task Queue)│  │ (Hardware-Aware Alloc)  │  │
//!        │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
//!        └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Components
//!
//! | Module | Purpose | Key Types |
//! |--------|---------|-----------|
//! | [`brain`] | LLM inference abstraction | [`Brain`], [`DesktopBrain`] |
//! | [`memory`] | Vector + relational storage | [`UnifiedMemory`] |
//! | [`orchestrator`] | Multi-agent task dispatch | [`Orchestrator`], [`AgentEvent`] |
//! | [`runtime`] | Autonomous task queue | [`AutonomousRuntime`], [`AutonomousTask`] |
//! | [`task_store`] | SQLite task persistence | [`TaskStore`] |
//! | [`tts`] | Text-to-speech synthesis | [`TtsEngine`], [`AudioBuffer`] |
//! | [`voice`] | Emotion & style control | [`VoiceOutput`], [`EmotionState`] |
//! | [`resource`] | Hardware-aware allocation | [`ResourceManager`], [`ResourceStats`] |
//! | [`device`] | GPU/NPU detection | [`DeviceCapabilities`] |
//!
//! ## Strix Halo Optimization
//!
//! This crate is optimized for AMD Ryzen AI Max 395+ with:
//! - 128GB unified memory (shared CPU/GPU)
//! - ROCm/Vulkan GPU acceleration
//! - 50 TOPS NPU (XDNA 2)
//!
//! ```rust,ignore
//! // Production configuration for Strix Halo
//! let brain = DesktopBrain::strix_halo();
//! let resources = ResourceManager::strix_halo();
//! ```
//!
//! ## Feature Flags
//!
//! - `desktop` - Enables native llama.cpp inference via [`DesktopBrain`]
//! - Default: Uses [`QuadradicalBrain`] for external LLM API

pub mod advanced_memory;
pub mod agent_builder;
pub mod agent_compiler;
pub mod agent_graph;
pub mod autopoietic;
pub mod brain;
#[cfg(feature = "desktop")]
#[path = "brain_desktop.rs"]
pub mod brain_desktop;
#[path = "brain_quadradical.rs"]
pub mod brain_quadradical;
pub mod config;
pub mod device;
pub mod input_injector;
pub mod memory;
#[cfg(feature = "desktop")]
pub mod npu_backend;
pub mod orchestrator;
pub mod resource;
#[cfg(feature = "desktop")]
pub mod rpc_pool;
pub mod runtime;
pub mod safety;
#[cfg(feature = "desktop")]
pub mod swappable_brain;
pub mod system_reaper;
pub mod systemd_control;
pub mod task_store;
pub mod todo_parser;
pub mod tts;
pub mod voice;
pub mod wasm_sandbox;

pub use advanced_memory::{
    AdvancedMemory, MemoryConfig, MemoryEntry, MemoryMetadata, MemorySource, MemoryStats,
    RecallResult,
};
pub use agent_builder::{AgentBuilder, AgentCapabilities, AgentDefinition, BrainTier, Tool};
pub use agent_compiler::{AgentCompiler, AgentSpec, CompilationMetadata, CompiledAgent};
pub use agent_graph::{
    AgentGraph, AgentGraphBuilder, AgentNode, GraphEdge, GraphResult, NodePort, NodeSpecialization,
    NodeStatus,
};
pub use brain::{Brain, GrammarSpec, MockBrain};
#[cfg(feature = "desktop")]
pub use brain_desktop::{DesktopBrain, DesktopBrainConfig};
pub use brain_quadradical::QuadradicalBrain;
pub use config::TrinityConfig;
pub use device::DeviceCapabilities;
pub use memory::UnifiedMemory;
#[cfg(feature = "desktop")]
pub use npu_backend::{ComputeTarget, NpuBrain, NpuConfig, NpuStats};
pub use orchestrator::{AgentEvent, AgentHandle, AgentSpecialization, Orchestrator};
pub use resource::{ResourceBudget, ResourceManager, ResourceStats};
#[cfg(feature = "desktop")]
pub use rpc_pool::{RemoteContentType, RemoteHandle, RpcMemoryPool, RpcNodeConfig, RpcPoolStats};
pub use runtime::{AutonomousRuntime, AutonomousTask, QueueStatus, TaskPriority, TaskType};
#[cfg(feature = "desktop")]
pub use swappable_brain::{ModelProfile, SwapStatus, SwappableBrain};
pub use system_reaper::ZombieReaper;
pub use task_store::TaskStore;
pub use todo_parser::{get_pending_items, parse_todo_file, TodoItem};
pub use tts::{AudioBuffer, TtsEngine};
pub use voice::{EmotionState, SpeakingResponse, VoiceOutput, VoiceStyle};
pub use wasm_sandbox::{Capability, CapabilitySet, SandboxConfig, SandboxStatus, WasmSandbox};

// Phase 2: System Control
pub use input_injector::{InputBackend, InputInjector, MouseButton};
pub use systemd_control::{SystemdController, UnitStatus};

// Phase 4: Autopoietic Soul
pub use autopoietic::{
    AutopoieticConfig, AutopoieticEngine, MutationRequest, MutationResult, MutationType,
};
pub use safety::{
    activate_kill_switch, check_persistent_kill_switch, deactivate_kill_switch, emergency_rollback,
    is_kill_switch_active, validate_critical_integrity, FailureTracker,
};
