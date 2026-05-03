use std::sync::{Arc, Mutex};

use fundsp::prelude32::*;

const BPM: f32 = 125.0;
const BEAT: f32 = 60.0 / BPM;
const BAR: f32 = BEAT * 4.0;
const STEP: f32 = BEAT / 4.0;
const SONG_LEN: f32 = 124.0;

const BRIDGE_START: f32 = 8.0;
const BRIDGE_FULL: f32 = 14.0;
const OVERDUB_START: f32 = BAR * 8.0;
const OVERDUB_FULL: f32 = BAR * 10.0;
const GUITAR_TIME_SCALE: f32 = 1.0;
const STATION_INSERT_TIME: f32 = 60.0;
const GRAPH_CHANGE_TIMES: [f32; 3] = [46.0, 60.0, 90.0];
const ROUTE_VISIT_DRAW_TIMES: [f32; 11] = [
    10.22, 11.31, 12.40, 13.49, 14.58, 15.67, 16.76, 17.85, 18.95, 20.04, 21.13,
];

const A4: f32 = 440.00;
const B4: f32 = 493.88;
const CS5: f32 = 554.37;
const D5: f32 = 587.33;
const E5: f32 = 659.25;
const FS5: f32 = 739.99;
const GS5: f32 = 830.61;
const A5: f32 = 880.00;
const B5: f32 = 987.77;
const C6: f32 = 1046.50;
const CS6: f32 = 1108.73;

const BASS_NOTES: [f32; 64] = [
    92.50, 92.50, 92.50, 92.50, 92.50, 92.50, 92.50, 92.50, 92.50, 92.50, 92.50, 92.50, 92.50,
    92.50, 92.50, 92.50, 73.42, 73.42, 73.42, 73.42, 73.42, 73.42, 73.42, 73.42, 73.42, 73.42,
    73.42, 73.42, 73.42, 73.42, 73.42, 73.42, 82.41, 82.41, 82.41, 82.41, 82.41, 82.41, 82.41,
    82.41, 82.41, 82.41, 82.41, 82.41, 82.41, 82.41, 82.41, 82.41, 69.30, 69.30, 69.30, 69.30,
    69.30, 69.30, 69.30, 69.30, 69.30, 69.30, 69.30, 69.30, 82.41, 82.41, 82.41, 82.41,
];

const BASS_GATE: [f32; 16] = [
    1.0, 0.62, 1.0, 0.58, 1.0, 0.55, 0.85, 0.46, 1.0, 0.60, 1.0, 0.52, 0.92, 0.58, 1.0, 0.72,
];

const LEAD_NOTES: [f32; 64] = [
    554.37, 0.0, 493.88, 0.0, 554.37, 0.0, 554.37, 0.0, 493.88, 0.0, 440.00, 0.0, 493.88, 0.0,
    554.37, 493.88, 440.00, 0.0, 369.99, 0.0, 440.00, 0.0, 493.88, 0.0, 440.00, 0.0, 369.99, 0.0,
    329.63, 0.0, 369.99, 0.0, 493.88, 0.0, 415.30, 0.0, 493.88, 0.0, 554.37, 0.0, 493.88, 0.0,
    415.30, 0.0, 369.99, 0.0, 415.30, 0.0, 415.30, 0.0, 349.23, 0.0, 415.30, 0.0, 493.88, 0.0,
    415.30, 0.0, 554.37, 0.0, 493.88, 0.0, 415.30, 554.37,
];

const ARP_NOTES: [f32; 64] = [
    554.37, 440.00, 369.99, 277.18, 554.37, 440.00, 369.99, 277.18, 554.37, 440.00, 369.99, 277.18,
    554.37, 440.00, 369.99, 277.18, 440.00, 369.99, 293.66, 220.00, 440.00, 369.99, 293.66, 220.00,
    440.00, 369.99, 293.66, 220.00, 440.00, 369.99, 293.66, 220.00, 493.88, 415.30, 329.63, 246.94,
    493.88, 415.30, 329.63, 246.94, 493.88, 415.30, 329.63, 246.94, 493.88, 415.30, 329.63, 246.94,
    415.30, 349.23, 277.18, 207.65, 415.30, 349.23, 277.18, 207.65, 415.30, 349.23, 277.18, 207.65,
    493.88, 415.30, 329.63, 246.94,
];

