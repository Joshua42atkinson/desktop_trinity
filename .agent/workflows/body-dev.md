---
description: Development workflow for Trinity Body zone (Bevy UI/Avatar)
---

# Body Zone Development

**Scope**: `trinity-genesis/crates/trinity-body`, `trinity-client`

## Local LLM Rules

You are working on Trinity **ZONE: BODY**

**DO NOT**:

- Modify files outside `crates/trinity-body/` or `crates/trinity-client/`
- Touch LLM/inference code
- Change RPC protocol types

**DO**:

- Focus on Bevy plugins and UI components
- Maintain avatar state machine consistency
- Test visual changes in the native app first

## Build & Test

```bash
cd /home/joshua/antigravity/trinity-genesis

# Build body (native desktop)
cargo build -p trinity-body

# Run desktop app
cargo run -p trinity-body

# Check client (WASM) - currently blocked
# cargo check -p trinity-client --target wasm32-unknown-unknown
```

## Key Files

| File | Purpose |
|------|---------|
| `trinity-body/src/main.rs` | Desktop app entry |
| `trinity-body/src/avatar.rs` | Avatar visuals |
| `trinity-body/src/hud.rs` | HUD overlay |
| `trinity-client/src/lib.rs` | WASM entry (blocked) |

## Bevy Plugin Structure

```rust
// Add new features as plugins
pub struct MyFeaturePlugin;

impl Plugin for MyFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, my_system);
    }
}
```

## Verification

1. Run `cargo run -p trinity-body`
2. Verify avatar renders (cyan sphere)
3. Check HUD elements display correctly
