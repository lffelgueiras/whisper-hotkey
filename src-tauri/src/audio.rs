use crate::error::AppError;
use parking_lot::Mutex;
use std::sync::Arc;

pub fn resample_to_16k_mono(samples: &[f32], from_hz: u32, channels: u16) -> Vec<f32> {
    let mono: Vec<f32> = if channels == 1 {
        samples.to_vec()
    } else {
        samples
            .chunks_exact(channels as usize)
            .map(|c| c.iter().sum::<f32>() / channels as f32)
            .collect()
    };
    if from_hz == 16_000 {
        return mono;
    }
    let ratio = 16_000f64 / from_hz as f64;
    let out_len = ((mono.len() as f64) * ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 / ratio;
        let idx = src.floor() as usize;
        let frac = src - idx as f64;
        let a = mono.get(idx).copied().unwrap_or(0.0);
        let b = mono.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac as f32);
    }
    out
}

pub struct AudioCapturer {
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Mutex<Option<cpal::Stream>>,
    sample_rate: Arc<Mutex<u32>>,
    channels: Arc<Mutex<u16>>,
}

// SAFETY: cpal::Stream contains platform handles that are not auto-Send/Sync,
// but in our design start()/stop() are called from a single tokio task and the
// stream's callback runs on cpal's dedicated audio thread. We never share the
// stream across threads concurrently.
unsafe impl Send for AudioCapturer {}
unsafe impl Sync for AudioCapturer {}

impl AudioCapturer {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: Mutex::new(None),
            sample_rate: Arc::new(Mutex::new(16_000)),
            channels: Arc::new(Mutex::new(1)),
        }
    }

    pub fn start(&self) -> Result<(), AppError> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::Mic("no default input device".into()))?;
        let config = device
            .default_input_config()
            .map_err(|e| AppError::Mic(format!("default config: {e}")))?;

        *self.sample_rate.lock() = config.sample_rate().0;
        *self.channels.lock() = config.channels();
        self.buffer.lock().clear();

        let buf = self.buffer.clone();
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| buf.lock().extend_from_slice(data),
                move |err| tracing::error!("audio stream err: {err}"),
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    let mut g = buf.lock();
                    for &s in data {
                        g.push(s as f32 / i16::MAX as f32);
                    }
                },
                move |err| tracing::error!("audio stream err: {err}"),
                None,
            ),
            other => {
                return Err(AppError::Mic(format!(
                    "unsupported sample format: {other:?}"
                )))
            }
        }
        .map_err(|e| AppError::Mic(format!("build stream: {e}")))?;
        stream
            .play()
            .map_err(|e| AppError::Mic(format!("play: {e}")))?;
        *self.stream.lock() = Some(stream);
        Ok(())
    }

    pub fn stop(&self) -> Result<Vec<f32>, AppError> {
        drop(self.stream.lock().take());
        let raw = std::mem::take(&mut *self.buffer.lock());
        let hz = *self.sample_rate.lock();
        let ch = *self.channels.lock();
        Ok(resample_to_16k_mono(&raw, hz, ch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_when_already_16k_mono() {
        let s = vec![0.1, 0.2, 0.3, 0.4];
        let out = resample_to_16k_mono(&s, 16_000, 1);
        assert_eq!(out, s);
    }

    #[test]
    fn downsamples_48k_mono_to_16k() {
        let s: Vec<f32> = (0..480).map(|i| (i as f32) / 480.0).collect();
        let out = resample_to_16k_mono(&s, 48_000, 1);
        assert!(out.len() >= 158 && out.len() <= 162, "got {}", out.len());
        for w in out.windows(2) {
            assert!(w[1] >= w[0] - 0.01);
        }
    }

    #[test]
    fn mixes_stereo_to_mono() {
        let s = vec![1.0, -1.0, 0.5, -0.5];
        let out = resample_to_16k_mono(&s, 16_000, 2);
        assert_eq!(out, vec![0.0, 0.0]);
    }
}
