// ═══════════════════════════════════════════════════════════════
// AUDIO ENGINE -- Four Phases. One Thread.
// ═══════════════════════════════════════════════════════════════
#![allow(dead_code)]

use fundsp::prelude32::*;

// ── Timing ───────────────────────────────────────────────────
pub const BPM: f32 = 125.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const STEP: f32 = BEAT / 4.0;

// ── Phase boundaries ────────────────────────────────────────
const PHASE2_START: f32 = 0.0;
const PHASE2_FULL: f32 = 0.0;
const BRIDGE_START: f32 = 8.0; // bridge tension begins
const BRIDGE_FULL: f32 = 12.0; // bridge at max intensity
const PHASE3_START: f32 = 12.0; // solo enters
const PHASE3_FULL: f32 = 14.0; // solo at full volume
const PHASE3_PEAK: f32 = 40.0; // maximum intensity

const BASS_NOTES: [f32; 64] = [
    // C section
    130.81, 130.81, 130.81, 130.81, 130.81, 130.81, 130.81, 130.81, 130.81, 130.81, 130.81, 130.81,
    130.81, 130.81, 130.81, 130.81, // Ab section
    103.83, 103.83, 103.83, 103.83, 103.83, 103.83, 103.83, 103.83, 103.83, 103.83, 103.83, 103.83,
    103.83, 103.83, 103.83, 103.83, // Bb section
    116.54, 116.54, 116.54, 116.54, 116.54, 116.54, 116.54, 116.54, 116.54, 116.54, 116.54, 116.54,
    116.54, 116.54, 116.54, 116.54, // G section + Bb turnaround
    98.00, 98.00, 98.00, 98.00, 98.00, 98.00, 98.00, 98.00, 98.00, 98.00, 98.00, 98.00, 116.54,
    116.54, 116.54, 116.54,
];

const BASS_GATE: [f32; 16] = [
    1.0, 0.6, 1.0, 0.6, 1.0, 0.6, 1.0, 0.6, 1.0, 0.6, 1.0, 0.6, 1.0, 0.6, 1.0, 0.6,
];

const LEAD_NOTES: [f32; 64] = [
    // C section
    523.25, 0.0, 466.16, 0.0, 523.25, 0.0, 523.25, 0.0, 466.16, 0.0, 523.25, 0.0, 523.25, 0.0,
    466.16, 523.25, // Ab section
    415.30, 0.0, 415.30, 0.0, 392.00, 0.0, 415.30, 0.0, 415.30, 0.0, 415.30, 0.0, 392.00, 0.0,
    415.30, 0.0, // Bb section
    466.16, 0.0, 466.16, 0.0, 415.30, 0.0, 466.16, 0.0, 466.16, 0.0, 466.16, 0.0, 415.30, 0.0,
    466.16, 0.0, // G section
    392.00, 0.0, 392.00, 0.0, 349.23, 0.0, 392.00, 0.0, 392.00, 0.0, 466.16, 0.0, 466.16, 0.0,
    466.16, 523.25,
];

const ARP_NOTES: [f32; 64] = [
    // C arps
    523.25, 392.00, 261.63, 196.00, 523.25, 392.00, 261.63, 196.00, 523.25, 392.00, 261.63, 196.00,
    523.25, 392.00, 261.63, 196.00, // Ab arps
    415.30, 311.13, 207.65, 155.56, 415.30, 311.13, 207.65, 155.56, 415.30, 311.13, 207.65, 155.56,
    415.30, 311.13, 207.65, 155.56, // Bb arps
    466.16, 349.23, 233.08, 174.61, 466.16, 349.23, 233.08, 174.61, 466.16, 349.23, 233.08, 174.61,
    466.16, 349.23, 233.08, 174.61, // G arps + Bb turnaround
    392.00, 293.66, 196.00, 146.83, 392.00, 293.66, 196.00, 146.83, 392.00, 293.66, 196.00, 146.83,
    466.16, 349.23, 233.08, 174.61,
];

const ARP_VOL: [f32; 4] = [1.0, 0.6, 0.35, 0.15];

const PAD_CHORDS: [(f32, f32, f32); 4] = [
    (130.81, 196.00, 261.63),
    (103.83, 155.56, 207.65),
    (116.54, 174.61, 233.08),
    (98.00, 146.83, 196.00),
];

