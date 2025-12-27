//! Trinity Hardware Verification CLI
//!
//! Diagnostic tool for verifying AMD Strix Halo hardware capabilities
//! and testing LLM model loading/inference.
//!
//! # Usage
//! ```bash
//! trinity-hw info          # Display hardware summary
//! trinity-hw models        # List discovered GGUF models
//! trinity-hw load <path>   # Test loading a model
//! trinity-hw chat <path>       # Interactive inference REPL
//! trinity-hw chat-mem <path>   # Interactive memory-augmented chat REPL
//! ```

use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use sysinfo::System;
use trinity_core::brain::desktop::DesktopBrain;
use trinity_core::brain::orchestrator::BrainOrchestrator;
use trinity_core::brain::Brain;
use trinity_core::chat::{ChatConfig, ChatEngine};
use trinity_core::learning::UnifiedMemory;

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("trinity=info".parse()?)
                .add_directive("llama_cpp_2=warn".parse()?),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "info" => cmd_info(),
        "models" => cmd_models(),
        "load" => {
            let path = args.get(2).map(|s| s.as_str());
            cmd_load(path)
        }
        "chat" => {
            let path = args.get(2).map(|s| s.as_str());
            cmd_chat(path)
        }
        "chat-mem" => {
            let path = args.get(2).map(|s| s.as_str());
            cmd_chat_mem(path)
        }
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════════╗
║           TRINITY HARDWARE VERIFICATION CLI                  ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  USAGE:                                                      ║
║    trinity-hw <command> [options]                            ║
║                                                              ║
║  COMMANDS:                                                   ║
║    info              Display hardware summary                ║
║    models            List discovered GGUF models             ║
║    load <path>       Test loading a model                    ║
║    chat <path>       Interactive inference REPL              ║
║    chat-mem <path>   Memory-augmented chat REPL              ║
║    help              Show this message                       ║
║                                                              ║
║  EXAMPLES:                                                   ║
║    trinity-hw info                                           ║
║    trinity-hw models                                         ║
║    trinity-hw chat ~/.lmstudio/models/.../model.gguf         ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
"#
    );
}

/// Display hardware information
fn cmd_info() -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_mem_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let avail_mem_gb = sys.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let cpu_count = sys.cpus().len();

    // Get CPU name
    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // Read ROCm version
    let rocm_version = std::fs::read_to_string("/opt/rocm/.info/version")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Not detected".to_string());

    // Check kernel parameters
    let cmdline = std::fs::read_to_string("/proc/cmdline").unwrap_or_default();
    let gtt_size = if cmdline.contains("amdgpu.gttsize=") {
        cmdline
            .split_whitespace()
            .find(|s| s.starts_with("amdgpu.gttsize="))
            .map(|s| s.replace("amdgpu.gttsize=", ""))
            .unwrap_or_else(|| "default".to_string())
    } else {
        "default (not set)".to_string()
    };

    // Check HSA override
    let hsa_override =
        std::env::var("HSA_OVERRIDE_GFX_VERSION").unwrap_or_else(|_| "not set".to_string());

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                  TRINITY HARDWARE INFO                       ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  CPU: {:55} ║", truncate(&cpu_name, 55));
    println!("║  Cores: {:53} ║", cpu_count);
    println!("║                                                              ║");
    println!(
        "║  Total Memory: {:7.1} GB                                    ║",
        total_mem_gb
    );
    println!(
        "║  Available:    {:7.1} GB                                    ║",
        avail_mem_gb
    );
    println!("║                                                              ║");
    println!("║  ROCm Version: {:46} ║", truncate(&rocm_version, 46));
    println!("║  GTT Size:     {:46} ║", truncate(&gtt_size, 46));
    println!("║  HSA Override: {:46} ║", truncate(&hsa_override, 46));
    println!("║                                                              ║");

    // GPU detection via rocm-smi
    if let Ok(output) = std::process::Command::new("rocm-smi")
        .arg("--showproductname")
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("GPU") || line.contains("Radeon") {
                let gpu_name = line
                    .split(':')
                    .next_back()
                    .map(|s| s.trim())
                    .unwrap_or("Unknown");
                println!("║  GPU: {:55} ║", truncate(gpu_name, 55));
            }
        }
    } else {
        println!("║  GPU: (rocm-smi not available)                               ║");
    }

    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Recommendations
    if gtt_size == "default (not set)" {
        println!("⚠️  RECOMMENDATION: Set kernel parameter for 124GB VRAM:");
        println!("   Add to /etc/default/grub:");
        println!("   GRUB_CMDLINE_LINUX=\"amdgpu.gttsize=126976 ttm.pages_limit=32505856\"");
        println!();
    }

    if hsa_override == "not set" {
        println!("⚠️  RECOMMENDATION: Set HSA override for gfx1151:");
        println!("   export HSA_OVERRIDE_GFX_VERSION=11.5.1");
        println!();
    }

    Ok(())
}

