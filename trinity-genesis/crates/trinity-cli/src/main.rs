use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use tarpc::{client, context, tokio_serde::formats::Bincode};
use tokio::net::TcpStream;
use trinity_protocol::brain::BrainServiceClient;
use trinity_protocol::task::TaskType;
use trinity_kernel::todo_parser::TodoItem;
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1:9000")]
    addr: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ping the brain
    Ping,
    /// Submit a code generation task (async, returns task ID)
    GenCode {
        #[arg(short, long)]
        prompt: String,
        #[arg(short, long)]
        lang: String,
        #[arg(short, long)]
        out: Option<String>,
    },
    /// Generate code synchronously (RPC call, waits for result)
    GenCodeSync {
        #[arg(short, long)]
        prompt: String,
        #[arg(short, long)]
        lang: String,
    },
    /// Generate a document synchronously (RPC call, waits for result)
    WriteDoc {
        #[arg(short, long)]
        topic: String,
        #[arg(short, long, default_value = "technical")]
        style: String,
        #[arg(short, long, default_value = "500")]
        words: u32,
    },
    /// Submit a generic thinking task
    Think {
        #[arg(short, long)]
        prompt: String,
    },
    /// Queue the standard documentation tasks
    QueueDocs,
    /// Check the status of the Brain (Queue, Agents, Hardware)
    Status,
    /// View recent task history
    History,
    /// Ingest TODOs from codebase
    IngestTodos,
    /// Test autopoietic mutation (DANGER: modifies code!)
    TestAutopoiesis {
        #[arg(short, long, default_value = "crates/trinity-kernel/src/lib.rs")]
        file: String,
        /// Mutation to apply (append marker comment)
        #[arg(short, long, default_value = "// Autopoietic test marker")]
        code: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    // println!("🔮 Trinity CLI - Connecting to {}...", cli.addr);

    let socket_addr: SocketAddr = cli.addr.parse()?;
    let stream = TcpStream::connect(&socket_addr).await?;
    let codec = tarpc::tokio_util::codec::LengthDelimitedCodec::new();
    let framed = tarpc::tokio_util::codec::Framed::new(stream, codec);
    let transport = tarpc::serde_transport::new(framed, Bincode::default());
    
    let client = BrainServiceClient::new(client::Config::default(), transport).spawn();

    match cli.command {
        Commands::Ping => {
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let alive = client.ping(ctx).await?;
            println!("Ping result: {}", alive);
        }
        Commands::Status => {
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);

            // 1. Hardware Stats
            let hw = client.get_hardware_stats(ctx).await?;
            
            // 2. Queue Status
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let queue = client.get_queue_status(ctx).await?;

            // 3. Agent Status
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let agents = client.get_agent_status(ctx).await?;

            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║                   TRINITY STATUS DASHBOARD                   ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ Hardware:  CPU: {:>3}%  RAM: {:>3}%  GPU: {:<17} ║", 
                hw.cpu_percent, hw.memory_percent, 
                if hw.gpu_available { "Online ✅" } else { "Offline ❌" });
            println!("║ Queue:     Pending: {:<4} Running: {:<4} Completed: {:<4}    ║", 
                queue.pending, queue.running, queue.completed);
            println!("╠══════════════════════════════════════════════════════════════╣");
            println!("║ Agents:                                                      ║");
            if agents.is_empty() {
                println!("║   No active agents                                           ║");
            } else {
                for agent in agents {
                    let status_icon = if agent.is_busy { "🔨" } else { "💤" };
                    let task = agent.current_task.unwrap_or_else(|| "Idle".to_string());
                    // Truncate task if too long
                    let task_disp = if task.len() > 30 {
                        format!("{}...", &task[..27])
                    } else {
                        task
                    };
                    println!("║   {} {:<10} : {:<35} ║", status_icon, agent.name, task_disp);
                }
            }
            println!("╚══════════════════════════════════════════════════════════════╝\n");
        }
        Commands::History => {
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            let results = client.list_completed_tasks(ctx, 10).await?;

            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║                     RECENT TASK HISTORY                      ║");
            println!("╠══════════════════════════════════════════════════════════════╣");
            for res in results {
                let status_icon = if res.success { "✅" } else { "❌" };
                println!("║ {} {:<45} ║", status_icon, res.task_name);
                if !res.success {
                     let err_msg = res.error.clone().unwrap_or_else(|| "Unknown error".to_string());
                     println!("║    Error: {:<46} ║", err_msg.chars().take(46).collect::<String>());
                } else {
                     let output_msg = res.output.clone().unwrap_or_else(|| "No output".to_string());
                     println!("║    Out:   {:<46} ║", output_msg.chars().take(46).collect::<String>());
                     println!("║    Time:  {:<46} ║", format!("{}ms", res.duration_ms));
                }
            }
            println!("╚══════════════════════════════════════════════════════════════╝\n");
        }
        Commands::GenCode { prompt, lang, out } => {
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
            let id = client.submit_task(
                ctx,
                format!("CLI: Generate {} Code", lang),
                TaskType::GenerateCode {
                    prompt,
                    language: lang,
                    output_path: out,
                },
                1 // Normal priority
            ).await??;
            println!("✅ Task Submitted: {}", id);
        }
        Commands::GenCodeSync { prompt, lang } => {
            println!("🔨 Generating {} code synchronously...", lang);
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
            
            let request = trinity_protocol::types::CodeRequest {
                prompt,
                language: lang.clone(),
                output_path: None,
                use_grammar: true,
            };
            
            match client.generate_code(ctx, request).await? {
                Ok(response) => {
                    println!("\n╔══════════════════════════════════════════════════════════════╗");
                    println!("║                    CODE GENERATION RESULT                    ║");
                    println!("╠══════════════════════════════════════════════════════════════╣");
                    println!("║ Language: {:50} ║", response.language);
                    println!("║ Syntax Valid: {:46} ║", if response.syntax_valid { "✅ Yes" } else { "❌ No" });
                    println!("║ Lines: {:49} ║", response.code.lines().count());
                    println!("╠══════════════════════════════════════════════════════════════╣");
                    println!("║ Code:                                                        ║");
                    println!("╚══════════════════════════════════════════════════════════════╝\n");
                    println!("{}", response.code);
                }
                Err(e) => {
                    eprintln!("❌ Code generation failed: {}", e.message);
                }
            }
        }
        Commands::WriteDoc { topic, style, words } => {
            println!("📝 Generating {} document about '{}'...", style, topic);
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
            
            let write_style = match style.to_lowercase().as_str() {
                "technical" => trinity_protocol::types::WriteStyle::Technical,
                "blog" | "blogpost" => trinity_protocol::types::WriteStyle::BlogPost,
                "tutorial" => trinity_protocol::types::WriteStyle::Tutorial,
                "creative" => trinity_protocol::types::WriteStyle::Creative,
                "formal" => trinity_protocol::types::WriteStyle::Formal,
                "casual" => trinity_protocol::types::WriteStyle::Casual,
                _ => trinity_protocol::types::WriteStyle::Technical,
            };
            
            let request = trinity_protocol::types::WriteRequest {
                topic,
                style: write_style,
                target_words: words,
                output_path: None,
            };
            
            match client.generate_document(ctx, request).await? {
                Ok(response) => {
                    println!("\n╔══════════════════════════════════════════════════════════════╗");
                    println!("║                   DOCUMENT GENERATION RESULT                 ║");
                    println!("╠══════════════════════════════════════════════════════════════╣");
                    println!("║ Word Count: {:48} ║", response.word_count);
                    println!("╠══════════════════════════════════════════════════════════════╣");
                    println!("╚══════════════════════════════════════════════════════════════╝\n");
                    println!("{}", response.content);
                }
                Err(e) => {
                    eprintln!("❌ Document generation failed: {}", e.message);
                }
            }
        }
        Commands::Think { prompt } => {
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
            
            let id = client.submit_task(
                ctx,
                "CLI: Think".to_string(),
                TaskType::Think { prompt },
                1
            ).await??;
            println!("✅ Task Submitted: {}", id);
        }
        Commands::QueueDocs => {
            println!("📄 Queueing Documentation Tasks...");
            let files = vec![
                "crates/trinity-kernel/src/lib.rs",
                "crates/trinity-kernel/src/brain.rs",
                "crates/trinity-kernel/src/orchestrator.rs",
                "crates/trinity-body/src/main.rs",
            ];

            for file in files {
                let mut ctx = context::current();
                ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
                
                let id = client.submit_task(
                    ctx,
                    format!("Document: {}", file),
                    TaskType::EditFile {
                        path: file.to_string(),
                        instructions: "Add comprehensive module-level documentation (//! ...) and struct/enum documentation (/// ...) to this file. Ensure it explains the 'Philosophy' and 'Key Types'.".to_string(),
                    },
                    0 // Low priority (overnight)
                ).await??;
                println!("   queued: {} ({})", file, id);
            }
            println!("✅ Doc queue populated!");
        }
        Commands::TestAutopoiesis { file, code } => {
            println!("⚠️  AUTOPOIETIC MUTATION TEST");
            println!("   File: {}", file);
            println!("   Code: {}", code);
            println!();
            println!("This will test the autopoietic engine by:");
            println!("  1. Copying source to staging");
            println!("  2. Appending the marker to the file");
            println!("  3. Validating syntax");
            println!("  4. Compiling in staging");
            println!("  5. Creating backup");
            println!("  6. Promoting to live");
            println!();
            
            // For now, just submit as a task - full autopoietic integration via CLI TBD
            let mut ctx = context::current();
            ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
            
            let id = client.submit_task(
                ctx,
                format!("Autopoietic Test: {}", file),
                TaskType::EditFile {
                    path: file,
                    instructions: format!("Append the following comment to the end of the file: {}", code),
                },
                2 // High priority
            ).await??;
            println!("✅ Autopoietic test task submitted: {}", id);
            println!("   Monitor with: trinity-cli status");
        }
        Commands::IngestTodos => {
            println!("🔍 Scanning workspace for TODOs...");
            // Get workspace root
            let root = std::env::current_dir()?;
            
            // Use the kernel parser
            let items: Vec<TodoItem> = trinity_kernel::todo_parser::scan_workspace_for_todos(&root)?;
            println!("   Found {} potential tasks.", items.len());
            
            let mut submitted = 0;
            for (i, item) in items.iter().enumerate() {
                // Submit task
                let mut ctx = context::current();
                ctx.deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
                
                let task_type = item.to_autonomous_task().task_type;
                
                // Map priority
                let priority_u8 = match item.priority {
                    trinity_protocol::task::TaskPriority::Low => 0,
                    trinity_protocol::task::TaskPriority::Normal => 1,
                    trinity_protocol::task::TaskPriority::High => 2,
                    trinity_protocol::task::TaskPriority::Critical => 3,
                };
                
                match client.submit_task(
                    ctx,
                    item.title.clone(),
                    task_type,
                    priority_u8
                ).await {
                    Ok(Ok(id)) => {
                        println!("   [{:>2}/{:<2}] [+] Queued: {} ({})", i+1, items.len(), item.title, id);
                        submitted += 1;
                    }
                    Ok(Err(e)) => {
                        eprintln!("   [{:>2}/{:<2}] [-] Brain rejected task: {} - {}", i+1, items.len(), item.title, e.message);
                    }
                    Err(e) => {
                        eprintln!("   [{:>2}/{:<2}] [-] RPC Error: {} - {}", i+1, items.len(), item.title, e);
                        // If connection dropped, stop trying this batch
                        if e.to_string().contains("shutdown") || e.to_string().contains("Broken pipe") {
                            eprintln!("❌ Fatal: RPC connection lost. Aborting batch.");
                            break;
                        }
                    }
                }
                
                // Rate limit to prevent overwhelming the server
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            
            println!("\n✅ Successfully ingested {}/{} tasks into Quadradical.", submitted, items.len());
        }
    }

    Ok(())
}
