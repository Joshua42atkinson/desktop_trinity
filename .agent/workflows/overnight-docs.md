---
description: Overnight code documentation and polish workflow for Trinity
---

# Trinity Overnight Documentation Workflow

// turbo-all

## Prerequisites

1. Ensure trinity-brain is running:

```bash
cd ~/antigravity/trinity-genesis
cargo run -p trinity-brain
```

1. Verify model is loaded (check for "TRINITY BRAIN - ONLINE" banner)

## Workflow Steps

### Step 1: Document trinity-kernel files

For each file in `crates/trinity-kernel/src/`:

1. Read the file with `view_file`
2. Analyze existing code structure
3. Add module-level `//!` documentation at top
4. Add `///` docs to all public items
5. Add `//` notes for implementation details
6. Write the updated file
7. Run `cargo check -p trinity-kernel` to verify

Files to process:

- lib.rs (already done)
- brain.rs
- brain_desktop.rs
- memory.rs
- orchestrator.rs
- runtime.rs
- tts.rs
- voice.rs
- resource.rs
- device.rs
- config.rs
- system_reaper.rs

### Step 2: Document trinity-protocol files

For each file in `crates/trinity-protocol/src/`:

- lib.rs
- brain.rs
- types.rs
- stream.rs
- task.rs
- memory.rs

### Step 3: Document trinity-body files

For each file in `crates/trinity-body/src/`:

- main.rs
- avatar.rs
- bridge.rs
- audio.rs
- panels/mod.rs
- panels/antigravity.rs
- panels/hardware.rs
- panels/tasks.rs
- panels/workspace.rs

### Step 4: Document trinity-brain

- crates/trinity-brain/src/main.rs

### Step 5: Document trinity-skills

For each file in `crates/trinity-skills/src/`:

- lib.rs
- media/mod.rs
- media/image_gen.rs
- coder.rs
- writer.rs
- web.rs
- drive.rs
- code_editor.rs

### Step 6: Verification

1. Run full build:

```bash
cargo build --workspace
```

1. Run clippy:

```bash
cargo clippy --workspace -- -W clippy::all
```

1. Generate docs:

```bash
cargo doc --workspace --no-deps
```

1. Create friction log:

```bash
cat > docs/FRICTION_LOG.md << 'EOF'
# Trinity Friction Log

## Date: 2024-12-25

### Issues Discovered During Documentation

| File | Issue | Severity | Suggested Fix |
|------|-------|----------|---------------|
| (to be filled) | | | |

### Lessons Learned

1. (to be filled during documentation)

### Next Steps

1. (improvements discovered during review)
EOF
```

### Step 7: Summary Report

Create a summary of work done:

```bash
echo "Documentation complete. Files processed: $(find crates -name '*.rs' | wc -l)"
cargo doc --workspace --no-deps 2>&1 | tail -5
```

## Documentation Templates

### Module Header Template

```rust
//! # [Module Name]
//!
//! [One-line description]
//!
//! ## Overview
//!
//! [What this module does and why]
//!
//! ## Key Types
//!
//! - [`Type1`] - Description
//! - [`Type2`] - Description
//!
//! ## Architecture
//!
//! [How this fits into the larger system]
```

### Struct Template

```rust
/// [One-line description]
///
/// [Longer explanation if needed]
///
/// # Example
///
/// ```rust,ignore
/// let instance = StructName::new();
/// ```
pub struct StructName {
    /// [Field description]
    pub field: Type,
}
```

### Function Template

```rust
/// [What this function does]
///
/// # Arguments
///
/// * `param` - [Description]
///
/// # Returns
///
/// [What is returned]
///
/// # Errors
///
/// [When this can fail]
pub fn function_name(param: Type) -> Result<T> {
    // Implementation
}
```
