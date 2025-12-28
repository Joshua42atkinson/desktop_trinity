// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)
//
// ═══════════════════════════════════════════════════════════════════════════════
// 🚂 ZONE: IRON ROAD | Workflow: /iron-road-dev | Context: /antigravity/CONTEXT.md
// ═══════════════════════════════════════════════════════════════════════════════
// VISION: Pure Rust • Constructivist Education • Cognitive Load Theory
// This is THE DEMO - the game sandbox that shows what Trinity can build.
// ═══════════════════════════════════════════════════════════════════════════════

//! # Iron Road Physics Engine
//!
//! This module implements the "Coal & Steam" economy derived from Cognitive Load Theory.
//!
//! ## Core Concepts
//! - **Mass (Intrinsic Load)**: The difficulty of the content.
//! - **Coal (Motivation)**: The finite resource required to do work.
//! - **Steam (Germane Load)**: The kinetic output of active processing.
//! - **Friction (Extraneous Load)**: Impediments to movement.
//!
//! ## The Equation
//! `Velocity = (Power * Steam_Efficiency) / (Mass * Friction)`

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub enum VocabularyTier {
    #[default]
    Basic, // Tier 1: Daily speech (Mass: 1-10)
    Academic,  // Tier 2: School speech (Mass: 10-50)
    Hazardous, // Tier 3: Domain specific (Mass: 50-100)
}

impl VocabularyTier {
    pub fn base_mass(&self) -> f32 {
        match self {
            Self::Basic => 5.0,
            Self::Academic => 30.0,
            Self::Hazardous => 80.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct CognitiveLoad {
    /// Intrinsic Load (1-100)
    pub base_mass: f32,
    /// Effective Load (Drops to 0 if mastered)
    pub effective_mass: f32,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Train {
    /// Current store of motivation (Coal). Burned to create Steam.
    pub coal: f32,
    /// Current kinetic potential (Steam). Used to move Mass.
    pub steam: f32,
    /// The engine's raw power output (Baseline cognitive capacity).
    pub power: f32,
    /// Current velocity (Learning rate).
    pub velocity: f32,
}

impl Train {
    pub fn new(power: f32) -> Self {
        Self {
            coal: 100.0, // Start fully motivated
            steam: 0.0,
            power,
            velocity: 0.0,
        }
    }

    /// Burns coal to generate steam.
    /// Returns true if coal was available, false if stalled.
    pub fn stoke_fire(&mut self, amount: f32) -> bool {
        if self.coal >= amount {
            self.coal -= amount;
            // Combustion efficiency could be a variable later
            self.steam += amount * 1.5;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Node {
    /// Intrinsic Cognitive Load (Difficulty).
    pub mass: f32,
    /// Extraneous Cognitive Load (Bad Design Friction).
    /// 1.0 = No Friction. > 1.0 = High Friction.
    pub friction: f32,
}

impl Node {
    pub fn new(mass: f32, friction: f32) -> Self {
        Self { mass, friction }
    }
}

/// Calculus of the Iron Road.
///
/// Determines the velocity of the train based on current physics state.
pub fn calculate_velocity(train: &mut Train, node: &Node) -> f32 {
    let power_output = train.power + train.steam;
    let resistance = node.mass * node.friction;

    if resistance <= 0.0 {
        return f32::MAX; // Infinite speed on zero resistance
    }

    let velocity = power_output / resistance;

    // Update train state (momentum decay logic could go here)
    train.velocity = velocity;

    // Steam is consumed by the act of moving
    train.steam = (train.steam - (node.mass * 0.1)).max(0.0);

    velocity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_baseline() {
        let mut train = Train::new(10.0);
        let node = Node::new(10.0, 1.0); // Mass 10, Friction 1 (Normal)

        // Base power 10 / Mass 10 = Velocity 1.0
        let v = calculate_velocity(&mut train, &node);
        assert_eq!(v, 1.0);
    }

    #[test]
    fn test_coal_boost() {
        let mut train = Train::new(10.0);
        train.stoke_fire(10.0); // +15 Steam

        let node = Node::new(10.0, 1.0);

        // (Power 10 + Steam 15) / Mass 10 = 2.5
        let v = calculate_velocity(&mut train, &node);
        assert_eq!(v, 2.5);
    }

    #[test]
    fn test_friction_stall() {
        let mut train = Train::new(10.0);
        let node = Node::new(10.0, 5.0); // High Friction (Bad Design)

        // Power 10 / (10 * 5) = 0.2 (Crawling)
        let v = calculate_velocity(&mut train, &node);
        assert_eq!(v, 0.2);
    }
}
