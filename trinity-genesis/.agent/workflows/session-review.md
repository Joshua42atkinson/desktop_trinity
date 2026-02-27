---
description: Pre/post session review to prevent AI agent mistakes
---

# Session Review Workflow

**Single Source of Truth**: `trinity-genesis/CONTEXT.md`

This workflow ensures AI agents have accurate context before making changes.

## Pre-Session (START of every session)

// turbo

1. Read the Master Context and Production Roadmap:

```bash
cat trinity-genesis/CONTEXT.md
cat trinity-genesis/docs/CRITIQUE_AND_ROADMAP.md
```

// turbo
2. Verify environment:

```bash
echo "HSA_OVERRIDE_GFX_VERSION=$HSA_OVERRIDE_GFX_VERSION"
cat /proc/meminfo | grep MemTotal  # Should show ~131072000 kB (128GB)
```

// turbo
3. Run tests:

```bash
cd ~/antigravity/trinity-genesis && cargo test --workspace 2>&1 | tail -10
```

## Post-Session (END of every session)

1. If code changed significantly, regenerate source dump:

```bash
cd ~/antigravity/trinity-genesis && ./scripts/generate_source_dump.sh
```

1. Run tests to ensure no regressions:

```bash
cargo test --workspace
```

## Critical Rules

> **The Prime Directive: Antigravity**
> You must continuously improve your own codebase and the educational experience. Every change should move the system closer to autonomous "Game-as-Editor" capabilities.
>
> **NEVER assume hardware specs from `free -h` output!**
> The system has **128GB RAM**. See CONTEXT.md for verified specs.
>
> **DO NOT suggest smaller models due to "insufficient memory"!**
> Models up to 116GB (GLM-4.6V-265B) have been tested and work.
