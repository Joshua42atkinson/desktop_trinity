#!/bin/bash
# Trinity AI OS Launcher

# Ensure we are in the project directory
PROJECT_DIR="/home/joshua/antigravity/day_dream"
cd "$PROJECT_DIR" || exit 1

# Set necessary environment variables
# LEPTOS_OUTPUT_NAME is critical for the backend to find client WASM/JS
export LEPTOS_OUTPUT_NAME="frontend" 
export RUST_LOG="info"

# Create log directory if it doesn't exist
mkdir -p "$HOME/.trinity"
LOG_FILE="$HOME/.trinity/trinity.log"

# Redirect stdout and stderr to log file
exec > >(tee -a "$LOG_FILE") 2>&1
echo "--- Starting Trinity Session at $(date) ---"


# Run the backend binary
# Using 'cargo run --release' allows for auto-recompile if needed, 
# but for a "production" app, we'll try to execute the binary directly if it exists.
BINARY_PATH="$PROJECT_DIR/target/release/backend"

if [ -f "$BINARY_PATH" ]; then
    echo "Starting Trinity AI OS (Release Binary)..."
    "$BINARY_PATH"
else
    echo "Binary not found. Compiling and running..."
    cargo run --release -p backend
fi
