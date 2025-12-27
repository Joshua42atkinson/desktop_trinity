#[cfg(feature = "voice_whisper")]
use anyhow::Context;
use anyhow::Result;
#[cfg(feature = "voice_whisper")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

pub struct SttEngine {
    #[cfg(feature = "voice_whisper")]
    context: Option<WhisperContext>,
}

impl SttEngine {
    #[allow(unused_variables)] // Used in voice_whisper feature
    pub fn new(model_path: &str) -> Result<Self> {
        #[cfg(feature = "voice_whisper")]
        {
            // Verify file exists
            if !std::path::Path::new(model_path).exists() {
                tracing::warn!("Whisper model not found at: {}", model_path);
                return Ok(Self { context: None });
            }

            let ctx = WhisperContext::new_with_params(model_path, Default::default())
                .context("Failed to load Whisper model")?;

            Ok(Self { context: Some(ctx) })
        }
        #[cfg(not(feature = "voice_whisper"))]
        {
            Ok(Self {})
        }
    }

    #[allow(unused_variables)] // Used in voice_whisper feature
    pub fn transcribe(&mut self, audio: &[f32]) -> Result<String> {
        #[cfg(feature = "voice_whisper")]
        {
            if let Some(ctx) = &mut self.context {
                let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
                params.set_language(Some("en"));
                params.set_print_special(false);
                params.set_print_progress(false);
                params.set_print_realtime(false);
                params.set_print_timestamps(false);

                let mut state = ctx
                    .create_state()
                    .context("Failed to create whisper state")?;
                state
                    .full(params, audio)
                    .context("Failed to run whisper inference")?;

                let num_segments = state.full_n_segments().context("Failed to get segments")?;
                let mut text = String::new();
                for i in 0..num_segments {
                    if let Ok(segment) = state.full_get_segment_text(i) {
                        text.push_str(&segment);
                        text.push(' ');
                    }
                }
                Ok(text.trim().to_string())
            } else {
                Ok(String::new())
            }
        }
        #[cfg(not(feature = "voice_whisper"))]
        {
            Ok("".to_string())
        }
    }
}
