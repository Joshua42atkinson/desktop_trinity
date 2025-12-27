use crate::brain::Brain;
use crate::voice::{input::AudioInput, stt::SttEngine, tts::TtsEngine};
use anyhow::Result;
use std::sync::Arc;
// use tokio::sync::Mutex;

/// Voice manager for audio input/output (Phase 8: Voice Integration)
/// Currently stubbed - full integration pending whisper-rs/llama-cpp-2 conflict resolution
#[allow(dead_code)]
pub struct VoiceManager {
    input: AudioInput,
    stt: SttEngine,
    tts: TtsEngine,
    brain: Arc<Box<dyn Brain>>,
}

impl VoiceManager {
    pub fn new(brain: Arc<Box<dyn Brain>>, stt_model: &str, tts_model: &str) -> Result<Self> {
        let input = AudioInput::new()?;
        let stt = SttEngine::new(stt_model)?;
        let tts = TtsEngine::new(tts_model)?;

        Ok(Self {
            input,
            stt,
            tts,
            brain,
        })
    }

    pub async fn run_loop(&mut self) -> Result<()> {
        loop {
            // 1. Read Audio
            let _chunk = self.input.read_chunk()?;

            // 2. VAD Check (TODO)

            // 3. STT
            // let text = self.stt.transcribe(&chunk)?;

            // 4. Brain Think
            // let response = self.brain.think(&text).await?;

            // 5. TTS
            // self.tts.speak(&response)?;

            // For now, allow yielding
            #[cfg(feature = "desktop")]
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            #[cfg(target_arch = "wasm32")]
            gloo_timers::future::TimeoutFuture::new(10).await;
        }
    }
}