const SOLO_MELODY: [(f32, f32); 101] = [
    // Phrase 1 (over C)
    (349.20, 0.5),
    (261.60, 0.75),
    (293.70, 0.5),
    (261.60, 0.25),
    (311.10, 0.5),
    (293.70, 0.25),
    (392.00, 0.25),
    (523.30, 2.0),
    (293.70, 0.25),
    (311.10, 0.75),
    (293.70, 0.75),
    (233.10, 0.75),
    (196.00, 1.0),
    (261.60, 0.75),
    (293.70, 0.5),
    (261.60, 0.25),
    (311.10, 0.5),
    (293.70, 0.25),
    (392.00, 0.25),
    (523.30, 2.0),
    (622.30, 0.25),
    (466.20, 0.75),
    (392.00, 0.5),
    (523.30, 0.25),
    (349.20, 0.75),
    (311.10, 0.5),
    // Phrase 2 (over Ab)
    (349.20, 0.5),
    (311.10, 0.75),
    (311.10, 0.5),
    (261.60, 0.25),
    (349.20, 0.5),
    (311.10, 0.25),
    (415.30, 0.25),
    (523.30, 2.0),
    (311.10, 0.25),
    (349.20, 0.75),
    (311.10, 0.75),
    (261.60, 0.75),
    (207.65, 1.0),
    (311.10, 0.75),
    (311.10, 0.5),
    (261.60, 0.25),
    (349.20, 0.5),
    (311.10, 0.25),
    (415.30, 0.25),
    (523.30, 2.0),
    (622.30, 0.25),
    (523.30, 0.75),
    (415.30, 0.5),
    (466.20, 0.25),
    (415.30, 0.75),
    (392.00, 0.5),
    // Phrase 3 (over Bb)
    (349.20, 0.5),
    (293.70, 0.75),
    (311.10, 0.5),
    (293.70, 0.25),
    (349.20, 0.5),
    (311.10, 0.25),
    (466.20, 0.25),
    (523.30, 2.0),
    (311.10, 0.25),
    (349.20, 0.75),
    (311.10, 0.75),
    (261.60, 0.75),
    (233.10, 1.0),
    (293.70, 0.75),
    (311.10, 0.5),
    (293.70, 0.25),
    (349.20, 0.5),
    (311.10, 0.25),
    (466.20, 0.25),
    (523.30, 2.0),
    (698.50, 0.25),
    (523.30, 0.75),
    (466.20, 0.5),
    (587.30, 0.25),
    (415.30, 0.75),
    (392.00, 0.5),
    // Phrase 4 (over G)
    (311.10, 0.5),
    (261.60, 0.75),
    (293.70, 0.5),
    (261.60, 0.25),
    (311.10, 0.5),
    (293.70, 0.25),
    (392.00, 0.25),
    (523.30, 2.0),
    (293.70, 0.25),
    (311.10, 0.75),
    (293.70, 0.75),
    (233.10, 0.75),
    (196.00, 1.0),
    (261.60, 0.75),
    (293.70, 0.5),
    (261.60, 0.25),
    (311.10, 0.5),
    (293.70, 0.25),
    (392.00, 0.25),
    (523.30, 2.0),
    (622.30, 0.25),
    (466.20, 0.75),
    (523.30, 2.0),
];

const SOLO_BEAT: f32 = 60.0 / BPM;

const fn solo_total_beats() -> f32 {
    let mut total = 0.0f32;
    let mut i = 0;
    while i < SOLO_MELODY.len() {
        total += SOLO_MELODY[i].1;
        i += 1;
    }
    total
}
const SOLO_TOTAL: f32 = solo_total_beats() * SOLO_BEAT;

/// Helper: smooth crossfade ramp
fn ramp(t: f32, start: f32, full: f32) -> f32 {
    if t < start {
        0.0
    } else if t < full {
        (t - start) / (full - start)
    } else {
        1.0
    }
}

/// Helper: look up current note in a melody table
fn melody_lookup(melody: &[(f32, f32)], beat_dur: f32, loop_time: f32) -> (f32, f32, f32) {
    // Returns (freq, note_phase 0..1, dur_beats)
    let mut cursor = 0.0f32;
    for &(freq, dur_beats) in melody {
        let dur = dur_beats * beat_dur;
        if loop_time < cursor + dur {
            let phase = (loop_time - cursor) / dur;
            return (freq, phase, dur_beats);
        }
        cursor += dur;
    }
    (0.0, 0.0, 1.0)
}

