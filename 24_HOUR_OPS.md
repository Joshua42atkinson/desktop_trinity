# 24/7 Autonomous Operations Guide

## Overview

This guide explains how to run the Trinity AI OS in a 24/7 autonomous mode, leveraging the AMD Strix Halo hardware for continuous self-improvement and goal seeking.

## Prerequisites

- **Models**: Ensure `Qwen3-235B` (Tier 1) and `GPT-OSS 120B` (Tier 2) are downloaded to `~/.lmstudio/models`.
- **Environment**: Your `run_trinity.sh` is already configured for `USE_NATIVE=true`.

## Running in Background (Screen/Tmux)

To keep Trinity running even when you close your terminal, use `screen` or `tmux`.

### Using Screen

1. Start a new session:

   ```bash
   screen -S trinity_core
   ```

2. Navigate to the project directory:

   ```bash
   cd /home/joshua/antigravity/day_dream
   ```

3. Run the launcher:

   ```bash
   ./run_trinity.sh
   ```

4. Detach from the session:
   Press `Ctrl+A`, then `D`.

### Reattaching

To view the logs or stop the agent:

```bash
screen -r trinity_core
```

## Goal Seeking

Trinity now actively looks for a `GOALS.md` file in the workspace root during its "Dream Cycle" (every hour).

1. Create a `GOALS.md` file in `/home/joshua/antigravity`:

   ```markdown
   # Autonomous Goals
   
   - [ ] Refactor the memory system to be more efficient
   - [ ] Create a new visualizer for the neural network
   - [ ] Write a blog post about Strix Halo performance
   ```

2. Trinity will pick up the first uncompleted item and create an `active_goal_plan.md` to track its progress.

## Status Indicators

- **Green Spirit Crystal**: Idle / Ready
- **Blue Spirit Crystal**: Thinking / Inference (High GPU Load)
- **Purple Spirit Crystal**: Dreaming / Memory Consolidation

> [!WARNING]
> This mode uses significant power and generates heat. Ensure your Strix Halo device has adequate cooling.
