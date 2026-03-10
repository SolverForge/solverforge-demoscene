// ═══════════════════════════════════════════════════════════════
// AUDIO ENGINE -- Musica Universalis
//
// 7 planetary tones, each a sine at the planet's current voice
// frequency. As the solver converges, the dissonant cluster
// resolves into a Pythagorean chord. The audience hears the
// score improving.
//
// Layers:
//   - 7 planetary tones (live frequency, morphing with solver)
//   - Slow celestial pad (sustained root chord, very quiet)
//   - Schroeder reverb (long tail, ethereal)
//   - Subtle rhythmic pulse on beat
// ═══════════════════════════════════════════════════════════════

use std::sync::{Arc, Mutex};

use fundsp::prelude32::*;

/// Shared state: current planetary frequencies (updated by main thread)
pub type FreqState = Arc<Mutex<[f32; 7]>>;

pub fn initial_freq_state() -> FreqState {
    // Start with the initial (dissonant) frequencies
    Arc::new(Mutex::new([
        116.54, // Luna:    A#2 (initial freq_idx = 1)
        146.83, // Mercury: D3  (initial freq_idx = 5)
        185.00, // Venus:   F#3 (initial freq_idx = 9)
        261.63, // Earth:   C4  (initial freq_idx = 14)
        311.13, // Mars:    D#4 (initial freq_idx = 17)
        369.99, // Jupiter: F#4 (initial freq_idx = 21)
        415.30, // Saturn:  G#4 (initial freq_idx = 23)
    ]))
}

/// Build the audio graph. Returns a stereo Box<dyn AudioUnit>.
pub fn build_audio(freq_state: FreqState) -> Box<dyn AudioUnit> {
    let fs0 = freq_state.clone();
    let fs1 = freq_state.clone();
    let fs2 = freq_state.clone();
    let fs3 = freq_state.clone();
    let fs4 = freq_state.clone();
    let fs5 = freq_state.clone();
    let fs6 = freq_state.clone();

    // Planet 0 (Luna) -- lightest, highest
    let tone0 = {
        let f = fs0;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[0]);
        let vol_lfo = lfo(|t| (t * 0.37 + 0.1).sin() * 0.08 + 0.18_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    // Planet 1 (Mercury)
    let tone1 = {
        let f = fs1;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[1]);
        let vol_lfo = lfo(|t| (t * 0.41 + 0.5).sin() * 0.08 + 0.18_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    // Planet 2 (Venus)
    let tone2 = {
        let f = fs2;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[2]);
        let vol_lfo = lfo(|t| (t * 0.29 + 1.2).sin() * 0.07 + 0.17_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    // Planet 3 (Earth)
    let tone3 = {
        let f = fs3;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[3]);
        let vol_lfo = lfo(|t| (t * 0.33 + 2.1).sin() * 0.08 + 0.19_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    // Planet 4 (Mars)
    let tone4 = {
        let f = fs4;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[4]);
        let vol_lfo = lfo(|t| (t * 0.27 + 0.7).sin() * 0.07 + 0.18_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    // Planet 5 (Jupiter)
    let tone5 = {
        let f = fs5;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[5]);
        let vol_lfo = lfo(|t| (t * 0.23 + 1.8).sin() * 0.09 + 0.22_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    // Planet 6 (Saturn)
    let tone6 = {
        let f = fs6;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[6]);
        let vol_lfo = lfo(|t| (t * 0.19 + 0.3).sin() * 0.08 + 0.20_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    let tones = tone0 + tone1 + tone2 + tone3 + tone4 + tone5 + tone6;

    // Celestial pad (sustained root + fifth)
    let pad = {
        let root = lfo(|t| 110.0_f32 + (t * 0.05).sin() * 1.5) >> saw();
        let fifth = lfo(|t| 165.0_f32 + (t * 0.07 + 1.0).sin() * 2.0) >> saw();
        (root + fifth) * 0.06
    };

    // Gentle rhythmic pulse
    let pulse = {
        let env = lfo(|t| {
            let phase = (t * 0.25) % 1.0;
            (phase * std::f32::consts::TAU).sin().max(0.0) * 0.05_f32
        });
        sine_hz(55.0) * env
    };

    let mono = tones * 0.6 + pad + pulse;

    // Plate reverb
    let reverb = {
        let dry_wet = 0.35_f32;
        let delay1 = mono.clone() >> delay(0.043);
        let delay2 = mono.clone() >> delay(0.079);
        let delay3 = mono.clone() >> delay(0.127);
        let delay4 = mono.clone() >> delay(0.191);
        let rev_mix = (delay1 + delay2 + delay3 + delay4) * (dry_wet * 0.25);
        mono.clone() * (1.0 - dry_wet) + rev_mix
    };

    // Soft limiter
    let master = reverb
        >> shape_fn(|x| {
            let x = x * 0.9;
            let s = x.signum();
            let a = x.abs();
            s * if a < 0.7 {
                a
            } else {
                0.7 + (1.0 - (-(a - 0.7) * 3.0).exp()) * 0.15
            }
        });

    let stereo = master >> split::<U2>();
    Box::new(stereo)
}

/// Update planetary frequencies from current solver state.
/// Called from main thread each frame.
pub fn update_frequencies(freq_state: &FreqState, planet_freqs: &[f64; 7]) {
    if let Ok(mut freqs) = freq_state.lock() {
        for (i, &f) in planet_freqs.iter().enumerate() {
            let f = f.clamp(55.0, 880.0) as f32;
            // Smooth toward target (exponential glide -- portamento)
            freqs[i] = freqs[i] * 0.97 + f * 0.03;
        }
    }
}
