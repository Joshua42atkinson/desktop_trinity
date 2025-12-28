// Trinity AI Agent System
// Copyright (c) Joshua
// Shared under license for Ask_Pete (Purdue University)

use tarpc::{client, context, tokio_serde::formats::Bincode};
use tokio_util::codec::LengthDelimitedCodec;
use trinity_protocol::{brain::BrainServiceClient, task::TaskType};
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Connect to Brain
    let server_addr: SocketAddr = "127.0.0.1:9000".parse()?;
    let stream = tokio::net::TcpStream::connect(server_addr).await?;
    
    let codec = LengthDelimitedCodec::new();
    let framed = tokio_util::codec::Framed::new(stream, codec);
    let transport = tarpc::serde_transport::new(framed, Bincode::default());
    
    let client = BrainServiceClient::new(client::Config::default(), transport).spawn();
    
    println!("🔌 Connected to Trinity Brain @ {}", server_addr);

    // 2. Submit Task
    // We'll ask it to generate a simple Usability Report artifact as a test of the flow
    // functioning with Quadradical (LM Studio)
    let task_name = "Generate Usability Report Template";
    let output_path = "usability_report_template.md";
    
    let task_type = TaskType::GenerateCode {
        prompt: "Create a markdown template for a Usability Report. Include sections for Executive Summary, Key Findings, and Recommendations.".to_string(),
        language: "markdown".to_string(),
        output_path: Some(output_path.to_string()),
    };
    
    println!("🚀 Submitting task: {}", task_name);
    let task_id = client.submit_task(context::current(), task_name.to_string(), task_type, 1).await??;
    
    println!("✅ Task submitted with ID: {}", task_id);
    
    // 3. Poll for Completion
    println!("⏳ Waiting for completion...");
    let mut attempts = 0;
    loop {
        if attempts > 60 {
            println!("❌ Timed out waiting for task completion.");
            break;
        }
        
        // Fix: Unwrap the Result from RPC call
        let completed = client.list_completed_tasks(context::current(), 10).await?;
        if let Some(result) = completed.iter().find(|r| r.task_id == task_id) {
            println!("🎉 Task Completed!");
            println!("   Success: {}", result.success);
            if let Some(output) = &result.output {
                println!("   Output: {:.100}...", output);
            } else if let Some(err) = &result.error {
                 println!("   Error: {}", err);
            }
            break;
        }
        
        // Fix: Unwrap the Result from RPC call
        let pending = client.list_pending_tasks(context::current()).await?;
        if let Some(info) = pending.iter().find(|t| t.id == task_id) {
            println!("   ... Task status: {} (Agent: {:?})", info.status, info.agent);
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        attempts += 1;
    }
    
    // Check if file exists
    if std::path::Path::new(output_path).exists() {
        println!("✅ File created: {}", output_path);
        let content = std::fs::read_to_string(output_path)?;
        println!("---\n{}\n---", content);
    } else {
        println!("⚠️ Output file not found!");
    }

    Ok(())
}
