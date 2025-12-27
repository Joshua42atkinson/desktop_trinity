# Research Plan: LLM Driver Stability on AMD Strix Halo

## Objective

Evaluate the stability and performance of HIP (ROCm) vs Vulkan drivers for 5 different Large Language Models on the AMD Strix Halo platform (128GB Unified Memory).

## Hardware

- **Platform:** AMD Ryzen AI Max+ 395 (Strix Halo)
- **GPU:** Radeon 8060S (gfx1151)
- **Memory:** 128GB Unified
- **OS:** Linux (Ubuntu 24.04 LTS kernel 6.8+)

## Drivers Tested

1. **HIP (ROCm 6.2+)**
   - Build flags: `-DGGML_HIP=ON -DAMDGPU_TARGETS=gfx1151`
   - Optimization: Flash Attention enabled
2. **Vulkan**
   - Build flags: `-DGGML_VULKAN=ON`
   - Optimization: Async transfer, coarse matrix?

## Models Under Test

| Model Name | Size | Quant | Path | Notes |
|------------|------|-------|------|-------|
| **Overthinking Rustacean** | 73B | Q4_K_M | `models/Overthinking-Rustacean-Behemoth.Q4_K_M.gguf` | Critical for Coding |
| **Llama 4 Scout** | 17B | Q4_K_M | `models/Llama-4-Scout-17B...` | Planner / Reasoning |
| **Devstral Small 2** | 24B | Q4_K_M | `models/Devstral-Small...` | General Assistant |
| **GLM-4.6V Flash** | ? | ? | `models/GLM-4.6V-Flash...` | Vision / Fast |
| **GPT-OSS** | 120B | Q4_K_M | `models/gpt-oss-120b...` | Stress Test (Limit of RAM) |

## Test Methodology

For each Driver + Model combination:

1. **Load Test**: Attempt to load the model into memory.
   - *Pass*: Server starts and reports ready.
   - *Fail*: OOM, Segmentation Fault, or Hang.

2. **Inference Test**: Send a standard "Hello, world" prompt.
   - *Metrics*: Time to First Token (TTFT), Generation Speed (t/s).

3. **Stability Test**:
   - Check logs for warnings/errors.
   - Verify process cleanup (no zombies).

## Execution Plan

1. [In Progress] Compile `llama-server` with HIP.
   - Save as `bin/llama-server-hip`.
2. [Pending] Compile `llama-server` with Vulkan.
   - Save as `bin/llama-server-vulkan`.
3. [Pending] Run automated test script `scripts/test_drivers.sh`.
4. [Pending] Compile results into `docs/RESEARCH_DRIVERS.md`.
