use crate::agent::workflow::systems::WorkflowPlugin;
use crate::agent::workflow::{SharedWorkflowState, SharedWorkflowStateResource};
use axum::{extract::FromRef, response::IntoResponse, Router};
use bevy::prelude::{
    default, App as BevyApp, EventReader, Name, PluginGroup, StateTransitionEvent, Update,
};
// use frontend::app::App; // Unused in SPA mode
use leptos::config::get_configuration;
use leptos::prelude::*;
// use leptos_axum::{generate_route_list, LeptosRoutes}; // Unused in SPA mode
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::sync::{Arc, RwLock};
// Trinity Core imports
use tokio::sync::RwLock as TokioRwLock;
use tower_http::cors::{Any, CorsLayer};
use trinity_core::chat::{ChatConfig, ChatEngine};
use trinity_core::learning::UnifiedMemory;
use trinity_core::notebook::TrinityNotebook;

use crate::agent::autonomous::{AutonomousRuntime, RuntimeConfig};
use crate::agent::self_coder::{SelfCoderConfig, SelfCodingAgent};

// Trinity Tiered Brain Orchestration
use trinity_core::brain::orchestrator::BrainOrchestrator;
use trinity_core::brain::tiered::strix_halo_presets;

mod agent; // [NEW] Swarm AI Agent Framework
mod ai;
mod domain;
mod error;
mod game;
mod handlers;
mod plugin; // [NEW] Trinity Plugin System
mod routes;
mod static_assets;
mod ui; // [NEW] Native Bevy+egui UI

use domain::player::get_simulated_character;
pub use error::{AppError, Result};
use routes::ai_mirror::ai_mirror_routes;
use routes::expert::expert_routes;
use routes::persona::persona_routes;
use routes::player::player_routes;
use routes::research::research_routes;
use static_assets::Assets; // [NEW]

use crate::game::components::*;
use crate::game::systems::*;

use bevy_yarnspinner::prelude::*;

// Define a shared application state
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: Option<PgPool>,
    pub shared_research_log: Arc<RwLock<ResearchLog>>,
    pub shared_virtues: Arc<RwLock<VirtueTopology>>,
    pub shared_workflow_state: Arc<RwLock<SharedWorkflowState>>,
    // Trinity Notebook (RAG-powered knowledge base)
    pub notebook: Arc<TokioRwLock<TrinityNotebook>>,
    pub memory: Arc<UnifiedMemory>,
    // Tiered AI Brain
    pub orchestrator: Arc<BrainOrchestrator>,
    pub chat_engine: Arc<ChatEngine>,
}

// Implement FromRef<AppState> for LeptosOptions
impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

// Implement FromRef<AppState> for PgPool
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone().expect(
            "Database pool not available. This handler should not be reachable in simulation mode.",
        )
    }
}

#[allow(dead_code)]
fn run_bevy_app(
    _shared_log: Arc<RwLock<ResearchLog>>,
    _shared_virtues: Arc<RwLock<VirtueTopology>>,
    _shared_workflow: Arc<RwLock<SharedWorkflowState>>,
) {
    // Deprecated: Logic moved to main()
}

