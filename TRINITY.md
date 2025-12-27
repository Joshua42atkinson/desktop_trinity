# Trinity AI OS - System Documentation

## Overview

Trinity is a pure-Rust AI OS built for the AMD Strix Halo platform. It combines:

- **Bevy ECS** for game-style runtime and 3D visuals
- **Native LLM inference** via `llama-cpp-2` (Pure ROCm/HIP, no Candle)
- **Semantic memory** with vector storage
- **Agent swarm** for autonomous task execution

### Current Status (Dec 2025)

- **Kernel/Backend**: 🟢 **Optimized** (128GB UMA Unlocked, Zombie Reaper active)
- **Frontend/UI**: 🟡 **Skeleton** (Basic window, waiting for Agent UI implementation)

---

## Core Systems

### 1. Avatar System (`trinity-core/src/visuals/avatar.rs`)

A 3D holographic avatar that represents Trinity visually.

```rust
pub struct TrinityAvatar {
    pub name: String,      // "Trinity Prime"
    pub state: AvatarState, // Idle, Thinking, Coding, Speaking
}
```

**States:**

- `Idle` - Default resting state
- `Thinking` - Processing a request
- `Coding` - Actively generating code
- `Speaking` - Responding to user

**Current Implementation:** Cyan glowing sphere ("Spirit Crystal") with hover animation.

---

### 2. Memory System (`trinity-core/src/learning/memory_system.rs`)

Unified memory combining vector search and relational storage.

```rust
let memory = UnifiedMemory::default_config()?;

// Start a conversation session
let session_id = memory.start_session();

// Store a conversation turn
memory.store_turn("What is Rust?", "Rust is a systems programming language...").await?;

// Recall relevant memories
let memories = memory.recall("memory safety", Some(5)).await?;

// Build context for LLM prompt
let context = memory.build_context("Tell me about Rust").await?;
```

**Features:**

- Session management (start, resume, track)
- Semantic recall using embeddings
- Context building for LLM prompts
- Vector + relational hybrid storage

**Storage Location:** `~/.trinity/trinity_vectors/`

---

### 3. Hardware Memory Manager (`trinity-core/src/memory.rs`)

Manages the AMD Strix Halo's 128GB unified memory.

```rust
let mgr = UnifiedMemoryManager::strix_halo_default(); // 128GB total, 96GB VRAM

// Check available memory
let stats = mgr.stats_live();
println!("Available: {} GB", stats.available_gb);

// Allocate memory
mgr.try_allocate(32 * 1024 * 1024 * 1024); // 32GB
```

**Key Stats:**

- Total RAM: 128 GB
- VRAM Limit: **Unlimited** (Hardware Pinning Enabled)
- **30GB Wall**: Shattered (Verified 105GB+ loads)

**Performance Tiers:**

- **Comfort**: < 64GB Model (Massive Context)
- **Sweet Spot**: 96GB Model (Recommended Daily Driver)
- **Bare Metal**: 109GB Model (Pure Compute, minimal OS headroom)

---

### 3.1 Process Stability (`trinity-core/src/system/reaper.rs`)

**Zombie Reaper**: A kernel-level service that ensures zero GPU memory leaks.

- Scans for stale `trinity` or `llama` processes on boot.
- Aggressively reaps zombies to free up the full 128GB UMA pool.
- Prevents "Out of Memory" errors caused by crashed sessions.

---

### 4. Agent Swarm (`backend/src/agent/`)

Multi-agent system for autonomous task execution.

**Agents:**

| Agent | Role |
|-------|------|
| Router | Distributes tasks to specialists |
| Core | Central reasoning and coordination |
| Research | Information gathering |
| Developer | Code generation and execution |
| Writer | Content creation |

**Workflow System:**

- Nodes represent tasks
- Tokens move through workflow graph
- Each node dispatches to appropriate agent

---

### 5. Chat API (`backend/src/routes/chat.rs`)

REST endpoint for conversation.

```bash
curl -X POST http://127.0.0.1:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello Trinity", "session_id": "optional-uuid"}'
```

**Response:**

```json
{
  "response": "Hello! I'm Trinity, your AI assistant...",
  "session_id": "uuid-here"
}
```

---

## Directory Structure

```
trinity-core/
├── src/
│   ├── brain/           # LLM inference (DesktopBrain)
│   ├── learning/        # Memory systems
│   │   ├── memory_system.rs   # UnifiedMemory
│   │   ├── vector_store.rs    # sled-based vectors
│   │   └── relational_store.rs
│   ├── visuals/         # 3D avatar
│   │   ├── avatar.rs    # TrinityAvatar
│   │   └── movement.rs  # Animations
│   ├── memory.rs        # Hardware memory manager
│   ├── config.rs        # Configuration
│   └── device.rs        # GPU detection

backend/
├── src/
│   ├── agent/           # Agent swarm
│   │   ├── workflow/    # Workflow execution
│   │   └── systems.rs   # Bevy ECS systems
│   ├── routes/          # API endpoints
│   │   ├── chat.rs      # Chat API
│   │   └── memory.rs    # Memory API
│   └── ui/              # Native Bevy+egui UI
```

---

## Network Access

| Service | Local | Remote (Tailscale) |
|---------|-------|-------------------|
| Backend API | <http://127.0.0.1:3000> | <http://100.115.247.4:3000> |
| SSH | localhost | joshua@100.115.247.4 |

**Connected Devices:**

- `trinity` (desktop): 100.115.247.4
- `quadratical` (laptop): 100.84.217.60
