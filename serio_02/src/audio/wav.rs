// ═══════════════════════════════════════════════════════════════
// WAV -- the headless render's soundtrack, synthesized in lockstep with
// the frames so the audio plays the notes that are on screen.
// ═══════════════════════════════════════════════════════════════

use fundsp::prelude32::AudioUnit;

pub const SAMPLE_RATE: u32 = 44100;

/// Collects stereo samples, one video frame at a time.
pub struct Recorder {
    synth: Box<dyn AudioUnit>,
    pcm: Vec<i16>,
    frames: u64,
    fps: u64,
}

impl Recorder {
    pub fn new(mut synth: Box<dyn AudioUnit>, fps: u64) -> Self {
        synth.reset();
        synth.set_sample_rate(SAMPLE_RATE as f64);
        Self {
            synth,
            pcm: Vec::new(),
            frames: 0,
            fps,
        }
    }

    /// Synthesize the samples that cover the next video frame.
    pub fn frame(&mut self) {
        self.frames += 1;
        let target = self.frames * SAMPLE_RATE as u64 / self.fps;
        while ((self.pcm.len() / 2) as u64) < target {
            let (l, r) = self.synth.get_stereo();
            self.pcm.push((l.clamp(-1.0, 1.0) * 32767.0) as i16);
            self.pcm.push((r.clamp(-1.0, 1.0) * 32767.0) as i16);
        }
    }

    /// Write 16-bit stereo PCM WAV.
    pub fn write(&self, path: &str) {
        let data_size = self.pcm.len() as u32 * 2;
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
        for s in &self.pcm {
            wav.extend_from_slice(&s.to_le_bytes());
        }
        std::fs::write(path, &wav).expect("Failed to write audio WAV");
        eprintln!(
            "[render] Audio written: {path} ({:.1} s)",
            self.pcm.len() as f64 / 2.0 / SAMPLE_RATE as f64
        );
    }
}
