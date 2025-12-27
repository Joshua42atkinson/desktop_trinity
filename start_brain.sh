#!/bin/bash
# Trinity Brain Launch Script
# Usage: ./start_brain.sh

# 1. Environment Setup
export RUST_LOG=info,trinity_core=debug

# Optional: Uncomment if GFX override is needed (unlikely for 6.4 but safe to keep handy)
# export HSA_OVERRIDE_GFX_VERSION=11.0.0

echo ">>> ---------------------------------------------------"
echo ">>> Trinity AI: Strix Halo Brain Initialization"
echo ">>> Target: Qwen 235B (105GB)"
echo ">>> ---------------------------------------------------"

# 2. Run Verification/Loader
# This loads the model to confirm VRAM health and basic inference stub.
cargo run -p trinity-core --example verify_load --features desktop
