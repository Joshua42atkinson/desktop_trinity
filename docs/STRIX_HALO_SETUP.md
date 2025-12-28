# Strix Halo Setup Guide

Configuration guide for running large LLMs (up to 235B parameters) on AMD Ryzen AI Max+ 395.

## Hardware

| Component | Specification |
|-----------|---------------|
| CPU | AMD Ryzen AI Max+ 395 (16C/32T, Zen 5) |
| GPU | Radeon 8060S (gfx1151, 40 CUs, RDNA 3.5) |
| RAM | 128GB Unified Memory |
| NPU | XDNA 2 (50 TOPS) |

---

## 1. Kernel Parameters (Enable 124GB VRAM)

Add to `/etc/default/grub`:

```bash
GRUB_CMDLINE_LINUX="amd_iommu=off amdgpu.gttsize=126976 ttm.pages_limit=32505856"
```

| Parameter | Purpose |
|-----------|---------|
| `amd_iommu=off` | Disables IOMMU for lower latency |
| `amdgpu.gttsize=126976` | Sets GPU memory limit to 124 GiB |
| `ttm.pages_limit=32505856` | Caps pinned memory to 124 GiB |

**Apply changes:**

```bash
sudo update-grub
sudo reboot
```

> [!IMPORTANT]
> **VRAM Override Note**:
> By manually overriding the GTT size and TTM page limits as shown above, we have successfully "bridged the gap," allowing `llama.cpp` (and other HIP-based backends) to utilize up to **128GB** of unified memory. This unlocks the ability to run massive models like Qwen 235B on consumer hardware.

---

## 2. ROCm Environment

```bash
export ROCM_PATH=/opt/rocm
export HIP_PATH=/opt/rocm/hip
export HSA_OVERRIDE_GFX_VERSION=11.5.1
export HIP_VISIBLE_DEVICES=0
export LLAMA_HIPBLAS=1
export AMDGPU_TARGETS=gfx1151
```

Add to `~/.bashrc` for persistence.

---

## 3. Build Trinity with llama-cpp

```bash
cd ~/antigravity/day_dream
LLAMA_HIPBLAS=1 cargo build -p trinity-core --features llama-cpp --release
```

---

## 4. Model Compatibility

| Model | Size | Fits? |
|-------|------|-------|
| Qwen 72B Q4 | ~45GB | ✅ |
| GPT-OSS-120B Q4 | ~75GB | ✅ |
| Qwen 235B Q3 | ~111GB | ✅ (with 124GB VRAM) |

---

## References

- [lhl/strix-halo-testing](https://github.com/lhl/strix-halo-testing)
- [kyuz0/amd-strix-halo-toolboxes](https://github.com/kyuz0/amd-strix-halo-toolboxes)
- [Ubuntu 24.04 GTT Memory Guide](https://github.com/technigmaai/technigmaai-wiki/wiki/AMD-Ryzen-AI-Max--395:-GTT--Memory-Step%E2%80%90by%E2%80%90Step-Instructions-(Ubuntu-24.04))
