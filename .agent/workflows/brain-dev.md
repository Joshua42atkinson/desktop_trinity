---
description: Development workflow for Trinity Brain zone (LLM/AI inference)
---

# Brain Zone Development

**Scope**: `trinity-genesis/crates/trinity-kernel`, `trinity-brain`, `trinity-protocol`

## Local LLM Rules

You are working on Trinity **ZONE: BRAIN**

**DO NOT**:

- Modify files outside `crates/trinity-kernel/`, `crates/trinity-brain/`, or `crates/trinity-protocol/`
- Add dependencies without listing them
- Change public API signatures without documenting

**DO**:

- Focus on one file at a time
- Maintain RPC interface stability
- Write unit tests for new logic

## Build & Test

```bash
# Navigate to genesis workspace
cd /home/joshua/antigravity/trinity-genesis

# Build brain zone
cargo build -p trinity-kernel -p trinity-brain -p trinity-protocol

# Run tests
cargo test -p trinity-kernel -p trinity-brain -p trinity-protocol

# Start brain server (verify RPC)
./start_brain.sh
```

## Key Files

| File | Purpose |
|------|---------|
| `trinity-kernel/src/orchestrator.rs` | Task orchestration |
| `trinity-kernel/src/wasm_sandbox.rs` | WASM tool execution |
| `trinity-brain/src/main.rs` | Inference server entry |
| `trinity-protocol/src/lib.rs` | RPC types |

## Verification

```bash
# Health check
curl -X POST http://localhost:50051/health

# Test inference (if running)
curl -X POST http://localhost:50051/inference -d '{"prompt":"Hello"}'
```
