# Trinity Genesis Documentation

## Quick Start

Generate and view API documentation:

```bash
cargo docs  # Opens in browser
```

## Crate Overview

### Core Crates

| Crate | Description |
|-------|-------------|
| `trinity-kernel` | Brain, Memory, TTS, Orchestrator - the cognitive core |
| `trinity-protocol` | RPC definitions and types for Brain↔Body communication |
| `trinity-skills` | Specialized capabilities (code gen, image gen, web search) |

### Application Crates

| Crate | Description |
|-------|-------------|
| `trinity-brain` | RPC server binary - runs on the Desktop (Strix Halo) |
| `trinity-body` | Bevy UI application - runs on laptop, connects to Brain |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     User's Laptop                               │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                  trinity-body (Bevy UI)                   │  │
│  │  • 3D Avatar visualization                                │  │
│  │  • Chat interface                                         │  │
│  │  • Task queue management                                  │  │
│  │  • Hardware status monitoring                             │  │
│  └────────────────────────────┬──────────────────────────────┘  │
└───────────────────────────────┼─────────────────────────────────┘
                                │ tarpc RPC (Tailscale VPN)
┌───────────────────────────────┼─────────────────────────────────┐
│                AMD Strix Halo Desktop                           │
│  ┌────────────────────────────▼──────────────────────────────┐  │
│  │                 trinity-brain (Server)                    │  │
│  │  • Llama 4 Scout (109B params, Q4_K_M)                   │  │
│  │  • 128GB unified VRAM for full GPU offload               │  │
│  │  • Local TTS via Piper                                    │  │
│  │  • SDXL Turbo image generation                           │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Documentation Standards

We follow Rust documentation best practices:

- **`//!`** - Module-level documentation (at top of lib.rs/mod.rs)
- **`///`** - Public API documentation (structs, functions, traits)
- **`//`** - Implementation notes, TODOs, internal reasoning

### Comment Markers

| Marker | Purpose |
|--------|---------|
| `// TODO:` | Feature to implement |
| `// FIXME:` | Bug or broken code |
| `// PERF:` | Performance-critical section |
| `// ADR:` | Architecture Decision Record |
| `// NOTE:` | Important context for future developers |

## Model Requirements

See [MODELS.md](./MODELS.md) for download instructions.

| Model | Purpose | Size | Required |
|-------|---------|------|----------|
| Llama 4 Scout Q4_K_M | LLM inference | ~65GB | ✅ Yes |
| Piper TTS voices | Speech synthesis | ~200MB | Optional |
| SDXL Turbo FP16 | Image generation | ~7GB | Optional |
| Cosmos 7B | Physics simulation | ~30GB | Future |
| TRELLIS 2-4B | 3D asset generation | ~8GB | Future |

## Building Documentation

```bash
# Full workspace docs with private items
cargo doc --workspace --document-private-items --no-deps

# Open in browser
cargo docs  # Uses alias from .cargo/config.toml

# Generate for a specific crate
cargo doc -p trinity-kernel --open
```

## Running Tests

```bash
# All tests including doc tests
cargo test-all  # Alias from .cargo/config.toml

# Specific crate
cargo test -p trinity-kernel
```
