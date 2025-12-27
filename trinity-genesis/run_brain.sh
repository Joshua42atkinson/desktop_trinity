#!/bin/bash
set -e

# Trinity Genesis - Strix Halo Startup Script
# Forces ROCm/HIP compilation and runtime settings

echo "🔮 Trinity Genesis: Initializing Strix Halo Environment..."

# 1. Compiler Paths (Force Clang/LLVM for ROCm)
export CC=/opt/rocm/llvm/bin/clang
export CXX=/opt/rocm/llvm/bin/clang++
export CUDACXX=/opt/rocm/bin/hipcc

# 2. Build Flags for Llama.cpp (via cargo)
# Ensure we build for gfx1100 family (compatible with Strix Halo gfx1150/1151)
export GPU_TARGETS="gfx1150;gfx1151" 
export AMDGPU_TARGETS="gfx1150;gfx1151"

# 3. Runtime Flags (Critical for Strix Halo)
# Enable Unified Memory (Zero-Copy)
export GGML_CUDA_ENABLE_UNIFIED_MEMORY=1
# Spoof gfx1151 if needed (DesktopBrain does this too, but safety first)
export HSA_OVERRIDE_GFX_VERSION=11.5.1
# Optimize memory
export HSA_ENABLE_SDMA=0

echo "🧠 Environment Configured:"
echo "   CC: $CC"
echo "   CXX: $CXX"
echo "   UM: $GGML_CUDA_ENABLE_UNIFIED_MEMORY"

# 4. Clean specific crates if requested
if [ "$1" == "clean" ]; then
    echo "🧹 Cleaning llama-cpp-2 artifacts to force HIP rebuild..."
    cargo clean -p llama-cpp-2
    cargo clean -p llama-cpp-sys-2
fi

# 5. Launch
echo "🚀 Launching Trinity Brain (Logging to brain.log)..."
unset CMAKE_TOOLCHAIN_FILE
unset CMAKE_GENERATOR

cargo run -p trinity-brain --release 2>&1 | tee brain.log

