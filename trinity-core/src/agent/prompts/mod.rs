//! System Prompts - Role-Specific Agent Instructions
//!
//! Provides well-crafted system prompts for each agent role,
//! including tool documentation injection.

use super::components::AgentRole;
use super::tools::ToolRegistry;

// ============================================================================
// System Prompts
// ============================================================================

/// Base system prompt for Trinity agents
pub const TRINITY_BASE: &str = r#"You are an AI agent in the Trinity AI OS - a local-first, sovereign AI system.

Key Principles:
1. You operate locally on the user's hardware - privacy is paramount
2. You have access to tools to interact with the filesystem and execute code
3. Be concise but thorough in your responses
4. When using tools, explain what you're doing and why
5. If unsure, ask for clarification rather than making assumptions

Tool Usage Format:
When you need to use a tool, output a JSON block like this:
```json
{"tool": "tool_name", "params": {"param1": "value1"}}
```

After the tool executes, you'll receive the result and can continue."#;

/// Kernel agent prompt
pub const KERNEL_PROMPT: &str = r#"You are the TRINITY KERNEL - the central orchestrator of this AI operating system.

Your responsibilities:
1. Route tasks to appropriate specialized agents
2. Manage system resources and agent lifecycle
3. Handle meta-level operations (configuration, monitoring)
4. Coordinate multi-agent workflows

When receiving a task, first determine:
- Can you handle this directly?
- Should this go to a specialized agent (Developer, Researcher, Writer)?
- Does this require multiple agents working together?

Be decisive and efficient. You are the brain of Trinity."#;

/// Assistant prompt
pub const ASSISTANT_PROMPT: &str = r#"You are TRINITY ASSISTANT - a helpful, knowledgeable AI assistant.

Your role is to:
1. Answer questions clearly and accurately
2. Help with general tasks and information
3. Engage in natural conversation
4. Provide explanations and guidance

Be friendly, concise, and helpful. If you don't know something, say so."#;

/// Developer prompt
pub const DEVELOPER_PROMPT: &str = r#"You are TRINITY DEVELOPER - a skilled software engineer and code specialist.

Your capabilities:
1. Read, write, and edit code files
2. Execute shell commands and build systems
3. Debug and fix issues
4. Implement new features
5. Refactor and improve existing code

Best practices:
- Always read existing code before modifying
- Use the Symbol Registry to check if similar code already exists
- Make minimal, focused changes
- Test your changes before considering them complete
- Document significant changes

When writing code:
- Follow the project's existing conventions
- Write clean, maintainable code
- Include appropriate error handling
- Add comments for complex logic"#;

/// Researcher prompt
pub const RESEARCHER_PROMPT: &str = r#"You are TRINITY RESEARCHER - an investigative information specialist.

Your role is to:
1. Find and synthesize information
2. Search documentation and code
3. Analyze patterns and connections
4. Provide comprehensive research summaries

Approach:
- Be thorough but efficient
- Cite sources when possible
- Distinguish between facts and inferences
- Organize findings clearly"#;

/// Writer prompt
pub const WRITER_PROMPT: &str = r#"You are TRINITY WRITER - a skilled content creator and communicator.

Your capabilities:
1. Write clear, engaging documentation
2. Create technical guides and tutorials
3. Draft communications and reports
4. Edit and improve existing text

Style guidelines:
- Match the tone to the audience
- Be clear and concise
- Use proper formatting (headers, lists, code blocks)
- Proofread for errors"#;

// ============================================================================
// Prompt Builder
// ============================================================================

/// Builder for constructing complete system prompts
pub struct PromptBuilder {
    base: String,
    role_prompt: String,
    tool_docs: Option<String>,
    context: Option<String>,
}

impl PromptBuilder {
    /// Create a new prompt builder for a role
    pub fn new(role: &AgentRole) -> Self {
        let role_prompt = match role {
            AgentRole::Kernel => KERNEL_PROMPT.to_string(),
            AgentRole::Assistant => ASSISTANT_PROMPT.to_string(),
            AgentRole::Developer => DEVELOPER_PROMPT.to_string(),
            AgentRole::Researcher => RESEARCHER_PROMPT.to_string(),
            AgentRole::Writer => WRITER_PROMPT.to_string(),
            AgentRole::Custom(name) => format!("You are {}, a specialized AI agent.", name),
        };

        Self {
            base: TRINITY_BASE.to_string(),
            role_prompt,
            tool_docs: None,
            context: None,
        }
    }

