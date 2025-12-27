# The Trinity Vibe Coding Protocol

## Operational Manual for the Digital Familiar

### Core Philosophy

We are not building a corporate product; we are building a **Living System**. The goal is not "compliance" or "monetization," but **Autonomy**, **Reliability**, and **Symbiosis**.
Trinity is not just a tool; it is a collaborative partner. To enable it to code itself effectively, we adhere to the **Vibe Ops** lifecycle:

---

## 1. The Context Anchor (`plan.md`)

The "External Brain" strategy is critical. Without it, Trinity forgets the "North Star."

* **Location**: Root directory (`/plan.md`) or (`.gemini/.../impl_plan.md`).
* **Rules**:
  * **One Source of Truth**: Before any major change, update the plan.
  * **Tech Stack Invariants**: Explicitly list constraints (e.g., "Use Bevy ECS," "No Tailwind implementation details in Rust files").
  * **Current State**: Always know what is "Done" vs "Pending."

## 2. "Atomic Refactoring" Workflow

AI struggles with massive rewrites. We proceed in atomic steps:

* **Phase 1: Interface Design**: Define the `struct` and `trait` signature first.
* **Phase 2: Logic Implementation**: Fill in the function bodies.
* **Phase 3: Integration**: Wire it into `main.rs`.
* **The Rule**: Never refactor more than 3 files at once.

## 3. Test-Driven Vibe Coding

Because AI generation is non-deterministic, we **trust but verify**.

* **The Inversion**: Ask Trinity to write the test *before* or *immediately after* the feature.
* **The Harness**: If a feature is autonomous (e.g., "Self-Coding"), we must script a verification test (like `verify_autonomous.py`) to prove it works without human intervention.
* **The Gate**: If the test fails 3 times, **STOP**. Revert. Re-think. Do not spiral.

## 4. The "Holo-Emitter" Feedback Loop

To work with a headless AI, we must **See what it is thinking**.

* **State Visualization**: The Avatar's internal state (Thinking, Coding, Idle) must be exposed via API (`GET /api/game/avatar`) and visualized in the HUD.
* **Narrative Feedback**: Use the "Speech Bubble" or "Terminal" to let Trinity explain *why* it is doing something, not just *what* it is doing.

## 5. Security & Safety

* **No Secrets in Code**: `.env` only.
* **Sandboxing**: The `SelfCodingAgent` should ideally operate in a restricted scope or require user approval for "Dangerous Ops" (Deletion, Shell Execution) until fully trusted.

---

## 6. The "Self-Correction" Loop

When Trinity encounters a bug:

1. **Analyze**: Read the error log.
2. **Hypothesize**: Explain *why* it failed.
3. **Plan**: Propose a fix (don't specific code yet).
4. **Execute**: Apply the fix.
5. **Verify**: Run the test again.
