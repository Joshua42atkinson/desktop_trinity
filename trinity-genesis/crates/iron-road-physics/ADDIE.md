# Iron Road (Physics) - ADDIE Analysis

## Analysis (Maturity Assessment)

**Status**: **[Maturity Level: 1/5 - Conceptual Prototype]**
The `iron-road-physics` crate is a minimal implementation of the "Cognitive Load Theory as Physics" concept. It is essentially a single file (`lib.rs`) with a basic `Train` and `Node` struct and a velocity calculation function.

### Strengths

* **Clear Vision**: The comments in `lib.rs` clearly articulate the pedagogy: Mass (Intrinsic Load), Steam (Germane Load), Friction (Extraneous Load).
* **Pure Rust**: No external dependencies (other than `serde` likely for state serialization), making it easy to embed anywhere.

### Friction Points

* **Lack of Gameplay**: It calculates a number (`velocity`), but there is no game loop, no world state, and no interaction with `trinity-client` yet.
* **Isolation**: It is currently just a math library. It needs to be integrated into the Bevy ECS to actually drive a train in the 3D world.
* **Simplicity**: The physics model is too simple (linear equation) to support a compelling "Tycoon" style game. It needs more variables (track gradient, boiler pressure, crew fatigue).

## Design

* **Pattern**: "Simulation Engine". A pure logic layer that accepts inputs (Coal added, Track parameters) and returns outputs (Velocity, Position).
* **Metaphor**:
  * **Coal**: Student Motivation (finite resource).
  * **Steam**: Learning Power (conversion of motivation into progress).
  * **Track**: The Curriculum (designed by the Teacher/Agent).

## Development

* **Language**: Rust (No_Std compatible potential).
* **API**:
  * `Train::stroke_fire(amount)`: Input action.
  * `calculate_velocity(train, node)`: Tick function.

## Implementation

* **Current State**: 134 lines of code. Basic unit tests passed.
* **Integration**: Currently NOT used by `trinity-client` or `trinity-brain`. It sits in `crates/` but is not wired up.

## Evaluation

* **Next Steps**:
    1. **ECS Integration**: Create a `trinity_client::game::physics` module that uses this crate to move the Train entity.
    2. **Visual Feedback**: Map `velocity` to the animation speed of the train wheels in Bevy.
    3. **Expansion**: Add "Tech Tree" elements (e.g., "Better Boiler" = "Better Study Habits" = Higher Coal-to-Steam efficiency).
