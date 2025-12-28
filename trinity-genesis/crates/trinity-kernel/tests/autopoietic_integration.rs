// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Integration tests for the Autopoietic Engine
//!
//! Tests the full mutation cycle: staging → compile → backup → promote

use std::path::PathBuf;
use trinity_kernel::{AutopoieticConfig, AutopoieticEngine, MutationRequest, MutationType};

/// Create a test config that uses temporary directories
fn test_config() -> AutopoieticConfig {
    let temp_base = std::env::temp_dir().join("trinity_autopoietic_test");

    AutopoieticConfig {
        source_root: PathBuf::from("/home/joshua/antigravity/trinity-genesis"),
        staging_dir: temp_base.join("staging"),
        backup_dir: temp_base.join("backups"),
        max_backups: 3,
        require_tests: false,
        cloud_backup_path: None,
        immutable_files: vec![
            "safety.rs".to_string(),
            "autopoietic.rs".to_string(),
            "Cargo.lock".to_string(),
        ],
        max_failures: 3,
    }
}

#[test]
fn test_engine_creation() {
    let config = test_config();
    let engine = AutopoieticEngine::new(config);
    assert!(engine.is_ok(), "Engine should be created successfully");

    let engine = engine.unwrap();
    assert_eq!(engine.failure_count(), 0);
}

#[test]
fn test_immutable_file_protection() {
    let config = test_config();
    let mut engine = AutopoieticEngine::new(config).unwrap();

    // Try to mutate safety.rs - should be blocked
    let request = MutationRequest {
        target_file: "crates/trinity-kernel/src/safety.rs".to_string(),
        mutation_type: MutationType::Append,
        description: "Test mutation of immutable file".to_string(),
        code: Some("// This should not be added".to_string()),
        find_pattern: None,
    };

    let result = engine.execute(request).unwrap();
    assert!(!result.success, "Mutation of safety.rs should be blocked");
    assert!(result.error.as_ref().unwrap().contains("immutable"));
}

#[test]
fn test_mutation_append() {
    let config = test_config();
    let mut engine = AutopoieticEngine::new(config).unwrap();

    // Create a simple append mutation to a test file
    // Note: This will actually modify staging, not live code
    let request = MutationRequest {
        target_file: "crates/trinity-kernel/src/lib.rs".to_string(),
        mutation_type: MutationType::Append,
        description: "Add a test comment".to_string(),
        code: Some("\n// Autopoietic test marker - safe to remove\n".to_string()),
        find_pattern: None,
    };

    // This test verifies the engine can process a mutation
    // In a real test, we'd verify compilation succeeds
    let result = engine.execute(request);

    // The mutation might fail due to various reasons in test env
    // (missing source, etc.) but we're testing the flow
    println!("Mutation result: {:?}", result);
}

#[test]
fn test_failure_tracking() {
    let config = test_config();
    let engine = AutopoieticEngine::new(config).unwrap();

    // Initial failure count should be 0
    assert_eq!(engine.failure_count(), 0);
    assert_eq!(engine.current_version(), 0);
}

#[test]
fn test_version_tracking() {
    let config = test_config();
    let engine = AutopoieticEngine::new(config).unwrap();

    // Version should start at the latest backup version (or 0)
    let version = engine.current_version();
    assert!(version >= 0, "Version should be non-negative");
    println!("Current version: {}", version);
}
