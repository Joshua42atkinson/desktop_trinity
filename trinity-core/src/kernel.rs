//! Trinity Kernel - The Heart of the AI OS
//!
//! Orchestrates the hardware abstraction layer (HAL), memory management,
//! and the agent process scheduler.

use anyhow::{Context, Result};
use bevy::ecs::schedule::Schedule;
use bevy::ecs::world::World;
use bevy::prelude::*;
use std::sync::Arc;

use crate::agent::{agent_scheduler_system, task_router_system, AgentId, TaskRequest};
use crate::device::TrinityDevice;
use crate::memory::UnifiedMemoryManager;
use crate::workflow::{workflow_execution_system, TaskCompleted};

/// Wrapper for sharing the Device with Bevy ECS
#[derive(Resource, Clone)]
pub struct SystemDevice(pub Arc<TrinityDevice>);

/// Wrapper for sharing Memory Manager with Bevy ECS
#[derive(Resource, Clone)]
pub struct SystemMemory(pub Arc<UnifiedMemoryManager>);

/// The Trinity Kernel
///
/// Functions as the central orchestrator for the AI Operating System.
pub struct Kernel {
    /// Hardware Abstraction Layer (HAL) for Strix Halo
    pub device: Arc<TrinityDevice>,
    /// Unified Memory Manager (128GB)
    pub memory_manager: Arc<UnifiedMemoryManager>,
    /// The "OS Scheduler" (Bevy ECS World)
    pub world: World,
    /// The Schedule (System execution order)
    pub schedule: Schedule,
}

impl Kernel {
    /// Boot the Trinity Kernel
    pub async fn boot() -> Result<Self> {
        tracing::info!("Kernel: Booting Trinity OS...");

        // 0. Reap stale processes to free GPU handles
        // crate::system::ZombieReaper::reap();

        // 1. Initialize Hardware (HAL)
        tracing::info!("Kernel: Initializing Hardware Abstraction Layer...");
        let device = Arc::new(TrinityDevice::new().context("Failed to initialize HAL")?);
        tracing::info!("Kernel: HAL/Device Ready: {}", device.device_type);

        // 2. Initialize Memory Manager
        tracing::info!("Kernel: Initializing Unified Memory Manager...");
        // Default Strix Halo config: 128GB Total, 96GB VRAM limit
        let memory_manager = Arc::new(UnifiedMemoryManager::strix_halo_default());
        tracing::info!("Kernel: Memory Manager Ready ({})", memory_manager.stats());

        // 3. Initialize Process Scheduler (ECS)
        tracing::info!("Kernel: Initializing Process Scheduler...");
        let mut world = World::new();
        let mut schedule = Schedule::default();

        // Register Core Resources
        // We wrap the Arcs in Resource structs to allow shared ownership between Kernel and ECS
        world.insert_resource(SystemDevice(device.clone()));
        world.insert_resource(SystemMemory(memory_manager.clone()));

        // Register Events
        world.init_resource::<Events<TaskRequest>>();
        world.init_resource::<Events<TaskCompleted>>();

        // Register Core Systems (The "OS Services")
        schedule.add_systems((
            task_router_system,        // Service: Router (Switchboard)
            agent_scheduler_system,    // Service: Scheduler (Process State)
            workflow_execution_system, // Service: Workflow Engine (Graph Propagation)
        ));

        tracing::info!("Kernel: Boot Complete. System Ready.");

        Ok(Self {
            device,
            memory_manager,
            world,
            schedule,
        })
    }

    /// Run one "tick" of the OS
    pub fn tick(&mut self) {
        self.schedule.run(&mut self.world);
    }

    /// Spawn a new Agent "Process"
    pub fn spawn_agent(&mut self, role: crate::agent::AgentRole) -> AgentId {
        let uuid = uuid::Uuid::new_v4();
        let id = AgentId(uuid);

        tracing::info!("Kernel: Spawning Agent Process {} ({:?})", uuid, role);

        self.world.spawn((
            id.clone(),
            role,
            crate::agent::AgentState::Idle,
            crate::agent::WorkingMemory::default(),
            crate::agent::AgentCapabilities::default(),
        ));

        id
    }

    /// Submit a task to the OS (User Input)
    pub fn submit_task(&mut self, content: String) {
        tracing::info!("Kernel: Received User Task -> '{}'", content);
        self.world.send_event(TaskRequest {
            content,
            preferred_agent: None,
        });
    }

    /// Get system diagnostics
    pub fn diagnostics(&self) -> String {
        format!(
            "--- Trinity Kernel Diagnostics ---\n\
             Device: {}\n\
             Memory: {}\n\
             Agents: {}\n\
             ----------------------------------",
            self.device.device_type,
            self.memory_manager.stats(),
            self.world.entities().len()
        )
    }
}
