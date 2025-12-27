# Trinity Overnight Documentation Task

## Mission: Christmas Morning UI Polish 🎄

Trinity will autonomously work overnight to:

1. Add comprehensive documentation comments to every file
2. Create a friction log of issues discovered
3. Polish the UI for a Christmas morning demo

---

## Time: ~8 hours (Dec 24 evening → Dec 25 morning)

## Target Files (Priority Order)

### Phase 1: Core Kernel (2 hours)

- [ ] `trinity-kernel/src/lib.rs` - Module overview ✓ (done)
- [ ] `trinity-kernel/src/brain.rs` - Brain trait docs
- [ ] `trinity-kernel/src/brain_desktop.rs` - Strix Halo inference
- [ ] `trinity-kernel/src/memory.rs` - Vector memory system
- [ ] `trinity-kernel/src/orchestrator.rs` - Multi-agent dispatch
- [ ] `trinity-kernel/src/runtime.rs` - Task queue
- [ ] `trinity-kernel/src/tts.rs` - Voice synthesis
- [ ] `trinity-kernel/src/voice.rs` - Emotion system
- [ ] `trinity-kernel/src/resource.rs` - Hardware management
- [ ] `trinity-kernel/src/device.rs` - GPU/NPU detection

### Phase 2: Protocol Layer (1 hour)

- [ ] `trinity-protocol/src/lib.rs` - RPC overview
- [ ] `trinity-protocol/src/brain.rs` - Service trait
- [ ] `trinity-protocol/src/types.rs` - Shared types
- [ ] `trinity-protocol/src/stream.rs` - Event streaming
- [ ] `trinity-protocol/src/task.rs` - Task definitions
- [ ] `trinity-protocol/src/memory.rs` - Memory types

### Phase 3: Body UI (2 hours)

- [ ] `trinity-body/src/main.rs` - Bevy app entry
- [ ] `trinity-body/src/avatar.rs` - Spirit Crystal animation
- [ ] `trinity-body/src/bridge.rs` - RPC client
- [ ] `trinity-body/src/audio.rs` - Audio playback
- [ ] `trinity-body/src/panels/mod.rs` - Panel system
- [ ] `trinity-body/src/panels/antigravity.rs` - Thought stream
- [ ] `trinity-body/src/panels/hardware.rs` - Stats display
- [ ] `trinity-body/src/panels/tasks.rs` - Task queue UI
- [ ] `trinity-body/src/panels/workspace.rs` - Mode switching

### Phase 4: Brain Server (1 hour)

- [ ] `trinity-brain/src/main.rs` - RPC server

### Phase 5: Skills (1 hour)

- [ ] `trinity-skills/src/lib.rs` - Skills overview
- [ ] `trinity-skills/src/media/mod.rs` - Media module
- [ ] `trinity-skills/src/media/image_gen.rs` - SDXL generation
- [ ] `trinity-skills/src/coder.rs` - Code generation
- [ ] `trinity-skills/src/writer.rs` - Content creation
- [ ] `trinity-skills/src/web.rs` - Web browsing

### Phase 6: Polish & Test (1 hour)

- [ ] Run `cargo doc --open` to verify
- [ ] Run `cargo clippy` and fix warnings
- [ ] Run `cargo build` to ensure compilation
- [ ] Create `FRICTION_LOG.md` with discovered issues

---

## Documentation Standard

For each file, add:

### 1. Module-level docs (`//!`)

```rust
//! # Module Name
//!
//! Brief description.
//!
//! ## Overview
//! What this module does and why it exists.
//!
//! ## Key Types
//! - [`MainType`] - Description
//!
//! ## Example
//! ```rust,ignore
//! // Usage example
//! ```
```

### 2. Struct/Enum docs (`///`)

```rust
/// Brief one-line description.
///
/// More detailed explanation if needed.
///
/// # Fields (for structs)
/// - `field_name` - What it does
///
/// # Variants (for enums)
/// - `Variant` - When to use it
```

### 3. Function docs (`///`)

```rust
/// Brief description of what the function does.
///
/// # Arguments
/// * `param` - Description
///
/// # Returns
/// Description of return value
///
/// # Errors
/// When this function can fail
///
/// # Example
/// ```rust,ignore
/// let result = function_name(arg);
/// ```
```

### 4. Implementation notes (`//`)

```rust
// NOTE: Why this design choice was made
// TODO: Future improvement needed
// FIXME: Known issue
// PERF: Performance consideration
// ADR: Architecture Decision Record
```

---

## Friction Log Template

Create `~/antigravity/trinity-genesis/docs/FRICTION_LOG.md`:

```markdown
# Trinity Friction Log

## Date: 2024-12-25

### Issues Discovered

| File | Issue | Severity | Fix |
|------|-------|----------|-----|
| file.rs | Description | Low/Med/High | Suggested fix |

### Lessons Learned

1. **Lesson title**
   - Context
   - What we learned
   - How to apply it

### Suggested Improvements

1. Improvement idea
   - Why it matters
   - How to implement
```

---

## Success Criteria

By Christmas morning, the codebase should:

✅ Have `///` docs on every public struct, enum, trait, function
✅ Have `//!` module docs at the top of every file  
✅ Have `//` implementation notes explaining non-obvious logic
✅ Compile with `cargo build` without errors
✅ Pass `cargo clippy` with minimal warnings
✅ Generate complete docs with `cargo doc`
✅ Have a FRICTION_LOG.md documenting all discovered issues

---

## How to Run

Trinity should be started with this task:

```bash
# Start Trinity Brain
cd ~/antigravity/trinity-genesis
cargo run -p trinity-brain

# In another terminal, submit the task
# (Or use the UI to submit these tasks)
```

Trinity will use the Orchestrator to:

1. Read each file
2. Analyze the code
3. Generate appropriate documentation comments
4. Write the updated file
5. Verify it compiles
6. Move to the next file
