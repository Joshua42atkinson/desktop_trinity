#![allow(unused)]
use super::conversation_memory::{Speaker, Turn};
use super::socratic_engine::SessionContext;

/// Strategies for Socratic prompting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptStrategy {
    Scaffolding, // User is stuck → Ask leading question
    Deepening,   // Superficial response → "What do you mean by...?"
    Mirroring,   // Reflect their words back
    Challenging, // Logical inconsistency detected
    Affirming,   // Breakthrough moment detected
}

impl PromptStrategy {
    /// Select the appropriate strategy based on user input and history
    pub fn select_strategy(user_input: &str, history: &[Turn]) -> Self {
        let word_count = user_input.split_whitespace().count();
        let input_lower = user_input.to_lowercase();

        // Simple heuristics for Phase 1
        // TODO: Implement more sophisticated analysis in Phase 2

        // Check for breakthrough indicators FIRST (before word count)
        let breakthrough_words = ["finally", "realize", "understand", "aha", "now I see"];
        if breakthrough_words.iter().any(|w| input_lower.contains(w)) {
            return Self::Affirming;
        }

        // Very short responses might indicate confusion or being stuck
        if word_count < 5 {
            return Self::Scaffolding;
        }

        // Short responses might be superficial
        if word_count < 10 {
            return Self::Deepening;
        }

        // Default to mirroring for moderate responses
        Self::Mirroring
    }

    /// Build the prompt for this strategy
    pub fn build_prompt(
        &self,
        user_input: &str,
        history: &[Turn],
        context: &SessionContext,
    ) -> String {
        let base_system_prompt = self.get_system_prompt();
        let conversation_context = self.format_history(history);
        let strategy_instructions = self.get_strategy_instructions();

        format!(
            "{}\n\nContext:\n- Session Focus: {}\n- Archetype: {}\n\n{}\n\nCurrent user input: \"{}\"\n\n{}",
            base_system_prompt,
            context.focus_area.as_deref().unwrap_or("General reflection"),
            context.archetype.as_deref().unwrap_or("Unknown"),
            conversation_context,
            user_input,
            strategy_instructions
        )
    }

    fn get_system_prompt(&self) -> &str {
        r#"You are a Socratic guide for a learner engaged in deep reflection. Your role is to:
1. NEVER give answers or solutions
2. Ask questions that help the learner discover insights themselves
3. Mirror their language back to reveal assumptions
4. Identify contradictions gently
5. Encourage deeper thinking without judgment

Guidelines:
- Keep responses to 1-3 sentences maximum
- Always end with a question
- Use the learner's own words when possible
- Be warm, curious, never condescending"#
    }

    fn format_history(&self, history: &[Turn]) -> String {
        if history.is_empty() {
            return "No previous conversation.".to_string();
        }

        let recent_turns: Vec<String> = history
            .iter()
            .rev()
            .take(3)
            .rev()
            .map(|turn| {
                let speaker = match turn.speaker {
                    Speaker::User => "Learner",
                    Speaker::AI => "You (AI Guide)",
                };
                format!("{}: {}", speaker, turn.content)
            })
            .collect();

        format!("Recent conversation:\n{}", recent_turns.join("\n"))
    }

    fn get_strategy_instructions(&self) -> &str {
        match self {
            Self::Scaffolding => {
                "The learner seems stuck or uncertain. Offer a gentle leading question that helps them see a path forward. Example: 'It sounds like you're noticing X. What might happen if...?'"
            }
            Self::Deepening => {
                "The learner's response is brief or superficial. Ask them to elaborate on a specific part. Example: 'You mentioned Y. What specifically do you mean by that?'"
            }
            Self::Mirroring => {
                "Reflect the learner's own words back to them to help them see connections. Example: 'You said \"A leads to B.\" How does that connect to your earlier point about C?'"
            }
            Self::Challenging => {
                "The learner has stated something that contradicts an earlier statement. Gently point this out. Example: 'Earlier you valued X, but now you're choosing Y. What changed for you?'"
            }
            Self::Affirming => {
                "The learner has reached an insight. Acknowledge it and help them deepen it. Example: 'I notice you used the word \"finally.\" What makes this moment significant for you?'"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_selection_short_input() {
        let strategy = PromptStrategy::select_strategy("I don't know", &[]);
        assert_eq!(strategy, PromptStrategy::Scaffolding);
    }

    #[test]
    fn test_strategy_selection_breakthrough() {
        let strategy = PromptStrategy::select_strategy("I finally understand what you mean!", &[]);
        assert_eq!(strategy, PromptStrategy::Affirming);
    }

    #[test]
    fn test_strategy_selection_moderate() {
        let strategy = PromptStrategy::select_strategy(
            "I think it relates to my earlier experiences with learning",
            &[],
        );
        assert_eq!(strategy, PromptStrategy::Mirroring);
    }
}
