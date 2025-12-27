#!/bin/bash
# Exit immediately if a command exits with a non-zero status.
set -e

# --- 1. CONFIGURATION ---
LEPTOS_WATCH_PORT="3000"
RELOAD_PORT="3001"
WASM_TARGET="wasm32-unknown-unknown"

print_message() {
  echo "===================================================================================================="
  echo "$1"
  echo "===================================================================================================="
}

# --- 2. TOOLCHAIN SANITY CHECK (AGGRESSIVE FIX) ---
# This is the aggressive fix for the core toolchain error.
print_message "Sanitizing WASM Toolchain (Deep fix for 'can't find crate for core')..."
# Remove and re-add the target to ensure rust-std component is fresh and not corrupted.
rustup target remove ${WASM_TARGET} --toolchain stable || true
rustup target add ${WASM_TARGET} --toolchain stable

# --- 3. EXECUTE IDEMPOTENT ENVIRONMENT SETUP ---
print_message "Executing idempotent environment setup..."
# Run the existing setup script to ensure all dependencies and folders are present.
./scripts/dev_setup.sh

# --- 4. PORT CONFLICT RESOLUTION ---
# Stop any processes blocking the default development and reload ports (3000 and 3001).
print_message "Checking for and terminating processes on ports ${LEPTOS_WATCH_PORT} and ${RELOAD_PORT}..."

# Find and kill processes on port 3000
if command -v lsof &> /dev/null; then
    if lsof -i :${LEPTOS_WATCH_PORT} -t; then
        print_message "Port ${LEPTOS_WATCH_PORT} occupied. Killing process."
        lsof -i :${LEPTOS_WATCH_PORT} -t | xargs kill -9 || true
    fi

    # Find and kill processes on port 3001
    if lsof -i :${RELOAD_PORT} -t; then
        print_message "Port ${RELOAD_PORT} occupied. Killing process."
        lsof -i :${RELOAD_PORT} -t | xargs kill -9 || true
    fi
fi

print_message "Ports confirmed clear (or check skipped). Starting application watcher."

# --- 5. LAUNCH APPLICATION ---
# Move to the project root directory
cd "$(dirname "$0")/.."

print_message "Launching cargo leptos watch from root..."
# This command launches the server and client in watch mode, using the fresh setup.
cargo leptos watch
