pub mod input;
pub mod manager;
pub mod stt;
pub mod tts;

use anyhow::Result;
use async_trait::async_trait;

/// Core Voice Loop interface
#[async_trait]
pub trait VoiceLoop: Send + Sync {
    /// Start listening for wake words or continuous speech
    async fn start_listening(&self) -> Result<()>;

    /// Synthesize speech from text
    async fn speak(&self, text: &str) -> Result<()>;
}
