# Trinity / VaaM Production Roadmap

**Status**: Active Construction
**Goal**: Transform the VaaM Blueprint into a shippable "Game-as-Editor" product.

## Phase 1: The Core Physics (Complete)

* [x] **Iron Road Engine**: `iron-road-physics` crate defining Mass, Friction, Steam.
* [x] **Unified Client**: `trinity-client` with Bevy 0.14 integration.
* [x] **VaaM Integration**: `vaam.rs` systems for Drag & Drop logistics checks.

## Phase 2: The Glass Interface (The Body)

*Focus: Allowing the user to SEE and TOUCH the physics.*

* [ ] **Game-as-Editor UI**:
  * Implement `bevy_egui` panels for the "Glass" overlay style.
  * Create the "Train Car" container UI (Lesson Nodes).
* [ ] **Visual Feedback**:
  * Render "Mass" as visual weight (scale/color) on vocabulary words.
  * Render "Friction" as environmental hazards (fog/static).

## Phase 3: The Weigh Station (The Brain)

*Focus: Feeding the physics engine with data.*

* [ ] **LLM Pipeline**:
  * Implement "Weigh Station" handler in `trinity-brain`.
  * Connect to `Llama 4 Scout` (or local equivalent) to analyze words.
* [ ] **Data Ingestion**:
  * Create the "Manifest" UI for bulk word submission.
  * Persist "Weighed" words to `vocabulary_bank` (SQL).

## Phase 4: The Editor Integration

*Focus: Curriculum Design.*

* [ ] **Yoleck Levels**:
  * Define `LessonLevel` struct compatible with Yoleck.
  * Implement Save/Load for lesson maps.
* [ ] **Node Graph**:
  * Integrate `egui_node_graph` for visual scripting of game logic.

## Phase 5: Verification & Polish

* [ ] **End-to-End Test**:
  * Administrator logs in -> Submits Word List -> LLM Weighs Words ->
  * Student logs in -> Sees "Heavy" words -> Solves Puzzle -> Gains Steam.
