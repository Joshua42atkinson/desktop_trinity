#!/bin/bash
set -e

echo "🌌 Initiating Trinity Genesis Migration Protocol..."

# 1. Terminate legacy systems
echo "Running cleanup..."
pkill -f "trinity-brain" || true
pkill -f "trinity-body" || true
pkill -f "day_dream" || true
pkill -f "backend" || true
echo "Cleanup complete."

# 2. Start Trinity Brain (The Mind)
echo "🧠 Awakening Local Intelligence (Trinity Brain)..."
cd /home/joshua/antigravity/trinity-genesis
# Run in background, log to file
nohup cargo run --release -p trinity-brain > brain.log 2>&1 &
BRAIN_PID=$!
echo "Brain Node active (PID: $BRAIN_PID). Logs at trinity-genesis/brain.log"

# Wait for Brain to initialize (Llama model loading takes time)
echo "⏳ Waiting 10s for Brain initialization..."
sleep 10

# 3. Start Trinity Body (The Avatar & IDE)
echo "💃 Materializing Avatar Interface (Trinity Body)..."
cargo run --release -p trinity-body &
BODY_PID=$!

echo "✅ Trinity Genesis is Live."
echo "Press Ctrl+C to stop the Body node (Brain will persist)."

wait $BODY_PID
