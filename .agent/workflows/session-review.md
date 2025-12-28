---
description: Pre/post session review to prevent AI agent mistakes
---

# Session Review Workflow

## Starting a Session

1. **Read CONTEXT.md first**

```bash
cat ~/antigravity/CONTEXT.md
```

1. **Check current state**

```bash
cat ~/antigravity/trinity-genesis/docs/SESSION_TURNOVER.md
```

1. **Pick a zone** - Use one of:
   - `/brain-dev`
   - `/body-dev`
   - `/tools-dev`
   - `/pete-dev`
   - `/iron-road-dev`

---

## Ending a Session

1. **Run quality checks**

```bash
cd ~/antigravity/trinity-genesis
cargo clippy --workspace -- -D warnings 2>&1 | head -50
cargo test --workspace 2>&1 | tail -30
```

1. **Update SESSION_TURNOVER.md**
   - What was accomplished
   - What's next
   - Any blockers

2. **Summarize for Joshua**
   - Plain English, no code jargon
   - What works now that didn't before
   - What to test
