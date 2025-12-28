# Trinity Body (The Avatar) - ADDIE Analysis

## Analysis (Maturity Assessment)

**Status**: **[Maturity Level: 2/5 - Split Personality]**
The "Body" concept is currently fragmented between two different implementations with overlapping goals. This represents a significant architectural fork that needs resolution.

### Components

1. **`crates/trinity-body`** (Native App): A tailored Bevy+Egui application designed to be the "Somatic Interface" for the Agent. It connects to the Brain via Tarpc and provides a dashboard (Chat, Hardware Monitor, Antigravity Panel). It is functional but relies on `bevy_egui` which limits its aesthetic potential compared to pure Bevy UI.
2. **`crates/trinity-client`** (WASM Game): A Web-ready Bevy application targeting the "Ask Pete" / "Iron Road" educational use case. It has a `RotateCube` placeholder and minimal logic.

### Friction Points

* **Dual Maintenance**: We have two potential frontends (`trinity-body` vs `trinity-client`). One is for the Agent's self-expression (Body), the other is for the Student's experience (Client).
* **Aesthetics**: `trinity-body` uses `egui` (functional, ugly) vs the vision of "Premium Aesthetics" (Game UI).
* **Connection**: `trinity-client` (Web) will need to connect to `trinity-brain` via the HTTP/Axum endpoints, whereas `trinity-body` (Native) uses the high-speed Tarpc bridge.

## Design

* **Pattern**: "Holographic Terminal". The Body should be a viewport into the Mind.
* **UX Vision**:
  * **Native**: A transparent overlay or desktop companion that visualizes the Agent's thought process (Task Queue, Logs, Chat).
  * **Web**: A gamified learning management system (LMS) where "Lesson Plans" are "Quests."

## Development

* **Language**: Rust (Bevy Engine).
* **Key Systems**:
  * `bridge.rs`: Handles the async communication loop with the Brain without blocking the render frame.
  * `panels/`: Modular UI components (Antigravity, Hardware, Task).
  * `avatar.rs`: Controls the 3D representation (currently primitive).

## Implementation

* **Current State**:
  * `trinity-body`: **Functional Prototype**. connectivity to Brain works, Chat works, Hardware monitoring works.
  * `trinity-client`: **Skeleton**. Basic Bevy setup, WASM bindings, but no real game logic or brain connection yet.

## Evaluation

* **Next Steps**:
    1. **Decide on Convergence**: Should `trinity-body` become the "Server Dashboard" and `trinity-client` the "Student Terminal"?
    2. **Upgrade Aesthetics**: Move `trinity-body` from `egui` to `bevy_ui` (or `sickle_ui`) to match the "Premium" requirement.
    3. **WASM Bridge**: Implement the Axios/Fetch bridge in `trinity-client` to talk to the Brain's Axum API.