const ARP_VOL: [f32; 4] = [1.0, 0.62, 0.38, 0.18];

const PAD_CHORDS: [(f32, f32, f32); 4] = [
    (92.50, 138.59, 185.00),
    (73.42, 110.00, 146.83),
    (82.41, 123.47, 164.81),
    (69.30, 103.83, 138.59),
];

const GUITAR_PHRASE: [(f32, f32); 21] = [
    (CS6, 0.5),
    (C6, 0.5),
    (A5, 1.0),
    (FS5, 1.0),
    (E5, 1.0),
    (D5, 1.0),
    (E5, 0.5),
    (FS5, 0.5),
    (A5, 1.0),
    (FS5, 1.0),
    (B5, 1.0),
    (A5, 0.5),
    (GS5, 0.5),
    (E5, 1.0),
    (B4, 1.0),
    (CS5, 1.0),
    (D5, 0.5),
    (CS5, 0.5),
    (B4, 1.0),
    (A4, 0.5),
    (B4, 0.5),
];

const GUITAR_TOTAL: f32 = guitar_total_beats() * BEAT;

const fn guitar_total_beats() -> f32 {
    let mut total = 0.0;
    let mut i = 0;
    while i < GUITAR_PHRASE.len() {
        total += GUITAR_PHRASE[i].1;
        i += 1;
    }
    total
}

#[derive(Clone)]
pub struct AudioClock {
    time: Arc<Mutex<f32>>,
}

