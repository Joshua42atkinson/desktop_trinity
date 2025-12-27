//! Audio Player System for Trinity Body
//!
//! Handles playback of synthesized speech from VoiceResponse audio packets.
//! Uses rodio for cross-platform audio output.

use anyhow::Result;
use rodio::{OutputStream, Sink, Source};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

// ============================================================================
// Audio Player
// ============================================================================

/// Audio packet for playback
#[derive(Debug, Clone)]
pub struct AudioPacket {
    /// Raw PCM samples (16-bit signed LE)
    pub audio_data: Vec<u8>,
    /// Sample rate in Hz
    pub sample_rate: u32,
}

impl AudioPacket {
    /// Create from VoicePacket data
    pub fn from_voice_packet(audio_data: Vec<u8>, sample_rate: u32) -> Self {
        Self {
            audio_data,
            sample_rate,
        }
    }
}

/// Audio player that runs in a background thread
pub struct AudioPlayer {
    /// Sender for audio packets
    tx: mpsc::UnboundedSender<AudioPacket>,
}

impl AudioPlayer {
    /// Create and start the audio player
    pub fn new() -> Result<Self> {
        let (tx, mut rx) = mpsc::unbounded_channel::<AudioPacket>();

        // Spawn the audio playback thread
        std::thread::spawn(move || {
            // Initialize audio output
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to initialize audio output: {}", e);
                    return;
                }
            };

            let sink = Sink::try_new(&stream_handle).unwrap();
            info!("Audio player initialized");

            // Process incoming audio packets
            while let Some(packet) = rx.blocking_recv() {
                debug!(
                    "Playing audio: {} bytes, {} Hz",
                    packet.audio_data.len(),
                    packet.sample_rate
                );

                // Convert to rodio source
                if let Some(source) = pcm_to_source(&packet.audio_data, packet.sample_rate) {
                    sink.append(source);
                    // Wait for playback to finish
                    sink.sleep_until_end();
                }
            }
        });

        Ok(Self { tx })
    }

    /// Queue audio for playback
    pub fn play(&self, packet: AudioPacket) {
        if let Err(e) = self.tx.send(packet) {
            error!("Failed to queue audio: {}", e);
        }
    }

    /// Queue audio from raw PCM data
    pub fn play_pcm(&self, audio_data: Vec<u8>, sample_rate: u32) {
        self.play(AudioPacket::from_voice_packet(audio_data, sample_rate));
    }

    /// Check if player is ready
    pub fn is_ready(&self) -> bool {
        !self.tx.is_closed()
    }
}

/// Convert raw 16-bit PCM to a rodio Source
fn pcm_to_source(data: &[u8], sample_rate: u32) -> Option<impl Source<Item = i16>> {
    if data.is_empty() || data.len() % 2 != 0 {
        return None;
    }

    // Convert bytes to i16 samples
    let samples: Vec<i16> = data
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    let buffer = rodio::buffer::SamplesBuffer::new(1, sample_rate, samples);
    Some(buffer)
}

// ============================================================================
// Bevy Resource
// ============================================================================

/// Bevy resource for audio playback
#[derive(bevy::prelude::Resource)]
pub struct AudioResource {
    player: Arc<AudioPlayer>,
}

impl AudioResource {
    pub fn new() -> Result<Self> {
        Ok(Self {
            player: Arc::new(AudioPlayer::new()?),
        })
    }

    pub fn play(&self, audio_data: Vec<u8>, sample_rate: u32) {
        self.player.play_pcm(audio_data, sample_rate);
    }

    pub fn is_ready(&self) -> bool {
        self.player.is_ready()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcm_conversion() {
        // Create simple test PCM data (silence)
        let data = vec![0u8; 100];
        let source = pcm_to_source(&data, 22050);
        assert!(source.is_some());
    }

    #[test]
    fn test_empty_pcm() {
        let data = vec![];
        let source = pcm_to_source(&data, 22050);
        assert!(source.is_none());
    }
}