/// List discovered GGUF models
fn cmd_models() -> Result<()> {
    println!();
    println!("🔍 Scanning for GGUF models...");
    println!();

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let search_paths = vec![
        PathBuf::from(&home).join(".lmstudio/models"),
        PathBuf::from(&home).join("antigravity"),
        PathBuf::from(&home).join("models"),
    ];

    let mut models: Vec<(PathBuf, u64)> = Vec::new();

    for search_path in search_paths {
        if search_path.exists() {
            find_gguf_files(&search_path, &mut models);
        }
    }

    if models.is_empty() {
        println!("No GGUF models found in standard locations.");
        println!("Searched: ~/.lmstudio/models, ~/antigravity, ~/models");
        return Ok(());
    }

    // Sort by size (largest first)
    models.sort_by(|a, b| b.1.cmp(&a.1));

    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                           DISCOVERED MODELS                                  ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");

    for (path, size) in &models {
        let size_gb = *size as f64 / (1024.0 * 1024.0 * 1024.0);
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let name_truncated = truncate(&name, 55);
        println!("║  {:55} {:>7.1} GB  ║", name_truncated, size_gb);
    }

    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Total: {} models found", models.len());
    println!();

    Ok(())
}

fn find_gguf_files(dir: &Path, results: &mut Vec<(PathBuf, u64)>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_gguf_files(&path, results);
            } else if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                if let Ok(meta) = std::fs::metadata(&path) {
                    results.push((path, meta.len()));
                }
            }
        }
    }
}

/// Test loading a model
fn cmd_load(path: Option<&str>) -> Result<()> {
    let model_path = match path {
        Some(p) => p.to_string(),
        None => {
            eprintln!("Usage: trinity-hw load <model_path>");
            return Ok(());
        }
    };

    // Expand ~ to home directory
    let model_path = expand_path(&model_path);

    if !Path::new(&model_path).exists() {
        eprintln!("Model file not found: {}", model_path);
        return Ok(());
    }

    let file_size = std::fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0);
    let size_gb = file_size as f64 / (1024.0 * 1024.0 * 1024.0);

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     MODEL LOAD TEST                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  File: {:54} ║", truncate(&model_path, 54));
    println!(
        "║  Size: {:7.2} GB                                            ║",
        size_gb
    );
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Set HSA override
    std::env::set_var("HSA_OVERRIDE_GFX_VERSION", "11.5.1");

    println!("⏳ Loading model (this may take a while for large models)...");
    let start = std::time::Instant::now();

    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(async {
        let brain = DesktopBrain::new();
        brain.load_model(&model_path).await
    });

    let elapsed = start.elapsed();

    match result {
        Ok(()) => {
            println!(
                "✅ Model loaded successfully in {:.1}s",
                elapsed.as_secs_f64()
            );
            println!();

            // Memory check
            let mut sys = System::new_all();
            sys.refresh_all();
            let avail_gb = sys.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
            println!("💾 Available memory after load: {:.1} GB", avail_gb);
        }
        Err(e) => {
            println!("❌ Failed to load model: {}", e);
            println!();
            println!("Troubleshooting:");
            println!("  - Check HSA_OVERRIDE_GFX_VERSION is set to 11.5.1");
            println!("  - Verify kernel parameters (amdgpu.gttsize)");
            println!("  - Ensure enough memory is available");
        }
    }

    Ok(())
}

