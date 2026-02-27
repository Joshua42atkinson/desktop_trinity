# Trinity: Critique and Roadmap to Completion

**Status**: Late Alpha / Early Beta (~60% Complete)
**Perspective**: Technical Architect & Instructional Designer
**Date**: 2025-12-25

---

## 🧐 Technical Critique

### 1. The "Fragmented Specialists" Problem (Partially Resolved)
Until recently, the system's "specialist" logic (how to write code, how to design a quiz) was duplicated between the `trinity-skills` crate and the `Orchestrator` in `trinity-kernel`. This led to inconsistent output quality.
*   **Progress**: We have centralized the prompts and cleaning logic into `trinity-protocol::skill_utils`.
*   **Remaining Work**: The `Educator` skill and `Media` skill still need end-to-end wiring into the Orchestrator.

### 2. Disconnected Somatics (Body vs. Brain)
The Bevy frontend (`trinity-body`) is a beautiful shell but lacks deep data-binding. Many UI panels (Task Queue, Hardware Stats) show static or placeholder data instead of streaming real-time events from the Brain.
*   **Critique**: A user cannot yet "see" the AI thinking in a way that provides educational value.

### 3. Hardware Moat (Strix Halo)
The system is designed for high-end local hardware (128GB unified memory), but the current environment lacks the drivers (ROCm/HIP) to prove full GPU offloading of 100B+ parameter models.
*   **Risk**: Performance on standard consumer hardware may be "crawling" (0.1 t/s) until the NPU/Vulkan paths are fully optimized.

---

## 🎓 Pedagogical Critique

### 1. The Missing Mastery Loop
The "Rule of Three" (Acquisition, Application, Reinforcement) is defined in the blueprints but not implemented in the ECS. Vocabulary items do not yet "get lighter" (drop in mass) as the student uses them.
*   **Impact**: Currently, Trinity is an AI Chatbot in a 3D window, not yet a "Game-as-Editor."

### 2. Feedback Saliency
The VaaM (Vocabulary-as-a-Mechanic) engine calculates physics correctly, but the UI doesn't make these physics "felt." The student should see a "heavy" word causing visual friction or smoke in the UI.

---

## 🗺️ Roadmap to 1.0 (Completion)

### Phase 1: The Unified Pipe (CURRENT)
*   [x] Centralize Skill prompts and processing logic.
*   [x] Enrich `ArtifactGenerated` events with metadata (syntax checks, word counts).
*   [ ] Wire `Educator` (Quiz generation) into the Orchestrator.

### Phase 2: Rich Somatics (Next Month)
*   [ ] **Code Editor**: Implement a functional `bevy_egui` code editor that displays generated code with syntax highlighting.
*   [ ] **Quiz Runner**: Build the UI component that parses `Artifact::Quiz` and allows the student to answer.
*   [ ] **Event Streaming**: Bind Hardware Stats and Task Queue panels to real RPC data.

### Phase 3: The Mastery Engine (Spring 2025)
*   [ ] **Mastery Component**: Implement `MasteryStatus` in the Bevy ECS.
*   [ ] **Rule of Three Logic**: Create the system that updates mastery after 3 successful physics checks.
*   [ ] **Dynamic Mass**: Ensure mastered words have `effective_mass = 0`.

### Phase 4: Hardware & Optimization
*   [ ] **Vulkan/ROCm Polish**: Ensure 100B+ models load across all 128GB of VRAM without crashing.
*   [ ] **NPU Offloading**: Implement the FastFlowLM runner for the XDNA 2 NPU.

### Phase 5: Classroom Trial (Purdue Pilot)
*   [ ] Administrator logs in -> Submits Word List -> LLM Weighs Words.
*   [ ] Student logs in -> Sees "Heavy" words -> Solves Puzzle -> Gains Steam.

---

## 🎯 The End Goal: "Autopoietic Education"
The project is finished when Trinity can **improve its own educational curriculum**.
The system should detect when a student is struggling with a concept (high friction), autonomously generate a new "Lab Project" (Skill: Educator), and update the game world without human intervention.
