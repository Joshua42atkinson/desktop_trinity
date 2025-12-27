#!/bin/bash
# Start Quadradical (Trinity Jr.) on Strix Halo
# This runs the 73B model as an OpenAI-compatible server

# 1. Kill Zombies
echo "💀 Killing zombie llama-server processes..."
pkill -9 -f llama-server
sleep 2

# 2. Set Strix Halo Environment Variables
echo "🔧 Configuring environment for AMD Strix Halo..."
export HSA_OVERRIDE_GFX_VERSION=11.5.1
export HIP_VISIBLE_DEVICES=0
export ROCR_VISIBLE_DEVICES=0
export ROCM_PATH=/opt/rocm

# 3. Start Server
echo "🚀 Starting Quadradical (73B Rustacean) on port 8081..."
echo "   Model: Overthinking-Rustacean-Behemoth.Q4_K_M.gguf (45GB)"
echo "   Context: 32k"
echo "   Backend: ROCm (gfx1151)"

# Note: --no-mmap is CRITICAL for Strix Halo stability with large models
# Note: -fa (Flash Attention) is CRITICAL for context performance
/home/joshua/antigravity/bin/llama-server \
    -m /home/joshua/antigravity/models/Overthinking-Rustacean-Behemoth.Q4_K_M.gguf \
    --port 8081 \
    --host 0.0.0.0 \
    -c 32768 \
    -ngl 999 \
    --no-mmap \
    --no-mlock \
    -fa \
    --alias quadradical \
    2>&1 | tee quadradical.log