impl AudioClock {
    pub fn new() -> Self {
        Self {
            time: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn set_time(&self, time: f32) {
        *self.time.lock().expect("audio clock mutex poisoned") = time;
    }

    pub fn time(&self) -> f32 {
        *self.time.lock().expect("audio clock mutex poisoned")
    }
}

pub struct RouteSynth {
    unit: Box<dyn AudioUnit>,
    last_time: f32,
}

impl RouteSynth {
    pub fn new(sample_rate: f32) -> Self {
        let mut unit = build_synth();
        unit.reset();
        unit.set_sample_rate(sample_rate as f64);
        Self {
            unit,
            last_time: 0.0,
        }
    }

    pub fn sample(&mut self, time: f32) -> (f32, f32) {
        if time + 0.25 < self.last_time {
            self.unit.reset();
        }
        self.last_time = time;
        let fade = ramp(time, 0.0, 1.5) * ramp(SONG_LEN - time, 0.0, 4.0);
        let (left, right) = self.unit.get_stereo();
        let (event_left, event_right) = route_event_sfx(time);
        (
            (left * fade + event_left).clamp(-1.0, 1.0),
            (right * fade + event_right).clamp(-1.0, 1.0),
        )
    }
}

fn route_event_sfx(time: f32) -> (f32, f32) {
    let mut left = 0.0;
    let mut right = 0.0;

    for event_time in GRAPH_CHANGE_TIMES {
        let (event_left, event_right) = graph_change_sfx(time - event_time);
        left += event_left;
        right += event_right;
    }

    for (idx, event_time) in ROUTE_VISIT_DRAW_TIMES.iter().copied().enumerate() {
        let (event_left, event_right) = visit_draw_sfx(time - event_time, idx);
        left += event_left;
        right += event_right;
    }

    let (insert_left, insert_right) = station_insert_sfx(time - STATION_INSERT_TIME);
    left += insert_left;
    right += insert_right;

    (left, right)
}

fn graph_change_sfx(event_t: f32) -> (f32, f32) {
    if !(0.0..=0.72).contains(&event_t) {
        return (0.0, 0.0);
    }

    let hit = transient_env(event_t, 0.004, 0.24);
    let tail = transient_env(event_t, 0.018, 0.56);
    let sweep_phase = event_t * (980.0 - event_t * 620.0).max(170.0);
    let sweep = (sweep_phase * std::f32::consts::TAU).sin();
    let low = (event_t * 92.0 * std::f32::consts::TAU).sin();
    let tick = (event_t * 2200.0 * std::f32::consts::TAU).sin() * hit;
    let noise = pseudo_noise(event_t * 8200.0) * hit * 0.18;
    let sample = (sweep * tail * 0.20 + low * tail * 0.16 + tick * 0.18 + noise) * 0.82;

    (sample * 0.92, sample)
}

fn visit_draw_sfx(event_t: f32, idx: usize) -> (f32, f32) {
    if !(0.0..=0.38).contains(&event_t) {
        return (0.0, 0.0);
    }

    let step = idx as f32;
    let freq = 880.0 * 2.0_f32.powf((step % 8.0) / 12.0);
    let ping_env = transient_env(event_t, 0.002, 0.18);
    let click_env = (-event_t / 0.018).exp();
    let tick = (event_t * freq * std::f32::consts::TAU).sin();
    let bright = (event_t * freq * 2.99 * std::f32::consts::TAU).sin() * 0.32;
    let click = pseudo_noise(event_t * 18_000.0 + step * 19.0) * click_env * 0.20;
    let impulse = if event_t < 0.006 { 0.22 } else { 0.0 };
    let sample = (tick + bright) * ping_env * 0.30 + click + impulse;
    let pan = (step / 10.0 - 0.5) * 0.36;

    (sample * (1.0 - pan), sample * (1.0 + pan))
}

fn station_insert_sfx(event_t: f32) -> (f32, f32) {
    if !(0.0..=1.12).contains(&event_t) {
        return (0.0, 0.0);
    }

    let notes: [(f32, f32, f32); 4] = [
        (0.00, 739.99, -0.25),
        (0.10, 932.33, 0.18),
        (0.20, 1108.73, -0.08),
        (0.34, 1479.98, 0.28),
    ];
    let mut left = 0.0;
    let mut right = 0.0;

    for (offset, freq, pan) in notes {
        let note_t = event_t - offset;
        if !(0.0..=0.46).contains(&note_t) {
            continue;
        }
        let env = transient_env(note_t, 0.006, 0.34);
        let body = (note_t * freq * std::f32::consts::TAU).sin();
        let bright = (note_t * freq * 2.01 * std::f32::consts::TAU).sin() * 0.25;
        let sample = (body + bright) * env * 0.18;
        left += sample * (1.0 - pan).clamp(0.65, 1.25);
        right += sample * (1.0 + pan).clamp(0.65, 1.25);
    }

    let settle_t = event_t - 0.46;
    if (0.0..=0.44).contains(&settle_t) {
        let env = transient_env(settle_t, 0.02, 0.30);
        let sample = (settle_t * 185.0 * std::f32::consts::TAU).sin() * env * 0.12;
        left += sample;
        right += sample * 0.88;
    }

    (left, right)
}

fn transient_env(t: f32, attack: f32, decay: f32) -> f32 {
    let attack = (t / attack).clamp(0.0, 1.0);
    attack * (-t / decay).exp()
}

fn pseudo_noise(x: f32) -> f32 {
    ((x * 12.9898).sin() * 43_758.547).fract() * 2.0 - 1.0
}

fn build_synth() -> Box<dyn AudioUnit> {
    let kick = {
        let freq = lfo(move |t| {
            let vol = kick_vol(t);
            if vol < 0.01 {
                return 42.0;
            }
            let p = (t % STEP) / STEP;
            180.0 * (-p * 30.0 * STEP).exp() + 42.0 * (1.0 - (-p * 30.0 * STEP).exp())
        });
        let env = lfo(move |t| {
            let vol = kick_vol(t);
            if vol < 0.01 {
                return 0.0;
            }
            let p = (t % STEP) / STEP;
            let click = (-p * 80.0 * STEP).exp() * 0.4;
            let body = (-p * 7.0 * STEP).exp();
            vol * (click + body) * (-p * 8.0).exp().min(1.0)
        });
        (freq >> sine()) * env * 3.5
    };

    let snare = {
        let env = lfo(move |t| {
            let vol = snare_vol(t);
            if vol < 0.01 {
                return 0.0;
            }
            let p = (t % STEP) / STEP;
            vol * (-p * 14.0).exp().min(1.0)
        });
        let body = sine_hz(190.0) * 0.34 + sine_hz(760.0) * 0.18 + sine_hz(1220.0) * 0.10;
        let crack_hi = noise() >> bandpass_hz(4500.0, 0.8);
        let crack_lo = noise() >> bandpass_hz(1800.0, 1.2);
        (body + crack_hi * 0.72 + crack_lo * 0.48) * env * 2.7
    };

    let hat = {
        let rhythm = lfo(move |t| {
            let step = ((t % BAR) / STEP) as usize;
            let vol = match step % 16 {
                0 | 8 => 0.9,
                2 | 6 | 10 | 14 => 0.7,
                4 | 12 => 0.82,
                7 | 15 => 0.38,
                _ => 0.24,
            };
            let phase = (t % STEP) / STEP;
            vol * (-phase * 55.0).exp().min(1.0)
        });
        (noise() >> highpass_hz(8500.0, 0.8)) * rhythm * 0.55
    };

    let open_hat = {
        let rhythm = lfo(move |t| {
            let step = ((t % (BAR * 2.0)) / STEP) as usize;
            if step == 9 || step == 25 || (t > BRIDGE_START && (step == 29 || step == 31)) {
                let phase = (t % STEP) / STEP;
                (-phase * 6.0).exp().min(1.0)
            } else {
                0.0
            }
        });
        (noise() >> highpass_hz(6500.0, 0.5)) * rhythm * 0.42
    };

    let drums = kick + snare + hat + open_hat;
    let drums = drums >> shape_fn(|x| drum_shape(x * 1.3));

    let bridge = lfo(move |t| ramp(t, BRIDGE_START, BRIDGE_FULL));
    let groove_vol = lfo(move |t| ramp(t, 0.0, 0.5));

    let bass = {
        let freq = lfo(move |t| {
            let step = ((t % (BAR * 4.0)) / STEP) as usize;
            BASS_NOTES[step % BASS_NOTES.len()]
        });
        let env = lfo(move |t| {
            let step = ((t % BAR) / STEP) as usize;
            let gate = BASS_GATE[step % BASS_GATE.len()];
            let phase = (t % STEP) / STEP;
            let atk = (phase / 0.01).min(1.0);
            let slide = (1.0 - phase * 0.7).max(0.1);
            gate * atk * slide
        });
        let sq = freq.clone() >> square();
        let sub = (freq.clone() * dc(0.5)) >> sine();
        let over = (freq * dc(2.0)) >> sine();
        (sq * 0.58 + sub * 0.25 + over * 0.10) * env * 1.05
    };

    let lead = {
        let freq = lfo(move |t| {
            let step = ((t % (BAR * 4.0)) / STEP) as usize;
            LEAD_NOTES[step % LEAD_NOTES.len()]
        });
        let gate = lfo(move |t| {
            let step = ((t % (BAR * 4.0)) / STEP) as usize;
            let freq = LEAD_NOTES[step % LEAD_NOTES.len()];
            if freq < 1.0 {
                return 0.0;
            }
            let phase = (t % STEP) / STEP;
            let atk = (phase / 0.02).min(1.0);
            let dec = (1.0 - (phase - 0.05).max(0.0) * 0.4).max(0.15);
            atk * dec
        });
        let carrier = freq.clone() >> saw();
        let modulator = ((freq.clone() * dc(3.0)) >> sine()) * 0.34;
        let dry = (carrier + modulator) * gate * 0.43;

        let echo_delay = STEP * 2.0;
        let echo_freq = lfo(move |t| {
            let td = (t - echo_delay).max(0.0);
            let step = ((td % (BAR * 4.0)) / STEP) as usize;
            let freq = LEAD_NOTES[step % LEAD_NOTES.len()];
            if freq < 1.0 {
                0.0
            } else {
                freq * 1.003
            }
        });
        let echo_gate = lfo(move |t| {
            let td = (t - echo_delay).max(0.0);
            let step = ((td % (BAR * 4.0)) / STEP) as usize;
            let freq = LEAD_NOTES[step % LEAD_NOTES.len()];
            if freq < 1.0 {
                return 0.0;
            }
            let phase = (td % STEP) / STEP;
            let atk = (phase / 0.02).min(1.0);
            let dec = (1.0 - (phase - 0.05).max(0.0) * 0.5).max(0.1);
            atk * dec
        });
        dry + (echo_freq >> saw()) * echo_gate * 0.23
    };

    let arp = {
        let env = lfo(move |t| {
            let step = ((t % (BAR * 4.0)) / STEP) as usize;
            let section = (step / 16) % 2;
            if section == 1 || t > BRIDGE_START {
                let phase = (t % STEP) / STEP;
                let atk = (phase / 0.005).min(1.0);
                let dec = (-phase * 8.0).exp();
                ARP_VOL[step % 4] * atk * dec
            } else {
                0.0
            }
        });
        let freq = lfo(move |t| {
            let step = ((t % (BAR * 4.0)) / STEP) as usize;
            ARP_NOTES[step % ARP_NOTES.len()]
        });
        (freq >> square()) * env * 0.26
    };

    let pad = pad_voice() * pad_volume();
    let lead = lead * (dc(1.0) + bridge.clone() * 0.35);
    let arp = arp * (dc(1.0) + bridge * 0.55);
    let groove = (bass + lead + arp * 0.70 + pad) * groove_vol;

    let overbeat_vol = lfo(move |t| ramp(t, OVERDUB_START, OVERDUB_FULL));
    let overbeat = overbeat() * overbeat_vol;
    let overbeat = overbeat >> shape_fn(|x| drum_shape(x * 1.15));
    let guitar = guitar_lead() * lfo(move |t| ramp(t, OVERDUB_START, OVERDUB_FULL)) * 0.72;

    let melodics_duck = lfo(move |t| 1.0 - ramp(t, OVERDUB_START, OVERDUB_FULL) * 0.35);
    let sidechain = lfo(move |t| {
        let phase = (t % BEAT) / BEAT;
        if phase < 0.05 {
            0.55
        } else if phase < 0.6 {
            0.55 + (phase - 0.05) / 0.55 * 0.45
        } else {
            1.0
        }
    });

    let mono = drums * 0.45 + groove * melodics_duck * sidechain * 0.92 + guitar + overbeat * 1.55;
    let master = mono >> shape_fn(master_shape);
    Box::new(master >> split::<U2>())
}

fn pad_voice() -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let lo = pad_freq(0);
    let mid = pad_freq(1);
    let hi = pad_freq(2);

    let voice_lo = ((lo.clone() * dc(1.004)) >> saw()) * 0.25
        + ((lo.clone() * dc(0.996)) >> saw()) * 0.25
        + (lo >> sine()) * 0.50;
    let voice_mid = ((mid.clone() * dc(1.005)) >> saw()) * 0.25
        + ((mid.clone() * dc(0.995)) >> saw()) * 0.25
        + (mid >> sine()) * 0.50;
    let voice_hi = ((hi.clone() * dc(1.003)) >> saw()) * 0.25
        + ((hi.clone() * dc(0.997)) >> saw()) * 0.25
        + (hi >> sine()) * 0.50;
    let shimmer = lfo(|t| (t * 0.8 * std::f32::consts::TAU).sin() * 0.15 + 0.85);
    (voice_lo + voice_mid * 0.9 + voice_hi * 0.7) * shimmer * 0.22
}

fn pad_freq(index: usize) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    lfo(move |t| {
        let cycle_t = t % (BAR * 4.0);
        let chord_idx = (cycle_t / BAR) as usize;
        let chord_phase = (cycle_t % BAR) / BAR;
        let curr = chord_tone(chord_idx, index);
        let next = chord_tone(chord_idx + 1, index);
        if chord_phase > 0.9 {
            let xf = (chord_phase - 0.9) / 0.1;
            curr * (1.0 - xf) + next * xf
        } else {
            curr
        }
    })
}

