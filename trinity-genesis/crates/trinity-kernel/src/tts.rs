// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Text-to-Speech Engine for Trinity Genesis
//!
//! Provides synthesized speech from text using local ONNX models.
//! Designed to work with the VoiceOutput from voice.rs.
//!
//! # Supported Backends
//! - **Piper** (offline, fast, ONNX-based) - Recommended
//! - **eSpeak** (fallback, robotic but always works)
//! - Future: Coqui XTTS, Zonos

use crate::voice::{EmotionState, VoiceOutput, VoiceStyle};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tracing::{debug, info};

// ============================================================================
// Audio Buffer
// ============================================================================

/// Raw audio buffer ready for playback
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// PCM samples (mono, 22050Hz typically)
    pub samples: Vec<f32>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
}

impl AudioBuffer {
    /// Create an empty buffer
    pub fn empty() -> Self {
        Self {
            samples: Vec::new(),
            sample_rate: 22050,
            channels: 1,
        }
    }

    /// Check if buffer has audio data
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Duration in seconds
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.samples.len() as f32 / self.sample_rate as f32 / self.channels as f32
    }
}

// ============================================================================
// TTS Backend Trait
// ============================================================================

/// Trait for TTS backends
#[async_trait::async_trait]
pub trait TtsBackend: Send + Sync {
    /// Synthesize speech from text
    async fn synthesize(&self, text: &str, style: &VoiceStyle) -> Result<AudioBuffer>;
    
    /// Get backend name
    fn name(&self) -> &'static str;
    
    /// Check if backend is available
    fn is_available(&self) -> bool;
}

// ============================================================================
// Piper TTS Backend (Recommended)
// ============================================================================

/// Piper TTS backend - fast, offline ONNX-based synthesis
///
/// See <https://github.com/rhasspy/piper> for more information.
pub struct PiperBackend {
    /// Path to piper executable
    piper_path: PathBuf,
    /// Path to voice model (.onnx)
    model_path: PathBuf,
    /// Model config path (.onnx.json)
    _config_path: PathBuf,
}

impl PiperBackend {
    /// Create a new Piper backend
    pub fn new(piper_path: PathBuf, model_path: PathBuf) -> Self {
        let config_path = model_path.with_extension("onnx.json");
        Self {
            piper_path,
            model_path,
            _config_path: config_path,
        }
    }

    /// Try to find piper in common locations
    pub fn discover() -> Option<Self> {
        // Check common locations
        let candidates = [
            // Antigravity installation (preferred)
            PathBuf::from("/home/joshua/antigravity/tools/piper/piper/piper"),
            PathBuf::from("/usr/bin/piper"),
            PathBuf::from("/usr/local/bin/piper"),
            dirs::home_dir()?.join(".local/bin/piper"),
            dirs::data_dir()?.join("piper/piper"),
        ];

        let piper_path = candidates.into_iter().find(|p| p.exists())?;

        // Look for a voice model
        let model_dirs = [
            // Antigravity installation (preferred)
            PathBuf::from("/home/joshua/antigravity/tools/piper"),
            // Trinity-specific location
            dirs::home_dir()?.join(".local/share/trinity/models/piper"),
            dirs::data_dir()?.join("piper/voices"),
            dirs::home_dir()?.join(".local/share/piper/voices"),
            PathBuf::from("/usr/share/piper/voices"),
        ];

        for dir in model_dirs {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "onnx").unwrap_or(false) {
                        info!("Found Piper voice model: {:?}", path);
                        return Some(Self::new(piper_path, path));
                    }
                }
            }
        }

        None
    }
}

#[async_trait::async_trait]
impl TtsBackend for PiperBackend {
    async fn synthesize(&self, text: &str, style: &VoiceStyle) -> Result<AudioBuffer> {
        // Calculate speaking rate (piper uses length_scale, inverse of speed)
        let length_scale = 1.0 / style.speed.clamp(0.5, 2.0);

        // Run piper and capture raw audio
        let mut child: Child = Command::new(&self.piper_path)
            .args([
                "--model",
                self.model_path.to_str().unwrap(),
                "--output-raw",
                "--length-scale",
                &length_scale.to_string(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn piper")?;

        // Write text to stdin and close it
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
            // stdin is dropped here, closing the pipe
        }

        let output = child.wait_with_output().context("Piper execution failed")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Piper failed: {}", stderr);
        }

        // Parse raw audio (16-bit signed LE mono 22050Hz by default)
        let samples: Vec<f32> = output
            .stdout
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect();

        Ok(AudioBuffer {
            samples,
            sample_rate: 22050,
            channels: 1,
        })
    }

    fn name(&self) -> &'static str {
        "Piper"
    }

    fn is_available(&self) -> bool {
        self.piper_path.exists() && self.model_path.exists()
    }
}

// ============================================================================
// eSpeak Fallback Backend
// ============================================================================

/// eSpeak fallback - always available on most Linux systems
pub struct ESpeakBackend;

