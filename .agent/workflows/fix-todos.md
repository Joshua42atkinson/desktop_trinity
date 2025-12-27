---
description: Workflow for addressing TODOs and FIXMEs in Trinity codebase
---
# TODO Resolution Workflow

Systematic approach to fixing TODOs and FIXMEs.

// turbo-all

## 1. Scan for TODOs

```bash
cd /home/joshua/antigravity/day_dream
grep -rn "TODO\|FIXME" --include="*.rs" ./backend/src ./trinity-core/src | head -50
```

## 2. Categorize by Priority

### Critical (Blocks Core)

- RAG not wired to LLM
- Missing embeddings
- Broken integrations

### Medium (Degrades UX)

- Stubbed functionality
- Missing validations
- Performance issues

### Low (Cleanup)

- Documentation
- Code style
- Minor enhancements

## 3. Fix One at a Time

For each TODO:

```bash
# 1. View the file
cat -n <file> | head -<line+10> | tail -20

# 2. Implement fix

# 3. Check compile
cargo check --package <pkg>

# 4. Test
cargo test --package <pkg>

# 5. Commit
git add <file>
git commit -m "TODO: <description>"
```

## 4. Common TODO Patterns

### "Replace with real embedding"

**Location**: learning/embedding.rs, notebook/ingest.rs, routes/memory.rs
**Fix**: When ort crate stabilizes, replace hash_based_embedding with ONNX model

### "Integrate with LLM"

**Location**: rag.rs, consolidation.rs
**Fix**: Call orchestrator.process() to synthesize answers

### "Track actual duration"

**Location**: autonomous.rs
**Fix**: Use std::time::Instant before/after task

### "Connect to ECS state"

**Location**: avatar_api.rs
**Fix**: Use shared resource or channel to Bevy world

## 5. Validation

After fixing batch of TODOs:

```bash
# Full check
cargo check --workspace
cargo clippy --workspace
cargo test --workspace

# Push if passing
git push origin main
```

## 6. Update AGENTS.md

Remove fixed items from Critical TODOs section in AGENTS.md.
