//! Writer Skill - Document and Content Generation
//!
//! ## Philosophy
//! "The Writer transforms ideas into structured prose. It produces
//!  documentation, articles, and content with appropriate style."
//!
//! ## Key Features
//! - Style-aware generation (technical docs, blog posts, etc.)
//! - Markdown output with proper formatting
//! - Integration with Brain RPC for LLM inference

use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, info, warn};

/// Request to generate written content
#[derive(Debug, Clone)]
pub struct WriteRequest {
    /// Topic or subject matter
    pub topic: String,
    /// Style of writing
    pub style: WriteStyle,
    /// Target word count (approximate)
    pub target_words: u32,
    /// Output format
    pub format: OutputFormat,
    /// Path to save the output (if any)
    pub output_path: Option<String>,
}

impl WriteRequest {
    /// Create a new write request
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            style: WriteStyle::Technical,
            target_words: 500,
            format: OutputFormat::Markdown,
            output_path: None,
        }
    }

    /// Set the writing style
    pub fn with_style(mut self, style: WriteStyle) -> Self {
        self.style = style;
        self
    }

    /// Set target word count
    pub fn with_words(mut self, count: u32) -> Self {
        self.target_words = count;
        self
    }

    /// Set output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set output path
    pub fn with_output(mut self, path: impl Into<String>) -> Self {
        self.output_path = Some(path.into());
        self
    }
}

/// Writing style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl WriteStyle {
    /// Get style description for prompt
    fn description(&self) -> &'static str {
        match self {
            WriteStyle::Technical => "technical documentation style - precise, concise, with code examples where appropriate",
            WriteStyle::BlogPost => "engaging blog post style - conversational yet informative, with clear structure",
            WriteStyle::Tutorial => "educational tutorial style - step-by-step, clear explanations, beginner-friendly",
            WriteStyle::Creative => "creative writing style - evocative, expressive, narrative-driven",
            WriteStyle::Formal => "formal business style - professional, objective, structured",
            WriteStyle::Casual => "casual conversational style - friendly, approachable, easy to read",
        }
    }
}

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Markdown format
    Markdown,
    /// Plain text
    PlainText,
    /// HTML
    Html,
}

impl OutputFormat {
    fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Markdown => "md",
            OutputFormat::PlainText => "txt",
            OutputFormat::Html => "html",
        }
    }
}

/// Response from writing generation
#[derive(Debug, Clone)]
pub struct WriteResponse {
    /// The generated content
    pub content: String,
    /// Approximate word count
    pub word_count: u32,
    /// Path where saved (if any)
    pub saved_path: Option<String>,
    /// Format used
    pub format: OutputFormat,
}

/// Writer skill for document generation
pub struct Writer {
    /// System prompt for writing
    system_prompt: String,
}

impl Writer {
    /// Create a new Writer skill
    pub fn new() -> Self {
        Self {
            system_prompt: DEFAULT_WRITER_SYSTEM_PROMPT.to_string(),
        }
    }

    /// Set a custom system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    /// Generate content using the provided Brain
    pub async fn generate<B: trinity_kernel::Brain + ?Sized>(
        &self,
        brain: &B,
        request: WriteRequest,
    ) -> Result<WriteResponse> {
        info!(
            "Generating {:?} content about: {}...",
            request.style,
            request.topic.chars().take(50).collect::<String>()
        );

        // Build the full prompt
        let full_prompt = self.build_prompt(&request);
        debug!("Full prompt length: {} chars", full_prompt.len());

        // Choose grammar based on format
        let grammar = match request.format {
            OutputFormat::Markdown => trinity_kernel::GrammarSpec::Markdown,
            _ => trinity_kernel::GrammarSpec::None,
        };

        // Generate content using the Brain
        let content = brain
            .think_with_grammar(&full_prompt, grammar)
            .await
            .context("Content generation failed")?;

        // Clean up the output
        let clean_content = clean_content(&content);
        let word_count = count_words(&clean_content);

        // Save to file if requested
        let saved_path = if let Some(ref path) = request.output_path {
            match save_content_to_file(path, &clean_content) {
                Ok(()) => {
                    info!("✓ Saved content to: {}", path);
                    Some(path.clone())
                }
                Err(e) => {
                    warn!("Failed to save content: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(WriteResponse {
            content: clean_content,
            word_count,
            saved_path,
            format: request.format,
        })
    }

    /// Build the full prompt for writing
    fn build_prompt(&self, request: &WriteRequest) -> String {
        format!(
            "{}\n\nWriting Style: {}\nFormat: {:?}\nTarget Length: approximately {} words\nTopic: {}\n\nGenerate the content now:\n",
            self.system_prompt,
            request.style.description(),
            request.format,
            request.target_words,
            request.topic
        )
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

/// Default system prompt for writing
const DEFAULT_WRITER_SYSTEM_PROMPT: &str = r#"You are an expert technical writer and content creator. Your task is to generate well-structured, engaging content.

Rules:
1. Follow the specified writing style closely
2. Use clear, professional language
3. Structure content with appropriate headings and sections
4. For Markdown: use proper formatting (headers, lists, code blocks)
5. Stay focused on the topic
6. Meet the approximate word count"#;

/// Clean up generated content
fn clean_content(content: &str) -> String {
    content.trim().to_string()
}

/// Count words in content
fn count_words(content: &str) -> u32 {
    content.split_whitespace().count() as u32
}

/// Save content to a file
fn save_content_to_file(path: &str, content: &str) -> Result<()> {
    let path = Path::new(path);

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
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

    /// Request to generate written content (Bevy Event)
    #[derive(Event, Debug, Clone)]
    pub struct RequestWriting {
        /// Style of writing
        pub style: String,
        /// Topic or subject matter
        pub topic: String,
        /// Approximate target word count
        pub target_words: u32,
    }

    /// Writing completed (Bevy Event)
    #[derive(Event, Debug, Clone)]
    pub struct WritingComplete {
        /// The generated content
        pub content: String,
        /// Word count
        pub word_count: u32,
        /// Path where saved
        pub saved_path: Option<String>,
    }

    /// Plugin for Writer skill
    pub struct WriterPlugin;

    impl Plugin for WriterPlugin {
        fn build(&self, app: &mut App) {
            app.add_event::<RequestWriting>()
                .add_event::<WritingComplete>();

            info!("Writer Skill Plugin initialized");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words("one two three four five"), 5);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_write_request_builder() {
        let req = WriteRequest::new("Rust async programming")
            .with_style(WriteStyle::Tutorial)
            .with_words(1000)
            .with_output("/tmp/tutorial.md");

        assert_eq!(req.topic, "Rust async programming");
        assert_eq!(req.style, WriteStyle::Tutorial);
        assert_eq!(req.target_words, 1000);
        assert_eq!(req.output_path, Some("/tmp/tutorial.md".to_string()));
    }

    #[test]
    fn test_style_descriptions() {
        // Just ensure they all return something
        assert!(!WriteStyle::Technical.description().is_empty());
        assert!(!WriteStyle::BlogPost.description().is_empty());
        assert!(!WriteStyle::Creative.description().is_empty());
    }
}
