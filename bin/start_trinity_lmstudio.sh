#!/bin/bash
# Start Trinity utilizing LM Studio as the backend
# Usage: ./start_with_lmstudio.sh [LM_STUDIO_URL]

# Default to standard LM Studio port
URL="${1:-http://localhost:1234}"

echo "🧠 Starting Trinity w/ LM Studio Backend"
echo "   Target: $URL"
echo ""
echo "👉 PRE-FLIGHT CHECKLIST:"
echo "   1. Open LM Studio"
echo "   2. Load 'Overthinking Rustacean' (73B) or 'Llama 4 Scout'"
echo "   3. Go to 'Developer' (Server) tab"
echo "   4. Start Server on port ${URL##*:}"
echo "   5. Ensure 'Apply Prompt Formatting' is ON"
echo ""

# Configuration
export USE_TRINITY_JR=1
export TRINITY_JR_URL="$URL"

# Disable local inference to prevent crashes
export TRINITY_GPU_LAYERS=0
export CUDA_VISIBLE_DEVICES=""
export HIP_VISIBLE_DEVICES=""
export ROCR_VISIBLE_DEVICES=""

# Build and Run
cargo build -p trinity-brain --release
./target/release/trinity-brain
