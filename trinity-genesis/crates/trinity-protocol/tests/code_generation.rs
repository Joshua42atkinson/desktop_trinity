// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Tests for grammar-constrained code generation RPC
//!
//! Verifies that the CodeRequest/CodeResponse types work correctly
//! and that the mock implementation returns valid responses.

use trinity_protocol::types::{CodeRequest, CodeResponse, WriteRequest, WriteResponse, WriteStyle};

#[test]
fn test_code_request_creation() {
    let request = CodeRequest {
        prompt: "Create a function that adds two numbers".to_string(),
        language: "rust".to_string(),
        output_path: None,
        use_grammar: true,
    };

    assert_eq!(request.language, "rust");
    assert!(request.use_grammar);
}

#[test]
fn test_code_request_new_helper() {
    let request = CodeRequest::new("Create a test function", "rust");

    assert_eq!(request.prompt, "Create a test function");
    assert_eq!(request.language, "rust");
    assert!(request.use_grammar); // Default enabled
    assert!(request.output_path.is_none());
}

#[test]
fn test_code_request_with_output_path() {
    let request = CodeRequest {
        prompt: "Create a test module".to_string(),
        language: "rust".to_string(),
        output_path: Some("/tmp/test_output.rs".to_string()),
        use_grammar: true,
    };

    assert_eq!(request.output_path, Some("/tmp/test_output.rs".to_string()));
}

#[test]
fn test_code_response() {
    let response = CodeResponse {
        code: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
        language: "rust".to_string(),
        saved_path: None,
        syntax_valid: true,
    };

    assert!(response.syntax_valid);
    assert!(response.code.contains("fn add"));
}

#[test]
fn test_write_request_creation() {
    let request = WriteRequest {
        topic: "Trinity Architecture Overview".to_string(),
        style: WriteStyle::Technical,
        target_words: 500,
        output_path: None,
    };

    assert_eq!(request.style, WriteStyle::Technical);
    assert_eq!(request.target_words, 500);
}

#[test]
fn test_write_request_new_helper() {
    let request = WriteRequest::new("Test Topic");

    assert_eq!(request.topic, "Test Topic");
    assert_eq!(request.style, WriteStyle::Technical); // Default
    assert_eq!(request.target_words, 500); // Default
    assert!(request.output_path.is_none());
}

#[test]
fn test_write_style_variants() {
    let styles = [
        WriteStyle::Technical,
        WriteStyle::Tutorial,
        WriteStyle::BlogPost,
        WriteStyle::Creative,
        WriteStyle::Formal,
        WriteStyle::Casual,
    ];

    for style in styles {
        let request = WriteRequest {
            topic: "Test topic".to_string(),
            style,
            target_words: 100,
            output_path: None,
        };
        assert!(request.topic.len() > 0);
    }
}

#[test]
fn test_write_response() {
    let response = WriteResponse {
        content: "# Trinity Architecture\n\nTrinity is an AI OS...".to_string(),
        word_count: 25,
        saved_path: None,
    };

    assert!(response.content.starts_with("#"));
    assert_eq!(response.word_count, 25);
}

#[test]
fn test_code_response_with_saved_path() {
    let response = CodeResponse {
        code: "fn main() {}".to_string(),
        language: "rust".to_string(),
        saved_path: Some("/tmp/main.rs".to_string()),
        syntax_valid: true,
    };

    assert_eq!(response.saved_path, Some("/tmp/main.rs".to_string()));
}

#[test]
fn test_write_style_default() {
    let style = WriteStyle::default();
    assert_eq!(style, WriteStyle::Technical);
}
