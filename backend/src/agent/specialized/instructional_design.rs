use crate::agent::autonomous::{AutonomousTask, TaskPriority, TaskType};
use crate::agent::self_coder::SelfCodingAgent;
use anyhow::Result;

/// Instructional Design Agent
/// Specialized in media development, curriculum design, and educational content.
pub struct InstructionalDesignAgent {
    name: String,
}

impl InstructionalDesignAgent {
    pub fn new() -> Self {
        Self {
            name: "Instructional Designer".to_string(),
        }
    }

    /// Analyze a topic and generate a curriculum structure
    pub fn create_curriculum(&self, topic: &str) -> AutonomousTask {
        AutonomousTask::new(
            format!("Design Curriculum: {}", topic),
            TaskType::GenerateCode {
                prompt: format!(
                    "You are an expert Instructional Designer. 
                    Create a comprehensive curriculum for the topic: '{}'.
                    Format the output as a JSON structure with modules, lessons, and learning objectives.",
                    topic
                ),
                language: "json".to_string(),
                output_path: Some(format!("curriculum_{}.json", topic.replace(" ", "_").to_lowercase())),
            }
        ).with_priority(TaskPriority::High)
    }

    /// Generate a lesson plan for a specific module
    pub fn create_lesson_plan(&self, module_title: &str) -> AutonomousTask {
        AutonomousTask::new(
            format!("Lesson Plan: {}", module_title),
            TaskType::GenerateCode {
                prompt: format!(
                    "Create a detailed lesson plan for '{}'. 
                    Include hook, direct instruction, guided practice, and independent practice.",
                    module_title
                ),
                language: "markdown".to_string(),
                output_path: Some(format!(
                    "lesson_plan_{}.md",
                    module_title.replace(" ", "_").to_lowercase()
                )),
            },
        )
        .with_priority(TaskPriority::Normal)
    }
}