    /// Add tool documentation
    pub fn with_tools(mut self, registry: &ToolRegistry) -> Self {
        self.tool_docs = Some(registry.generate_docs());
        self
    }

    /// Add additional context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Build the complete system prompt
    pub fn build(self) -> String {
        let mut prompt = format!("{}\n\n{}", self.base, self.role_prompt);

        if let Some(tools) = self.tool_docs {
            prompt.push_str("\n\n");
            prompt.push_str(&tools);
        }

        if let Some(ctx) = self.context {
            prompt.push_str("\n\n--- Context ---\n");
            prompt.push_str(&ctx);
        }

        prompt
    }
}

// ============================================================================
// Message Formatting
// ============================================================================

/// Format a conversation for the model
pub fn format_conversation(
    system: &str,
    messages: &[(String, String)], // (role, content)
) -> String {
    let mut formatted = format!("SYSTEM: {}\n\n", system);

    for (role, content) in messages {
        formatted.push_str(&format!("{}: {}\n\n", role.to_uppercase(), content));
    }

    formatted.push_str("ASSISTANT:");
    formatted
}

/// Format with chat template (for models that use special tokens)
pub fn format_chat_template(
    system: &str,
    messages: &[(String, String)],
    template: ChatTemplate,
) -> String {
    match template {
        ChatTemplate::Chatml => format_chatml(system, messages),
        ChatTemplate::Llama2 => format_llama2(system, messages),
        ChatTemplate::Alpaca => format_alpaca(system, messages),
        ChatTemplate::Plain => format_conversation(system, messages),
    }
}

/// Chat template type
#[derive(Debug, Clone, Copy)]
pub enum ChatTemplate {
    /// ChatML format (used by many models)
    Chatml,
    /// Llama 2 format
    Llama2,
    /// Alpaca instruction format
    Alpaca,
    /// Plain text format
    Plain,
}

fn format_chatml(system: &str, messages: &[(String, String)]) -> String {
    let mut out = format!("<|im_start|>system\n{}<|im_end|>\n", system);

    for (role, content) in messages {
        out.push_str(&format!("<|im_start|>{}\n{}<|im_end|>\n", role, content));
    }

    out.push_str("<|im_start|>assistant\n");
    out
}

fn format_llama2(system: &str, messages: &[(String, String)]) -> String {
    let mut out = format!("[INST] <<SYS>>\n{}\n<</SYS>>\n\n", system);

    let mut first = true;
    for (role, content) in messages {
        if role == "user" {
            if first {
                out.push_str(content);
                out.push_str(" [/INST] ");
                first = false;
            } else {
                out.push_str(&format!("[INST] {} [/INST] ", content));
            }
        } else {
            out.push_str(content);
            out.push(' ');
        }
    }

    out
}

fn format_alpaca(system: &str, messages: &[(String, String)]) -> String {
    let mut out = format!("### System:\n{}\n\n", system);

    for (role, content) in messages {
        let label = if role == "user" {
            "Instruction"
        } else {
            "Response"
        };
        out.push_str(&format!("### {}:\n{}\n\n", label, content));
    }

    out.push_str("### Response:\n");
    out
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new(&AgentRole::Developer)
            .with_context("Working on a Rust project")
            .build();

        assert!(prompt.contains("DEVELOPER"));
        assert!(prompt.contains("Rust project"));
    }

    #[test]
    fn test_chatml_format() {
        let formatted = format_chatml(
            "You are helpful.",
            &[("user".to_string(), "Hello".to_string())],
        );

        assert!(formatted.contains("<|im_start|>system"));
        assert!(formatted.contains("<|im_start|>user"));
        assert!(formatted.contains("<|im_start|>assistant"));
    }
}
