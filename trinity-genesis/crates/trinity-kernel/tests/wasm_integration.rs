// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

//! Integration test for WASM sandbox execution
//!
//! Tests loading and executing actual WASM plugins.

use std::path::PathBuf;
use trinity_kernel::wasm_sandbox::{WasmSandbox, SandboxConfig, CapabilitySet, Capability};

/// Get the plugins directory path
fn plugins_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("plugins")
}

#[tokio::test]
async fn test_calculator_wasm_plugin() {
    // Initialize sandbox with workspace root
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    
    let sandbox = WasmSandbox::with_workspace(workspace_root).expect("Failed to create sandbox");
    
    // Load the calculator plugin
    let calculator_path = plugins_dir().join("calculator.wasm");
    
    // Skip test if plugin doesn't exist
    if !calculator_path.exists() {
        eprintln!("Skipping test: calculator.wasm not found at {:?}", calculator_path);
        eprintln!("Build it with: cd quadradical-tools/calculator && cargo build --target wasm32-unknown-unknown --release");
        return;
    }
    
    sandbox.load_module_from_file(calculator_path.clone())
        .await
        .expect("Failed to load calculator.wasm");
    
    // Verify module is listed
    let modules = sandbox.list_modules().await;
    assert!(modules.contains(&calculator_path), "Calculator should be in module list");
}

#[tokio::test]
async fn test_calculator_execution() {
    // Initialize sandbox
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    
    let sandbox = WasmSandbox::with_workspace(workspace_root).expect("Failed to create sandbox");
    
    let calculator_path = plugins_dir().join("calculator.wasm");
    if !calculator_path.exists() {
        eprintln!("Skipping test: calculator.wasm not found");
        return;
    }
    
    sandbox.load_module_from_file(calculator_path.clone())
        .await
        .expect("Failed to load calculator.wasm");
    
    // Execute a calculation: 2 + 3 * 4 = 14
    let input = r#"{"expression": "2 + 3 * 4"}"#;
    let result = sandbox.execute(&calculator_path, "calculate", input)
        .await
        .expect("Execution should succeed");
    
    // Parse result JSON
    let output: serde_json::Value = serde_json::from_str(&result)
        .expect("Result should be valid JSON");
    
    assert_eq!(output["result"].as_f64(), Some(14.0), "2 + 3 * 4 should equal 14");
    assert_eq!(output["expression"].as_str(), Some("2 + 3 * 4"));
    
    // Test another expression with parentheses
    let input2 = r#"{"expression": "(2 + 3) * 4"}"#;
    let result2 = sandbox.execute(&calculator_path, "calculate", input2)
        .await
        .expect("Execution should succeed");
    
    let output2: serde_json::Value = serde_json::from_str(&result2)
        .expect("Result should be valid JSON");
    
    assert_eq!(output2["result"].as_f64(), Some(20.0), "(2 + 3) * 4 should equal 20");
}

#[tokio::test]
async fn test_calculator_error_handling() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    
    let sandbox = WasmSandbox::with_workspace(workspace_root).expect("Failed to create sandbox");
    
    let calculator_path = plugins_dir().join("calculator.wasm");
    if !calculator_path.exists() {
        eprintln!("Skipping test: calculator.wasm not found");
        return;
    }
    
    sandbox.load_module_from_file(calculator_path.clone())
        .await
        .expect("Failed to load calculator.wasm");
    
    // Test invalid expression
    let input = r#"{"expression": "invalid syntax !@#"}"#;
    let result = sandbox.execute(&calculator_path, "calculate", input)
        .await
        .expect("Execution should succeed even for invalid expressions");
    
    let output: serde_json::Value = serde_json::from_str(&result)
        .expect("Result should be valid JSON");
    
    // Should have an error field
    assert!(output.get("error").is_some(), "Should return error for invalid expression");
}

#[tokio::test]
async fn test_sandbox_capabilities() {
    // Test capability set creation and checking
    let caps = CapabilitySet::new()
        .with(Capability::FileRead { paths: vec![PathBuf::from("/tmp")] })
        .with(Capability::MemoryStore { read: true, write: true });
    
    assert!(caps.can_read_file(std::path::Path::new("/tmp/test.txt")));
    assert!(!caps.can_read_file(std::path::Path::new("/etc/passwd")));
    assert!(!caps.can_write_file(std::path::Path::new("/tmp/test.txt")));
}

#[tokio::test]
async fn test_sandbox_config_presets() {
    let trusted = SandboxConfig::trusted();
    let sandboxed = SandboxConfig::default();
    let ephemeral = SandboxConfig::ephemeral();
    
    // Trusted has more fuel than sandboxed
    assert!(trusted.max_fuel > sandboxed.max_fuel);
    
    // Ephemeral has less execution time
    assert!(ephemeral.max_execution_ms < sandboxed.max_execution_ms);
    
    // Ephemeral has WASI disabled
    assert!(!ephemeral.enable_wasi);
}

