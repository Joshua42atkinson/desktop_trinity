//! Build script for Trinity Core - AMD Strix Halo (gfx1151) configuration
//!
//! This configures the llama-cpp-2 build for optimal performance on Strix Halo
//! by setting the correct environment variables for HIP/ROCm compilation.

fn main() {
    // Only configure for llama-cpp feature
    #[cfg(feature = "llama-cpp")]
    configure_strix_halo();
}

#[cfg(feature = "llama-cpp")]
fn configure_strix_halo() {
    // AMD Strix Halo GPU architecture (Radeon 8060S iGPU)
    // gfx1151 = RDNA 3.5, 40 Compute Units, 2.9 GHz

    // Set ROCm paths
    println!("cargo:rustc-env=ROCM_PATH=/opt/rocm");
    println!("cargo:rustc-env=HIP_PATH=/opt/rocm/hip");

    // Target GPU architecture for llama.cpp HIPBLAS compilation
    // gfx1151 is the official arch for Strix Halo, but some builds use gfx1103
    println!("cargo:rustc-env=AMDGPU_TARGETS=gfx1151");

    // HSA override for compatibility with gfx1151
    println!("cargo:rustc-env=HSA_OVERRIDE_GFX_VERSION=11.5.1");

    // Enable HIP/HIPBLAS for llama.cpp
    println!("cargo:rustc-env=LLAMA_HIPBLAS=1");

    // Force single GPU device (the iGPU)
    println!("cargo:rustc-env=HIP_VISIBLE_DEVICES=0");

    // Rerun if ROCm installation changes
    println!("cargo:rerun-if-env-changed=ROCM_PATH");
    println!("cargo:rerun-if-env-changed=HIP_PATH");

    // Link paths for HIP libraries (if building with native HIP)
    if std::path::Path::new("/opt/rocm/lib").exists() {
        println!("cargo:rustc-link-search=native=/opt/rocm/lib");
    }
    if std::path::Path::new("/opt/rocm/hip/lib").exists() {
        println!("cargo:rustc-link-search=native=/opt/rocm/hip/lib");
    }
}
