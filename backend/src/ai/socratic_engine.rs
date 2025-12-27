#![allow(unused)]
use super::conversation_memory::{ConversationMemory, Speaker, Turn};
use super::llm::{GgufConfig, GgufGenerateConfig, GgufModel};
use super::prompts::PromptStrategy;
use anyhow::Result;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Response from the Socratic engine
#[derive(Debug, Clone)]
pub struct SocraticResponse {
    pub text: String,
    pub strategy_used: PromptStrategy,
}

/// Context for the current session
pub struct SessionContext {
    pub session_id: Uuid,
    pub user_id: i64,
    pub archetype: Option<String>,
    pub focus_area: Option<String>,
}

/// Main Socratic dialogue engine
pub struct SocraticEngine {
    model: Option<Arc<Mutex<GgufModel>>>,
    memory: ConversationMemory,
}

impl SocraticEngine {
    /// Create a new Socratic engine
    pub fn new(memory: ConversationMemory) -> Self {
        Self {
            model: None,
            memory,
        }
    }

    /// Initialize the LLM model (called after construction)
    pub fn load_model(&mut self, config: GgufConfig) -> Result<()> {
        log::info!("Loading GGUF model for Socratic engine...");
        let model = GgufModel::load(config)?;
        self.model = Some(Arc::new(Mutex::new(model)));
        Ok(())
    }

    /// Generate a Socratic response to user input
    pub async fn respond(
        &mut self,
        user_input: &str,
        context: &SessionContext,
    ) -> Result<SocraticResponse> {
        log::debug!(
            "Generating Socratic response for session {}",
            context.session_id
        );

        // 1. Save user's turn to memory
        let user_turn = Turn {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            speaker: Speaker::User,
            content: user_input.to_string(),
            metadata: Default::default(),
        };
        self.memory.add_turn(context.session_id, user_turn).await?;

        // 2. Retrieve recent conversation history
        let history = self.memory.get_recent(context.session_id, 10).await?;
        log::debug!(
            "Retrieved {} turns from conversation history",
            history.len()
        );

        // 3. Select prompting strategy based on user input
        let strategy = PromptStrategy::select_strategy(user_input, &history);
        log::debug!("Selected strategy: {:?}", strategy);

        // 4. Build prompt with template
        let prompt = strategy.build_prompt(user_input, &history, context);
        log::debug!("Built prompt: {} chars", prompt.len());

        // 5. Generate response using LLM
        let response_text = if let Some(model_arc) = self.model.clone() {
            // Actual inference (in spawn_blocking to avoid blocking async runtime)
            let prompt_owned = prompt.clone();

            let result = tokio::task::spawn_blocking(move || {
                let mut model = model_arc.lock().unwrap();
                let gen_config = GgufGenerateConfig::default();
                model.generate(&prompt_owned, &gen_config)
            })
            .await??;

            result
        } else {
            // Fallback if model not loaded
            log::warn!("Model not loaded, using fallback response");
            "I'm listening. Can you tell me more about that?".to_string()
        };

        // 6. Post-process response
        let processed_response = Self::post_process_response(&response_text);

        // 7. Save AI's turn to memory
        let ai_turn = Turn {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            speaker: Speaker::AI,
            content: processed_response.clone(),
            metadata: Default::default(),
        };
        self.memory.add_turn(context.session_id, ai_turn).await?;

        Ok(SocraticResponse {
            text: processed_response,
            strategy_used: strategy,
        })
    }

    /// Post-process AI response to ensure it's Socratic
    fn post_process_response(response: &str) -> String {
        let mut processed = response.trim().to_string();

        // Ensure response doesn't give direct answers (basic heuristic)
        // TODO: More sophisticated filtering

        // Ensure response ends with a question mark
        if !processed.ends_with('?') && !processed.is_empty() {
            processed.push('?');
        }

        processed
    }
}
