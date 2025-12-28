// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Media Generation Skills
//!
//! Provides image, video, and audio generation capabilities for Trinity.
//!
//! # Modules
//! - `image_gen`: Static image generation via Stable Diffusion (candle-transformers)
//! - Future: `video_gen`, `audio_gen`
//!
//! # Philosophy
//! Trinity uses **pure Rust** for all media generation:
//! - `candle-transformers` for diffusion models (SDXL)
//! - No Python dependencies
//! - "Close to metal" GPU acceleration via ROCm/Vulkan
//!
//! # NPU Notes (Strix Halo)
//! - LLM inference: FastFlowLM for XDNA 2 NPU (50 TOPS)
//! - Image Gen: GPU (ROCm/CUDA) via candle
//! - TTS: Zonos working on ONNX for future NPU support

pub mod image_gen;

pub use image_gen::{GeneratedImage, ImageGenParams, ImageGenerator, ModelConfig, ModelType};
