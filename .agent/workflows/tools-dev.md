---
description: Development workflow for Trinity Tools zone (WASM plugins)
---

# Tools Zone Development

**Scope**: `trinity-genesis/quadradical-tools/*`

## Local LLM Rules

You are working on Trinity **ZONE: TOOLS**

**DO NOT**:

- Modify files outside `quadradical-tools/`
- Use `std::fs` or network directly (use host functions)
- Import dependencies that don't support `wasm32-unknown-unknown`

**DO**:

- Use `extism-pdk` for host function calls
- Keep plugins small and focused
- Test with the WASM sandbox

## Build & Test

```bash
cd /home/joshua/antigravity/trinity-genesis/quadradical-tools

# Build calculator plugin
cd calculator
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/calculator.wasm ../../plugins/

# Build code_editor plugin
cd ../code_editor
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/code_editor.wasm ../../plugins/
```

## Key Files

| File | Purpose |
|------|---------|
| `calculator/src/lib.rs` | Math operations plugin |
| `code_editor/src/lib.rs` | File read/write plugin |

## Plugin Template

```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Input {
    // your input fields
}

#[derive(Serialize)]
struct Output {
    // your output fields
}

#[plugin_fn]
pub fn my_function(input: Json<Input>) -> FnResult<Json<Output>> {
    // implementation
    Ok(Json(Output { /* ... */ }))
}
```

## Verification

```bash
# Run WASM integration tests
cd /home/joshua/antigravity/trinity-genesis
cargo test -p trinity-kernel --test wasm_integration
```