impl ESpeakBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn is_installed() -> bool {
        Command::new("espeak-ng")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for ESpeakBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl TtsBackend for ESpeakBackend {
    async fn synthesize(&self, text: &str, style: &VoiceStyle) -> Result<AudioBuffer> {
        // espeak speed is in words per minute (default 175)
        let speed = (175.0 * style.speed).clamp(80.0, 450.0) as u32;
        
        // Pitch adjustment (espeak uses 0-99, default 50)
        let pitch = (50.0 + style.pitch * 5.0).clamp(0.0, 99.0) as u32;

        let output = Command::new("espeak-ng")
            .args([
                "-s", &speed.to_string(),
                "-p", &pitch.to_string(),
                "--stdout",
                text,
            ])
            .output()
            .context("Failed to run espeak-ng")?;

        if !output.status.success() {
            anyhow::bail!("espeak-ng failed");
        }

        // espeak outputs WAV format - skip 44 byte header, parse 16-bit samples
        let wav_data = &output.stdout;
        if wav_data.len() < 44 {
            return Ok(AudioBuffer::empty());
        }

        // Parse sample rate from WAV header (bytes 24-27)
        let sample_rate = u32::from_le_bytes([wav_data[24], wav_data[25], wav_data[26], wav_data[27]]);

        let samples: Vec<f32> = wav_data[44..]
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect();

        Ok(AudioBuffer {
            samples,
            sample_rate,
            channels: 1,
        })
    }

    fn name(&self) -> &'static str {
        "eSpeak-NG"
    }

    fn is_available(&self) -> bool {
        Self::is_installed()
    }
}

// ============================================================================
// TTS Engine (Main Interface)
// ============================================================================

/// Main TTS engine that manages backends and synthesis
pub struct TtsEngine {
    /// Active backend
    backend: Arc<dyn TtsBackend>,
    /// Emotion modulation enabled
    emotion_enabled: bool,
}

impl TtsEngine {
    /// Create a new TTS engine with auto-detected backend
    pub fn auto_detect() -> Result<Self> {
        // Try backends in order of preference
        if let Some(piper) = PiperBackend::discover() {
            info!("TTS: Using Piper backend");
            return Ok(Self {
                backend: Arc::new(piper),
                emotion_enabled: true,
            });
        }

        if ESpeakBackend::is_installed() {
            info!("TTS: Using eSpeak-NG fallback");
            return Ok(Self {
                backend: Arc::new(ESpeakBackend::new()),
                emotion_enabled: false,
            });
        }

        anyhow::bail!("No TTS backend available. Install piper or espeak-ng.")
    }

    /// Create with a specific backend
    pub fn with_backend(backend: Arc<dyn TtsBackend>) -> Self {
        Self {
            backend,
            emotion_enabled: true,
        }
    }

    /// Get the active backend name
    pub fn backend_name(&self) -> &'static str {
        self.backend.name()
    }

    /// Synthesize speech from VoiceOutput
    pub async fn synthesize(&self, output: &VoiceOutput) -> Result<AudioBuffer> {
        debug!(
            "TTS synthesizing: {} chars, emotion: {:?}",
            output.text.len(),
            output.emotion
        );

        // Apply emotion modulation to style
        let mut style = output.style.clone();
        if self.emotion_enabled {
            self.apply_emotion_modulation(&mut style, &output.emotion);
        }

        self.backend.synthesize(&output.text, &style).await
    }

    /// Synthesize simple text with default style
    pub async fn speak(&self, text: &str) -> Result<AudioBuffer> {
        self.synthesize(&VoiceOutput::simple(text)).await
    }

    /// Apply emotion to voice style parameters
    fn apply_emotion_modulation(&self, style: &mut VoiceStyle, emotion: &EmotionState) {
        // Happiness -> slightly faster, higher pitch
        if emotion.happiness > 0.5 {
            style.speed *= 1.0 + (emotion.happiness * 0.15);
            style.pitch += emotion.happiness * 3.0;
            style.energy = (style.energy + emotion.happiness * 0.2).min(1.0);
        }

        // Sadness -> slower, lower pitch
        if emotion.sadness > 0.5 {
            style.speed *= 1.0 - (emotion.sadness * 0.2);
            style.pitch -= emotion.sadness * 3.0;
            style.energy = (style.energy - emotion.sadness * 0.15).max(0.3);
        }

        // Anger -> faster, louder, slight pitch raise
        if emotion.anger > 0.5 {
            style.speed *= 1.0 + (emotion.anger * 0.1);
            style.pitch += emotion.anger * 2.0;
            style.energy = (style.energy + emotion.anger * 0.3).min(1.0);
        }

        // Fear -> faster, higher pitch, quieter
        if emotion.fear > 0.5 {
            style.speed *= 1.0 + (emotion.fear * 0.2);
            style.pitch += emotion.fear * 4.0;
            style.energy = (style.energy - emotion.fear * 0.1).max(0.4);
        }

        // Surprise -> pitch spike
        if emotion.surprise > 0.5 {
            style.pitch += emotion.surprise * 5.0;
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_buffer_duration() {
        let buffer = AudioBuffer {
            samples: vec![0.0; 22050], // 1 second at 22050Hz mono
            sample_rate: 22050,
            channels: 1,
        };
        assert!((buffer.duration_secs() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_empty_buffer() {
        let buffer = AudioBuffer::empty();
        assert!(buffer.is_empty());
        assert_eq!(buffer.duration_secs(), 0.0);
    }

    #[test]
    fn test_emotion_modulation() {
        let engine = TtsEngine {
            backend: Arc::new(ESpeakBackend::new()),
            emotion_enabled: true,
        };

        let mut style = VoiceStyle::default();
        let emotion = EmotionState::happy(0.8);
        
        let original_speed = style.speed;
        engine.apply_emotion_modulation(&mut style, &emotion);
        
        // Happy should increase speed
        assert!(style.speed > original_speed);
    }
}
