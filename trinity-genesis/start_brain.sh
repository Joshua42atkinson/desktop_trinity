#!/bin/bash
# =============================================================================
# TRINITY BRAIN STARTUP SCRIPT
# =============================================================================
# This script starts the Trinity Brain node with Llama-4-Scout model.
# 
# KEY FIXES DOCUMENTED HERE (don't repeat troubleshooting!):
# 1. Kill zombie processes first - they hold GPU memory
# 2. Set HSA_OVERRIDE_GFX_VERSION=11.5.1 for gfx1151 (Strix Halo)
# 3. Set HIP_VISIBLE_DEVICES=0 for single GPU
# 4. Multi-part models (.gguf-00001-of-00002) load automatically
# =============================================================================

set -e

echo "🔧 Trinity Brain Startup Script"
echo "================================"

# Step 1: Kill any zombie processes (CRITICAL - they hold GPU memory!)
echo "🧹 Killing zombie processes..."
pkill -9 -f trinity-brain 2>/dev/null || true
pkill -9 -f "llama.*gguf" 2>/dev/null || true
sleep 2

# Step 2: Set environment variables for Vulkan (Stable Strix Halo)
echo "⚙️  Setting environment for Strix Halo (Vulkan Mode)..."
# STABILITY FIRST: Force CPU-only until GPU issues are resolved
# GPU crashes during model loading - need to fix Vulkan driver or llama.cpp config
# Enable GPU for 73B model performance (Strix Halo Unified Memory)
export TRINITY_GPU_LAYERS=-1
echo "   TRINITY_GPU_LAYERS=-1 (GPU OFFLOAD ENABLED)"
# Use nemotron profile for the NVIDIA Nemotron 3 Nano
export TRINITY_PROFILE=nemotron
# Enable Hybrid Architecture: Use local Llama for planning + LM Studio for Nemotron
export TRINITY_HYBRID_MODE=1
export TRINITY_JR_URL="http://localhost:1234/v1"
echo "   TRINITY_PROFILE set to: $TRINITY_PROFILE (Hybrid Mode Active)"
echo "   TRINITY_JR_URL: $TRINITY_JR_URL (Connecting to LM Studio)"
# Phase 2 Stability: Limit threads to prevent CPU starvation on Strix Halo
export OMP_NUM_THREADS=8
export GGML_VK_DISABLE_INTEGER_DOT_PRODUCT=1
export GGML_VK_DISABLE_F16=1
export TRINITY_CTX_SIZE=32768
export GGML_VULKAN_DEVICE=0
# Disable ROCm variables to prevent conflict
unset HSA_OVERRIDE_GFX_VERSION
unset HIP_VISIBLE_DEVICES
export ROCM_PATH=/opt/rocm
# Step 3: Verify model exists (safeguard)
MODEL_PATH="/home/joshua/.lmstudio/models/lmstudio-community/NVIDIA-Nemotron-3-Nano-30B-A3B-GGUF/NVIDIA-Nemotron-3-Nano-30B-A3B-Q4_K_M.gguf"
if [ ! -f "$MODEL_PATH" ]; then
    echo "❌ Model not found: $MODEL_PATH"
    echo "   Please ensure NVIDIA-Nemotron-3-Nano-30B is downloaded"
    exit 1
fi
echo "✓ Model found: Nemotron 3 Nano (A3B)"

# Step 4: Build if needed
echo "🔨 Building Trinity Brain (release mode)..."
cargo build -p trinity-brain --release 2>&1 | tail -5

# Step 5: Start the brain
echo ""
echo "🧠 Starting Trinity Brain..."
echo "   Model: Llama-4-Scout-17B-16E-Instruct"
echo "   Config: gpu_layers=${TRINITY_GPU_LAYERS:-default} context=${TRINITY_CTX_SIZE:-default}"
echo "   RPC: 0.0.0.0:9000"
echo ""

./target/release/trinity-brain
