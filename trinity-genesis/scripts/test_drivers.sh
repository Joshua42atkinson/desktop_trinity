#!/bin/bash
# test_drivers.sh - Test Driver+Model combinations

DRIVER=$1
MODEL_PATH=$2
MODEL_NAME=$(basename "$MODEL_PATH")
LOG_FILE="test_${DRIVER}_${MODEL_NAME}.log"

echo "🧪 Testing Driver: $DRIVER | Model: $MODEL_NAME"
echo "---------------------------------------------------"

# Select Binary
if [ "$DRIVER" == "hip" ]; then
    SERVER_BIN="/home/joshua/antigravity/bin/llama-server-hip"
    export HSA_OVERRIDE_GFX_VERSION=11.5.1
    export HIP_VISIBLE_DEVICES=0
    export ROCR_VISIBLE_DEVICES=0
elif [ "$DRIVER" == "vulkan" ]; then
    SERVER_BIN="/home/joshua/antigravity/bin/llama-server-vulkan"
    export GGML_VULKAN_DEVICE=0
else
    echo "❌ Unknown driver: $DRIVER"
    exit 1
fi

if [ ! -f "$SERVER_BIN" ]; then
    echo "❌ Binary not found: $SERVER_BIN"
    exit 1
fi

# Cleanup
pkill -9 -f llama-server
sleep 2

# Start Server (Background)
echo "🚀 Starting Server..."
$SERVER_BIN \
    -m "$MODEL_PATH" \
    --port 8089 \
    --host 0.0.0.0 \
    -c 2048 \
    -ngl 999 \
    --no-mmap \
    -fa on \
    > "$LOG_FILE" 2>&1 &

SERVER_PID=$!

# Wait for Ready
echo "⏳ Waiting for server to be ready..."
RETRIES=0
MAX_RETRIES=60 # 60 seconds
READY=0
while [ $RETRIES -lt $MAX_RETRIES ]; do
    if grep -q "HTTP server listening" "$LOG_FILE"; then
        READY=1
        break
    fi
    # Check if process died
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo "❌ Server process died prematurely!"
        break
    fi
    sleep 1
    RETRIES=$((RETRIES+1))
done

if [ $READY -eq 1 ]; then
    echo "✅ Server Ready in ${RETRIES}s"
    
    # Run Inference
    echo "🧠 Running Inference..."
    START_TIME=$(date +%s%N)
    
    curl -s -X POST http://localhost:8089/completion \
        -H "Content-Type: application/json" \
        -d '{"prompt": "Hello, world! Explain quantum physics in one sentence.", "n_predict": 50}' \
        > response.json
        
    END_TIME=$(date +%s%N)
    DURATION=$(( (END_TIME - START_TIME) / 1000000 )) # ms
    
    if grep -q "content" response.json; then
        echo "✅ Inference Successful in ${DURATION}ms"
        echo "📄 Response: $(cat response.json | jq -r '.content' | tr -d '\n')"
        
        # Determine Pass
        echo "PASS" > "result_${DRIVER}_${MODEL_NAME}.txt"
    else
        echo "❌ Inference Failed (No content in response)"
        cat response.json
        echo "FAIL" > "result_${DRIVER}_${MODEL_NAME}.txt"
    fi

else
    echo "❌ Server failed to start within ${MAX_RETRIES}s"
    echo "FAIL" > "result_${DRIVER}_${MODEL_NAME}.txt"
    tail -n 10 "$LOG_FILE"
fi

# Cleanup
kill -9 $SERVER_PID 2>/dev/null
echo "---------------------------------------------------"
echo ""
