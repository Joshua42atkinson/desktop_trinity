//! Voice Output System for Trinity Genesis
//!
//! Defines the voice synthesis interface with emotion/style control,
//! designed for integration with Zonos or similar TTS models.

use serde::{Deserialize, Serialize};

// ============================================================================
// Emotion Parameters
// ============================================================================

/// Emotional state for voice synthesis (Zonos-compatible)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmotionState {
    /// Happiness level (0.0 - 1.0)
    pub happiness: f32,
    /// Anger level (0.0 - 1.0)
    pub anger: f32,
    /// Sadness level (0.0 - 1.0)
    pub sadness: f32,
    /// Fear level (0.0 - 1.0)
    pub fear: f32,
    /// Surprise level (0.0 - 1.0)
    pub surprise: f32,
    /// Disgust level (0.0 - 1.0)
    pub disgust: f32,
}

impl EmotionState {
    pub fn neutral() -> Self {
        Self::default()
    }

    pub fn happy(intensity: f32) -> Self {
        Self {
            happiness: intensity.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    pub fn angry(intensity: f32) -> Self {
        Self {
            anger: intensity.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    pub fn sad(intensity: f32) -> Self {
        Self {
            sadness: intensity.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    pub fn excited() -> Self {
        Self {
            happiness: 0.8,
            surprise: 0.5,
            ..Default::default()
        }
    }

    pub fn concerned() -> Self {
        Self {
            sadness: 0.3,
            fear: 0.4,
            ..Default::default()
        }
    }
}

// ============================================================================
// Voice Style Parameters
// ============================================================================

/// Voice style parameters for TTS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceStyle {
    /// Speaking rate multiplier (0.5 = half speed, 2.0 = double speed)
    pub speed: f32,
    /// Pitch shift in semitones (-12 to +12)
    pub pitch: f32,
    /// Energy/volume level (0.0 - 1.0)
    pub energy: f32,
    /// Voice ID or persona name
    pub voice_id: String,
}

impl Default for VoiceStyle {
    fn default() -> Self {
        Self {
            speed: 1.0,
            pitch: 0.0,
            energy: 0.7,
            voice_id: "trinity".to_string(),
        }
    }
}

// ============================================================================
// Voice Output (Brain -> Voice Service)
// ============================================================================

/// Complete voice output request from Brain to Voice Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceOutput {
    /// Text to synthesize
    pub text: String,
    /// Emotional state for this utterance
    pub emotion: EmotionState,
    /// Voice style parameters
    pub style: VoiceStyle,
    /// Optional stage direction (e.g., "[whispers]", "[shouts]")
    pub direction: Option<String>,
}

impl VoiceOutput {
    /// Create a simple voice output with default emotion/style
    pub fn simple(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            emotion: EmotionState::neutral(),
            style: VoiceStyle::default(),
            direction: None,
        }
    }

    /// Create voice output with emotion
    pub fn with_emotion(text: impl Into<String>, emotion: EmotionState) -> Self {
        Self {
            text: text.into(),
            emotion,
            style: VoiceStyle::default(),
            direction: None,
        }
    }

    /// Set speaking speed
    pub fn speed(mut self, speed: f32) -> Self {
        self.style.speed = speed;
        self
    }

    /// Set pitch
    pub fn pitch(mut self, pitch: f32) -> Self {
        self.style.pitch = pitch;
        self
    }

    /// Add stage direction
    pub fn direction(mut self, dir: impl Into<String>) -> Self {
        self.direction = Some(dir.into());
        self
    }
}

// ============================================================================
// Brain Response (includes voice control)
// ============================================================================

/// Enhanced brain response with voice synthesis parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingResponse {
    /// The text response from the LLM
    pub text: String,
    /// Voice output parameters (if voice synthesis is enabled)
    pub voice: Option<VoiceOutput>,
    /// Avatar state hint (thinking, speaking, etc.)
    pub avatar_state: String,
}

impl SpeakingResponse {
    /// Parse a brain response that may contain stage directions
    /// Format: [EMOTION: value] [SPEED: value] text content
    pub fn parse(raw_text: &str) -> Self {
        let mut emotion = EmotionState::neutral();
        let mut speed = 1.0f32;
        let mut text = raw_text.to_string();
        let mut direction = None;

        // Parse [EMOTION: X] tags
        if let Some(start) = raw_text.find("[EMOTION:") {
            if let Some(end) = raw_text[start..].find(']') {
                let tag = &raw_text[start + 9..start + end].trim().to_lowercase();
                emotion = match tag.as_str() {
                    "happy" | "joy" => EmotionState::happy(0.8),
                    "angry" | "anger" => EmotionState::angry(0.8),
                    "sad" | "sadness" => EmotionState::sad(0.7),
                    "excited" => EmotionState::excited(),
                    "concerned" | "worried" => EmotionState::concerned(),
                    _ => EmotionState::neutral(),
                };
                text = raw_text[start + end + 1..].trim().to_string();
            }
        }

        // Parse [SPEED: X] tags
        if let Some(start) = text.find("[SPEED:") {
            if let Some(end) = text[start..].find(']') {
                let val = &text[start + 7..start + end].trim();
                speed = val.parse().unwrap_or(1.0);
                text = text[start + end + 1..].trim().to_string();
            }
        }

        // Parse [ACT: X] tags
        if let Some(start) = text.find("[ACT:") {
            if let Some(end) = text[start..].find(']') {
                direction = Some(text[start + 5..start + end].trim().to_string());
                text = text[start + end + 1..].trim().to_string();
            }
        }

        Self {
            text: text.clone(),
            voice: Some(VoiceOutput {
                text,
                emotion,
                style: VoiceStyle {
                    speed,
                    ..Default::default()
                },
                direction,
            }),
            avatar_state: "speaking".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_emotion() {
        let resp = SpeakingResponse::parse("[EMOTION: happy] Hello world!");
        assert!(resp.voice.as_ref().unwrap().emotion.happiness > 0.5);
        assert_eq!(resp.text, "Hello world!");
    }

    #[test]
    fn test_parse_with_speed() {
        let resp = SpeakingResponse::parse("[SPEED: 1.5] Fast talking here");
        assert!((resp.voice.as_ref().unwrap().style.speed - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_parse_plain() {
        let resp = SpeakingResponse::parse("Just plain text");
        assert_eq!(resp.text, "Just plain text");
    }
}
