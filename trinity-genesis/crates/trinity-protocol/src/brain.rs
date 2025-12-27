use crate::stream::{AgentConfig, AgentStatus, OrchestratorConfig, StreamEvent};
use crate::task::{QueueStatus, TaskInfo, TaskResult, TaskType};
use crate::types::{ChatMessage, CodeRequest, CodeResponse, ImageRequest, ImageResponse, ModelInfo, ProtocolError, VoicePacket, VoiceResponse, WriteRequest, WriteResponse};
use uuid::Uuid;

#[tarpc::service]
pub trait BrainService {
    /// Process a text chat message with conversation history and return the response text
    async fn chat(message: ChatMessage, history: Vec<ChatMessage>) -> String;

    /// Process a voice packet and return the audio response
    async fn voice_chat(audio: VoicePacket) -> VoicePacket;
    
    /// Process a chat message and return text + synthesized voice
    async fn chat_with_voice(message: ChatMessage, synthesize_audio: bool) -> VoiceResponse;
    
    /// Generate an image from a text prompt
    async fn generate_image(request: ImageRequest) -> Result<ImageResponse, ProtocolError>;

    /// Check if the brain is alive
    async fn ping() -> bool;

    /// Get information about the loaded model
    async fn model_info() -> Option<ModelInfo>;

    // ------------------------------------------------------------------------
    // Skill Endpoints (Coder & Writer)
    // ------------------------------------------------------------------------

    /// Generate code using the Coder skill (grammar-constrained)
    async fn generate_code(request: CodeRequest) -> Result<CodeResponse, ProtocolError>;

    /// Generate a document using the Writer skill
    async fn generate_document(request: WriteRequest) -> Result<WriteResponse, ProtocolError>;

    // ------------------------------------------------------------------------
    // Autonomous Task Features
    // ------------------------------------------------------------------------

    /// Submit a new task to the autonomous runtime
    async fn submit_task(name: String, task_type: TaskType, priority: u8) -> Result<Uuid, ProtocolError>;

    /// Cancel a task by ID
    async fn cancel_task(task_id: Uuid) -> Result<bool, ProtocolError>;

    /// Get current queue status
    async fn get_queue_status() -> QueueStatus;

    /// List all pending tasks
    async fn list_pending_tasks() -> Vec<TaskInfo>;

    /// List recently completed tasks
    async fn list_completed_tasks(limit: usize) -> Vec<TaskResult>;

    // ------------------------------------------------------------------------
    // Streaming & Orchestrator (Antigravity Window)
    // ------------------------------------------------------------------------

    /// Get agent status for all active agents
    async fn get_agent_status() -> Vec<AgentStatus>;

    /// Get orchestrator configuration
    async fn get_orchestrator_config() -> OrchestratorConfig;

    /// Update an agent's configuration
    async fn update_agent_config(config: AgentConfig) -> Result<(), ProtocolError>;

    /// Poll for new stream events (returns batch of recent events)
    async fn poll_events(since_id: u64) -> Vec<StreamEvent>;

    /// Get real-time hardware statistics
    async fn get_hardware_stats() -> crate::types::HardwareStats;
}

