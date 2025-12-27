# Trinity Genesis

> A pure-Rust, closest-to-metal AI Operating System with animated avatar UI.

## Architecture

```
Desktop (Brain) ←─ Tarpc/Tailscale ─→ Laptop (Body)
     │                                      │
     ├── LLM Inference (Qwen 235B)         ├── Bevy 3D UI
     ├── ROCm/HIPBLAS                       ├── Animated Avatars
     └── 128GB RAM / 96GB VRAM             └── Chat Interface
```

## Crates

| Crate | Description |
|-------|-------------|
| `trinity-kernel` | Core library: Brain trait, memory, device detection |
| `trinity-protocol` | Tarpc RPC service definitions |
| `trinity-brain` | Desktop inference server binary |
| `trinity-body` | Laptop Bevy UI binary |
| `trinity-skills` | Agent specialist plugins (coder, writer, etc.) |

## Building

```bash
# Full workspace
cargo build --workspace

# Brain node (desktop)
cargo build -p trinity-brain --release

# Body node (laptop)
cargo build -p trinity-body --release
```

## Running

**Desktop (Brain Node):**

```bash
cd trinity-genesis
cargo run -p trinity-brain --release
# Listens on 0.0.0.0:9000
```

**Laptop (Body Node):**

```bash
cd trinity-genesis
cargo run -p trinity-body --release
# Connects to Brain at 100.115.247.4:9000
```

## Network (Tailscale)

| Node | IP Address | Role |
|------|------------|------|
| trinity (desktop) | 100.115.247.4 | Brain - LLM inference |
| quadratical (laptop) | 100.84.217.60 | Body - UI client |

## License

GPL-3.0
