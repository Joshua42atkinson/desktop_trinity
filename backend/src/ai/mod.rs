#![allow(unused)]
pub mod conversation_memory;
pub mod gguf_agent;
pub mod hardware;
pub mod llm;
pub mod lmstudio_brain;
pub mod lmstudio_client;
pub mod model_manager;
pub mod prompts;
pub mod qwen3;
pub mod socratic_engine;
pub mod tool_executor;
pub mod tool_factory;
pub mod tools;

pub use conversation_memory::{ConversationMemory, Speaker, Turn, TurnMetadata};
pub use gguf_agent::{GgufAgent, GgufAgentConfig};
pub use hardware::{detect_hardware, print_hardware_summary, ComputeConfig, HardwareInfo};
pub use lmstudio_brain::LMStudioBrain;
pub use lmstudio_client::{ChatMessage, LMStudioClient};
pub use model_manager::ModelManager;
pub use qwen3::{parse_qwen3_response, Qwen3Config, Qwen3Response};
pub use socratic_engine::{SessionContext, SocraticEngine, SocraticResponse};
pub use tool_executor::{ExecutorConfig, ToolExecutor};
pub use tool_factory::{create_research_tools, ToolFactory};
pub use tools::{create_builtin_tools, Tool, ToolCall, ToolRegistry, ToolResult};
