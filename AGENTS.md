# AGENTS.md - Trinity AI OS Control Document

> **Purpose**: Single source of truth for AI agent sessions.
> **Last Updated**: 2025-12-22
> **Architecture**: Rust native (Axum + Leptos + llama-cpp-2)

---

## 🎯 Project Vision

**Trinity AI OS** is a local-first, self-improving AI operating system for AMD Strix Halo hardware with 128GB unified memory. It replaces cloud AI with on-device LLM inference.

**NOT** the old Python/Flask version. This is Rust-native.

---

## 🏗️ Architecture

```
day_dream/
├── backend/          # Axum server + Bevy ECS
├── frontend/         # Leptos WASM
├── trinity-core/     # Core AI library
│   ├── brain/        # LLM inference (llama-cpp-2)
│   ├── learning/     # Memory + embeddings
│   ├── notebook/     # RAG engine
│   └── agent/        # Self-coder tools
├── trinity-desktop/  # Native panels (bevy_egui)
└── common/           # Shared types
```

### Three-Tier LLM System

| Tier | Model | Size | Purpose |
|------|-------|------|---------|
| **Reflection** | Qwen3-235B Thinking | 105GB | Deep reasoning |
| **Tasks** | GPT-OSS 120B | 60GB | Daily work (default) |
| **Swarm** | Gemma-3 27B | 15GB | Parallel ops |

---

## 🚀 Quick Start

```bash
cd /home/joshua/antigravity/day_dream

# Build all
cargo build --workspace

# Run backend (serves frontend too)
cargo run -p backend
# Access: http://localhost:3000

# Run desktop (Bevy panels)
cargo run -p trinity-desktop
```

---

## 📂 Key Files

| File | Purpose |
|------|---------|
| `backend/src/main.rs` | Server entry + route registration |
| `trinity-core/src/brain/desktop.rs` | LLM inference |
| `trinity-core/src/brain/tiered.rs` | Model tier config |
| `trinity-core/src/brain/orchestrator.rs` | Smart routing |
| `backend/src/routes/terminal.rs` | Shell execution API |
| `.agent/workflows/trinity.md` | Build workflow |

---

## 🔧 Development Rules

### Coding Standards

- **No unwrap()** in production code
- **Use spawn_blocking** for CPU-heavy work
- **Async everywhere** in Axum handlers
- **snake_case** file names

### Testing

```bash
cargo test --workspace
cargo clippy --workspace
```

### Feature Flags

```toml
desktop = ["llama-cpp-2", "sled", "sqlx"]  # Full desktop
memory = ["sled", "sqlx"]                   # DB only
embeddings = ["ndarray", "tokenizers"]      # Semantic search
```

---

## 📋 TODO Workflow

When addressing TODOs:

1. **Find all**: `grep -rn "TODO\|FIXME" --include="*.rs" ./`
2. **Categorize**: Critical (blocks core), Medium, Low
3. **Fix or document why not**
4. **Run tests after each**

### Current Critical TODOs

- `rag.rs:119` - Wire to LLM orchestrator
- `embedding.rs` - Real ONNX embeddings (blocked on ort crate)
- `consolidation.rs:97` - Memory consolidation

---

## 🧹 Dead Code Policy

Files with `#[allow(dead_code)]`:

| Category | Action |
|----------|--------|
| Planned features | Keep, document intent |
| Old implementation | Delete if replaced |
| Temporary stubs | Remove or implement |

**Don't delete** if:

- Test/example code
- Interface for future expansion
- Used conditionally by feature flags

---

## ⚠️ Common Mistakes

1. **Blocking async runtime** - Use spawn_blocking for LLM inference
2. **Hardcoded paths** - Use TrinityConfig
3. **Panic on error** - Return Result
4. **Split GGUF** - Point to first shard, llama.cpp loads all

---

## 🗂️ Files to Ignore/Delete

These are **old Python version** files - NOT part of current Rust project:

```
/home/joshua/antigravity/
├── PROJECT_TRINITY.md   # OLD - Python Flask version
├── README.md            # OLD - wrong architecture
├── LM_STUDIO_GUIDE.md   # OUTDATED
├── trinity_*.py         # OLD Python files (if any)
└── _archive/            # Safe to delete
```

Current code is in `/home/joshua/antigravity/day_dream/`

---

## 🔑 For Future AI Sessions

1. **Read this file first**: `day_dream/AGENTS.md`
2. **Workflow**: `.agent/workflows/trinity.md`
3. **Main code**: `backend/src/main.rs`, `trinity-core/src/`
4. **Architecture**: Rust, Axum, Leptos, llama-cpp-2, Bevy ECS
5. **NOT Python** - ignore PROJECT_TRINITY.md

---

## 📡 API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/chat` | POST | LLM conversation |
| `/api/memory/recall` | GET | Semantic search |
| `/api/memory/store` | POST | Store memory |
| `/api/notebook/query` | POST | RAG query |
| `/api/terminal/execute` | POST | Run shell command |
| `/api/terminal/quick` | POST | Quick command |
| `/api/autonomous/status` | GET | Self-coding status |

---

*This document supersedes PROJECT_TRINITY.md and old AGENTS.md*