#[tokio::test]
async fn test_spawn_and_status() {
    let sandbox = WasmSandbox::new().expect("Failed to create sandbox");
    
    // Spawn an empty instance
    let id = sandbox.spawn(&[], SandboxConfig::default(), None)
        .await
        .expect("Failed to spawn instance");
    
    // Check status
    let status = sandbox.status(id).await.expect("Status should exist");
    assert_eq!(status.id, id);
    assert!(!status.running);
    assert_eq!(status.fuel_consumed, 0);
    
    // Terminate
    sandbox.terminate(id).await.expect("Terminate should succeed");
    
    let status = sandbox.status(id).await.expect("Status should still exist");
    assert!(status.exit_code.is_some());
}

#[tokio::test]
async fn test_code_editor_plugin() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tmp_test_workspace");
    
    // Create temp workspace
    if !workspace_root.exists() {
        std::fs::create_dir_all(&workspace_root).expect("Failed to create temp workspace");
    }
    
    // Clean up potentially existing files
    let test_file = workspace_root.join("test_file.txt");
    if test_file.exists() {
        std::fs::remove_file(&test_file).expect("Failed to remove test file");
    }

    let sandbox = WasmSandbox::with_workspace(workspace_root.clone())
        .expect("Failed to create sandbox");
    
    let plugin_path = plugins_dir().join("code_editor.wasm");
    if !plugin_path.exists() {
        eprintln!("Skipping test: code_editor.wasm not found");
        return;
    }
    
    sandbox.load_module_from_file(plugin_path.clone())
        .await
        .expect("Failed to load plugin");

    // 1. Write a file
    let write_cmd = r#"{
        "action": "Write",
        "args": {
            "path": "test_file.txt",
            "content": "Hello Autopoietic World!"
        }
    }"#;
    
    // Create config with write permissions
    let mut config = SandboxConfig::default();
    config.capabilities = CapabilitySet::new()
        .with(Capability::FileWrite { paths: vec![PathBuf::from("test_file.txt")] })
        .with(Capability::FileRead { paths: vec![PathBuf::from("test_file.txt")] });
        
    let result = sandbox.execute_with_config(&plugin_path, "edit", write_cmd, config.clone())
        .await
        .expect("Execution failed");
    
    let output: serde_json::Value = serde_json::from_str(&result).expect("Invalid JSON");
    assert!(output["success"].as_bool().unwrap(), "Write failed: {:?}", output);

    // Verify file exists on disk
    assert!(test_file.exists(), "File was not created on disk");
    let content = std::fs::read_to_string(&test_file).expect("Failed to read file");
    assert_eq!(content, "Hello Autopoietic World!");

    // 2. Read the file
    let read_cmd = r#"{
        "action": "Read",
        "args": {
            "path": "test_file.txt"
        }
    }"#;

    let result = sandbox.execute_with_config(&plugin_path, "edit", read_cmd, config)
        .await
        .expect("Execution failed");

    let output: serde_json::Value = serde_json::from_str(&result).expect("Invalid JSON");
    assert!(output["success"].as_bool().unwrap(), "Read failed: {:?}", output);
    assert_eq!(output["data"].as_str().unwrap(), "Hello Autopoietic World!");

    // Cleanup
    std::fs::remove_dir_all(&workspace_root).unwrap_or_default();
}

#[tokio::test]
async fn test_code_editor_permissions() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tmp_perm_test");

    std::fs::create_dir_all(&workspace_root).expect("Failed to create workspace");

    let sandbox = WasmSandbox::with_workspace(workspace_root.clone())
        .expect("Failed to create sandbox");

    let plugin_path = plugins_dir().join("code_editor.wasm");
    if !plugin_path.exists() {
        eprintln!("Skipping test: code_editor.wasm not found");
        return;
    }

    sandbox.load_module_from_file(plugin_path.clone())
        .await
        .expect("Failed to load plugin");

    // Sandbox with READ-ONLY permissions
    let mut config = SandboxConfig::default();
    config.capabilities = CapabilitySet::new()
        .with(Capability::FileRead { paths: vec![PathBuf::from("/")] }); // Only read

    // Try to Write
    let write_cmd = r#"{
        "action": "Write",
        "args": {
            "path": "forbidden.txt",
            "content": "Should fail"
        }
    }"#;

    let result = sandbox.execute_with_config(&plugin_path, "edit", write_cmd, config)
        .await
        .expect("Execution failed");

    let output: serde_json::Value = serde_json::from_str(&result).expect("Invalid JSON");
    
    // Should fail
    assert!(!output["success"].as_bool().unwrap(), "Write should have failed due to permissions");
    if let Some(err) = output["error"].as_str() {
        assert!(err.contains("Permission denied"), "Error should be Permission denied, got: {}", err);
    } else {
        panic!("Expected error message in output: {:?}", output);
    }

    // Cleanup
    std::fs::remove_dir_all(&workspace_root).unwrap_or_default();
}
