---
description: Checkpoint - Build and test all zones to verify project health
---

# Checkpoint Workflow

Run this to create a known-good state you can return to.

## Quick Checkpoint (Build Only)

```bash
cd ~/antigravity/trinity-genesis
cargo build -p trinity-kernel -p trinity-brain -p trinity-body -p iron-road-physics
```

## Full Checkpoint (Build + Test)

// turbo

```bash
cd ~/antigravity/trinity-genesis
cargo build --workspace 2>&1 | tail -20
```

// turbo

```bash
cd ~/antigravity/trinity-genesis  
cargo test --workspace 2>&1 | tail -30
```

## Update Session Turnover

After checkpoint passes, update the session doc:

```bash
cat ~/antigravity/trinity-genesis/docs/SESSION_TURNOVER.md
```

Then edit `SESSION_TURNOVER.md` with:

- What was accomplished
- Current build status
- Next steps
