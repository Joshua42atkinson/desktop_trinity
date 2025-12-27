# ROCm Issues on AMD Strix Halo - Deep Dive Research Document

**System:** AMD Ryzen AI Max+ 395 (Strix Halo) with Radeon 8060S (gfx1151)  
**Date:** 2024-12-24  
**Purpose:** Document all ROCm/VRAM issues for troubleshooting and future reference

---

## Executive Summary

The AMD Strix Halo is an APU with **128GB unified memory** that should be available as GPU VRAM. However, ROCm/HIP only exposes ~30GB for single allocations due to missing kernel parameters and llama.cpp bugs.

**Status:**

- ✅ Small models (<30GB) work with full GPU acceleration
- ❌ Large models (>30GB) fail with "cudaMalloc failed: out of memory"

---

## Issue 1: Kernel TTM Pages Limit

### Problem

The Translation Table Manager (TTM) limits pinned memory allocations. Without explicit configuration, large GPU buffer allocations fail even with huge system RAM.

### Symptoms

```
ggml_backend_cuda_buffer_type_alloc_buffer: allocating 63849.62 MiB on device 0: cudaMalloc failed: out of memory
alloc_tensor_range: failed to allocate ROCm0 buffer of size 66951181056
```

### Current State

```bash
$ cat /proc/cmdline | grep amdgpu
amdgpu.gttsize=126976  # ✓ Set (124GB GTT)
# ttm.pages_limit NOT SET ✗
```

### Root Cause

- `amdgpu.gttsize=126976` reserves 124GB for GPU Translation Table
- But TTM (kernel memory manager) still limits individual allocations
- Need `ttm.pages_limit=33554432` to allow 128GB pinned allocations

### Fix

Add to `/etc/default/grub`:

```bash
GRUB_CMDLINE_LINUX="amdgpu.gttsize=131072 ttm.pages_limit=33554432 amd_iommu=off"
```

Then: `sudo update-grub && sudo reboot`

### Why `amd_iommu=off`?

- IOMMU adds latency to GPU memory access
- For unified memory APUs, disabling it improves performance
- Safe on single-user systems (no VM isolation needed)

---

## Issue 2: llama.cpp UMA Detection Bug

### Problem

llama.cpp incorrectly detects Strix Halo as a "Unified Memory Architecture" system and limits memory to `MemAvailable` (~32GB) instead of using the full VRAM.

### Location in Code

`/patches/llama-cpp-sys-2/llama.cpp/ggml/src/ggml-cuda/ggml-cuda.cu` line 3840:

```cpp
bool is_uma = prop.unifiedAddressing > 0 || uma_env;
```

### What Happens

1. ROCm reports `prop.unifiedAddressing > 0` (true for APUs)
2. llama.cpp reads `/proc/meminfo` → gets `MemAvailable: ~32GB`
3. Uses 32GB instead of the 96GB+ VRAM available

### Our Fix (Applied)

```cpp
#ifdef GGML_USE_HIP
    bool is_uma = false;  // Force disable UMA for AMD APUs
#else
    bool is_uma = prop.unifiedAddressing > 0 || uma_env;
#endif
```

### Why This Alone Didn't Work

The UMA fix changes memory *reporting* but doesn't change the actual `hipMalloc` limit. The kernel TTM parameter is still required for large allocations.

---

## Issue 3: Unsupported Model Architectures

### Problem

Our patched llama.cpp version doesn't support some newer model architectures.

### Symptoms

```
llama_model_load: error loading model: unknown model architecture: 'nemotron_h_moe'
llama_model_load: error loading model: unknown model architecture: 'mistral3'
```

### Affected Models

| Model | Architecture | Status |
|-------|-------------|--------|
| NVIDIA Nemotron-30B | nemotron_h_moe | ❌ Not supported |
| Devstral-24B | mistral3 | ❌ Not supported |
| Llama-4-Scout | llama4 | ❌ Needs newer llama.cpp |
| GLM-4.6V-Flash | glm4 | ✅ Works |
| Qwen models | qwen/qwen2 | ✅ Works |
| Llama-3 | llama | ✅ Works |

### Fix Options

