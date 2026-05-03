pub mod synth;

use std::fs;
use std::path::Path;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub use synth::AudioClock;

pub fn start_audio(clock: AudioClock) -> Option<cpal::Stream> {
    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(device) => device,
        None => {
            eprintln!("[audio] No output device found -- running silent");
            return None;
        }
    };

    let config = match device.default_output_config() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("[audio] Could not get output config: {error} -- running silent");
            return None;
        }
    };

    let sample_rate = config.sample_rate() as f32;
    let channels = config.channels() as usize;
    eprintln!("[audio] Starting route-splice track at {}Hz", sample_rate);

    let mut synth = synth::RouteSynth::new(sample_rate);
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
            let base_time = clock.time();
            let frames = data.len() / channels;
            for frame in 0..frames {
                let t = base_time + frame as f32 / sample_rate;
                let (l, r) = synth.sample(t);
                for channel in 0..channels {
                    data[frame * channels + channel] = if channel == 0 { l } else { r };
                }
            }
        },
        |error| eprintln!("[audio] Stream error: {error}"),
        None,
    );

    match stream {
        Ok(stream) => {
            if let Err(error) = stream.play() {
                eprintln!("[audio] Could not play stream: {error} -- running silent");
                None
            } else {
                Some(stream)
            }
        }
        Err(error) => {
            eprintln!("[audio] Could not build stream: {error} -- running silent");
            None
        }
    }
}

pub fn render_audio_wav(duration: f32, path: &str) {
    const SAMPLE_RATE: u32 = 44_100;
    let total_samples = (duration.max(0.0) * SAMPLE_RATE as f32).ceil() as usize;
    eprintln!("[render] Rendering audio: {total_samples} samples @ {SAMPLE_RATE}Hz -> {path}");

    let mut synth = synth::RouteSynth::new(SAMPLE_RATE as f32);
    let mut pcm = Vec::with_capacity(total_samples * 2);
    for sample_idx in 0..total_samples {
        let t = sample_idx as f32 / SAMPLE_RATE as f32;
        let (l, r) = synth.sample(t);
        pcm.push((l.clamp(-1.0, 1.0) * 32767.0) as i16);
        pcm.push((r.clamp(-1.0, 1.0) * 32767.0) as i16);
    }

    let channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let byte_rate = SAMPLE_RATE * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;
    let data_size = pcm.len() as u32 * 2;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for sample in pcm {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    if let Some(parent) = Path::new(path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).expect("failed to create audio output directory");
    }
    fs::write(path, &wav).expect("failed to write generated audio");
    eprintln!(
        "[render] Audio written: {path} ({:.1} MB)",
        wav.len() as f64 / 1_048_576.0
    );
}
