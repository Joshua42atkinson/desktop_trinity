//! Coder Skill - Code Generation with Grammar-Constrained Output
//!
//! ## Philosophy
//! "The Coder is the most critical Skill for autopoiesis. It must generate
//!  syntactically valid code that compiles on the first try."
//!
//! ## Key Features
//! - Grammar-constrained output (GBNF) for valid Rust syntax
//! - Integration with Brain RPC for LLM inference
//! - File output with optional path specification
//! - Syntax validation before output

use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, info, warn};

/// Request to generate code
#[derive(Debug, Clone)]
pub struct CodeRequest {
    /// Description of the code to generate
    pub prompt: String,
    /// Language (e.g., "rust", "python", "typescript")
    pub language: String,
    /// Path to save the output code (if any)
    pub output_path: Option<String>,
    /// Whether to use grammar-constrained sampling
    pub use_grammar: bool,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
}

impl CodeRequest {
    /// Create a new code generation request
    pub fn new(prompt: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            language: language.into(),
            output_path: None,
            use_grammar: true,
            max_tokens: None,
        }
    }

    /// Set the output path
    pub fn with_output(mut self, path: impl Into<String>) -> Self {
        self.output_path = Some(path.into());
        self
    }

    /// Disable grammar constraints (for free-form output)
    pub fn without_grammar(mut self) -> Self {
        self.use_grammar = false;
        self
    }

    /// Set maximum tokens
    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }
}

/// Response from code generation
#[derive(Debug, Clone)]
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

/// Coder skill for code generation
pub struct Coder {
    /// System prompt prefix for code generation
    system_prompt: String,
}

impl Coder {
    /// Create a new Coder skill
    pub fn new() -> Self {
        Self {
            system_prompt: DEFAULT_CODER_SYSTEM_PROMPT.to_string(),
        }
    }

    /// Set a custom system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Generate code using the provided Brain
    ///
    /// The brain should implement the `Brain` trait from `trinity-kernel`.
    pub async fn generate<B: trinity_kernel::Brain + ?Sized>(
        &self,
        brain: &B,
        request: CodeRequest,
    ) -> Result<CodeResponse> {
        info!(
            "Generating {} code: {}...",
            request.language,
            request.prompt.chars().take(50).collect::<String>()
        );

        // Build the full prompt
        let full_prompt = self.build_prompt(&request);
        debug!("Full prompt length: {} chars", full_prompt.len());

        // Choose grammar based on language and request
        let grammar = if request.use_grammar {
            match request.language.to_lowercase().as_str() {
                "rust" => trinity_kernel::GrammarSpec::Rust,
                "json" => trinity_kernel::GrammarSpec::Json,
                "markdown" | "md" => trinity_kernel::GrammarSpec::Markdown,
                _ => {
                    debug!(
                        "No grammar available for language: {}, using unconstrained",
                        request.language
                    );
                    trinity_kernel::GrammarSpec::None
                }
            }
        } else {
            trinity_kernel::GrammarSpec::None
        };

        // Generate code using the Brain
        let code = brain
            .think_with_grammar(&full_prompt, grammar)
            .await
            .context("Code generation failed")?;

        // Clean up the output (remove markdown fences if present)
        let clean_code = clean_code_output(&code, &request.language);

        // Basic syntax check
        let syntax_valid = check_syntax(&clean_code, &request.language);

        if !syntax_valid {
            warn!("Generated code may have syntax issues");
        }

        // Save to file if requested
        let saved_path = if let Some(ref path) = request.output_path {
            match save_code_to_file(path, &clean_code) {
                Ok(()) => {
                    info!("✓ Saved code to: {}", path);
                    Some(path.clone())
                }
                Err(e) => {
                    warn!("Failed to save code: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(CodeResponse {
            code: clean_code,
            language: request.language,
            saved_path,
            syntax_valid,
        })
    }

    /// Build the full prompt with system context
    fn build_prompt(&self, request: &CodeRequest) -> String {
        format!(
            "{}\n\nLanguage: {}\nTask: {}\n\nGenerate ONLY the code, no explanations or markdown fences:\n",
            self.system_prompt, request.language, request.prompt
        )
    }
}

impl Default for Coder {
    fn default() -> Self {
        Self::new()
    }
}

/// Default system prompt for code generation
const DEFAULT_CODER_SYSTEM_PROMPT: &str = r#"You are an expert programmer. Your task is to generate clean, efficient, production-ready code.

Rules:
1. Output ONLY valid code - no explanations, no markdown, no comments about what you're doing
2. Use best practices for the language
3. Handle errors appropriately
4. Write self-documenting code with clear naming
5. If generating Rust, ensure it compiles without warnings"#;

/// Clean up code output by removing markdown fences and extra whitespace
fn clean_code_output(code: &str, _language: &str) -> String {
    let mut output = code.to_string();

    // Remove leading/trailing whitespace
    output = output.trim().to_string();

    // Remove markdown code fences
    if output.starts_with("```") {
        // Find the end of the first line (after ```rust or similar)
        if let Some(first_newline) = output.find('\n') {
            output = output[first_newline + 1..].to_string();
        }
    }
    if output.ends_with("```") {
        output = output[..output.len() - 3].trim_end().to_string();
    }

    // Remove common LLM preambles
    let preambles = [
        "Here is the code:",
        "Here's the code:",
        "Here is your code:",
        "Here's your code:",
        "```rust",
        "```python",
        "```typescript",
        "```javascript",
    ];
    for preamble in preambles {
        if let Some(idx) = output.find(preamble) {
            if idx < 50 {
                // Only if at the start
                output = output[idx + preamble.len()..].trim_start().to_string();
            }
        }
    }

    output
}

/// Basic syntax check for generated code
fn check_syntax(code: &str, language: &str) -> bool {
    match language.to_lowercase().as_str() {
        "rust" => check_rust_syntax(code),
        "json" => check_json_syntax(code),
        _ => true, // Can't check, assume ok
    }
}

/// Check Rust syntax (very basic - balanced braces)
fn check_rust_syntax(code: &str) -> bool {
    let mut brace_count = 0i32;
    let mut paren_count = 0i32;
    let mut bracket_count = 0i32;

    for ch in code.chars() {
        match ch {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }

        // Negative count means unbalanced
        if brace_count < 0 || paren_count < 0 || bracket_count < 0 {
            return false;
        }
    }

    brace_count == 0 && paren_count == 0 && bracket_count == 0
}

/// Check JSON syntax
fn check_json_syntax(code: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(code).is_ok()
}

/// Save code to a file
fn save_code_to_file(path: &str, code: &str) -> Result<()> {
    let path = Path::new(path);

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, code)?;
    Ok(())
}

// ============================================================================
// Bevy Plugin (for trinity-body integration)
// ============================================================================

#[cfg(feature = "bevy")]
pub use bevy_plugin::*;

#[cfg(feature = "bevy")]
mod bevy_plugin {
    use bevy::prelude::*;

