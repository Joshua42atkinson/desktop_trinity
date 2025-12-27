use anyhow::Result;
use trinity_core::brain::{create_brain, Brain};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let model_path = "Qwen3-235B-A22B-Thinking-2507-GGUF/Qwen3-235B-A22B-Thinking-2507-Q3_K_L-00001-of-00003.gguf";

    println!(">>> Starting Memory Load Verification");
    println!(">>> ---------------------------------");
    println!(">>> Target Model: {}", model_path);
    println!(">>> This will attempt to load the 235B Qwen model (~105GB).");
    println!(">>> This is the definitive test for Strix Halo 128GB UMA.");
    println!(">>> Ensure you are monitoring VRAM/GTT with `amdgpu_top`.");

    if !std::path::Path::new(model_path).exists() {
        anyhow::bail!(
            "Model not found at: {}. Please check your current working directory.",
            model_path
        );
    }

    let start = std::time::Instant::now();

    // Load config and clear default model to prevent dual loading
    let mut config = trinity_core::config::TrinityConfig::load().unwrap_or_default();
    config.models.default_model_path = std::path::PathBuf::new();

    // Use our brain factory with the modified config
    let brain = trinity_core::brain::desktop::DesktopBrain::with_config(config);

    println!(">>> Starting model load into GPU (Shard 1)...");
    brain.load_model(model_path).await?;

    let elapsed = start.elapsed();
    println!(">>> ---------------------------------");
    println!(">>> SUCCESS: Model Loaded in {:.2?}!", elapsed);
    println!(">>> If you see this, the 30GB Wall is officially shattered!");
    println!(">>> ---------------------------------");

    // Quick smoketest of generation
    println!(">>> Running quick inference smoketest...");
    let response = brain
        .think("Explain why a 128GB unified memory pool is game-changing for LLMs.")
        .await?;
    println!(">>> Brain Response: {}", response);

    Ok(())
}
