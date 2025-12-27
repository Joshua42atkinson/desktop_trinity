//! # Educator Skill - Assessment and Curriculum Generation
//!
//! ## Philosophy
//! "The Educator transforms raw knowledge into mastery. It designs
//!  paths for students to climb, from theory to code."

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info};
use trinity_kernel::GrammarSpec;

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

/// Difficulty level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Request to generate an assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentRequest {
    pub topic: String,
    pub assessment_type: AssessmentType,
    pub difficulty: Difficulty,
    pub target_audience: String,
}

/// A question in a quiz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub question: String,
    pub options: Vec<String>,
    pub correct_answer_idx: usize,
    pub explanation: String,
}

/// A lab project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lab {
    pub title: String,
    pub objective: String,
    pub steps: Vec<String>,
    pub starter_code: Option<String>,
    pub solution: Option<String>,
}

/// Response containing the generated assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AssessmentResponse {
    Quiz { questions: Vec<Question> },
    Lab(Lab),
}

/// Educator skill for generating educational content
pub struct Educator {
    system_prompt: String,
}

impl Educator {
    pub fn new() -> Self {
        Self {
            system_prompt: "You are an expert Professor and Curriculum Designer. Your task is to generate high-quality, pedagogically sound assessments that challenge students while providing clear paths to mastery.".to_string(),
        }
    }

    pub async fn generate<B: trinity_kernel::Brain + ?Sized>(
        &self,
        brain: &B,
        request: AssessmentRequest,
    ) -> Result<AssessmentResponse> {
        info!("Generating {:?} assessment for: {}...", request.assessment_type, request.topic);

        match request.assessment_type {
            AssessmentType::Quiz => {
                let prompt = format!(
                    "{}\n\nTask: Generate a 5-question multiple choice quiz.\nTopic: {}\nDifficulty: {:?}\nAudience: {}\n\nOutput ONLY valid JSON matching this schema: [{{ \"question\": \"...\", \"options\": [\"...\", \"...\"], \"correct_answer_idx\": 0, \"explanation\": \"...\" }}]",
                    self.system_prompt, request.topic, request.difficulty, request.target_audience
                );

                let response = brain.think_with_grammar(&prompt, GrammarSpec::Json).await?;
                let questions: Vec<Question> = serde_json::from_str(&response)
                    .context("Failed to parse quiz JSON")?;
                
                Ok(AssessmentResponse::Quiz { questions })
            }
            AssessmentType::Lab | AssessmentType::Challenge => {
                let prompt = format!(
                    "{}\n\nTask: Generate a hands-on lab project.\nTopic: {}\nDifficulty: {:?}\nAudience: {}\n\nOutput ONLY valid JSON matching this schema: {{ \"title\": \"...\", \"objective\": \"...\", \"steps\": [\"step 1\", \"step 2\"], \"starter_code\": \"...\", \"solution\": \"...\" }}",
                    self.system_prompt, request.topic, request.difficulty, request.target_audience
                );

                let response = brain.think_with_grammar(&prompt, GrammarSpec::Json).await?;
                let lab: Lab = serde_json::from_str(&response)
                    .context("Failed to parse lab JSON")?;
                
                Ok(AssessmentResponse::Lab(lab))
            }
        }
    }
}

impl Default for Educator {
    fn default() -> Self {
        Self::new()
    }
}
