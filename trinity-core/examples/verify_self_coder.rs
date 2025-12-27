use anyhow::Result;
use std::path::PathBuf;
use trinity_core::agent::specialized::self_coder::{SelfCoder, SelfCoderConfig};
use trinity_core::config::TrinityConfig;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!(">>> Starting SelfCoder Verification (Llama-4-Scout)");
    println!(">>> ---------------------------------------------");

    // Configure model to use Llama-4-Scout
    // We update the config before initialization so create_brain() picks it up
    let mut config = TrinityConfig::load().unwrap_or_default();
    config.models.default_model_path = PathBuf::from(
        "/home/joshua/.lmstudio/models/lmstudio-community/Llama-4-Scout-17B-16E-Instruct-GGUF/Llama-4-Scout-17B-16E-Instruct-Q4_K_M-00001-of-00002.gguf"
    );
    // Persist this config override for the test session
    // (Note: In a real app we'd pass config, but create_brain loads from disk/env.
    //  So we'll use env var override for the test).
    std::env::set_var(
        "TRINITY_MODEL_PATH",
        config.models.default_model_path.to_str().unwrap(),
    );

    let coder_config = SelfCoderConfig::default();
    let coder = SelfCoder::new(coder_config);

    println!(">>> Requesting Code Generation...");
    let instruction = "Write a Rust function that calculates the nth Fibonacci number efficiently.";
    let context = "We are in a high-performance Rust environment.";

    let start = std::time::Instant::now();
    let code = coder.generate_code(instruction, context).await?;
    let elapsed = start.elapsed();

    println!(">>> ---------------------------------------------");
    println!(">>> Code Generated in {:.2?}!", elapsed);
    println!(">>> Output:\n{}", code);
    println!(">>> ---------------------------------------------");

    Ok(())
}