pub fn build_synth(_sample_rate: f32) -> Box<dyn AudioUnit> {
    // ════════════════════════════════════════════════════════
    // DRUMS -- Always on. The spine.
    // ════════════════════════════════════════════════════════

    // Helper: kick volume for a given global time (rhythm pattern)
    fn kick_vol(t: f32) -> f32 {
        let bar4 = BAR * 4.0;
        let step = ((t % bar4) / STEP) as usize;
        let in_bridge = t > BRIDGE_START && t < PHASE3_START;
        match step % 64 {
            // Every beat = kick (four-on-the-floor)
            0 | 4 | 8 | 12 | 16 | 20 | 24 | 28 | 32 | 36 | 40 | 44 | 48 | 52 | 56 | 60 => 1.0,
            // Offbeat 16ths for drive
            2 | 6 | 10 | 14 | 18 | 22 | 26 | 30 => 0.45,
            // Ghost notes for shuffle
            11 | 27 | 43 | 59 => 0.3,
            // Second half offbeat fills
            34 | 38 | 42 | 46 | 50 | 54 | 58 | 62 => 0.45,
            // Bridge fills
            33 | 37 | 41 | 45 | 49 | 53 | 57 | 61 if in_bridge => 0.55,
            _ => 0.0,
        }
    }

    let kick = {
        // Pitch sweep re-triggers every step: 180Hz -> 42Hz
        let kick_freq = lfo(move |t| {
            let vol = kick_vol(t);
            if vol < 0.01 {
                return 42.0;
            }
            let p = (t % STEP) / STEP; // per-hit phase 0..1
            180.0 * (-p * 30.0 * STEP).exp() + 42.0 * (1.0 - (-p * 30.0 * STEP).exp())
        });
        // Amp envelope re-triggers every step
        let kick_env = lfo(move |t| {
            let vol = kick_vol(t);
            if vol < 0.01 {
                return 0.0;
            }
            let p = (t % STEP) / STEP;
            let click = (-p * 80.0 * STEP).exp() * 0.4;
            let body = (-p * 7.0 * STEP).exp();
            vol * (click + body) * (-p * 8.0).exp().min(1.0)
        });
        (kick_freq >> sine()) * kick_env * 3.5
    };

    // Helper: snare volume for a given global time (rhythm pattern)
    fn snare_vol(t: f32) -> f32 {
        let bar4 = BAR * 4.0;
        let step = ((t % bar4) / STEP) as usize;
        let in_bridge = t > BRIDGE_START && t < PHASE3_START;
        match step % 64 {
            4 | 12 | 20 | 28 | 36 | 44 | 52 | 60 => 1.0,
            15 | 31 | 47 | 63 => 0.35,
            3 | 19 | 35 | 51 => 0.2,
            59 | 61 | 62 if in_bridge => 0.7,
            _ => 0.0,
        }
    }

    let snare = {
        // All envelopes re-trigger per step
        let snare_env = lfo(move |t| {
            let vol = snare_vol(t);
            if vol < 0.01 {
                return 0.0;
            }
            let p = (t % STEP) / STEP;
            vol * (-p * 14.0).exp().min(1.0)
        });
        let body = sine_hz(200.0) * 0.35 + sine_hz(800.0) * 0.2 + sine_hz(1200.0) * 0.1;
        let crack_hi = noise() >> bandpass_hz(4500.0f32, 0.8f32);
        let crack_lo = noise() >> bandpass_hz(1800.0f32, 1.2f32);
        (body + crack_hi * 0.7 + crack_lo * 0.5) * snare_env * 2.8
    };

    let hat = {
        let hat_rhythm = lfo(move |t| {
            let step = ((t % BAR) / STEP) as usize;
            // Standard industrial hat pattern: accent on downbeats and offbeats (8th notes)
            let vol = match step % 16 {
                0 => 0.9,   // downbeat - accent
                1 => 0.25,  // ghost
                2 => 0.7,   // offbeat 8th
                3 => 0.25,  // ghost
                4 => 0.85,  // beat 2
                5 => 0.25,  // ghost
                6 => 0.7,   // offbeat 8th
                7 => 0.35,  // ghost (slightly louder before beat 3)
                8 => 0.9,   // beat 3 - accent
                9 => 0.25,  // ghost
                10 => 0.7,  // offbeat 8th
                11 => 0.25, // ghost
                12 => 0.85, // beat 4
                13 => 0.25, // ghost
                14 => 0.7,  // offbeat 8th
                15 => 0.4,  // pickup into next bar
                _ => 0.0,
            };
            let phase = (t % STEP) / STEP;
            vol * (-phase * 55.0).exp().min(1.0)
        });
        (noise() >> highpass_hz(8500.0f32, 0.8f32)) * hat_rhythm * 0.6
    };

    let open_hat = {
        let oh_rhythm = lfo(move |t| {
            let bar2 = BAR * 2.0;
            let step = ((t % bar2) / STEP) as usize;
            if step == 9 || step == 25 {
                let phase = (t % STEP) / STEP;
                (-phase * 6.0).exp().min(1.0)
            } else {
                0.0
            }
        });
        (noise() >> highpass_hz(6500.0f32, 0.5f32)) * oh_rhythm * 0.45
    };

    let drums = kick + snare + hat + open_hat;

    // Drum bus: harder saturation for glue and punch
    let drums_saturated = drums
        >> shape_fn(|x| {
            let x = x * 1.3; // drive into saturation harder
            let s = x.signum();
            let a = x.abs();
            s * if a < 0.5 {
                a * 1.1 // slight boost to transients
            } else if a < 0.9 {
                0.55 + (a - 0.5) * 0.35
            } else {
                0.69 + (1.0 - (-((a - 0.9) * 3.0)).exp()) * 0.15
            }
        });

    // ════════════════════════════════════════════════════════
    // BASS + LEAD + ARP: The groove.
    // Plays from Phase 2 through the end. Never fully stops.
    // During bridge: intensity rises. During solo: ducks but
    // keeps pumping underneath.
    // ════════════════════════════════════════════════════════

    let phase2_vol = lfo(move |t| ramp(t, PHASE2_START, PHASE2_FULL));

    // Bridge intensity: ramps up lead/arp volume and adds urgency
    let bridge_intensity = lfo(move |t| ramp(t, BRIDGE_START, BRIDGE_FULL));

    // ── Bass ──────────────────────────────────────────────────
    let bass = {
        let bass_freq = lfo(move |t| {
            let bar4 = BAR * 4.0;
            let step = ((t % bar4) / STEP) as usize;
            BASS_NOTES[step % BASS_NOTES.len()]
        });
        let bass_env = lfo(move |t| {
            let step = ((t % BAR) / STEP) as usize;
            let gate = BASS_GATE[step % BASS_GATE.len()];
            if gate < 0.01 {
                return 0.0;
            }
            let phase = (t % STEP) / STEP;
            let atk = (phase / 0.01).min(1.0);
            let slide = (1.0 - phase * 0.7).max(0.1);
            gate * atk * slide
        });
        let sq = bass_freq.clone() >> square();
        let sub = (bass_freq.clone() * dc(0.5f32)) >> sine();
        let over = (bass_freq * dc(2.0f32)) >> sine();
        (sq * 0.55 + sub * 0.25 + over * 0.1) * bass_env * 1.0
    };

    // ── Lead with echo ────────────────────────────────────────
    let lead = {
        let lead_freq = lfo(move |t| {
            let bar4 = BAR * 4.0;
            let step = ((t % bar4) / STEP) as usize;
            LEAD_NOTES[step % LEAD_NOTES.len()]
        });
        let lead_gate = lfo(move |t| {
            let bar4 = BAR * 4.0;
            let step = ((t % bar4) / STEP) as usize;
            let freq = LEAD_NOTES[step % LEAD_NOTES.len()];
            if freq < 1.0 {
                return 0.0;
            }
            let phase = (t % STEP) / STEP;
            let atk = (phase / 0.02).min(1.0);
            let dec = (1.0 - (phase - 0.05).max(0.0) * 0.4).max(0.15);
            atk * dec
        });
        let carrier = lead_freq.clone() >> saw();
        let modulator = ((lead_freq.clone() * dc(3.0f32)) >> sine()) * 0.35;
        let dry = (carrier + modulator) * lead_gate * 0.45;

        // Echo channel (2 rows delayed)
        let echo_delay = STEP * 2.0;
        let echo_freq = lfo(move |t| {
            let td = (t - echo_delay).max(0.0);
            let bar4 = BAR * 4.0;
            let step = ((td % bar4) / STEP) as usize;
            let f = LEAD_NOTES[step % LEAD_NOTES.len()];
            if f < 1.0 {
                0.0
            } else {
                f * 1.003
            }
        });
        let echo_gate = lfo(move |t| {
            let td = (t - echo_delay).max(0.0);
            let bar4 = BAR * 4.0;
            let step = ((td % bar4) / STEP) as usize;
            let freq = LEAD_NOTES[step % LEAD_NOTES.len()];
            if freq < 1.0 {
                return 0.0;
            }
            let phase = (td % STEP) / STEP;
            let atk = (phase / 0.02).min(1.0);
            let dec = (1.0 - (phase - 0.05).max(0.0) * 0.5).max(0.1);
            atk * dec
        });
        let echo_osc = echo_freq >> saw();
        let echo_ch = echo_osc * echo_gate * 0.25;

        dry + echo_ch
    };

    // ── Arp ───────────────────────────────────────────────────
    let arp = {
        let arp_env = lfo(move |t| {
            let bar4 = BAR * 4.0;
            let step = ((t % bar4) / STEP) as usize;
            let arp_idx = step % 4; // 4-note arp groups: root, 5th, oct, 5th
            let vol = ARP_VOL[arp_idx];
            // Active in second half of each 2-bar group
            let section = (step / 16) % 2;
            if section == 1 {
                let phase = (t % STEP) / STEP;
                let atk = (phase / 0.005).min(1.0);
                let dec = (-phase * 8.0).exp();
                vol * atk * dec
            } else {
                0.0
            }
        });
        let arp_freq = lfo(move |t| {
            let bar4 = BAR * 4.0;
            let step = ((t % bar4) / STEP) as usize;
            ARP_NOTES[step % ARP_NOTES.len()]
        });
        let arp_osc = arp_freq >> square();
        arp_osc * arp_env * 0.3
    };

    // ── Pad -- Cheesy synth pads. Pure cheese. ─────────────
    // Three-voice detuned saws + sine per chord tone.
    // Slow crossfade between chords. Lush and wide.
    let pad = {
        // LFO that outputs the three frequencies for the current chord
        // with smooth crossfade between chord changes
        let pad_freq_lo = lfo(move |t| {
            let bar4 = BAR * 4.0;
            let cycle_t = t % bar4;
            let chord_idx = (cycle_t / BAR) as usize;
            let chord_phase = (cycle_t % BAR) / BAR;
            let curr = PAD_CHORDS[chord_idx % 4].0;
            let next = PAD_CHORDS[(chord_idx + 1) % 4].0;
            // Smooth crossfade in last 10% of each bar
            if chord_phase > 0.9 {
                let xf = (chord_phase - 0.9) / 0.1;
                curr * (1.0 - xf) + next * xf
            } else {
                curr
            }
        });
        let pad_freq_mid = lfo(move |t| {
            let bar4 = BAR * 4.0;
            let cycle_t = t % bar4;
            let chord_idx = (cycle_t / BAR) as usize;
            let chord_phase = (cycle_t % BAR) / BAR;
            let curr = PAD_CHORDS[chord_idx % 4].1;
            let next = PAD_CHORDS[(chord_idx + 1) % 4].1;
            if chord_phase > 0.9 {
                let xf = (chord_phase - 0.9) / 0.1;
                curr * (1.0 - xf) + next * xf
            } else {
                curr
            }
        });
        let pad_freq_hi = lfo(move |t| {
            let bar4 = BAR * 4.0;
            let cycle_t = t % bar4;
            let chord_idx = (cycle_t / BAR) as usize;
            let chord_phase = (cycle_t % BAR) / BAR;
            let curr = PAD_CHORDS[chord_idx % 4].2;
            let next = PAD_CHORDS[(chord_idx + 1) % 4].2;
            if chord_phase > 0.9 {
                let xf = (chord_phase - 0.9) / 0.1;
                curr * (1.0 - xf) + next * xf
            } else {
                curr
            }
        });

        // Each voice: detuned saw pair + sine for warmth
        let lo_a = (pad_freq_lo.clone() * dc(1.004f32)) >> saw();
        let lo_b = (pad_freq_lo.clone() * dc(0.996f32)) >> saw();
        let lo_s = pad_freq_lo >> sine();
        let voice_lo = (lo_a + lo_b) * 0.25 + lo_s * 0.5;

        let mid_a = (pad_freq_mid.clone() * dc(1.005f32)) >> saw();
        let mid_b = (pad_freq_mid.clone() * dc(0.995f32)) >> saw();
        let mid_s = pad_freq_mid >> sine();
        let voice_mid = (mid_a + mid_b) * 0.25 + mid_s * 0.5;

        let hi_a = (pad_freq_hi.clone() * dc(1.003f32)) >> saw();
        let hi_b = (pad_freq_hi.clone() * dc(0.997f32)) >> saw();
        let hi_s = pad_freq_hi >> sine();
        let voice_hi = (hi_a + hi_b) * 0.25 + hi_s * 0.5;

        // Slow pulsing amplitude (cheesy shimmer)
        let shimmer = lfo(|t| {
            let pulse = (t * 0.8 * std::f32::consts::TAU).sin() * 0.15 + 0.85;
            pulse
        });

        (voice_lo + voice_mid * 0.9 + voice_hi * 0.7) * shimmer * 0.22
    };

    // Pad volume: fades in during Phase 2, swells during bridge
    let pad_vol = lfo(move |t| {
        let base = ramp(t, PHASE2_START, PHASE2_FULL);
        let bridge_swell = ramp(t, BRIDGE_START, BRIDGE_FULL);
        // During solo: pad stays but ducks slightly
        let solo_duck = if t > PHASE3_FULL {
            0.7
        } else if t > PHASE3_START {
            1.0 - ramp(t, PHASE3_START, PHASE3_FULL) * 0.3
        } else {
            1.0
        };
        base * (1.0 + bridge_swell * 0.5) * solo_duck
    });

    let pad_final = pad * pad_vol;

    // Lead + arp get louder during bridge
    let lead_with_bridge = lead * (dc(1.0f32) + bridge_intensity.clone() * 0.4);
    let arp_with_bridge = arp * (dc(1.0f32) + bridge_intensity * 0.6);

    let melodics = (bass + lead_with_bridge + arp_with_bridge * 0.7 + pad_final) * phase2_vol;

    // ════════════════════════════════════════════════════════
    // SOLO -- A-minor shred locked to bar boundaries.
    //
    // Each 4-bar phrase starts on beat 1 of the groove cycle.
    // The solo knows what chord is underneath and anchors to it.
    // Phrase 1 (C): establishes over C minor
    // Phrase 2 (Ab): fast run over Ab
    // Phrase 3 (Bb): chromatic chaos over Bb
    // Phrase 4 (G): triumphant descent over G
    //
    // The groove keeps playing. The solo rides it.
    // ════════════════════════════════════════════════════════

    let phase3_vol = lfo(move |t| {
        let base = ramp(t, PHASE3_START, PHASE3_FULL);
        let peak_boost = if t > PHASE3_PEAK {
            1.0
        } else if t > PHASE3_FULL {
            1.0 + ((t - PHASE3_FULL) / (PHASE3_PEAK - PHASE3_FULL)) * 0.4
        } else {
            1.0
        };
        base * peak_boost
    });

    let solo = {
        let solo_freq = lfo(move |t| {
            if t < PHASE3_START {
                return 0.0;
            }
            let solo_t = t - PHASE3_START;
            let loop_t = solo_t % SOLO_TOTAL;
            melody_lookup(&SOLO_MELODY, SOLO_BEAT, loop_t).0
        });

        // Vibrato: gentle, always-on wobble
        let vibrato_depth = lfo(move |t| {
            if t < PHASE3_START {
                return 1.0;
            }
            let solo_t = t - PHASE3_START;
            let loop_t = solo_t % SOLO_TOTAL;
            let (_freq, note_phase, dur_beats) = melody_lookup(&SOLO_MELODY, SOLO_BEAT, loop_t);
            let rate = 5.0;
            let vib = (t * rate * std::f32::consts::TAU).sin();
            let ramp_in = (note_phase * 4.0).min(1.0);
            let depth = 0.006 + ((dur_beats - 0.5) / 1.0).clamp(0.0, 1.0) * 0.015 * ramp_in;
            1.0 + vib * depth
        });

        let freq_with_vib = solo_freq * vibrato_depth;

        // Softer timbre: detuned saws + warm sine, no sub (leave low end for bass)
        let osc1a = (freq_with_vib.clone() * dc(1.002f32)) >> saw();
        let osc1b = (freq_with_vib.clone() * dc(0.998f32)) >> saw();
        let osc_warm = freq_with_vib >> sine();

        let raw = (osc1a + osc1b) * 0.18 + osc_warm * 0.25;

        // Heavy tremolo: always on, deep and psychedelic
        // Two rates beating against each other for warble
        let tremolo = lfo(move |t| {
            if t < PHASE3_START {
                return 1.0;
            }
            let solo_t = t - PHASE3_START;
            let loop_t = solo_t % SOLO_TOTAL;
            let (_freq, note_phase, dur_beats) = melody_lookup(&SOLO_MELODY, SOLO_BEAT, loop_t);
            // Primary tremolo: ~7 Hz, deep
            let trem1 = (t * 7.0 * std::f32::consts::TAU).sin();
            // Secondary tremolo: ~4.3 Hz, offset — creates beating/warble
            let trem2 = (t * 4.3 * std::f32::consts::TAU).sin();
            // Short notes get even deeper tremolo
            let short_boost = (1.0 - dur_beats).clamp(0.0, 1.0) * 0.15;
            let depth = 0.55 + short_boost; // base 55% depth, up to 70% on short notes
            let ramp_in = (note_phase * 5.0).min(1.0);
            let combined = trem1 * 0.65 + trem2 * 0.35;
            let modulation = depth * ramp_in;
            (1.0 - modulation) + modulation * (combined * 0.5 + 0.5)
        });

        // Softer note envelope: slower attack, gentler decay
        let note_env = lfo(move |t| {
            if t < PHASE3_START {
                return 0.0;
            }
            let solo_t = t - PHASE3_START;
            let loop_t = solo_t % SOLO_TOTAL;
            let (freq, note_phase, dur_beats) = melody_lookup(&SOLO_MELODY, SOLO_BEAT, loop_t);
            if freq < 1.0 {
                return 0.0;
            }
            let atk = (note_phase / 0.03).min(1.0); // slower attack (was 0.01)
            let sustain_decay = if dur_beats < 0.5 {
                0.10
            } else if dur_beats < 1.0 {
                0.04
            } else {
                0.015
            };
            let sustain = (1.0 - (note_phase - 0.03).max(0.0) * sustain_decay).max(0.15);
            let rel = if note_phase > 0.85 {
                ((1.0 - note_phase) / 0.15).max(0.0)
            } else {
                1.0
            };
            atk * sustain * rel
        });

        raw * tremolo * note_env
    };

    // Gentle saturation instead of high-gain distortion
    let solo_shaped = solo
        >> shape_fn(|x| {
            let x = x * 1.8; // mild drive
            x / (1.0 + x.abs()) // soft tanh-style saturation
        });

    // ── Psychedelic plate reverb ─────────────────────────────
    // Schroeder-style: multi-tap delays with allpass diffusion.
    // Long tail, modulated, dark and washy. Pure math.
    //
    // We simulate it with multiple delayed+filtered copies of
    // the solo signal at prime-number-spaced tap times, each
    // feeding back through LFO-modulated decay. The result is
    // a dense, shimmering tail that floats behind the dry sound.
    let reverb_tap = |delay_ms: f32, decay: f32, lfo_rate: f32, lfo_depth: f32| {
        // Each tap: re-derive the solo signal at a delayed time
        // with its own decay envelope and LFO modulation
        let tap_delay = delay_ms / 1000.0;
        let tap_env = lfo(move |t| {
            if t < PHASE3_START + tap_delay {
                return 0.0;
            }
            let dt = t - tap_delay;
            let solo_t = dt - PHASE3_START;
            if solo_t < 0.0 {
                return 0.0;
            }
            let loop_t = solo_t % SOLO_TOTAL;
            let (freq, note_phase, _dur_beats) = melody_lookup(&SOLO_MELODY, SOLO_BEAT, loop_t);
            if freq < 1.0 {
                return 0.0;
            }
            // Reverb tail envelope: ramps up then decays with the delay
            let atk = (note_phase / 0.05).min(1.0);
            let tail = (-note_phase * 0.8).exp(); // long decay tail
                                                  // LFO modulation on amplitude for shimmer
            let shimmer = 1.0 + (t * lfo_rate * std::f32::consts::TAU).sin() * lfo_depth;
            atk * tail * decay * shimmer
        });
        let tap_freq = lfo(move |t| {
            if t < PHASE3_START + tap_delay {
                return 0.0;
            }
            let dt = t - tap_delay;
            let solo_t = dt - PHASE3_START;
            if solo_t < 0.0 {
                return 0.0;
            }
            let loop_t = solo_t % SOLO_TOTAL;
            let freq = melody_lookup(&SOLO_MELODY, SOLO_BEAT, loop_t).0;
            if freq < 1.0 {
                0.0
            } else {
                freq
            }
        });
        // Warm sine for reverb taps (darker than the dry signal)
        (tap_freq >> sine()) * tap_env
    };

    // Prime-spaced taps for maximal diffusion (no comb filtering)
    let verb_a = reverb_tap(37.0, 0.45, 0.7, 0.12); // early reflection
    let verb_b = reverb_tap(79.0, 0.38, 0.53, 0.15); // mid reflection
    let verb_c = reverb_tap(127.0, 0.30, 0.37, 0.18); // late reflection
    let verb_d = reverb_tap(191.0, 0.22, 0.23, 0.20); // diffuse tail
    let verb_e = reverb_tap(263.0, 0.15, 0.17, 0.25); // deep tail
    let verb_f = reverb_tap(347.0, 0.10, 0.11, 0.30); // ghostly far tail

    // Reverb sum — highpassed to keep it out of the bass region
    let plate_verb = (verb_a + verb_b + verb_c + verb_d + verb_e + verb_f) * 0.55
        >> highpass_hz(280.0f32, 0.5f32);

    let solo_wet = solo_shaped * 0.55 + plate_verb; // dry/wet blend

    // Highpass the entire solo chain so it never fights the bass or kick
    let solo_final = (solo_wet >> highpass_hz(220.0f32, 0.6f32)) * phase3_vol * 0.24;

    // ════════════════════════════════════════════════════════
    // HARD ROCK OVERBEAT -- Layers on top of the existing beat
    // the moment the solo enters. Classic rock pattern:
    //
    //   tutu cha tu tu tut cha
    //   K K  S  K  K  K   S
    //
    // On the 8th-note grid (8 steps per bar):
    //   0:K  1:K  2:S  3:—  4:K  5:K  6:K  7:S
    //
    // Heavier kick, fatter snare, crash on the snare hits.
    // Fades in with the solo so it doesn't just slam in.
    // ════════════════════════════════════════════════════════

    let overbeat_vol = lfo(move |t| ramp(t, PHASE3_START, PHASE3_FULL));

    let rock_kick = {
        let eighth = BEAT / 2.0;
        // Pitch sweep re-triggers per 8th note
        let rk_freq = lfo(move |t| {
            if t < PHASE3_START {
                return 48.0;
            }
            let bar_t = t % BAR;
            let step = (bar_t / eighth) as usize;
            let _is_hit = match step % 8 {
                0 | 1 | 4 | 5 | 6 => true,
                _ => return 48.0,
            };
            let p = (bar_t % eighth) / eighth;
            140.0 * (-p * 22.0 * eighth).exp() + 48.0
        });
        // Amp envelope re-triggers per 8th note
        let rk_env = lfo(move |t| {
            if t < PHASE3_START {
                return 0.0;
            }
            let bar_t = t % BAR;
            let step = (bar_t / eighth) as usize;
            let vol = match step % 8 {
                0 => 1.0,
                1 => 0.85,
                4 => 1.0,
                5 => 0.8,
                6 => 0.7,
                _ => return 0.0,
            };
            let p = (bar_t % eighth) / eighth;
            let click = (-p * 90.0 * eighth).exp() * 0.5;
            let body = (-p * 6.0 * eighth).exp();
            vol * (click + body) * (-p * 7.0).exp().min(1.0)
        });
        (rk_freq >> sine()) * rk_env * 2.8
    };

    let rock_snare = {
        let eighth = BEAT / 2.0;
        // All-in-one envelope for snare hits
        let rs_env = lfo(move |t| {
            if t < PHASE3_START {
                return 0.0;
            }
            let bar_t = t % BAR;
            let step = (bar_t / eighth) as usize;
            if step % 8 != 2 && step % 8 != 7 {
                return 0.0;
            }
            let p = (bar_t % eighth) / eighth;
            (-p * 10.0).exp().min(1.0)
        });
        let rs_body = sine_hz(180.0) * 0.4 + sine_hz(330.0) * 0.2;
        let rs_crack = noise() >> bandpass_hz(3800.0f32, 0.7f32);
        let rs_rattle = noise() >> bandpass_hz(6000.0f32, 1.5f32);
        (rs_body + rs_crack * 0.6 + rs_rattle * 0.25) * rs_env * 2.5
    };

    let rock_crash = {
        let eighth = BEAT / 2.0;
        let crash_noise = noise() >> highpass_hz(4000.0f32, 0.3f32);
        let crash_mid = noise() >> bandpass_hz(2500.0f32, 0.4f32);
        let crash_env = lfo(move |t| {
            if t < PHASE3_START {
                return 0.0;
            }
            let bar_t = t % BAR;
            let step = (bar_t / eighth) as usize;
            if step % 8 != 2 && step % 8 != 7 {
                return 0.0;
            }
            let p = (bar_t % eighth) / eighth;
            (-p * 3.5).exp().min(1.0)
        });
        (crash_noise * 0.5 + crash_mid * 0.3) * crash_env * 0.4
    };

    let overbeat = (rock_kick + rock_snare + rock_crash) * overbeat_vol;

    // Overbeat through its own saturation for glue
    let overbeat_saturated = overbeat
        >> shape_fn(|x| {
            let x = x * 1.2;
            let s = x.signum();
            let a = x.abs();
            s * if a < 0.6 {
                a
            } else {
                0.6 + (1.0 - (-((a - 0.6) * 2.5)).exp()) * 0.25
            }
        });

    // ════════════════════════════════════════════════════════
    // FINAL MIX -- The silver line runs through everything.
    //
    // Phase 1: drums alone
    // Phase 2: drums + full groove
    // Bridge: groove intensifies, tension builds
    // Phase 3: groove (ducked) + SOLO on top + ROCK OVERBEAT
    //
    // The groove never fully drops out. It ducks by 30% when
    // the solo is screaming. The beat is always there.
    // ════════════════════════════════════════════════════════

    let melodics_duck = lfo(move |t| {
        let solo_level = ramp(t, PHASE3_START, PHASE3_FULL);
        1.0 - solo_level * 0.35 // duck by 35% max when solo screams
    });

    // Sidechain pump: duck melodics on kick hits for that pumping cohesion
    let sidechain = lfo(move |t| {
        let phase = (t % BEAT) / BEAT;
        // Quick duck on each beat, recover over ~60% of the beat
        let duck = if phase < 0.05 {
            0.55 // sharp duck
        } else if phase < 0.6 {
            0.55 + (phase - 0.05) / 0.55 * 0.45 // recover
        } else {
            1.0
        };
        duck
    });

    let mono = drums_saturated * 0.45
        + melodics * melodics_duck * sidechain * 0.9
        + solo_final
        + overbeat_saturated * 1.6;

    // Tighter master limiter with more headroom for transients
    let master = mono
        >> shape_fn(|x| {
            let x = x * 1.15; // slight master drive for density
            let s = x.signum();
            let a = x.abs();
            s * if a < 0.5 {
                a
            } else if a < 0.85 {
                0.5 + (a - 0.5) * 0.5
            } else {
                0.675 + (1.0 - (-((a - 0.85) * 4.0)).exp()) * 0.12
            }
        });

    let stereo = master >> split::<U2>();
    Box::new(stereo)
}

/// Bar phase [0,1)
pub fn bar_phase(time: f64) -> f32 {
    ((time as f32 / BAR) % 1.0).abs()
}

/// Current 16th step within bar (0..15)
pub fn step_in_bar(time: f64) -> usize {
    ((time as f32 / STEP) as usize) % 16
}
