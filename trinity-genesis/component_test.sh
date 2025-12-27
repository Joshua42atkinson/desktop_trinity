#!/bin/bash
# =============================================================================
# TRINITY COMPONENT ISOLATION TESTING
# =============================================================================
# Tests each component individually to find the crash source.
# Run each test one at a time, rebooting between if needed.
# =============================================================================

set -e
cd /home/joshua/antigravity/trinity-genesis

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        TRINITY COMPONENT ISOLATION TESTING                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Choose which test to run
TEST_MODE="${1:-help}"

case "$TEST_MODE" in

# =============================================================================
# TEST 1: Pure Library Tests (No GPU, No Model)
# =============================================================================
"test1"|"lib")
    echo "🔬 TEST 1: Library Tests (Safe - No GPU)"
    echo "   Tests: memory, runtime, resource manager, etc."
    echo ""
    cargo test -p trinity-kernel --lib 2>&1 | tail -20
    echo ""
    echo "✅ If this passes, Rust code is healthy."
    ;;

# =============================================================================
# TEST 2: Memory System Only
# =============================================================================
"test2"|"memory")
    echo "🔬 TEST 2: Memory System Tests"
    echo "   Tests: AdvancedMemory, UnifiedMemory"
    echo ""
    cargo test -p trinity-kernel advanced_memory 2>&1
    cargo test -p trinity-kernel memory 2>&1 || true
    echo ""
    echo "✅ If this passes, memory DB is healthy."
    ;;

# =============================================================================
# TEST 3: LLM Isolation (CPU-Only, No Memory)
# =============================================================================
"test3"|"llm-cpu")
    echo "🔬 TEST 3: LLM CPU-Only (No Memory Ops)"
    echo "   Uses: test_gpu_isolation.sh"
    echo ""
    export TRINITY_GPU_LAYERS=0
    export TRINITY_PROFILE=planner
    if [ -f "./test_gpu_isolation.sh" ]; then
        ./test_gpu_isolation.sh
    else
        echo "   ⚠️ test_gpu_isolation.sh not found!"
        echo "   Run: llama-cli -m /path/to/model -p 'Hello' -n 10"
    fi
    ;;

# =============================================================================
# TEST 4: LLM Isolation (GPU, No Memory)
# =============================================================================
"test4"|"llm-gpu")
    echo "🔬 TEST 4: LLM with GPU (No Memory Ops)"
    echo "   ⚠️ This may crash if llama.cpp has issues!"
    echo ""
    export TRINITY_GPU_LAYERS=-1
    export TRINITY_PROFILE=planner
    export GGML_VK_DISABLE_INTEGER_DOT_PRODUCT=1
    export GGML_VK_DISABLE_F16=1
    export OMP_NUM_THREADS=4
    
    if [ -f "./test_gpu_isolation.sh" ]; then
        ./test_gpu_isolation.sh
    else
        echo "   ⚠️ test_gpu_isolation.sh not found!"
    fi
    ;;

# =============================================================================
# TEST 5: Full Brain (CPU-Only Mode)
# =============================================================================
"test5"|"brain-cpu")
    echo "🔬 TEST 5: Full Brain (CPU-Only)"
    echo "   This is what's running now - should be stable but slow"
    echo ""
    export TRINITY_GPU_LAYERS=0
    export TRINITY_PROFILE=planner
    export OMP_NUM_THREADS=4
    
    cargo build -p trinity-brain --release
    timeout 30 ./target/release/trinity-brain || echo "Stopped after 30s"
    ;;

# =============================================================================
# TEST 6: Full Brain (GPU Enabled)
# =============================================================================
"test6"|"brain-gpu")
    echo "🔬 TEST 6: Full Brain with GPU"
    echo "   ⚠️ THIS MAY CRASH! Save your work first."
    echo ""
    export TRINITY_GPU_LAYERS=-1
    export TRINITY_PROFILE=planner
    export GGML_VK_DISABLE_INTEGER_DOT_PRODUCT=1
    export GGML_VK_DISABLE_F16=1
    export OMP_NUM_THREADS=4
    
    cargo build -p trinity-brain --release
    ./target/release/trinity-brain
    ;;

# =============================================================================
# TEST 7: Brain WITHOUT Memory (Bypass Test)
# =============================================================================
"test7"|"no-memory")
    echo "🔬 TEST 7: Brain WITHOUT Memory System"
    echo "   Requires code edit to bypass memory in chat()"
    echo "   Edit main.rs line 53-74 to: let memory_context = String::new();"
    echo "   And comment out store() calls on lines 161-166"
    ;;

# =============================================================================
# HELP
# =============================================================================
*)
    echo "USAGE: ./component_test.sh <test>"
    echo ""
    echo "TESTS (run in order, stop when crash occurs):"
    echo "  test1, lib       - Library tests (safe)"
    echo "  test2, memory    - Memory system tests (safe)"
    echo "  test3, llm-cpu   - LLM CPU-only isolation"
    echo "  test4, llm-gpu   - LLM GPU isolation (may crash)"
    echo "  test5, brain-cpu - Full brain CPU mode (current stable)"
    echo "  test6, brain-gpu - Full brain GPU mode (may crash)"
    echo "  test7, no-memory - Instructions for no-memory test"
    echo ""
    echo "RECOMMENDED ORDER:"
    echo "  1. Run test1 - if fails, Rust code is broken"
    echo "  2. Run test2 - if fails, memory system is broken"
    echo "  3. Run test3 - if fails, llama.cpp CPU issue"
    echo "  4. Run test4 - if fails, GPU/Vulkan is the problem"
    echo "  5. Run test6 - if fails but test4 passed, it's memory+GPU combo"
    ;;

esac