// [NEW] Handler for static assets
async fn static_handler(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match <Assets as rust_embed::RustEmbed>::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => {
            if path.contains('.') {
                return axum::http::StatusCode::NOT_FOUND.into_response();
            }
            // Fallback to index.html for SPA routing
            match <Assets as rust_embed::RustEmbed>::get("index.html") {
                Some(index) => (
                    [(axum::http::header::CONTENT_TYPE, "text/html")],
                    index.data,
                )
                    .into_response(),
                None => axum::http::StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}

// Browser auto-open removed for single-window experience

fn main() {
    println!("Starting Trinity AI OS...");
    dotenv::dotenv().ok();

    // 1. Initialize Runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // 2. Initialize Shared State
    let shared_research_log = Arc::new(RwLock::new(ResearchLog::default()));
    let shared_virtues = Arc::new(RwLock::new(VirtueTopology::default()));
    let shared_workflow_state = Arc::new(RwLock::new(SharedWorkflowState::default()));

    // 3. Initialize Database (Blocking)
    let pool = match env::var("DATABASE_URL") {
        Ok(database_url) => {
            println!("DATABASE_URL found, connecting...");
            rt.block_on(async {
                Some(
                    PgPoolOptions::new()
                        .max_connections(5)
                        .connect(&database_url)
                        .await
                        .expect("Failed to create database pool"),
                )
            })
        }
        Err(_) => {
            println!("WARN: DATABASE_URL not found. Running in SIMULATION MODE.");
            None
        }
    };

    // 4. Initialize Core Systems (Orchestrator, Agent, Runtime)
    // Note: These need to be cloned for the server thread
    let tiered_manager = strix_halo_presets();
    // 4 swarm slots
    let orchestrator = Arc::new(BrainOrchestrator::new(tiered_manager, 4));
    println!("✓ Tiered AI Orchestrator initialized");

    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let allow_root = std::env::var("TRINITY_ALLOW_ROOT").unwrap_or_default() == "true";
    let workspace_root = if allow_root {
        std::path::PathBuf::from("/")
    } else {
        current_dir
    };

    let mut coder_config = SelfCoderConfig::default()
        .with_workspace(workspace_root)
        .allow_dangerous();

    if let Ok(password) =
        std::env::var("SUDO_PASSWORD").or_else(|_| std::env::var("TRINITY_SUDO_PW"))
    {
        coder_config = coder_config.with_sudo(password);
    }

    // Choose brain based on environment: USE_NATIVE=true for native llama-cpp, otherwise LM Studio
    use trinity_core::brain::Brain;
    let brain: Arc<dyn Brain> = if std::env::var("USE_NATIVE").unwrap_or_default() == "true" {
        println!("✓ Using native BrainOrchestrator for inference (slow)");
        orchestrator.clone()
    } else {
        println!("✓ Using LM Studio API for inference (http://localhost:1234)");
        let lm_brain = crate::ai::LMStudioBrain::from_env();
        Arc::new(lm_brain)
    };

    let self_coding_agent = SelfCodingAgent::with_brain(coder_config, brain.clone());
    let shared_agent = Arc::new(tokio::sync::Mutex::new(self_coding_agent));

    // TaskStore & Notebook
    let trinity_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".trinity");
    let task_store_path = trinity_dir.join("tasks.db");
    let task_store = Arc::new(
        crate::agent::TaskStore::new(&task_store_path)
            .unwrap_or_else(|_| crate::agent::TaskStore::in_memory().unwrap()),
    );

    let runtime = AutonomousRuntime::new(RuntimeConfig::default(), task_store.clone());
    let shared_runtime = Arc::new(std::sync::Mutex::new(runtime));

    // Vector Store & Notebook

    // Vector Store & Notebook
    let (memory, notebook) = rt.block_on(async {
        let mem = Arc::new(
            UnifiedMemory::default_config()
                .await
                .expect("Failed UnifiedMemory"),
        );
        let nb = Arc::new(TokioRwLock::new(
            TrinityNotebook::with_vector_store(mem.vector_store().clone())
                .expect("Failed Notebook"),
        ));
        (mem, nb)
    });

    let chat_config = ChatConfig::default();
    let chat_engine = Arc::new(ChatEngine::new(
        memory.clone(),
        orchestrator.clone(),
        chat_config,
    ));

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    let app_state = AppState {
        leptos_options,
        pool,
        shared_research_log: shared_research_log.clone(),
        shared_virtues: shared_virtues.clone(),
        shared_workflow_state: shared_workflow_state.clone(),
        notebook,
        memory,
        orchestrator: orchestrator.clone(),
        chat_engine: chat_engine.clone(),
    };

    // 5. Spawn Server on Runtime
    let server_app_state = app_state.clone();
    let server_runtime = shared_runtime.clone();

    // autonomous loop setup
    let runtime_for_loop = shared_runtime.clone();
    let agent_for_loop = shared_agent.clone();

        // Autonomous Loop
        tokio::spawn(async move {
            println!("Initializing Autonomous Runtime (Async)...");
            let mut local_runtime = {
                let guard = runtime_for_loop.lock().unwrap();
                guard.clone()
            };
            println!("Starting Autonomous Loop...");
            if let Err(e) = local_runtime.run(agent_for_loop).await {
                eprintln!("CRITICAL: Autonomous runtime crashed: {}", e);
            }
        });

        // Web Server
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        let autonomous_router = routes::autonomous::autonomous_routes(server_runtime);

        let app = Router::new()
            .merge(player_routes(&server_app_state))
            .merge(persona_routes(&server_app_state))
            .merge(expert_routes(&server_app_state))
            .merge(research_routes(&server_app_state))
            .merge(routes::workflow::workflow_routes(&server_app_state))
            .merge(routes::notebook::notebook_routes(&server_app_state))
            .merge(routes::memory::memory_routes(&server_app_state))
            .merge(routes::avatar_api::avatar_routes(&server_app_state))
            .merge(routes::terminal::terminal_routes(&server_app_state))
            .merge(routes::hardware::hardware_routes()) // [NEW] Hardware API
            .nest("/api/chat", routes::chat::chat_routes()) // Chat API
            .nest("/api/ai-mirror", ai_mirror_routes())
            .fallback(static_handler)
            .layer(cors.clone())
            .with_state(server_app_state)
            .merge(autonomous_router.layer(cors));

        println!("Backend listening on http://{}", &addr);
        axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app)
            .await
            .unwrap();
    });

    // 6. Run Bevy App (Main Thread)
    let mut app = BevyApp::new();

    // Use DefaultPlugins but configure Window
    // Use DefaultPlugins but configure Window
    app.add_plugins(bevy::DefaultPlugins.set(bevy::window::WindowPlugin {
        primary_window: Some(bevy::window::Window {
            title: "Trinity AI OS".to_string(),
            resolution: bevy::window::WindowResolution::new(1280.0, 800.0),
            resizable: true, // Allow resizing
            decorations: true,
            ..default()
        }),
        ..default()
    }));

    app.add_plugins(YarnSpinnerPlugin::new());
    app.add_plugins(agent::systems::AgentSwarmPlugin);
    app.add_plugins(WorkflowPlugin);

    // Add Trinity UI Plugin
    app.add_plugins(crate::ui::TrinityUiPlugin);

    // Add Native Avatar Plugin from Trinity Core
    app.add_plugins(trinity_core::visuals::avatar::AvatarPlugin);

    // Insert Shared Resources
    app.insert_resource(SharedResearchLogResource(shared_research_log));
    app.insert_resource(SharedVirtuesResource(shared_virtues));
    app.insert_resource(SharedWorkflowStateResource(shared_workflow_state));

    // Register Systems
    app.add_systems(
        Update,
        (
            update_virtue_topology,
            monitor_cognitive_load,
            log_research_events,
            sync_yarn_to_story_progress,
            // open_browser_system removed
        ),
    );

    // Spawn Initial Entities (Student, Avatar) - Same as before
    let simulated_player = get_simulated_character();
    app.world_mut().spawn(StudentBundle {
        name: Name::new(simulated_player.name),
        persona: Persona {
            archetype: Archetype::Novice,
            shadow_trait: "None".to_string(),
            projective_dissonance: 0.0,
        },
        virtue_topology: VirtueTopology::default(),
        cognitive_load: CognitiveLoad::default(),
        story_progress: StoryProgress {
            current_quest_id: simulated_player.current_quest_id,
            current_step_id: simulated_player.current_step_id,
            current_step_description: simulated_player.current_step_description,
            history: Vec::new(),
            inventory: simulated_player.inventory,
            quest_flags: simulated_player.quest_flags,
            learned_vocab: simulated_player.learned_vocab,
        },
        research_log: ResearchLog::default(),
        level: Level(1),
        xp: Experience(0),
    });

    app.world_mut()
        .spawn(trinity_core::visuals::avatar::TrinityAvatar {
            name: "Trinity".to_string(),
            state: trinity_core::visuals::avatar::AvatarState::Idle,
        });

    app.run();
}
