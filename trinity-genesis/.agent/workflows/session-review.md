---
description: Pre/post session review to prevent AI agent mistakes
---

# Session Review Workflow

**Single Source of Truth**: `~/antigravity/TECHNICAL_BIBLE.md`

This workflow ensures AI agents have accurate context before making changes.

## Pre-Session (START of every session)

// turbo

1. Read the HARDWARE CONTEXT section at the top of the Tech Bible:

```bash
head -60 ~/antigravity/TECHNICAL_BIBLE.md
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

> **NEVER assume hardware specs from `free -h` output!**
> The system has **128GB RAM**. See TECHNICAL_BIBLE.md for verified specs.
>
> **DO NOT suggest smaller models due to "insufficient memory"!**
> Models up to 116GB (GLM-4.6V-265B) have been tested and work.