/// Interactive chat REPL
fn cmd_chat(path: Option<&str>) -> Result<()> {
    let model_path = match path {
        Some(p) => expand_path(p),
        None => {
            eprintln!("Usage: trinity-hw chat <model_path>");
            return Ok(());
        }
    };

    if !Path::new(&model_path).exists() {
        eprintln!("Model file not found: {}", model_path);
        return Ok(());
    }

    // Set HSA override
    std::env::set_var("HSA_OVERRIDE_GFX_VERSION", "11.5.1");

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    TRINITY CHAT REPL                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Type your message and press Enter.                          ║");
    println!("║  Type 'exit' or Ctrl+C to quit.                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    println!("⏳ Loading model...");
    let start = std::time::Instant::now();

    let rt = tokio::runtime::Runtime::new()?;
    let brain = rt.block_on(async {
        let brain = DesktopBrain::new();
        brain.load_model(&model_path).await?;
        Ok::<_, anyhow::Error>(brain)
    })?;

    let elapsed = start.elapsed();
    println!("✅ Model loaded in {:.1}s", elapsed.as_secs_f64());
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("You: ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        print!("Trinity: ");
        stdout.flush()?;

        let start = std::time::Instant::now();
        let response = rt.block_on(brain.think(input))?;
        let elapsed = start.elapsed();

        println!("{}", response);
        println!(
            "  [{:.1}s, ~{} tokens]",
            elapsed.as_secs_f64(),
            response.split_whitespace().count()
        );
        println!();
    }

    Ok(())
}

/// Interactive memory-augmented chat REPL
fn cmd_chat_mem(path: Option<&str>) -> Result<()> {
    let model_path = match path {
        Some(p) => expand_path(p),
        None => {
            eprintln!("Usage: trinity-hw chat-mem <model_path>");
            return Ok(());
        }
    };

    if !Path::new(&model_path).exists() {
        eprintln!("Model file not found: {}", model_path);
        return Ok(());
    }

    // Set HSA override
    std::env::set_var("HSA_OVERRIDE_GFX_VERSION", "11.5.1");

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║             TRINITY MEMORY CHAT REPL                         ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Type your message and press Enter.                          ║");
    println!("║  Type 'exit' or Ctrl+C to quit.                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    println!("⏳ Initializing system...");
    let start_init = std::time::Instant::now();

    let rt = tokio::runtime::Runtime::new()?;

    // Initialize components
    let (chat_engine, session_id) = rt.block_on(async {
        // 1. Memory
        println!("   - Initializing Unified Memory (~/.trinity)...");
        let memory = std::sync::Arc::new(UnifiedMemory::default_config().await?);
        let session_id = memory.start_session().await;

        // 2. Brain (Orchestrator wrapping DesktopBrain)
        println!("   - Initializing Brain...");
        let brain = DesktopBrain::new();
        brain.load_model(&model_path).await?;

        // Use a simplified orchestrator for the CLI test
        // Creating a BrainOrchestrator requires a TieredBrainManager which manages loading.
        // For this simple CLI, we'll manually inject the loaded brain into a MockOrchestrator behavior or construct properly.
        // Actually, let's use the Orchestrator properly but we need to trick it or configure it to use our already loaded brain
        // OR better yet, let's just use ChatEngine with a MockBrain that wraps our DesktopBrain if Orchestrator is too complex to setup here.
        // WAIT: ChatEngine requires BrainOrchestrator. BrainOrchestrator manages loading.
        // So we should configure the Orchestrator with a preset that points to our model file.

        // Simpler approach for CLI test: Construct ChatEngine but mock the Orchestrator?
        // No, let's build a real Orchestrator but verify the config.
        // Strix Halo presets use hardcoded paths.
        // We want to force it to use the user-provided path.
        // Let's create a custom single-tier manager.

        let mut tiers = trinity_core::brain::tiered::TieredBrainManager::new(128.0);
        let tier_config = trinity_core::brain::tiered::TierConfig {
            model_path: std::path::PathBuf::from(&model_path),
            ..Default::default()
        };
        // Register for all tiers so any request works
        tiers.configure_tier(
            trinity_core::brain::tiered::BrainTier::Reflection,
            tier_config.clone(),
        );
        tiers.configure_tier(
            trinity_core::brain::tiered::BrainTier::Tasks,
            tier_config.clone(),
        );
        tiers.configure_tier(trinity_core::brain::tiered::BrainTier::Swarm, tier_config);

        let orchestrator = std::sync::Arc::new(BrainOrchestrator::new(tiers, 1)); // 1 slot

        // 3. Chat Engine
        let config = ChatConfig::default();
        let engine = ChatEngine::new(memory, orchestrator, config);

        Ok::<_, anyhow::Error>((engine, session_id))
    })?;

    let elapsed = start_init.elapsed();
    println!("✅ System ready in {:.1}s", elapsed.as_secs_f64());
    println!("📝 Session ID: {}", session_id);
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("You: ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        print!("Trinity: ");
        stdout.flush()?;

        let start = std::time::Instant::now();
        let response = rt.block_on(chat_engine.chat(session_id, input));
        let elapsed = start.elapsed();

        match response {
            Ok(text) => {
                println!("{}", text);
                println!(
                    "  [{:.1}s, ~{} tokens]",
                    elapsed.as_secs_f64(),
                    text.split_whitespace().count()
                );
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        path.replacen("~", &home, 1)
    } else {
        path.to_string()
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