    /// Request to generate code (Bevy Event)
    #[derive(Event, Debug, Clone)]
    pub struct RequestCodeGeneration {
        /// Description of the code to generate
        pub prompt: String,
        /// Language (e.g., "rust", "python")
        pub language: String,
        /// Path to save the output code (if any)
        pub output_path: Option<String>,
    }

    /// Code generation completed (Bevy Event)
    #[derive(Event, Debug, Clone)]
    pub struct CodeGenerationComplete {
        /// The generated code
        pub code: String,
        /// Language used
        pub language: String,
        /// Path where saved
        pub saved_path: Option<String>,
        /// Whether syntax is valid
        pub syntax_valid: bool,
    }

    /// Plugin for Coder skill
    pub struct CoderPlugin;

    impl Plugin for CoderPlugin {
        fn build(&self, app: &mut App) {
            app.add_event::<RequestCodeGeneration>()
                .add_event::<CodeGenerationComplete>();

            info!("Coder Skill Plugin initialized");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_code_output() {
        let input = "```rust\nfn main() {}\n```";
        let output = clean_code_output(input, "rust");
        assert_eq!(output, "fn main() {}");
    }

    #[test]
    fn test_rust_syntax_check() {
        assert!(check_rust_syntax("fn main() { let x = 1; }"));
        assert!(!check_rust_syntax("fn main() { let x = 1; ")); // Missing }
        assert!(!check_rust_syntax("fn main() } let x = 1; {")); // Wrong order
    }

    #[test]
    fn test_json_syntax_check() {
        assert!(check_json_syntax(r#"{"key": "value"}"#));
        assert!(!check_json_syntax(r#"{"key": "value""#)); // Missing }
    }

    #[test]
    fn test_code_request_builder() {
        let req = CodeRequest::new("Create a fibonacci function", "rust")
            .with_output("/tmp/fib.rs")
            .with_max_tokens(500);

        assert_eq!(req.language, "rust");
        assert_eq!(req.output_path, Some("/tmp/fib.rs".to_string()));
        assert_eq!(req.max_tokens, Some(500));
        assert!(req.use_grammar);
    }
}