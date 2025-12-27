# Phoenix Protocol: Trinity Reliability System

## Philosophy

"Built like a Toyota, not a Ford."
The system must be self-healing, persistent, and capable of "reincarnation" (clean restarts from a known good state) when catastrophic failure occurs.

## Components

### 1. `trinity-monitor` (The Watchdog)

A lightweight Rust daemon that runs independently of the Brain/Body.

- **Role**: Heartbeat Monitor & Repairman.
- **Checks**:
  - **Port 9000**: Is the Brain accepting connections?
  - **Process State**: Are `trinity-brain` processes zombies?
  - **GPU Memory**: Is VRAM leaked? (via `rocm-smi` parsing)
- **Actions**:
  - **Restart**: `systemctl restart trinity-brain`
  - **Kill**: `pkill -9` if stuck in D/Z state.
  - **Alert**: Pushes notification to "Antigravity" (Cloud/UI).

### 2. Systemd Integration

We move away from user-space scripts (`start_brain.sh`) to managed services.

**`trinity-brain.service`**:

```ini
[Unit]
Description=Trinity AI Brain (Nucleus)
After=network.target

[Service]
Type=simple
User=joshua
ExecStart=/home/joshua/antigravity/trinity-genesis/target/release/trinity-brain
Restart=always
RestartSec=5
# Critical: Resource Limits
LimitNOFILE=65536
Environment=HSA_OVERRIDE_GFX_VERSION=11.5.1
Environment=HIP_VISIBLE_DEVICES=0

[Install]
WantedBy=default.target
```

**`trinity-monitor.service`**:
Runs the watchdog. Restart=always.

### 3. "Reincarnation" (Cloud Tether)

A failsafe mechanism if local restarts fail loop.

- **Concept**: If `trinity-monitor` detects 3 failed restarts in 5 minutes:
    1. It queries a remote endpoint (e.g., `Jules.google.com/status` or a gist).
    2. If the remote says "NUKE", it wipes the local `memory.db` (caches only) and performs a clean factory boot.
    3. This mimics "divine intervention" for unrecoverable states.

## Implementation Plan

1. **Scaffold**: Create `crates/trinity-monitor`.
2. **Service Files**: Write `.service` files to `~/.config/systemd/user/`.
3. **Enable**: `systemctl --user enable trinity-brain`.
