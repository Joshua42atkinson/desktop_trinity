# Walkthrough: Expert Module & Authoring Tools

## Goal

Implement the core "Expert Module" functionality, including the backend API for saving/loading story graphs, and the frontend "Authoring" environment with node editing, linking, and property management.

## Changes

### Backend

- Verified `backend/src/routes/expert.rs` and `backend/src/handlers/expert.rs` are correctly implemented to handle `GET /api/expert/graph` and `POST /api/expert/graph`.
- Verified `story_graphs` table exists in database schema.

### Frontend

- **New Page**: `AuthoringPage` accessible at `/authoring`.
- **New Components** (`frontend/src/components/authoring/`):
  - `NodeCanvas`: Main workspace. Handles graph state, loading/saving, and connection logic.
  - `StoryNodeComponent`: Visual representation of a node with Input/Output ports.
  - `PropertyEditor`: Side panel to edit node Title and Content.
- **API**: Added `get_graph` and `save_graph` to `frontend/src/api.rs`.
- **Routing**: Added `/authoring` route to `App.rs` and navigation link.

## Verification Steps

1. **Open Authoring Page**:
    - Navigate to `http://localhost:8080/authoring` (or whatever port frontend runs on).
    - You should see the "DAYDREAM AUTHOR" header and a dark canvas.

2. **Load/Save Graph**:
    - If the database is empty, it should load a default "New Story" with empty nodes.
    - Click "Save Graph". Check the browser console or backend logs to confirm success.

3. **Node Interaction**:
    - **Move**: Drag nodes around the canvas.
    - **Edit**: Click on a node. The "Properties" panel should slide in from the right.
    - Type in "Title" or "Content". The node on the canvas should update immediately.

4. **Linking Nodes**:
    - Click and drag from a node's **Right (Output)** port.
    - A dashed line should follow your mouse.
    - Release the mouse over another node's **Left (Input)** port.
    - A solid cyan line should appear, connecting the nodes.

## Next Steps

- Implement "Add Node" button (currently nodes must exist in DB or be manually added to state).
- Implement "Delete Node/Connection".
- Improve visual styling of connections (Bezier curves).
