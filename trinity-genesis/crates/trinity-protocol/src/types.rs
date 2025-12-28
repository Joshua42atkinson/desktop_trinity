// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePacket {
    pub audio_data: Vec<u8>,
    pub sample_rate: u32,
}

/// Emotion values for voice synthesis (0.0 - 1.0 each)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmotionData {
    pub happiness: f32,
    pub anger: f32,
    pub sadness: f32,
    pub fear: f32,
    pub surprise: f32,
}

/// Response from chat with voice synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceResponse {
    /// Text content of the response
    pub text: String,
    /// Synthesized audio (WAV format, 16-bit PCM)
    pub audio: Option<VoicePacket>,
    /// Emotion for this response
    pub emotion: EmotionData,
    /// Avatar state hint
    pub avatar_state: AvatarState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFact {
    pub id: String,
    pub content: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub quantization: String,
    pub context_size: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AvatarState {
    Idle,
    Thinking,
    Coding,
    Speaking,
    Sleeping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: u32,
    pub message: String,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProtocolError {}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProtocolError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStats {
    pub memory_used_bytes: u64,
    pub memory_available_bytes: u64,
    pub memory_percent: f32,
    pub cpu_percent: f32,
    pub load_avg_1m: f32,
    pub gpu_available: bool,
}

// ============================================================================
// Image Generation Types
// ============================================================================

/// Request for image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRequest {
    /// Text prompt describing the image
    pub prompt: String,
    /// Negative prompt (things to avoid)
    pub negative_prompt: Option<String>,
    /// Output width (default: 1024)
    pub width: Option<u32>,
    /// Output height (default: 1024)
    pub height: Option<u32>,
    /// Number of inference steps (default: 30)
    pub steps: Option<u32>,
    /// Random seed (None = random)
    pub seed: Option<u64>,
}

impl ImageRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            negative_prompt: None,
            width: None,
            height: None,
            steps: None,
            seed: None,
        }
    }
}

/// Response from image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
    /// PNG image data
    pub image_data: Vec<u8>,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Prompt used
    pub prompt: String,
    /// Seed used
    pub seed: u64,
}

// ============================================================================
// Code Generation Types
// ============================================================================

/// Request for code generation (Coder skill)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRequest {
    /// Description of the code to generate
    pub prompt: String,
    /// Language (e.g., "rust", "python", "typescript")
    pub language: String,
    /// Path to save the output code (if any)
    pub output_path: Option<String>,
    /// Whether to use grammar-constrained sampling
    pub use_grammar: bool,
}

impl CodeRequest {
    pub fn new(prompt: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            language: language.into(),
            output_path: None,
            use_grammar: true,
        }
    }
}

/// Response from code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeResponse {
    /// The generated code
    pub code: String,
    /// Language used
    pub language: String,
    /// Path where code was saved (if any)
    pub saved_path: Option<String>,
    /// Whether syntax appears valid (basic check)
    pub syntax_valid: bool,
}

// ============================================================================
// Document Generation Types
// ============================================================================

/// Writing style for document generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteStyle {
    /// Technical documentation (API docs, READMEs)
    Technical,
    /// Blog post / article style
    BlogPost,
    /// Educational / tutorial
    Tutorial,
    /// Creative / storytelling
    Creative,
    /// Formal / business communication
    Formal,
    /// Casual / conversational
    Casual,
}

impl Default for WriteStyle {
    fn default() -> Self {
        WriteStyle::Technical
    }
}

/// Request for document generation (Writer skill)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
    /// Topic or subject matter
    pub topic: String,
    /// Style of writing
    pub style: WriteStyle,
    /// Target word count (approximate)
    pub target_words: u32,
    /// Path to save the output (if any)
    pub output_path: Option<String>,
}

impl WriteRequest {
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            style: WriteStyle::Technical,
            target_words: 500,
            output_path: None,
        }
    }
}

/// Response from document generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResponse {
    /// The generated content
    pub content: String,
    /// Approximate word count
    pub word_count: u32,
    /// Path where saved (if any)
    pub saved_path: Option<String>,
}

// ============================================================================
// Assessment Generation Types (Educator Skill)
// ============================================================================

/// Type of assessment to generate
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AssessmentType {
    /// Multiple choice quiz
    Quiz,
    /// Hands-on lab project
    Lab,
    /// Coding challenge
    Challenge,
}

/// Difficulty level for assessments
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Request for assessment generation (Educator skill)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentRequest {
    /// Topic for the assessment
    pub topic: String,
    /// Type of assessment to generate
    pub assessment_type: AssessmentType,
    /// Difficulty level
    pub difficulty: Difficulty,
    /// Target audience description
    pub target_audience: String,
}

impl AssessmentRequest {
    pub fn new(topic: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            assessment_type: AssessmentType::Quiz,
            difficulty: Difficulty::Intermediate,
            target_audience: audience.into(),
        }
    }

    pub fn with_type(mut self, assessment_type: AssessmentType) -> Self {
        self.assessment_type = assessment_type;
        self
    }

    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;
        self
    }
}

/// A quiz question with multiple choice options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub correct_answer_idx: usize,
    pub explanation: String,
}

/// A hands-on lab project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabProject {
    pub title: String,
    pub objective: String,
    pub steps: Vec<String>,
    pub starter_code: Option<String>,
    pub solution: Option<String>,
}

/// Response from assessment generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssessmentResponse {
    /// A quiz with questions
    Quiz { questions: Vec<QuizQuestion> },
    /// A lab project
    Lab(LabProject),
}
