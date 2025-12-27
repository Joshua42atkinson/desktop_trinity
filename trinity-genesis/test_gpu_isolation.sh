#!/bin/bash
# Minimal Brain Test - No Memory, No Orchestrator, No Tasks
# Purpose: Isolate whether the GPU crash is from model loading or from usage

echo "🔬 Minimal Brain Test (GPU Isolation)"
echo "======================================"

# Kill any existing
pkill -f trinity-brain 2>/dev/null || true
sleep 1

# Set environment
export TRINITY_PROFILE=planner
export TRINITY_GPU_LAYERS=-1  # Full GPU
export TRINITY_CTX_SIZE=16384
export GGML_VULKAN_DEVICE=0
export OMP_NUM_THREADS=8
export GGML_VK_DISABLE_INTEGER_DOT_PRODUCT=1

# Use a minimal test binary that just loads the model and does nothing else
# For now, we'll use llama-cli directly to isolate the issue

MODEL="/home/joshua/antigravity/models/Llama-4-Scout-17B-16E-Instruct-GGUF/Llama-4-Scout-17B-16E-Instruct-UD-Q4_K_XL.gguf"

if [ ! -f "$MODEL" ]; then
    echo "❌ Model not found: $MODEL"
    exit 1
fi

echo "📊 Monitoring system resources..."
echo "   CPU cores: $(nproc)"
echo "   Memory: $(free -h | grep Mem | awk '{print $2}')"
echo ""

echo "⏳ Loading model with llama-cli (no inference, just load test)..."
echo "   If this freezes, the issue is in llama.cpp/Vulkan driver"
echo "   If this completes, the issue is in our Rust code"
echo ""

# Just load the model and exit (--prompt "" with -n 0 = no generation)
timeout 120 llama-cli \
    --model "$MODEL" \
    --n-gpu-layers -1 \
    --ctx-size 4096 \
    --threads 8 \
    --prompt "Hello" \
    -n 1 \
    --no-display-prompt \
    2>&1 | head -100

EXIT_CODE=$?
if [ $EXIT_CODE -eq 124 ]; then
    echo "❌ TIMEOUT: Model loading took >120 seconds (GPU hang suspected)"
elif [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Model loaded and generated 1 token successfully!"
    echo "   The issue is likely in our Rust startup code, not llama.cpp"
else
    echo "❌ llama-cli exited with code $EXIT_CODE"
fi
