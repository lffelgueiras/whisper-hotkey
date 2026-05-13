/// In-process tone playback for recording cues.
///
/// Background — why this is a persistent stream, not on-demand playback:
///
/// Every prior approach (`afplay` subprocess, `NSSound`, Web Audio in the
/// overlay webview) had to *open the macOS audio output route on demand*.
/// Right after the start cue is requested, `cpal` opens the input device
/// for recording. CoreAudio renegotiates the default device's routing on
/// input open, and the still-uninitialized output stream feeding the cue
/// loses the race — sometimes the beep plays, sometimes it gets clipped
/// to nothing. The stop cue never has this problem because by then the
/// output route is settled.
///
/// Fix: hold an output stream open for the lifetime of the app. Beeps are
/// just samples pushed into a ring buffer that the audio callback drains.
/// No stream-open handshake on the hot path, no race.
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct ToneOutput {
    pending: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: u32,
    _stream: cpal::Stream,
}

// SAFETY: cpal::Stream's platform handles aren't auto-Send/Sync, but we
// never touch the stream concurrently — the callback runs on cpal's audio
// thread, and `pending` is the only shared state (already a Mutex).
unsafe impl Send for ToneOutput {}
unsafe impl Sync for ToneOutput {}

impl ToneOutput {
    pub fn new() -> Option<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let supported = device.default_output_config().ok()?;
        let sample_rate = supported.sample_rate().0;
        let channels = supported.channels() as usize;
        let sample_format = supported.sample_format();
        let cfg: cpal::StreamConfig = supported.into();

        let pending: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));
        let pending_cb = pending.clone();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &cfg,
                move |data: &mut [f32], _| {
                    let mut buf = pending_cb.lock();
                    for frame in data.chunks_mut(channels) {
                        let s = buf.pop_front().unwrap_or(0.0);
                        for ch in frame.iter_mut() {
                            *ch = s;
                        }
                    }
                },
                |e| tracing::warn!("tone output err: {e}"),
                None,
            ),
            cpal::SampleFormat::I16 => device.build_output_stream(
                &cfg,
                move |data: &mut [i16], _| {
                    let mut buf = pending_cb.lock();
                    for frame in data.chunks_mut(channels) {
                        let s = buf.pop_front().unwrap_or(0.0);
                        let si = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        for ch in frame.iter_mut() {
                            *ch = si;
                        }
                    }
                },
                |e| tracing::warn!("tone output err: {e}"),
                None,
            ),
            cpal::SampleFormat::U16 => device.build_output_stream(
                &cfg,
                move |data: &mut [u16], _| {
                    let mut buf = pending_cb.lock();
                    for frame in data.chunks_mut(channels) {
                        let s = buf.pop_front().unwrap_or(0.0);
                        let su = ((s.clamp(-1.0, 1.0) * 0.5 + 0.5) * u16::MAX as f32) as u16;
                        for ch in frame.iter_mut() {
                            *ch = su;
                        }
                    }
                },
                |e| tracing::warn!("tone output err: {e}"),
                None,
            ),
            other => {
                tracing::warn!("tone output: unsupported sample format {other:?}");
                return None;
            }
        }
        .map_err(|e| tracing::warn!("tone output build_output_stream: {e}"))
        .ok()?;

        stream
            .play()
            .map_err(|e| tracing::warn!("tone output play: {e}"))
            .ok()?;

        tracing::info!(
            "tone output stream opened: sr={sample_rate} ch={channels} fmt={sample_format:?}"
        );

        Some(Self {
            pending,
            sample_rate,
            _stream: stream,
        })
    }

    pub fn play_start(&self) {
        self.push(sweep(self.sample_rate, 660.0, 990.0, 0.14));
    }

    pub fn play_stop(&self) {
        self.push(sweep(self.sample_rate, 660.0, 330.0, 0.18));
    }

    fn push(&self, samples: Vec<f32>) {
        let mut buf = self.pending.lock();
        // Bound the queue so a stuck audio thread can't grow it unboundedly.
        if buf.len() > self.sample_rate as usize {
            buf.clear();
        }
        buf.extend(samples);
    }
}

/// Generate an exponential-sweep sine with a short attack/release envelope.
fn sweep(sr: u32, f_start: f32, f_end: f32, dur_s: f32) -> Vec<f32> {
    let n = (sr as f32 * dur_s) as usize;
    let mut out = Vec::with_capacity(n);
    let sr_f = sr as f32;
    let mut phase = 0.0f32;
    let attack = 0.04;
    let release_start = 0.7;
    for i in 0..n {
        let t = i as f32 / n as f32;
        let f = f_start * (f_end / f_start).powf(t);
        phase += 2.0 * std::f32::consts::PI * f / sr_f;
        if phase > 2.0 * std::f32::consts::PI {
            phase -= 2.0 * std::f32::consts::PI;
        }
        let env = if t < attack {
            t / attack
        } else if t > release_start {
            ((1.0 - t) / (1.0 - release_start)).max(0.0)
        } else {
            1.0
        };
        out.push(0.18 * env * phase.sin());
    }
    out
}
