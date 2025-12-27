//! Output Parser - Extract Structured Data from LLM Output
//!
//! Parses LLM responses to extract tool calls, thoughts, and final answers.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// ============================================================================
// Parsed Output
// ============================================================================

/// Complete parsed output from an LLM response
#[derive(Debug, Clone, Default)]
pub struct ParsedOutput {
    /// Extracted thought process/reasoning
    pub thoughts: Vec<String>,
    /// Tool/function calls extracted
    pub tool_calls: Vec<ParsedToolCall>,
    /// The final answer/response (if any)
    pub final_answer: Option<String>,
    /// Code blocks extracted (language, code)
    pub code_blocks: Vec<(String, String)>,
}

/// A parsed tool/function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedToolCall {
    /// Name of the tool
    pub name: String,
    /// Parameters as JSON
    pub parameters: JsonValue,
    /// Raw text of the call (for debugging)
    pub raw: String,
}

// ============================================================================
// Output Parser
// ============================================================================

/// Parser for extracting structured data from LLM output
pub struct OutputParser {
    /// Custom thought markers
    thought_markers: Vec<(&'static str, &'static str)>,
}

impl Default for OutputParser {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputParser {
    pub fn new() -> Self {
        Self {
            thought_markers: vec![
                ("<think>", "</think>"),
                ("<thinking>", "</thinking>"),
                ("Thought:", "\n"),
                ("Reasoning:", "\n"),
            ],
        }
    }

    /// Parse an LLM output string
    pub fn parse(&self, output: &str) -> ParsedOutput {
        let mut result = ParsedOutput::default();

        // Extract thoughts
        result.thoughts = self.extract_thoughts(output);

        // Extract code blocks
        result.code_blocks = self.extract_code_blocks(output);

        // Extract tool calls from code blocks
        result.tool_calls = self.extract_tool_calls(&result.code_blocks);

        // Extract final answer (text after last tool call or thought)
        result.final_answer = self.extract_final_answer(output, &result);

        result
    }

    /// Extract thought/reasoning sections
    fn extract_thoughts(&self, output: &str) -> Vec<String> {
        let mut thoughts = Vec::new();

        for (start, end) in &self.thought_markers {
            let mut remaining = output;
            while let Some(start_idx) = remaining.find(start) {
                let after_start = &remaining[start_idx + start.len()..];
                if let Some(end_idx) = after_start.find(end) {
                    let thought = after_start[..end_idx].trim().to_string();
                    if !thought.is_empty() {
                        thoughts.push(thought);
                    }
                    remaining = &after_start[end_idx + end.len()..];
                } else {
                    break;
                }
            }
        }

        thoughts
    }

    /// Extract code blocks (```language\ncode```)
    fn extract_code_blocks(&self, output: &str) -> Vec<(String, String)> {
        let mut blocks = Vec::new();
        let pattern = regex::Regex::new(r"```(\w*)\s*([\s\S]*?)```").ok();

        if let Some(re) = pattern {
            for cap in re.captures_iter(output) {
                let lang = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let code = cap
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !code.is_empty() {
                    blocks.push((lang, code));
                }
            }
        }

        blocks
    }

    /// Extract tool calls from JSON code blocks
    fn extract_tool_calls(&self, code_blocks: &[(String, String)]) -> Vec<ParsedToolCall> {
        let mut calls = Vec::new();

        for (lang, code) in code_blocks {
            if lang != "json" && lang != "tool" {
                continue;
            }

            if let Ok(value) = serde_json::from_str::<JsonValue>(code) {
                // Format 1: {"tool": "name", "params": {...}}
                if let (Some(tool), Some(params)) = (
                    value.get("tool").and_then(|v| v.as_str()),
                    value.get("params"),
                ) {
                    calls.push(ParsedToolCall {
                        name: tool.to_string(),
                        parameters: params.clone(),
                        raw: code.clone(),
                    });
                    continue;
                }

                // Format 2: {"name": "...", "arguments": {...}}
                if let (Some(name), Some(args)) = (
                    value.get("name").and_then(|v| v.as_str()),
                    value.get("arguments"),
                ) {
                    calls.push(ParsedToolCall {
                        name: name.to_string(),
                        parameters: args.clone(),
                        raw: code.clone(),
                    });
                    continue;
                }

                // Format 3: {"function": "...", "input": {...}}
                if let (Some(func), Some(input)) = (
                    value.get("function").and_then(|v| v.as_str()),
                    value.get("input"),
                ) {
                    calls.push(ParsedToolCall {
                        name: func.to_string(),
                        parameters: input.clone(),
                        raw: code.clone(),
                    });
                }
            }
        }

        calls
    }

    /// Extract the final answer from output
    fn extract_final_answer(&self, output: &str, parsed: &ParsedOutput) -> Option<String> {
        // If there are no tool calls and no thoughts, the whole output is the answer
        if parsed.tool_calls.is_empty() && parsed.thoughts.is_empty() {
            let trimmed = output.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }

        // Look for explicit final answer markers
        let markers = ["Final Answer:", "Answer:", "Response:", "Output:"];
        for marker in markers {
            if let Some(idx) = output.find(marker) {
                let after = output[idx + marker.len()..].trim();
                // Take until next code block or end
                let end = after.find("```").unwrap_or(after.len());
                let answer = after[..end].trim();
                if !answer.is_empty() {
                    return Some(answer.to_string());
                }
            }
        }

        // If we have tool calls, there might not be a final answer yet
        if !parsed.tool_calls.is_empty() {
            return None;
        }

        // Otherwise, text after thoughts/code blocks
        // This is a simplified heuristic
        let mut last_end = 0;
        for thought in &parsed.thoughts {
            if let Some(idx) = output.find(thought) {
                last_end = last_end.max(idx + thought.len());
            }
        }

        if last_end > 0 && last_end < output.len() {
            let remaining = output[last_end..].trim();
            if !remaining.is_empty() && !remaining.starts_with("```") {
                return Some(remaining.to_string());
            }
        }

        None
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Quick parse of LLM output
pub fn parse_output(output: &str) -> ParsedOutput {
    OutputParser::new().parse(output)
}

/// Check if output contains any tool calls
pub fn has_tool_calls(output: &str) -> bool {
    !parse_output(output).tool_calls.is_empty()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_thoughts() {
        let output = r#"
<think>
I should read the file first.
</think>

Let me help you with that.
        "#;

        let parsed = parse_output(output);
        assert_eq!(parsed.thoughts.len(), 1);
        assert!(parsed.thoughts[0].contains("read the file"));
    }

    #[test]
    fn test_extract_tool_calls() {
        let output = r#"
I'll read that file for you.

```json
{"tool": "read_file", "params": {"path": "/home/user/test.txt"}}
```

Let me know if you need anything else.
        "#;

        let parsed = parse_output(output);
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].name, "read_file");
    }

    #[test]
    fn test_extract_code_blocks() {
        let output = r#"
Here's the code:

```rust
fn main() {
    println!("Hello, world!");
}
```

And in Python:

```python
print("Hello, world!")
```
        "#;

        let parsed = parse_output(output);
        assert_eq!(parsed.code_blocks.len(), 2);
        assert_eq!(parsed.code_blocks[0].0, "rust");
        assert_eq!(parsed.code_blocks[1].0, "python");
    }

    #[test]
    fn test_final_answer() {
        let output = r#"
<think>Let me think about this.</think>

Final Answer: The answer is 42.
        "#;

        let parsed = parse_output(output);
        assert!(parsed.final_answer.is_some());
        assert!(parsed.final_answer.unwrap().contains("42"));
    }
}
