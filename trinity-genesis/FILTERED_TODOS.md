./patches/llama-cpp-2/src/log.rs:236:                // TODO: Support logging this to stdout directly via options?
./patches/llama-cpp-2/src/lib.rs:519:    // TODO: Reinitialize the state to support calling send_logs_to_tracing multiple times.
./AGENTS.md:99:## 📋 TODO Workflow
./AGENTS.md:101:When addressing TODOs:
./AGENTS.md:103:1. **Find all**: `grep -rn "TODO\|FIXME" --include="*.rs" ./`
./AGENTS.md:108:### Current Critical TODOs
./AI_MIRROR_QUICKSTART.md:170:**Fix**: This is expected until full GGUF loading is implemented. The API structure works, inference is TODO.
./tools/piper/piper/espeak-ng-data/lang/roa/ht:4:maintainer  // TODO somebody should take responsibility for this
./backend/src/agent/self_coder.rs:121:    // TODO: Ideally this should also be async-safe or use a channel,
./backend/src/agent/autonomous.rs:295:                    duration: Duration::from_secs(0), // TODO: track actual duration
./backend/src/agent/autonomous.rs:474:                        log::info!("Found {} TODOs via self-grooming. scheduling fixes for top 3.", count);
./backend/src/agent/autonomous.rs:478:                                format!("Auto-Fix TODO in {}", file_path),
./backend/src/agent/autonomous.rs:481:                                    instructions: "Analyze this file, locate the TODO or FIXME comments, and attempt to implement the missing functionality. If the task is too complex, add a detailed explanation comment instead.".to_string() 
./backend/src/agent/autonomous.rs:489:                    Ok(format!("Scanned for TODOs in {}. Found {}, scheduled 3 fixes.", payload, count))
./backend/src/agent/autonomous.rs:508:            "Self-Grooming: Scan for TODOs",
./backend/src/agent/autonomous.rs:542:// Helper to scan for TODOs
./backend/src/agent/autonomous.rs:562:                if content.contains("TODO") || content.contains("FIXME") {
./backend/src/routes/memory.rs:55:/// TODO: Replace with real embedding model via trinity-core
./backend/src/routes/memory.rs:190:                conversations_stored: 0, // TODO: Track by source type
./backend/src/routes/memory.rs:204:    // TODO: Integrate MemoryConsolidator when PostgreSQL is connected
./backend/src/routes/autonomous.rs:251:        recommendations: vec!["Consider reviewing TODO fixes from dream cycles".to_string()],
./backend/src/ui/mod.rs:175:                    // TODO: Show about dialog
./backend/src/ai/conversation_memory.rs:54:        // TODO: Implement actual sentiment analysis
./backend/src/ai/conversation_memory.rs:57:        // TODO: Implement virtue keyword extraction
./backend/src/ai/prompts/mod.rs:22:        // TODO: Implement more sophisticated analysis in Phase 2
./backend/src/ai/socratic_engine.rs:127:        // TODO: More sophisticated filtering
./backend/src/handlers/ai_mirror.rs:45:    // TODO: Move engine initialization to app state
./backend/src/handlers/ai_mirror.rs:99:    // TODO: Implement actual history retrieval from ConversationMemory
./trinity-core/src/agent/specialized/self_coder.rs:96:    // model: Option<Arc<Mutex<GgufModel>>>, // TODO: Re-integrate with Brain trait
./trinity-core/src/learning/memory_system.rs:252:                            // TODO: Merge with local? For now, return remote if successful
./trinity-core/src/learning/scanner.rs:93:    // TODO: Add PDF support later (needs pdf-extract)
./trinity-core/src/brain/desktop.rs:215:    /// TODO: Wire up to UI streaming in Phase 6
./trinity-core/src/brain/orchestrator.rs:175:        // TODO: True parallel execution with separate model instances
./trinity-core/src/visuals/procedural.rs:26:    // TODO: Self-Coding Agent will populate this with actual mesh generation code.
./trinity-core/src/system_check.rs:14:        // Self::check_accelerator()?; // TODO: Re-implement with llama-cpp check if needed
./trinity-core/src/voice/manager.rs:36:            // 2. VAD Check (TODO)
./trinity-core/Cargo.toml:128:    # "embeddings", # TODO: Enable when ort stabilizes
./trinity-genesis/quadradical-ui/index.html:66:                        <div class="stat-label">Total TODOs</div>
./trinity-genesis/quadradical-ui/index.html:138:                        <div class="term-line">   found 507 TODO items across 48 files.</div>
./trinity-genesis/docs/scope_creep/self_coding.md:21:1. Workspace scanner (find TODOs/FIXMEs)
./trinity-genesis/docs/README.md:60:- **`//`** - Implementation notes, TODOs, internal reasoning
./trinity-genesis/docs/README.md:66:| `// TODO:` | Feature to implement |
./trinity-genesis/docs/USABILITY_TODO.md:1:# Trinity Educational AI OS — Usability Audit & TODO
./trinity-genesis/docs/OVERNIGHT_TASK.md:134:// TODO: Future improvement needed
./trinity-genesis/crates/trinity-skills/src/web.rs:35:        // TODO: Improve extraction logic (remove scripts, styles, etc.)
./trinity-genesis/crates/trinity-skills/src/media/image_gen.rs:233:        // TODO: Implement actual SDXL inference with candle
./trinity-genesis/crates/trinity-body/src/bridge.rs:459:                    vram_used_gb: 0.0, // TODO: Get from GPU
./trinity-genesis/crates/trinity-body/src/agent_systems.rs:206:                // TODO: Actually spawn particle entities
./trinity-genesis/crates/trinity-brain/src/main.rs:483:        // TODO: Actually update orchestrator config
./trinity-genesis/crates/trinity-brain/src/main.rs:776:                    // TODO: Handle failure (remove from map, fail runtime)
./trinity-genesis/crates/trinity-cli/src/main.rs:60:    /// Ingest TODOs from codebase
./trinity-genesis/crates/trinity-cli/src/main.rs:311:            println!("🔍 Scanning workspace for TODOs...");
./trinity-genesis/crates/trinity-kernel/src/wasm_sandbox.rs:342:        // TODO: Actually compile and instantiate the Wasm module using wasmtime
./trinity-genesis/crates/trinity-kernel/src/wasm_sandbox.rs:374:        // TODO: Actually call the Wasm function
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:1://! # TODO Parser — Self-Improvement Task Extraction
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:4://! "Trinity reads its own conscience. The TODO file is the moral compass;
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:7://! This module parses markdown TODO/roadmap files into structured
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:13:/// A parsed TODO item from markdown
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:40:    /// Parse a markdown file into TODO items
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:161:            .with_description(format!("Auto-generated from TODO: {}", self.title))
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:164:    /// Infer TaskType from the TODO item content
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:208:                "You are a Rust expert. Complete this TODO item:\n\n{}\n\nProvide the implementation.",
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:220:/// Parse a TODO file and return actionable tasks
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:226:/// Scan a workspace directory for TODO comments in code files
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:268:/// Scan a single file for TODO patterns
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:304:/// Extract TODO content from a line comment
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:309:    // // TODO: message
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:311:    // /// TODO: message
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:312:    // # TODO: message
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:313:    // <!-- TODO: message -->
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:316:    let keywords = ["TODO", "FIXME", "XXX", "HACK"];
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:339:/// Get only incomplete items from a TODO file
./trinity-genesis/crates/trinity-kernel/src/todo_parser.rs:352:# Test TODO
./trinity-genesis/crates/trinity-kernel/src/npu_backend.rs:159:        // TODO: Initialize RyzenAI SDK when available
./trinity-genesis/crates/trinity-kernel/src/npu_backend.rs:183:        // TODO: Use RyzenAI loader with W4ABF16 format
./trinity-genesis/crates/trinity-kernel/src/npu_backend.rs:207:        // TODO: Actual RyzenAI inference call
./trinity-genesis/crates/trinity-kernel/src/npu_backend.rs:230:        // TODO: Query actual NPU metrics via RyzenAI SDK
./trinity-genesis/crates/trinity-kernel/src/orchestrator.rs:574:                    thought: format!("Scanning workspace for TODOs: {}", path),
./trinity-genesis/crates/trinity-kernel/src/orchestrator.rs:582:                let mut content = format!("Found {} TODO items in {}\n\n", items.len(), path);
./trinity-genesis/crates/trinity-kernel/src/orchestrator.rs:612:                Ok(format!("Scan complete: Found {} TODOs", items.len()))
./trinity-genesis/crates/trinity-kernel/src/agent_graph.rs:421:        // For now, simple sequential execution (TODO: parallel with topological sort)
./trinity-genesis/crates/trinity-kernel/src/rpc_pool.rs:197:            // TODO: Actual TCP connection with TCP_NODELAY
./trinity-genesis/crates/trinity-kernel/src/rpc_pool.rs:239:        // TODO: Actually send data over RPC
./trinity-genesis/crates/trinity-kernel/src/rpc_pool.rs:267:        // TODO: Actually retrieve data over RPC
./ai:58:# Fix a TODO
./ai:64:        -d "{\"messages\":[{\"role\":\"user\",\"content\":\"How should I implement this TODO in Rust? Be specific with code: $context\"}],\"max_tokens\":1024}" \
./ai:83:        echo "  $0 todo <description>  Get implementation for a TODO"
./trinity_staging/pending/game_screen_v5.rs:63:        // TODO: transition to game scene
./trinity_staging/pending/game_screen_v3.rs:48:                    // TODO: navigate to game screen
./trinity_staging/pending/game_screen_v3.rs:52:                    // TODO: open settings modal
./trinity_staging/pending/game_screen_v3.rs:56:                    // TODO: open quest log
./trinity_staging/pending/game_screen_v3.rs:60:                    // TODO: exit
./trinity_staging/pending/game_screen_v4.rs:79:        // TODO: navigate to game scene
./trinity_staging/pending/game_screen_v4.rs:86:        // TODO: open settings overlay
./trinity_staging/pending/game_screen_v4.rs:93:        // TODO: close the app / navigate away
./frontend/src/components/mod.rs:7:// pub mod help_panel;  // TODO: Fix Leptos closure threading issues
./frontend/src/components/mod.rs:10:// pub mod persona_quiz;  // TODO: Fix Leptos 0.8 compatibility issues
./frontend/src/components/mod.rs:13:// pub mod tutorial_overlay;  // TODO: Fix Leptos closure threading issues
./frontend/src/pages/daydream.rs:96:                        on:click=move |_| { /* TODO: show help */ }
./frontend/src/pages/autonomous.rs:231:                            "🔍 Scan Codebase for TODOs"
./trinity-desktop/src/main.rs:121:                                // TODO: Integrate with ModelManager
./trinity-desktop/src/main.rs:127:                                // TODO: Integrate with ModelManager
./.agent/workflows/fix-todos.md:2:description: Workflow for addressing TODOs and FIXMEs in Trinity codebase
./.agent/workflows/fix-todos.md:4:# TODO Resolution Workflow
./.agent/workflows/fix-todos.md:6:Systematic approach to fixing TODOs and FIXMEs.
./.agent/workflows/fix-todos.md:10:## 1. Scan for TODOs
./.agent/workflows/fix-todos.md:14:grep -rn "TODO\|FIXME" --include="*.rs" ./backend/src ./trinity-core/src | head -50
./.agent/workflows/fix-todos.md:39:For each TODO:
./.agent/workflows/fix-todos.md:55:git commit -m "TODO: <description>"
./.agent/workflows/fix-todos.md:58:## 4. Common TODO Patterns
./.agent/workflows/fix-todos.md:82:After fixing batch of TODOs:
./.agent/workflows/fix-todos.md:96:Remove fixed items from Critical TODOs section in AGENTS.md.
./models/tokenizer.json:5628:      "ĠTODO": 5343,
./models/tokenizer.json:15017:      "TODO": 14732,
./models/tokenizer.json:87746:      "_TODO": 87461,
./models/tokenizer.json:91273:      ".TODO": 90988,
