---
description: Development workflow for Iron Road zone (physics/game sandbox)
---

# Iron Road Zone Development

**Scope**: `trinity-genesis/crates/iron-road-physics`

## Local LLM Rules

You are working on Trinity **ZONE: IRON ROAD**

**DO NOT**:

- Modify Trinity core systems
- Touch UI/avatar code
- Change LLM inference

**DO**:

- Focus on physics and game logic
- Keep it pure Rust (no external physics engines)
- Write comprehensive unit tests

## Build & Test

```bash
cd /home/joshua/antigravity/trinity-genesis

# Build physics crate
cargo build -p iron-road-physics

# Run all unit tests
cargo test -p iron-road-physics

# Run specific test
cargo test -p iron-road-physics -- test_coal_consumption
```

## Key Files

| File | Purpose |
|------|---------|
| `iron-road-physics/src/lib.rs` | Main exports |
| `iron-road-physics/src/train.rs` | Train entity logic |
| `iron-road-physics/src/node.rs` | Network nodes |
| `iron-road-physics/src/economy.rs` | Coal/steam economy |

## Core Concepts

```rust
// Train moves along nodes, consuming coal
pub struct Train {
    pub coal: f32,      // Current fuel
    pub steam: f32,     // Generated power
    pub velocity: f32,  // Current speed
}

// Nodes form the railway network
pub struct Node {
    pub position: Vec2,
    pub connections: Vec<NodeId>,
}

// Economy calculates costs
pub fn calculate_velocity(train: &Train) -> f32 {
    // Steam-to-speed conversion
}
```

## Verification

1. All unit tests pass
2. Physics calculations are deterministic
3. Economy values are balanced for gameplay
