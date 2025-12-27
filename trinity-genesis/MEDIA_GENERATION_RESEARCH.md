# Trinity Genesis - Media Generation Integration Research

## Current Source Code

**File**: `TRINITY_SOURCE_DUMP.rs` (5028 lines)

- All Rust source code from trinity-genesis crates

---

## TTS System (Already Built)

**File**: `crates/trinity-kernel/src/voice.rs`

```rust
// Already supports:
- EmotionState (happy, angry, sad, excited, concerned)
- VoiceStyle (speed, pitch, energy, voice_id)
- VoiceOutput (text + emotion + style + stage directions)
- SpeakingResponse parsing ([EMOTION: X] [SPEED: X] tags)
```

**To activate TTS:**

1. Integrate Zonos/Coqui/Bark TTS service
2. Wire VoiceOutput to audio synthesis
3. Play audio through Body UI

---

## NVIDIA Nemotron (NOT for Media Gen)

Nemotron is for **text agents**, not media generation:

- Nemotron 3 Nano 30B - Coding/reasoning agents
- Llama Nemotron Super 49B - Research agents  
- Nemotron Nano VL 12B - Vision-language (document/video understanding)
- Nemotron RAG - Document retrieval

**Not** image/video/audio generation.

---

## Actual Media Generation Options

### Image Generation

| Model | Integration | Notes |
|-------|-------------|-------|
| **SDXL** (Stable Diffusion) | candle-transformers | Already in Cargo.toml |
| **Flux** | API or local | High quality |
| **DALLE-3** | OpenAI API | Easy but not local |

### Video Generation

| Model | Integration | Notes |
|-------|-------------|-------|
| **CogVideoX** | Hugging Face | Open source |
| **Runway Gen-3** | API | High quality |
| **Kling** | API | Good for short clips |

### TTS (Voice)

| Model | Integration | Notes |
|-------|-------------|-------|
| **Zonos** | Local ONNX | Already planned |
| **Coqui XTTS** | Local | Voice cloning |
| **Bark** | Local | Expressive speech |
| **Sesame CSM** | Local | Emotional speech |
| **ElevenLabs** | API | Premium quality |

### Music

| Model | Integration | Notes |
|-------|-------------|-------|
| **MusicGen** | Hugging Face | Meta's model |
| **Suno** | API | Song generation |

---

## Recommended Integration Path

### Phase 1: TTS (Voice Avatar)

```rust
// crates/trinity-kernel/src/tts.rs
pub struct TtsEngine {
    model: ZonosModel,  // or CoquiXTTS
}

impl TtsEngine {
    pub fn synthesize(&self, output: VoiceOutput) -> AudioBuffer;
}
```

### Phase 2: Image Generation

```rust
// crates/trinity-skills/src/media/image_gen.rs
pub struct ImageGenerator {
    pipeline: StableDiffusionPipeline,  // candle
}

impl ImageGenerator {
    pub async fn generate(&self, prompt: &str) -> Image;
}
```

### Phase 3: Video (Storyboard → Clips)

1. Generate storyboard images with SDXL
2. Animate with img2video model
3. Concatenate clips

---

## Files to Create

1. `crates/trinity-kernel/src/tts.rs` - TTS engine
2. `crates/trinity-skills/src/media/mod.rs` - Media module
3. `crates/trinity-skills/src/media/image_gen.rs` - Image generation
4. `crates/trinity-skills/src/media/video_gen.rs` - Video generation
5. `crates/trinity-body/src/panels/media_studio.rs` - Media UI

---

## Google Drive Integration

- Mount at: `/mnt/gdrive` or use rclone
- Store generated assets for Brain/Body sharing
- Use for project files and exports
