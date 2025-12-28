use crate::game::vaam::{CognitiveWeight, VocabularyItem};
use bevy::prelude::*;
use iron_road_physics::VocabularyTier;
use serde::{Deserialize, Serialize};

/// The schema that the LLM (Llama 4 Scout) must adhere to when "weighing" a word.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WordPhysics {
    pub word: String,
    pub definition: String,
    pub tier: VocabularyTier,
    /// 1-100 (Intrinsic Load)
    pub mass: f32,
    /// Semantic tags for socket compatibility (e.g., ["Time", "Decay"])
    pub tags: Vec<String>,
}

/// Trait for the AI Pipeline.
/// In production, this will make async HTTP calls to the Brain (Llama 4).
pub trait WeighStation: Send + Sync {
    fn weigh_word(&self, word: &str) -> Option<WordPhysics>;
}

pub struct Llama3WeighStation {
    // In future: client: reqwest::Client
}

impl Default for Llama3WeighStation {
    fn default() -> Self {
        Self {}
    }
}

impl WeighStation for Llama3WeighStation {
    fn weigh_word(&self, word: &str) -> Option<WordPhysics> {
        // MOCK IMPLEMENTATION (For now)
        // In the full system, this would POST to /api/brain/weigh

        info!("🤖 Llama 3 (Mock) Weighing Word: '{}'", word);

        // Simple heuristic fallback for prototype
        let len = word.len();
        let mass = (len * 5) as f32; // Longer words are heavier?

        Some(WordPhysics {
            word: word.to_string(),
            definition: format!("Mock definition for {}", word),
            tier: if len > 8 {
                VocabularyTier::Hazardous
            } else {
                VocabularyTier::Basic
            },
            mass: mass.clamp(5.0, 100.0),
            tags: vec!["MockTag".to_string()],
        })
    }
}

// System to process generic "IngestWord" events could go here
