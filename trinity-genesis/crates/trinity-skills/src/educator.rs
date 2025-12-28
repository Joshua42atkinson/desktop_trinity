// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! # Educator Skill - Assessment and Curriculum Generation
//!
//! ## Philosophy
//! "The Educator transforms raw knowledge into mastery. It designs
//!  paths for students to climb, from theory to code."

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info};
use trinity_kernel::GrammarSpec;

use trinity_protocol::types::{AssessmentType, QuizQuestion, LabProject, AssessmentRequest, AssessmentResponse};

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

        let system_prompt = &self.system_prompt;


        match request.assessment_type {
            AssessmentType::Quiz => {
                let prompt = format!(
                    "{}\n\nTask: Generate a 5-question multiple choice quiz.\nTopic: {}\nDifficulty: {:?}\nAudience: {}\n\n\
                    Output ONLY valid JSON matching this schema:\n\
                    [\n  {{ \n    \"question\": \"Question text here...\", \n    \"options\": [\"Option A\", \"B\", \"C\", \"D\"],\n    \"correct_answer_idx\": 0,\n    \"explanation\": \"Detailed explanation...\"\n  }}\n]",
                    system_prompt, request.topic, request.difficulty, request.target_audience
                );

                let response = brain.think_with_grammar(&prompt, GrammarSpec::Json).await?;
                let questions: Vec<QuizQuestion> = serde_json::from_str(&response)
                    .context("Failed to parse quiz JSON")?;
                
                Ok(AssessmentResponse::Quiz { questions })
            }
            AssessmentType::Lab | AssessmentType::Challenge => {
                let prompt = format!(
                    "{}\n\nTask: Generate a hands-on lab project.\nTopic: {}\nDifficulty: {:?}\nAudience: {}\n\n\
                    Output ONLY valid JSON matching this schema:\n\
                    {{\n  \"title\": \"Lab Title\",\n  \"objective\": \"Goal...\",\n  \"steps\": [\"Step 1...\"],\n  \"starter_code\": \"Code...\",\n  \"solution\": \"Solution...\"\n}}",
                    system_prompt, request.topic, request.difficulty, request.target_audience
                );

                let response = brain.think_with_grammar(&prompt, GrammarSpec::Json).await?;
                let lab: LabProject = serde_json::from_str(&response)
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