1. Update llama.cpp submodule to latest version
2. Use models with supported architectures
3. Wait for upstream support

---

## Issue 4: ROCm Memory Pool Reporting

### Problem

`rocminfo` shows ~32GB per memory pool, not the full 128GB.

### Output

```
Pool 1: Size: 32486512 KB (~31GB)
Pool 2: Size: 32486512 KB (~31GB)
Pool 3: Size: 32486512 KB (~31GB)
Pool 4: Size: 32486512 KB (~31GB)
```

### Explanation

ROCm divides memory into pools for different purposes:

- Host memory pool
- Device memory pool
- Fine-grained coherent pool
- Coarse-grained coherent pool

The pools don't represent allocation limits, but the TTM kernel limit does.

---

## Issue 5: HSA_OVERRIDE_GFX_VERSION

### Problem

gfx1151 (Strix Halo) is very new and may not be fully recognized by ROCm.

### Current Setting

```bash
export HSA_OVERRIDE_GFX_VERSION=11.5.1
```

### Why Needed

- ROCm uses this to select GPU-specific optimizations
- Without it, ROCm may use suboptimal code paths
- `11.5.1` corresponds to gfx1151 (RDNA 3.5)

### Alternative Values

- `11.0.0` - Generic RDNA 3
- `11.0.3` - gfx1103 (different Strix chip)

---

## Issue 6: Multi-Part GGUF Loading

### Problem

Large models split into multiple files (e.g., `-00001-of-00002.gguf`) caused confusion.

### Solution

Point to the **first file only**. llama.cpp automatically loads subsequent parts:

```rust
model_path: "model-00001-of-00002.gguf"  // Loads -00002 automatically
```

---

## Issue 7: Zombie Processes Hold GPU Memory

### Problem

Crashed or improperly terminated brain processes hold GPU memory, causing "out of memory" on next run.

### Symptoms

- GPU shows less free memory than expected
- Same model that worked before now fails

### Fix

Always kill zombie processes before starting:

```bash
pkill -9 -f trinity-brain
pkill -9 -f llama
sleep 2  # Wait for GPU memory to be released
```

---

## Environment Variables Reference

### Required for Strix Halo

```bash
export HSA_OVERRIDE_GFX_VERSION=11.5.1  # GPU version override
export HIP_VISIBLE_DEVICES=0            # Use first GPU
export ROCR_VISIBLE_DEVICES=0           # Same for ROCR runtime
export ROCM_PATH=/opt/rocm              # ROCm installation path
```

### Optional/Debugging

```bash
export HSA_ENABLE_SDMA=0          # Disable SDMA (sometimes helps stability)
export GPU_MAX_HEAP_SIZE=100      # Allow 100% heap allocation
export GPU_MAX_ALLOC_PERCENT=100  # Allow 100% single allocation
export AMD_LOG_LEVEL=4            # Enable AMD debug logging
```

---

## Verification Commands

### Check Kernel Parameters

```bash
cat /proc/cmdline | tr ' ' '\n' | grep -E "amdgpu|ttm|iommu"
```

### Check GPU Info

```bash
rocminfo | head -100
```

### Check Available Memory

```bash
cat /proc/meminfo | grep -E "MemTotal|MemAvailable"
cat /sys/class/drm/card*/device/mem_info_vram_total
```

### Monitor GPU Usage

```bash
rocm-smi
watch -n1 rocm-smi
```

---

## Recommended Next Steps

1. **Immediate:** Use GLM-4.6V-Flash (verified working)
2. **Requires Reboot:** Add TTM kernel parameter for Llama-4-Scout
3. **Future:** Update llama.cpp for newer architectures (Nemotron, Llama-4)

---

## References

- [llama.cpp Issue #18159](https://github.com/ggml-org/llama.cpp/issues/18159) - UMA detection bug
- [llama.cpp PR #17368](https://github.com/ggml-org/llama.cpp/pull/17368) - UMA memory fix
- [Framework Forums Strix Halo](https://community.frame.work/t/amd-framework-laptop-13-running-large-llms) - Community fixes
- [ROCm Documentation](https://rocm.docs.amd.com/) - Official AMD docs
