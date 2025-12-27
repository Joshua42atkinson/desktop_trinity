use tarpc::{client, context, server::{BaseChannel, Channel}, tokio_serde::formats::Bincode};
use tokio::net::TcpListener;
use tokio_util::codec::LengthDelimitedCodec;
use trinity_protocol::{
    brain::{BrainService, BrainServiceClient},
    types::{ChatMessage, VoicePacket, VoiceResponse, EmotionData, AvatarState, ModelInfo, ImageRequest, ImageResponse, ProtocolError, HardwareStats},
};
use futures::StreamExt;
use uuid;

#[derive(Clone)]
struct MockBrain;

impl BrainService for MockBrain {
    async fn chat(self, _: context::Context, msg: ChatMessage, _history: Vec<ChatMessage>) -> String {
        format!("Echo: {}", msg.content)
    }

    async fn voice_chat(self, _: context::Context, _: VoicePacket) -> VoicePacket {
        VoicePacket {
            audio_data: vec![],
            sample_rate: 44100,
        }
    }

    async fn ping(self, _: context::Context) -> bool {
        true
    }

    async fn model_info(self, _: context::Context) -> Option<ModelInfo> {
        Some(ModelInfo {
            name: "MockBrain-1.0".to_string(),
            quantization: "f16".to_string(),
            context_size: 4096,
        })
    }

    async fn submit_task(self, _: context::Context, _name: String, _task_type: trinity_protocol::task::TaskType, _priority: u8) -> Result<uuid::Uuid, trinity_protocol::types::ProtocolError> {
        Ok(uuid::Uuid::new_v4())
    }

    async fn cancel_task(self, _: context::Context, _task_id: uuid::Uuid) -> Result<bool, trinity_protocol::types::ProtocolError> {
        Ok(true)
    }

    async fn get_queue_status(self, _: context::Context) -> trinity_protocol::task::QueueStatus {
        trinity_protocol::task::QueueStatus {
            pending: 0,
            running: 0,
            completed: 0,
            failed: 0,
            is_running: true,
            uptime_secs: Some(100),
        }
    }

    async fn list_pending_tasks(self, _: context::Context) -> Vec<trinity_protocol::task::TaskInfo> {
        vec![]
    }

    async fn list_completed_tasks(self, _: context::Context, _limit: usize) -> Vec<trinity_protocol::task::TaskResult> {
        vec![]
    }

    async fn get_agent_status(self, _: context::Context) -> Vec<trinity_protocol::stream::AgentStatus> {
        vec![
            trinity_protocol::stream::AgentStatus {
                id: "agent-0".to_string(),
                name: "MockCoder".to_string(),
                model_tier: trinity_protocol::stream::ModelTier::Standard,
                is_busy: false,
                current_task: None,
            },
        ]
    }

    async fn get_orchestrator_config(self, _: context::Context) -> trinity_protocol::stream::OrchestratorConfig {
        trinity_protocol::stream::OrchestratorConfig::default()
    }

    async fn update_agent_config(self, _: context::Context, _config: trinity_protocol::stream::AgentConfig) -> Result<(), trinity_protocol::types::ProtocolError> {
        Ok(())
    }

    async fn poll_events(self, _: context::Context, _since_id: u64) -> Vec<trinity_protocol::stream::StreamEvent> {
        vec![]
    }

    async fn chat_with_voice(self, _: context::Context, _message: ChatMessage, _synthesize_audio: bool) -> VoiceResponse {
        VoiceResponse {
            text: "Mock voice response".to_string(),
            audio: None,
            emotion: EmotionData::default(),
            avatar_state: AvatarState::Idle,
        }
    }

    async fn generate_image(self, _: context::Context, _request: ImageRequest) -> Result<ImageResponse, ProtocolError> {
        Err(ProtocolError {
            code: 501,
            message: "Mock: Not implemented".to_string(),
        })
    }

    async fn get_hardware_stats(self, _: context::Context) -> HardwareStats {
        HardwareStats {
            memory_used_bytes: 0,
            memory_available_bytes: 128 * 1024 * 1024 * 1024,
            memory_percent: 0.0,
            cpu_percent: 0.0,
            load_avg_1m: 0.0,
            gpu_available: true,
        }
    }

    async fn generate_code(self, _: context::Context, request: trinity_protocol::types::CodeRequest) -> Result<trinity_protocol::types::CodeResponse, ProtocolError> {
        Ok(trinity_protocol::types::CodeResponse {
            code: format!("// Mock code for: {}", request.prompt),
            language: request.language,
            saved_path: None,
            syntax_valid: true,
        })
    }

    async fn generate_document(self, _: context::Context, request: trinity_protocol::types::WriteRequest) -> Result<trinity_protocol::types::WriteResponse, ProtocolError> {
        Ok(trinity_protocol::types::WriteResponse {
            content: format!("# {}\n\nMock document content.", request.topic),
            word_count: 5,
            saved_path: None,
        })
    }
}

#[tokio::test]
async fn test_rpc_flow() -> anyhow::Result<()> {
    // 1. Start Server
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    println!("Mock Server listening on {}", addr);

    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let codec = LengthDelimitedCodec::new();
                    let framed = tokio_util::codec::Framed::new(stream, codec);
                    let transport = tarpc::serde_transport::new(framed, Bincode::default());
                    
                    let channel = BaseChannel::with_defaults(transport);
                    let server = MockBrain;
                    let _ = channel.execute(server.serve())
                        .for_each(|r| async move {
                            tokio::spawn(r);
                        }).await;
                });
            }
        }
    });

    // 2. Start Client
    let stream = tokio::net::TcpStream::connect(addr).await?;
    let codec = LengthDelimitedCodec::new();
    let framed = tokio_util::codec::Framed::new(stream, codec);
    let transport = tarpc::serde_transport::new(framed, Bincode::default());
    let client = BrainServiceClient::new(client::Config::default(), transport).spawn();

    // 3. Test Ping
    let alive = client.ping(context::current()).await?;
    assert!(alive, "Brain should be alive");

    // 4. Test Model Info
    let info = client.model_info(context::current()).await?.expect("Should have info");
    assert_eq!(info.name, "MockBrain-1.0");

    // 5. Test Chat
    let response = client.chat(context::current(), ChatMessage {
        role: "user".to_string(),
        content: "Hello".to_string(),
        timestamp: 0,
    }, vec![]).await?;
    assert_eq!(response, "Echo: Hello");

    Ok(())
}
