#!/bin/bash
# Start Trinity in Hybrid Mode (Local Vulkan Planner + Remote LM Studio Worker)

# Default to standard LM Studio port
URL="${1:-http://localhost:1234}"

echo "🧠 Starting Trinity Hybrid Architecture"
echo "   [Planner] Local Llama 4 Scout (Vulkan)"
echo "   [Worker]  Remote LM Studio ($URL)"
echo ""
echo "👉 PRE-FLIGHT CHECKLIST:"
echo "   1. Open LM Studio"
echo "   2. Load 'Overthinking Rustacean' (73B)"
echo "   3. Start Server on port 1234"
echo "   4. Ensure Local Model 'Llama-4-Scout-17B' is in ~/antigravity/models"
echo ""

# Enable Hybrid Mode
export TRINITY_HYBRID_MODE=1
export TRINITY_JR_URL="$URL"

# Enable Vulkan for Planner
export GGML_VULKAN_DEVICE=0
# Ensure ROCm variables don't conflict (unset them just in case)
export HSA_OVERRIDE_GFX_VERSION="" 
export HIP_VISIBLE_DEVICES=""
export ROCR_VISIBLE_DEVICES=""

# Build and Run
cargo build -p trinity-brain --release
./target/release/trinity-brain
