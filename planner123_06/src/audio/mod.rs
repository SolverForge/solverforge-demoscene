pub mod song;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use song::SAMPLE_RATE;

/// The rendered track plus a shared playhead (source frames, f64 bits).
/// In windowed mode the playhead is the demo's master clock.
#[derive(Clone)]
pub struct Track {
    samples: Arc<Vec<[f32; 2]>>,
    playhead: Arc<AtomicU64>,
}

impl Track {
    pub fn render() -> Self {
        Self {
            samples: Arc::new(song::render()),
            playhead: Arc::new(AtomicU64::new(0f64.to_bits())),
        }
    }

    pub fn time(&self) -> f32 {
        (f64::from_bits(self.playhead.load(Ordering::Relaxed)) / SAMPLE_RATE as f64) as f32
    }

    pub fn seek(&self, seconds: f32) {
        let pos = seconds.max(0.0) as f64 * SAMPLE_RATE as f64;
        self.playhead.store(pos.to_bits(), Ordering::Relaxed);
    }

    /// Start live playback. None when there is no usable output device.
    pub fn play(&self) -> Option<cpal::Stream> {
        let device = cpal::default_host().default_output_device()?;
        let config = device
            .default_output_config()
            .map_err(|e| eprintln!("[audio] no output config: {e} -- running silent"))
            .ok()?;
        let device_rate = config.sample_rate() as f64;
        let channels = config.channels() as usize;
        let step = SAMPLE_RATE as f64 / device_rate;
        let samples = self.samples.clone();
        let playhead = self.playhead.clone();
        eprintln!("[audio] playing at {device_rate}Hz");

        let stream = device
            .build_output_stream(
                &config.into(),
                move |data: &mut [f32], _| {
                    let len = samples.len() as f64;
                    let start = playhead.load(Ordering::Relaxed);
                    let mut pos = f64::from_bits(start);
                    for frame in data.chunks_mut(channels) {
                        let i = pos as usize;
                        let frac = (pos - i as f64) as f32;
                        let a = samples[i % samples.len()];
                        let b = samples[(i + 1) % samples.len()];
                        let lr = [a[0] + (b[0] - a[0]) * frac, a[1] + (b[1] - a[1]) * frac];
                        for (c, out) in frame.iter_mut().enumerate() {
                            *out = lr[c.min(1)];
                        }
                        pos += step;
                        if pos >= len {
                            pos -= len;
                        }
                    }
                    // A seek that landed while this buffer was filling wins.
                    let _ = playhead.compare_exchange(
                        start,
                        pos.to_bits(),
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    );
                },
                |e| eprintln!("[audio] stream error: {e}"),
                None,
            )
            .map_err(|e| eprintln!("[audio] could not build stream: {e} -- running silent"))
            .ok()?;
        stream
            .play()
            .map_err(|e| eprintln!("[audio] could not play: {e} -- running silent"))
            .ok()?;
        Some(stream)
    }

    pub fn write_wav(&self, seconds: f32, path: &str) {
        let frames =
            ((seconds.max(0.0) * SAMPLE_RATE as f32).ceil() as usize).min(self.samples.len());
        let data_size = (frames * 4) as u32;
        let mut wav = Vec::with_capacity(44 + data_size as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_size).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
        wav.extend_from_slice(&(SAMPLE_RATE * 4).to_le_bytes());
        wav.extend_from_slice(&4u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        for [l, r] in &self.samples[..frames] {
            wav.extend_from_slice(&((l.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
            wav.extend_from_slice(&((r.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
        if let Some(dir) = Path::new(path)
            .parent()
            .filter(|d| !d.as_os_str().is_empty())
        {
            fs::create_dir_all(dir).expect("create audio output dir");
        }
        fs::write(path, wav).expect("write audio WAV");
        eprintln!("[render] audio -> {path}");
    }
}
