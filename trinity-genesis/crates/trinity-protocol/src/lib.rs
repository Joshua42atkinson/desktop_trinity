//! # Trinity Protocol (The Language)
//!
//! ## Philosophy (Architectonics)
//! "Language is the substrate of thought. The Protocol defines the potential limits of
//!  communication between the Mind (Brain) and the Body. It must be expressive, type-safe,
//!  and future-proof."
//!
//! ## Instructions for Developers
//! 1. **Type Safety**: Use Rust enums to make illegal states unrepresentable (e.g., `AvatarState`).
//! 2. **Compatibility**: Changes here affect the whole ecosystem. Append, don't break.
//! 3. **Clarity**: Type names should be self-documenting (e.g., `ThinkResult`, `ActionRequest`).

pub mod artifact;
pub mod brain;
pub mod memory;
pub mod stream;
pub mod task;
pub mod types;

pub use artifact::{
    AgentMode, Artifact, GraphEdge, GraphNode, NodeStatus, PlanTask, StepItem, StepStatus,
};
pub use brain::BrainServiceClient;
pub use memory::MemoryServiceClient;
pub use stream::{AgentConfig, AgentStatus, ModelTier, OrchestratorConfig, StreamEvent};
pub use task::TaskServiceClient;
pub use types::*;
