# Trinity AI OS: Technical Bible

**Status**: Strix Halo "Close to Metal" Edition (Dec 2025)

## Overview

Trinity is a pure-Rust AI OS built specifically for the **AMD Strix Halo** platform. It leverages the 128GB Unified Memory Architecture (UMA) to run massive LLMs natively.

**Core Stack:**

- **Runtime**: Bevy ECS (Game Engine Architecture)
- **Inference**: `llama-cpp-2` (Pure ROCm/HIP 6.4.0)
- **Memory**: Unified Vector/Relational Store
- **Hardware**: AMD Ryzen AI Max+ 395 (16c/32t) + Radeon 8060S (40 WGP)

### 🟢 Current System Status

- **Kernel**: Optimized. 128GB UMA Unlocked. Zombie Reaper Active.
- **Frontend**: Skeleton. Basic Window operational.

---

## Core Systems

### 1. Hardware Memory Manager (`trinity-core/src/memory.rs`)

Manages the AMD Strix Halo's 128GB unified memory pool for "Close to Metal" inference.

**Hardware Stats (Strix Halo):**

- **Total RAM**: 128 GB (LPDDR5X-8000)
- **VRAM Limit**: **Unlimited** (Hardware Pinning Enabled)
- **Verified Max Load**: 105.7 GB (Qwen-235B)

**Performance Tiers:**

| Tier | Model Size (Max) | Headroom (Context) | Use Case |
| :--- | :--- | :--- | :--- |
| **Comfort** | < 64 GB | ~60 GB | Massive Context Research / Deep Retrieval |
| **Sweet Spot** | **96 GB** | ~22 GB | **Recommended Daily Driver** (High Speed + Good Context) |
| **Bare Metal** | 109 GB | ~9 GB | Pure Compute / Benchmarking (Close background apps) |

### 2. Avatar System (`trinity-core/src/visuals/avatar.rs`)

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

### 3. Memory System (`trinity-core/src/learning/memory_system.rs`)

Unified memory combining vector search and relational storage.

```rust
let memory = UnifiedMemory::default_config()?;

// Start a conversation session
let session_id = memory.start_session();

// Store a conversation turn
memory.store_turn("What is Rust?", "Rust is a systems programming language...").await?;

// Recall relevant memories
let memories = memory.recall("memory safety", Some(5)).await?;
```

**Features:**

- Session management (start, resume, track)
- Semantic recall using embeddings
- Context building for LLM prompts
- Vector + relational hybrid storage

**Storage Location:** `~/.trinity/trinity_vectors/`

---

## Process Stability (`trinity-core/src/system/reaper.rs`)

**Zombie Reaper**: Kernel service to prevent GPU memory leaks.

- Auto-detects stale `trinity` or `llama` processes.
- Agressively reaps zombies on boot to ensure the VRAM pool is clean.
- Prevents "Out of Memory" errors caused by crashed sessions.

---

## Agent Swarm (`backend/src/agent/`)

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

## Chat API (`backend/src/routes/chat.rs`)

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
│   ├── visuals/         # 3D avatar
│   ├── memory.rs        # Hardware memory manager
│   ├── config.rs        # Configuration
│   └── device.rs        # GPU detection

backend/
├── src/
│   ├── agent/           # Agent swarm
│   ├── routes/          # API endpoints
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
