#[cfg(feature = "desktop")]
use anyhow::Context;
use anyhow::Result;
#[cfg(feature = "desktop")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "desktop")]
use ringbuf::{traits::*, HeapCons, HeapRb};

/// Audio input for voice capture (Phase 8: Voice Integration)
#[allow(dead_code)]
pub struct AudioInput {
    #[cfg(feature = "desktop")]
    stream: Option<cpal::Stream>,
    #[cfg(feature = "desktop")]
    receiver: Option<HeapCons<f32>>,
}

impl AudioInput {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "desktop")]
        {
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .context("No input device available")?;

            tracing::info!("Using input device: {}", device.name()?);

            let config = device.default_input_config()?;
            // We need 16000Hz for Whisper usually, but let's just capture native for now
            // and assume we might need resampling later if not 16k.

            let ring = HeapRb::<f32>::new(16000 * 10); // 10 seconds buffer
            let (mut producer, consumer) = ring.split();

            let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        let _ = producer.push_slice(data);
                    },
                    err_fn,
                    None,
                )?,
                _ => return Err(anyhow::anyhow!("Unsupported sample format")),
            };

            stream.play()?;

            Ok(Self {
                stream: Some(stream),
                receiver: Some(consumer),
            })
        }
        #[cfg(not(feature = "desktop"))]
        {
            Ok(Self {})
        }
    }

    pub fn read_chunk(&mut self) -> Result<Vec<f32>> {
        #[cfg(feature = "desktop")]
        {
            if let Some(consumer) = &mut self.receiver {
                let mut chunk = Vec::new();
                while let Some(v) = consumer.try_pop() {
                    chunk.push(v);
                }
                Ok(chunk)
            } else {
                Ok(vec![])
            }
        }
        #[cfg(not(feature = "desktop"))]
        {
            Ok(vec![])
        }
    }
}
