// Agent ECS Module - Swarm AI Agent Framework
// Agents are Bevy entities with specialized roles

#![allow(unused)]
pub mod autonomous;
pub mod components;
pub mod events;
pub mod self_coder;
pub mod specialized;
pub mod systems;
pub mod task_store;
pub mod workflow;

pub use autonomous::{AutonomousRuntime, AutonomousTask, RuntimeConfig, TaskPriority, TaskType};
pub use components::*;
pub use events::*;
pub use self_coder::{CodeResult, SelfCoderConfig, SelfCodingAgent};
pub use systems::*;
pub use task_store::TaskStore;
