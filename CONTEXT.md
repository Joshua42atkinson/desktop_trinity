# Trinity AI OS - Master Context

> **Read this first.** This file is the single source of truth for any AI working on Trinity.

## 🎯 What Trinity Is

Trinity is a **Pure-Rust AI OS for teachers** built on constructivist educational philosophy. The user (Joshua) is an instructional designer who does NOT read code - he directs AI agents to build it.

**The Vision**: Teachers use Trinity to create educational video games that teach through discovery (constructivism), not memorization.

---

## 🏗️ The 5 Development Zones

| Zone | Crates | Purpose |
|------|--------|---------|
| 🧠 **BRAIN** | `trinity-kernel`, `trinity-brain`, `trinity-protocol` | LLM inference, memory, orchestration |
| 🎮 **BODY** | `trinity-body`, `trinity-client` | Bevy UI, avatar, visual interaction |
| 🔧 **TOOLS** | `quadradical-tools/*` | WASM sandboxed plugins |
| 📚 **PETE** | (scattered, migrating) | Educational content, personas |
| 🚂 **IRON ROAD** | `iron-road-physics` | Physics game sandbox (the demo) |

**Rule**: Only work on ONE zone at a time. Use `/zone-dev` workflows.

---

## ⚙️ Technical Constraints

- **Pure Rust** - No Python, no JavaScript in core
- **Bevy** - All UI via Bevy ECS (currently 0.14)
- **llama-cpp-2** - Patched for AMD HIP/ROCm
- **Minimal dependencies** - Prefer writing code over adding crates
- **AMD Strix Halo** - 128GB unified memory, ROCm GPU

---

## 📁 Directory Structure

```
antigravity/
├── trinity-genesis/     # THE MAIN PROJECT (work here)
│   ├── crates/          # All Rust code
│   └── docs/            # Genesis-specific docs
├── docs/                # High-level documentation
├── patches/             # llama-cpp AMD fixes (don't touch)
├── llama.cpp/           # Submodule (don't touch)
├── models/              # GGUF model files
├── _archive/            # Legacy code (preserved but unused)
└── _tools/              # External tools (piper TTS)
```

---

## 🔧 Workflows

Use slash commands to load zone context:

- `/brain-dev` - LLM/AI development
- `/body-dev` - Bevy UI development
- `/tools-dev` - WASM plugin development
- `/pete-dev` - Educational content
- `/iron-road-dev` - Physics/game development
- `/pre-commit` - Quality checks before ending session

---

## 📋 Current Blockers

1. **`trinity-client` WASM** - Bevy 0.13/0.14 version conflict
2. **`getrandom`** - WASM target needs explicit feature flags

---

## 🚨 Rules for AI Assistants

1. **Never modify `patches/` or `llama.cpp/`** - These are stable
2. **Stay in your zone** - Don't touch files outside assigned zone
3. **Document everything** - Joshua can't read code, so explain in comments
4. **Test before finishing** - Run `cargo build` and `cargo test`
5. **Update SESSION_TURNOVER.md** - At end of session, document what was done

---

## 📞 Build Commands

```bash
# Build main components
cd ~/antigravity/trinity-genesis
cargo build -p trinity-brain -p trinity-body -p trinity-kernel

# Run brain server
./start_brain.sh

# Run desktop app
cargo run -p trinity-body

# Quality checks
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
