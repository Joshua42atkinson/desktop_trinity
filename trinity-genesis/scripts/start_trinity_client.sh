#!/bin/bash
# Start Trinity Brain in Client Mode (Connecting to Quadradical)
# This keeps the main process lightweight and stable

set -e

echo "🧠 Starting Trinity Brain (Client Mode)..."
echo "   Target: Quadradical (http://localhost:8081)"

# 1. Set Client Flags
export USE_TRINITY_JR=1
export TRINITY_JR_URL="http://localhost:8081"
# Disable local GPU usage to ensure this process is purely CPU/Orchestrator
export TRINITY_GPU_LAYERS=0
export CUDA_VISIBLE_DEVICES=""
export HIP_VISIBLE_DEVICES=""
export ROCR_VISIBLE_DEVICES=""

# 2. Build and Run
# We use release mode for performance in the orchestrator
cargo build -p trinity-brain --release

echo "🚀 Launching Trinity Orchestrator..."
./target/release/trinity-brain
