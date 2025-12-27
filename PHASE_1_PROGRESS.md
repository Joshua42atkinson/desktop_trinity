# Phase 1: The Authoring Core (In Progress)

## Goal

Create a stable, private authoring tool that synthesizes Twine (node-based) and Storyline (trigger-based) paradigms.

## Implemented Features (Frontend)

### 1. Node Editor Canvas (`frontend/src/components/authoring/node_canvas.rs`)

- **Feature**: A visual canvas that renders a list of nodes.
- **Interaction**: Supports dragging nodes around the canvas.
- **State**: Manages local state for node positions and dragging status.

### 2. Story Node Component (`frontend/src/components/authoring/story_node.rs`)

- **Feature**: A visual representation of a narrative passage.
- **UI**: Displays title, content preview, and input/output ports (visual only).
- **Styling**: Uses Tailwind CSS for a "cyberpunk/premium" look (slate/cyan theme).

### 3. Authoring Page (`frontend/src/pages/authoring.rs`)

- **Feature**: The main container for the authoring environment.
- **UI**: Includes a header with "Save" and "Settings" buttons, and a placeholder floating toolbar.
- **Routing**: Accessible via `/authoring`.

## Completed Features (Backend & Logic)

### 1. Backend Integration (Expert Module)

- **Status**: ✅ Completed
- **Details**: Implemented `GET` and `POST` endpoints for `StoryGraph`. Connected to Postgres `story_graphs` table.

### 2. Link Logic (Connections)

- **Status**: ✅ Completed
- **Details**: Implemented SVG-based connection rendering and drag-to-connect interaction (Output -> Input).

### 3. Property Editor

- **Status**: ✅ Completed
- **Details**: Created side panel for editing node properties (Title, Content). Syncs with canvas state.

### 4. Graph Management (Quality of Life)

- **Status**: ✅ Completed
- **Details**: Implemented "Add Node" button and "Delete Node" functionality.

### 5. Visual Polish

- **Status**: ✅ Completed
- **Details**: Upgraded connection lines to smooth Bezier curves.

### 6. Deployment (Single Executable)

- **Status**: ✅ Completed
- **Details**: Implemented `rust-embed` to bundle frontend assets into the backend binary. Created `build_release.ps1` for one-click build.

## Next Steps

1. **Canvas Navigation**: Implement Zoom and Pan functionality for the NodeCanvas.
2. **Storyline Triggers**: Implement logic for node transitions based on game state (e.g., "Requires Strength > 5").
3. **Play Mode**: Create a way to "play" the graph from the perspective of a user.
