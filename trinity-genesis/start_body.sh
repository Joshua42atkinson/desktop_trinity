#!/bin/bash
# Trinity Genesis - Start Body Node
# Connects to Brain via Tailscale

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              TRINITY GENESIS - BODY STARTUP                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"

# Default Brain address (Desktop via Tailscale)
BRAIN_ADDR="${BRAIN_ADDR:-127.0.0.1:9000}"

echo "🎨 Starting Trinity Body..."
echo "   Brain address: $BRAIN_ADDR"
echo "   Press 'A' to toggle Antigravity Window"
echo "   Press 'ESC' to exit"
echo ""

cd "$(dirname "$0")"

# Build if needed
if [ ! -f "target/release/trinity-body" ]; then
    echo "📦 Building trinity-body (release)..."
    cargo build -p trinity-body --release
fi

# Start the body
echo "🚀 Launching Body UI..."
export BRAIN_ADDR="$BRAIN_ADDR"
exec cargo run -p trinity-body --release