fn chord_tone(chord_idx: usize, tone_idx: usize) -> f32 {
    let chord = PAD_CHORDS[chord_idx % PAD_CHORDS.len()];
    match tone_idx {
        0 => chord.0,
        1 => chord.1,
        _ => chord.2,
    }
}

fn pad_volume() -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    lfo(move |t| {
        let base = ramp(t, 0.0, 2.0);
        let bridge = ramp(t, BRIDGE_START, BRIDGE_FULL);
        let overdub_duck = if t > OVERDUB_FULL {
            0.72
        } else if t > OVERDUB_START {
            1.0 - ramp(t, OVERDUB_START, OVERDUB_FULL) * 0.28
        } else {
            1.0
        };
        base * (1.0 + bridge * 0.45) * overdub_duck
    })
}

fn guitar_lead() -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let freq = lfo(move |t| {
        if t < OVERDUB_START {
            return 0.0;
        }
        let loop_t = ((t - OVERDUB_START) * GUITAR_TIME_SCALE) % GUITAR_TOTAL;
        let (freq, phase, dur_beats, _) = guitar_lookup(loop_t);
        if freq < 1.0 {
            return 0.0;
        }
        let bend = if dur_beats >= 0.75 {
            1.0 + (1.0 - phase).powi(2) * 0.010
        } else {
            1.0
        };
        let vibrato_depth = if dur_beats >= 0.75 { 0.015 } else { 0.006 };
        let vibrato =
            1.0 + (t * 4.7 * std::f32::consts::TAU).sin() * vibrato_depth * (phase * 2.2).min(1.0);
        freq * 0.5 * bend * vibrato
    });
    let env = guitar_env();
    let raw = ((freq.clone() * dc(0.996)) >> saw()) * 0.26
        + ((freq.clone() * dc(1.004)) >> saw()) * 0.26
        + ((freq.clone() * dc(0.5)) >> sine()) * 0.18
        + (freq.clone() >> square()) * 0.20
        + (freq >> sine()) * 0.16;
    let picked = raw * env;
    let driven = picked >> shape_fn(guitar_shape);
    let cab = driven.clone() >> lowpass_hz(3600.0, 0.7) >> highpass_hz(150.0, 0.7);
    let neck = cab.clone() >> bandpass_hz(760.0, 1.1);
    let upper_mid = cab.clone() >> bandpass_hz(1650.0, 0.9);
    cab * 0.36 + neck * 0.56 + upper_mid * 0.28
}

