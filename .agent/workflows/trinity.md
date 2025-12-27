---
description: Trinity AI OS development and build workflow
---

# Trinity AI OS Development Workflow

This workflow guides development of the Trinity Rust-native AI agent OS.

## User Preferences

> **IMPORTANT**: The user strongly prefers **pure Rust** implementations.
>
> - Avoid Python, Node.js, or other runtimes when Rust alternatives exist
> - Use `candle-transformers` for ML instead of Python HuggingFace
> - Use `llama-cpp-2` (Rust bindings) instead of Python llama APIs
> - "Close to metal" philosophy: Native code > Interpreted code

## Prerequisites

- Rust toolchain installed (`rustup`)
- WASM target: `rustup target add wasm32-unknown-unknown`
- LM Studio running with GPT-OSS model loaded (optional fallback)
- Docker for PostgreSQL (optional)

---

## Build & Test

// turbo-all

1. Navigate to day_dream workspace

```bash
cd /home/joshua/antigravity/day_dream
```

1. Check compilation

```bash
cargo check --workspace
```

1. Run all tests

```bash
cargo test --workspace
```

1. Build release

```bash
cargo build --workspace --release
```

---

## Run Trinity Backend

1. Start the Axum server

```bash
cd /home/joshua/antigravity/day_dream
cargo run -p backend
```

1. Access at <http://localhost:3000>

---

## LM Studio Configuration

Trinity expects LM Studio to be running at `http://localhost:1234` with an OpenAI-compatible API.

### Verify LM Studio

```bash
curl http://localhost:1234/v1/models
```

### Test Completion

```bash
curl http://localhost:1234/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-oss-120b", "messages": [{"role": "user", "content": "Hello Trinity"}]}'
```

---

## Agent Development

### Run single agent test

```bash
cargo test --package backend -- agent::components --nocapture
```

### Generate documentation

```bash
cargo doc --package backend --no-deps --open
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Leptos SSR errors | Ensure `wasm32-unknown-unknown` target installed |
| LM Studio timeout | Check model is loaded, increase timeout in client |
| Bevy ECS conflicts | Only one Bevy app can run at a time |
