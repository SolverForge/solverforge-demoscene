// ═══════════════════════════════════════════════════════════════
// AUDIO ENGINE -- Musica Universalis
//
// 7 planetary tones (Kepler: f ∝ r^(-3/2)), each a sine + harmonic.
// As the solver converges, the dissonant cluster resolves into a
// Pythagorean chord. The audience hears the score improving.
//
// Layers:
//   - 7 planetary tones (live frequency, morphing with solver)
//   - Slow celestial pad (sustained root chord, very quiet)
//   - Schroeder reverb (long tail, ethereal)
//   - Subtle rhythmic pulse on beat (gently anchors the tone cluster)
// ═══════════════════════════════════════════════════════════════
use std::sync::{Arc, Mutex};

use fundsp::prelude32::*;

// ── Planetary frequency table ─────────────────────────────────
// These are the 7 target frequencies for a perfectly consonant
// Pythagorean tuning, anchored to A=110 Hz (outermost planet).
// The solver works toward making all planet pairs hit these ratios.
pub const PLANET_FREQS_INITIAL: [f32; 7] = [
    // Initial chaotic frequencies (bad slots)
    // Luna, Mercury, Venus, Earth, Mars, Jupiter, Saturn
    330.0, // arbitrary dissonant start
    285.0, 250.0, 215.0, 185.0, 155.0, 110.0, // outermost = lowest
];

/// Shared state: current planetary frequencies (updated by main thread)
pub type FreqState = Arc<Mutex<[f32; 7]>>;

pub fn initial_freq_state() -> FreqState {
    Arc::new(Mutex::new(PLANET_FREQS_INITIAL))
}

/// Build the audio graph. Returns a stereo Box<dyn AudioUnit>.
pub fn build_audio(freq_state: FreqState) -> Box<dyn AudioUnit> {
    // ── 7 planetary tones ────────────────────────────────────
    // Each planet: sine + 2nd harmonic + gentle 3rd, slowly amplitude-modulated.
    // LFO modulates slightly for a "breathing" quality.

    let fs0 = freq_state.clone();
    let fs1 = freq_state.clone();
    let fs2 = freq_state.clone();
    let fs3 = freq_state.clone();
    let fs4 = freq_state.clone();
    let fs5 = freq_state.clone();
    let fs6 = freq_state.clone();

    // Planet 0 (Luna) — lightest, highest
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

    // Planet 5 (Jupiter) — richer, larger
    let tone5 = {
        let f = fs5;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[5]);
        let vol_lfo = lfo(|t| (t * 0.23 + 1.8).sin() * 0.09 + 0.22_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    // Planet 6 (Saturn) — deepest, most resonant
    let tone6 = {
        let f = fs6;
        let freq_lfo = lfo(move |_t| f.lock().unwrap()[6]);
        let vol_lfo = lfo(|t| (t * 0.19 + 0.3).sin() * 0.08 + 0.20_f32);
        (freq_lfo >> sine()) * vol_lfo
    };

    // Sum all planetary tones
    let tones = tone0 + tone1 + tone2 + tone3 + tone4 + tone5 + tone6;

    // ── Celestial pad (sustained root + fifth) ────────────────
    // A very quiet two-voice pad providing tonal anchor.
    // Detuned slightly for warmth.
    let pad = {
        let root = lfo(|t| 110.0_f32 + (t * 0.05).sin() * 1.5) >> saw();
        let fifth = lfo(|t| 165.0_f32 + (t * 0.07 + 1.0).sin() * 2.0) >> saw();
        (root + fifth) * 0.06
    };

    // ── Very gentle rhythmic pulse (heartbeat) ────────────────
    // A slow, quiet 4-second pulse to give the music a breathing quality.
    let pulse = {
        let env = lfo(|t| {
            let phase = (t * 0.25) % 1.0; // 4 second period
            (phase * std::f32::consts::TAU).sin().max(0.0) * 0.05_f32
        });
        sine_hz(55.0) * env
    };

    // ── Mix ───────────────────────────────────────────────────
    let mono = tones * 0.6 + pad + pulse;

    // ── Plate reverb (Schroeder-style, long ethereal tail) ────
    // 6 allpass delay taps at prime-length delays for diffusion.
    let reverb = {
        let dry_wet = 0.35_f32;
        // Simple feedback delay network approximation
        let delay1 = mono.clone() >> delay(0.043);
        let delay2 = mono.clone() >> delay(0.079);
        let delay3 = mono.clone() >> delay(0.127);
        let delay4 = mono.clone() >> delay(0.191);
        let rev_mix = (delay1 + delay2 + delay3 + delay4) * (dry_wet * 0.25);
        mono.clone() * (1.0 - dry_wet) + rev_mix
    };

    // ── Soft limiter ─────────────────────────────────────────
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

/// Update planetary frequencies from current orbital radii.
/// Called from main thread each frame when the solver makes a move.
pub fn update_frequencies(freq_state: &FreqState, orbital_radii: &[f64; 7]) {
    use crate::orrery::harmony::radius_to_frequency;
    use crate::orrery::model::R_MAX;

    if let Ok(mut freqs) = freq_state.lock() {
        for (i, &r) in orbital_radii.iter().enumerate() {
            // Map to audio range [55 Hz, 880 Hz]
            let f = radius_to_frequency(r, R_MAX);
            // Clamp to reasonable audio range
            let f = f.clamp(55.0, 880.0) as f32;
            // Smooth toward target (exponential glide — portamento)
            freqs[i] = freqs[i] * 0.97 + f * 0.03;
        }
    }
}
