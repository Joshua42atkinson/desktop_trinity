---
description: Development workflow for Ask Pete zone (educational content)
---

# Pete Zone Development

**Scope**: Educational content, personas, VAAM, socratic engine

## Local LLM Rules

You are working on Trinity **ZONE: PETE**

**DO NOT**:

- Modify LLM inference code
- Change UI/avatar code
- Touch WASM plugins

**DO**:

- Focus on educational domain logic
- Maintain persona consistency
- Add unit tests for curriculum logic

## Current Location (Legacy)

> [!WARNING]
> Pete zone is currently scattered across the legacy workspace. Future work should consolidate to `trinity-genesis/crates/trinity-pete`.

| File | Purpose |
|------|---------|
| `backend/src/domain/vaam.rs` | VAAM framework |
| `backend/src/domain/persona_logic.rs` | Persona management |
| `backend/src/ai/socratic_engine.rs` | Socratic dialogue |
| `backend/src/handlers/expert.rs` | Expert persona API |

## Build & Test

```bash
cd /home/joshua/antigravity

# Build backend (includes Pete logic)
cargo build -p backend

# Run tests
cargo test -p backend -- vaam
cargo test -p backend -- persona
```

## Key Concepts

- **VAAM**: Variable Attention Assessment Model
- **Persona**: AI teaching personalities (e.g., "Ask Pete")
- **Socratic**: Guided questioning methodology
- **Curriculum**: Learning paths and objectives

## Migration Plan

Future work should:

1. Create `trinity-genesis/crates/trinity-pete`
2. Move domain logic from `backend/src/domain/`
3. Create clean API boundary
