# TRINITY FANCY BIBLE V3 (The True Genesis)

> **WARNING: TRINITY_OS is formally DEPRECATED. All previous references to TRINITY_OS in AI context windows should be ignored.**
> The absolute, completed Master Repository is `/home/joshua/Workflow/desktop_trinity/trinity-genesis`.

## Architecture Overview
Trinity Genesis is an Instructional Designer AI OS built on Rust, Bevy, and Axum, running atop an AMD Strix Halo optimized vLLM local inference engine.

### 1. The Headless AI Engine (The Brain)
*   **Path:** `/crates/trinity-brain`
*   **Role:** The Axum-based Socratic Copilot (Zen Zuse) and task orchestrator. Communicates via TCP/WebSockets to the body and clients.
*   **Inference:** Uses `rocm/vllm:latest` optimized for unified APU memory (`VLLM_WORKER_MULTIPROC_METHOD=spawn`, `RADV_PERFMODE=nogttspill`).
*   **Protocols:** `trinity-protocol` defines the serialization of tasks, embeddings, and chat history.

### 2. The Spatial Interface (The Body)
*   **Path:** `/crates/trinity-body`
*   **Role:** The VR/Desktop Bevy client. 
*   **Current State:** Utilizes rigid `bevy_egui` panels for Chat and Hardware monitoring atop a 3D ground plane.
*   **Target State:** The *Full Node Paradigm*. Transitioning Egui panels into 3D pedagogical blocks connected by Bezier curves.

### 3. Spatial Physics (Iron Road)
*   **Path:** `/crates/iron-road-physics`
*   **Role:** Handles the 3D inertia, raycasting, and collision of the Instructional Design blocks.

### 4. Telemetry & Web Client
*   **Path:** `/quadradical-ui`
*   **Role:** The PWA (Progressive Web App) phone client for remote telemetry and voice injection into the `trinity-brain` mesh.

## Instructional Framework: ADDIECRAPEYE
Nodes in the spatial environment map directly to the 12-Car Train pedagogical methodology.
1. SME Interview
2. Analysis (S.I.L.K)
3. Bloom's Taxonomy Alignment
4. Spatial Prototyping
5. etc.