fn guitar_env() -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    lfo(move |t| {
        if t < OVERDUB_START {
            return 0.0;
        }
        let loop_t = ((t - OVERDUB_START) * GUITAR_TIME_SCALE) % GUITAR_TOTAL;
        let (freq, note_phase, dur_beats, note_start_beats) = guitar_lookup(loop_t);
        if freq < 1.0 {
            return 0.0;
        }
        let note_start_half_beats = (note_start_beats * 2.0).round() as usize;
        let accent = if note_start_half_beats.is_multiple_of(8) {
            1.48
        } else if note_start_half_beats.is_multiple_of(2) {
            1.14
        } else {
            1.0
        };
        let pick = (1.0 - note_phase * 18.0).max(0.0) * 0.24;
        let attack = (note_phase / 0.012).min(1.0);
        let sustain = if dur_beats >= 0.75 {
            (1.0 - note_phase * 0.18).max(0.42)
        } else {
            (1.0 - note_phase * 0.82).max(0.16)
        };
        let release = if note_phase > 0.78 {
            ((1.0 - note_phase) / 0.22).max(0.0)
        } else {
            1.0
        };
        (attack * sustain + pick) * release * accent
    })
}

fn overbeat() -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let eighth = BEAT / 2.0;
    let kick = {
        let freq = lfo(move |t| {
            if t < OVERDUB_START {
                return 48.0;
            }
            let bar_t = t % BAR;
            let step = (bar_t / eighth) as usize;
            match step % 8 {
                0 | 1 | 4 | 5 | 6 => {}
                _ => return 48.0,
            }
            let p = (bar_t % eighth) / eighth;
            140.0 * (-p * 22.0 * eighth).exp() + 48.0
        });
        let env = lfo(move |t| {
            if t < OVERDUB_START {
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
        (freq >> sine()) * env * 2.8
    };
    let snare = {
        let env = lfo(move |t| {
            if t < OVERDUB_START {
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
        let body = sine_hz(180.0) * 0.4 + sine_hz(330.0) * 0.2;
        let crack = noise() >> bandpass_hz(3800.0, 0.7);
        let rattle = noise() >> bandpass_hz(6000.0, 1.5);
        (body + crack * 0.6 + rattle * 0.25) * env * 2.5
    };
    let crash = {
        let env = lfo(move |t| {
            if t < OVERDUB_START {
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
        ((noise() >> highpass_hz(4000.0, 0.3)) * 0.5 + (noise() >> bandpass_hz(2500.0, 0.4)) * 0.3)
            * env
            * 0.4
    };
    kick + snare + crash
}

fn kick_vol(t: f32) -> f32 {
    let step = ((t % (BAR * 4.0)) / STEP) as usize;
    let in_bridge = t > BRIDGE_START && t < OVERDUB_START;
    match step % 64 {
        0 | 4 | 8 | 12 | 16 | 20 | 24 | 28 | 32 | 36 | 40 | 44 | 48 | 52 | 56 | 60 => 1.0,
        2 | 6 | 10 | 14 | 18 | 22 | 26 | 30 => 0.45,
        11 | 27 | 43 | 59 => 0.3,
        34 | 38 | 42 | 46 | 50 | 54 | 58 | 62 => 0.45,
        33 | 37 | 41 | 45 | 49 | 53 | 57 | 61 if in_bridge => 0.55,
        _ => 0.0,
    }
}

fn snare_vol(t: f32) -> f32 {
    let step = ((t % (BAR * 4.0)) / STEP) as usize;
    let in_bridge = t > BRIDGE_START && t < OVERDUB_START;
    match step % 64 {
        4 | 12 | 20 | 28 | 36 | 44 | 52 | 60 => 1.0,
        15 | 31 | 47 | 63 => 0.35,
        3 | 19 | 35 | 51 => 0.2,
        59 | 61 | 62 if in_bridge => 0.7,
        _ => 0.0,
    }
}

fn guitar_lookup(loop_time: f32) -> (f32, f32, f32, f32) {
    let mut cursor = 0.0;
    for &(freq, dur_beats) in &GUITAR_PHRASE {
        let dur = dur_beats * BEAT;
        if loop_time < cursor + dur {
            return (freq, (loop_time - cursor) / dur, dur_beats, cursor / BEAT);
        }
        cursor += dur;
    }
    (0.0, 0.0, 1.0, 0.0)
}

fn ramp(t: f32, start: f32, full: f32) -> f32 {
    if t < start {
        0.0
    } else if t < full {
        (t - start) / (full - start)
    } else {
        1.0
    }
}

fn drum_shape(x: f32) -> f32 {
    let sign = x.signum();
    let abs = x.abs();
    sign * if abs < 0.5 {
        abs * 1.1
    } else if abs < 0.9 {
        0.55 + (abs - 0.5) * 0.35
    } else {
        0.69 + (1.0 - (-((abs - 0.9) * 3.0)).exp()) * 0.15
    }
}

fn guitar_shape(x: f32) -> f32 {
    let driven = x * 4.2;
    (driven * 0.82).tanh() * 0.78 + driven / (1.0 + driven.abs() * 0.85) * 0.22
}

fn master_shape(x: f32) -> f32 {
    let x = x * 1.15;
    let sign = x.signum();
    let abs = x.abs();
    sign * if abs < 0.5 {
        abs
    } else if abs < 0.85 {
        0.5 + (abs - 0.5) * 0.5
    } else {
        0.675 + (1.0 - (-((abs - 0.85) * 4.0)).exp()) * 0.12
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overdub_enters_on_bar_boundary() {
        let start_bars = OVERDUB_START / BAR;
        let full_bars = OVERDUB_FULL / BAR;

        assert!((start_bars - start_bars.round()).abs() < f32::EPSILON);
        assert!((full_bars - full_bars.round()).abs() < f32::EPSILON);
    }

    #[test]
    fn guitar_phrase_is_four_bar_slow_lead() {
        let mut actual_beats = 0.0;

        for &(_, dur_beats) in &GUITAR_PHRASE {
            let actual_note_beats = dur_beats / GUITAR_TIME_SCALE;
            assert!(actual_note_beats >= 0.5);
            assert!(
                (actual_note_beats * 2.0 - (actual_note_beats * 2.0).round()).abs() < f32::EPSILON
            );
            assert!((actual_beats * 2.0 - (actual_beats * 2.0).round()).abs() < f32::EPSILON);
            actual_beats += actual_note_beats;
        }

        assert!((actual_beats - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn guitar_downbeats_track_existing_backing() {
        assert_eq!(GUITAR_PHRASE[0].0, CS6);
        assert_eq!(GUITAR_PHRASE[5].0, D5);
        assert_eq!(GUITAR_PHRASE[10].0, B5);
        assert_eq!(GUITAR_PHRASE[15].0, CS5);
    }

    #[test]
    fn route_event_sfx_marks_graph_and_station_changes() {
        let graph_hit = route_event_sfx(GRAPH_CHANGE_TIMES[0] + 0.03);
        let station_hit = route_event_sfx(STATION_INSERT_TIME + 0.12);
        let visit_hit = route_event_sfx(ROUTE_VISIT_DRAW_TIMES[3] + 0.003);
        let quiet = route_event_sfx(12.0);

        assert!(graph_hit.0.abs() + graph_hit.1.abs() > 0.01);
        assert!(station_hit.0.abs() + station_hit.1.abs() > 0.01);
        assert!(visit_hit.0.abs() + visit_hit.1.abs() > 0.30);
        assert_eq!(quiet, (0.0, 0.0));
    }
}
