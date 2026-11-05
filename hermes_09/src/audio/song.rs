// HERMES_09 soundtrack: synthwave at 120 BPM in D harmonic minor,
// Dm - Bb - Gm - A, one chord per bar. Detuned supersaw pads, gliding octave
// bass, a gated snare, tom fills and noise risers, and leads that bend in the
// tracker style of the One Must Fall soundtrack: legato notes slide into each
// other, held notes scoop up from below and grow a vibrato, and phrase
// endings dive an octave.
//
// The arrangement follows the picture's sections bar for bar (see
// timeline.rs). The intro builds to the drop; under the live take the band
// steps back to pads, a held bass and light hats so the narration sits on
// top; the outro brings the groove and the hook back. Every task block that
// lands in the intro's week is a blip in the mix. Every note is rendered once
// into buffers at startup; live playback and the WAV export read the same
// samples.
use crate::timeline::{bars, now, Section, BAR, BEAT, DURATION, TOTAL_BARS};

pub const SAMPLE_RATE: u32 = 44_100;
const SR: f32 = SAMPLE_RATE as f32;
const TAU: f32 = std::f32::consts::TAU;

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

/// One-pole smoothing coefficient for a time constant in seconds.
fn coeff(tau: f32) -> f32 {
    1.0 - (-1.0 / (tau * SR)).exp()
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

// D harmonic minor (C sharp, never C): Dm - Bb - Gm - A, the A major chord
// pulling back to Dm every four bars.
const ROOTS: [f32; 4] = [38.0, 34.0, 31.0, 33.0];
const PADS: [[f32; 4]; 4] = [
    [62.0, 65.0, 69.0, 76.0], // Dm add9
    [58.0, 62.0, 65.0, 69.0], // Bbmaj7
    [62.0, 67.0, 70.0, 74.0], // Gm
    [61.0, 64.0, 67.0, 69.0], // A7
];

/// Two four-bar phrases on an eighth-note grid: (start, length, midi).
const HOOK_A: [(u8, u8, u8); 18] = [
    (0, 2, 74),
    (2, 1, 76),
    (3, 1, 77),
    (4, 2, 81),
    (6, 2, 82),
    (8, 3, 81),
    (11, 1, 77),
    (12, 4, 74),
    (16, 2, 79),
    (18, 1, 82),
    (19, 1, 86),
    (20, 2, 82),
    (22, 2, 81),
    (24, 2, 79),
    (26, 1, 77),
    (27, 1, 76),
    (28, 3, 73),
    (31, 1, 76),
];
const HOOK_B: [(u8, u8, u8); 14] = [
    (0, 3, 81),
    (3, 1, 82),
    (4, 2, 81),
    (6, 2, 86),
    (8, 2, 86),
    (10, 2, 82),
    (12, 4, 77),
    (16, 2, 79),
    (18, 1, 82),
    (19, 1, 81),
    (20, 4, 79),
    (24, 2, 76),
    (26, 2, 73),
    (28, 4, 81),
];

struct Mix {
    dry: Vec<[f32; 2]>,
    pad: Vec<[f32; 2]>,
    send: Vec<f32>,
    kicks: Vec<f32>,
    noise: u64,
}

impl Mix {
    fn new(len: usize) -> Self {
        Self {
            dry: vec![[0.0; 2]; len],
            pad: vec![[0.0; 2]; len],
            send: vec![0.0; len],
            kicks: Vec::new(),
            noise: 0x1234_5678_9abc_def1,
        }
    }

    fn rnd(&mut self) -> f32 {
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 7;
        self.noise ^= self.noise << 17;
        (self.noise >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
    }

    /// Write a voice: `f(t)` gives the mono sample at t seconds into it.
    fn voice(
        &mut self,
        at: f32,
        dur: f32,
        gain: f32,
        pan: f32,
        send: f32,
        mut f: impl FnMut(f32, &mut Self) -> f32,
    ) {
        let i0 = (at * SR) as usize;
        let n = (dur * SR) as usize;
        let (gl, gr) = (
            (1.0 - pan) * 0.5 * gain * 1.4,
            (1.0 + pan) * 0.5 * gain * 1.4,
        );
        for k in 0..n {
            let i = i0 + k;
            if i >= self.dry.len() {
                break;
            }
            let s = f(k as f32 / SR, self);
            self.dry[i][0] += s * gl;
            self.dry[i][1] += s * gr;
            self.send[i] += s * send * gain;
        }
    }
}

fn env(t: f32, attack: f32, hold: f32, release: f32) -> f32 {
    if t < attack {
        t / attack
    } else if t < hold {
        1.0
    } else {
        (1.0 - (t - hold) / release).max(0.0)
    }
}

fn saw(phase: f32) -> f32 {
    2.0 * phase.fract() - 1.0
}

/// A pulse wave with its DC offset removed (a narrow pulse is not centred).
fn pulse(phase: f32, duty: f32) -> f32 {
    let raw = if phase.fract() < duty { 1.0 } else { -1.0 };
    raw - (2.0 * duty - 1.0)
}

pub fn render(landings: &[f32]) -> Vec<[f32; 2]> {
    let len = (DURATION * SR) as usize + 1;
    let mut m = Mix::new(len);

    for b in 0..TOTAL_BARS as usize {
        let t0 = bars(b as f32);
        let section = now(t0 + 0.01).section;
        let next = now(t0 + BAR + 0.01).section;
        let chord = b % 4;
        let prev_root = ROOTS[(b + 3) % 4];
        let last_bar = b + 1 == TOTAL_BARS as usize;
        let k = b - first_bar(section);
        let left = section_bars(section) - k;

        pad(
            &mut m,
            t0,
            chord,
            if last_bar { 3.5 } else { BAR + 0.35 },
            b,
        );
        match section {
            Section::Intro => {
                arp(&mut m, t0, chord, if k >= 2 { 1.0 } else { 0.6 });
                if k >= 2 {
                    bass(&mut m, t0, prev_root, ROOTS[chord], Bass::Eighths, 0.8);
                    kick(&mut m, t0);
                    kick(&mut m, t0 + 2.5 * BEAT);
                    gated_snare(&mut m, t0 + 2.0 * BEAT, 0.8);
                }
                if k >= 4 {
                    four_on_floor(&mut m, t0);
                    hats(&mut m, t0, 2);
                }
            }
            Section::Live => {
                // Under the voice: pads, a held bass, quiet hats. No lead, no snare.
                arp(&mut m, t0, chord, 0.35);
                bass(&mut m, t0, prev_root, ROOTS[chord], Bass::Held, 0.5);
                if k >= 1 && left > 1 {
                    hats(&mut m, t0, 2);
                }
            }
            Section::Outro => {
                arp(&mut m, t0, chord, 1.0);
                if left > 2 {
                    bass(&mut m, t0, prev_root, ROOTS[chord], Bass::Octaves, 1.0);
                    four_on_floor(&mut m, t0);
                    gated_snare(&mut m, t0 + BEAT, 1.0);
                    gated_snare(&mut m, t0 + 3.0 * BEAT, 1.0);
                    hats(&mut m, t0, 4);
                } else if !last_bar {
                    bass(&mut m, t0, prev_root, ROOTS[chord], Bass::Held, 0.8);
                }
            }
        }
        // Toms and a noise riser into every new section.
        if next != section && b + 1 < TOTAL_BARS as usize {
            tom_fill(&mut m, t0);
            riser(&mut m, t0);
        }

        // Leads: the intro's last bars tease the hook an octave down; the
        // outro plays it in full.
        match section {
            Section::Intro if k == 2 => lead(&mut m, t0, &HOOK_B, -12.0, Osc::Brass, 0.6, true),
            Section::Outro if k == 0 => {
                lead(&mut m, t0, &HOOK_A, 0.0, Osc::Super, 1.0, false);
                lead(&mut m, t0, &HOOK_A, -12.0, Osc::Pulse, 0.5, false);
            }
            _ => {}
        }
    }

    // A blip for every task block landing in the intro's week.
    for (i, &at) in landings.iter().enumerate() {
        let chord = ((at / BAR) as usize) % 4;
        let note = PADS[chord][i % 4] + 24.0;
        blip(&mut m, at, hz(note), if i % 2 == 0 { -0.5 } else { 0.5 });
    }

    mixdown(m)
}

fn section_span(section: Section) -> (usize, usize) {
    crate::timeline::SECTIONS
        .iter()
        .find(|(s, _, _)| *s == section)
        .map(|(_, b0, b1)| (*b0 as usize, *b1 as usize))
        .unwrap_or((0, 0))
}

fn first_bar(section: Section) -> usize {
    section_span(section).0
}

fn section_bars(section: Section) -> usize {
    let (b0, b1) = section_span(section);
    b1 - b0
}

/// Supersaw pad: three detuned saws per chord tone through a lowpass whose
/// cutoff breathes over eight bars.
fn pad(m: &mut Mix, t0: f32, chord: usize, dur: f32, bar: usize) {
    let open = 0.035 + 0.035 * (0.5 - 0.5 * (bar as f32 * TAU / 8.0).cos());
    for (k, &note) in PADS[chord].iter().enumerate() {
        for (d, det) in [-0.12f32, 0.0, 0.12].iter().enumerate() {
            let f = hz(note + det);
            let pan = (k as f32 - 1.5) * 0.25 + (d as f32 - 1.0) * 0.35;
            let i0 = (t0 * SR) as usize;
            let n = (dur * SR) as usize;
            let (mut lp, mut lp2) = (0.0f32, 0.0f32);
            let phase0 = (k * 3 + d) as f32 * 0.21;
            for s in 0..n {
                let i = i0 + s;
                if i >= m.pad.len() {
                    break;
                }
                let t = s as f32 / SR;
                let raw = saw(phase0 + f * t);
                lp += (raw - lp) * open;
                lp2 += (lp - lp2) * open * 1.5;
                let e = env(t, 0.3, dur - 0.45, 0.45);
                let v = lp2 * e * 0.026;
                m.pad[i][0] += v * (1.0 - pan) * 0.5 * 1.4;
                m.pad[i][1] += v * (1.0 + pan) * 0.5 * 1.4;
            }
        }
    }
}

fn arp(m: &mut Mix, t0: f32, chord: usize, gain: f32) {
    let notes = PADS[chord];
    let order = [0, 1, 2, 3, 2, 1, 0, 1];
    for step in 0..16 {
        let note = notes[order[step % 8]] + if step >= 8 { 12.0 } else { 0.0 };
        let f = hz(note + 12.0);
        let at = t0 + step as f32 * BEAT / 4.0;
        let mut lp = 0.0f32;
        m.voice(
            at,
            0.2,
            0.045 * gain,
            if step % 2 == 0 { -0.4 } else { 0.4 },
            0.6,
            move |t, _| {
                let raw = pulse(f * t, 0.25);
                lp += (raw - lp) * 0.25;
                lp * (-t * 18.0).exp()
            },
        );
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Bass {
    /// Eighths on the root.
    Eighths,
    /// Eighths jumping root / octave, sliding between them.
    Octaves,
    /// One long note.
    Held,
}

/// One bar of bass as a single gliding voice: it slides in from the last
/// bar's root, and in Octaves it slides up and down every eighth.
fn bass(m: &mut Mix, t0: f32, from: f32, root: f32, kind: Bass, gain: f32) {
    let eighth = BEAT / 2.0;
    let mut pitch = from + 12.0;
    let mut phase = 0.0f32;
    let mut lp = 0.0f32;
    let k_bar = coeff(0.035);
    let k_oct = coeff(0.012);
    m.voice(t0, BAR + 0.02, 0.17 * gain, 0.0, 0.0, move |t, _| {
        let step = (t / eighth) as usize;
        let up = if kind == Bass::Octaves && step % 2 == 1 {
            12.0
        } else {
            0.0
        };
        let target = root + 12.0 + up;
        pitch += (target - pitch) * if t < 0.1 { k_bar } else { k_oct };
        phase += hz(pitch) / SR;
        let local = if kind == Bass::Held {
            t
        } else {
            t - step as f32 * eighth
        };
        let pluck = if kind == Bass::Held {
            (-t * 0.8).exp()
        } else {
            (-local * 9.0).exp()
        };
        let raw = 0.65 * saw(phase) + 0.35 * pulse(phase, 0.5);
        lp += (raw - lp) * (0.03 + 0.22 * pluck);
        let gate = if kind == Bass::Held {
            env(t, 0.01, BAR - 0.1, 0.1)
        } else {
            env(local, 0.003, eighth - 0.03, 0.02)
        };
        lp * gate * (0.55 + 0.45 * pluck)
    });
}

fn kick(m: &mut Mix, at: f32) {
    m.kicks.push(at);
    let mut phase = 0.0f32;
    m.voice(at, 0.45, 0.55, 0.0, 0.0, move |t, _| {
        let f = 46.0 + 120.0 * (-t * 30.0).exp();
        phase += f / SR;
        (phase * TAU).sin() * (-t * 6.5).exp() + 0.3 * (-t * 300.0).exp()
    });
}

fn four_on_floor(m: &mut Mix, t0: f32) {
    for beat in 0..4 {
        kick(m, t0 + beat as f32 * BEAT);
    }
}

/// The 80s gated snare: a crack, then a dense noise tail that stops dead.
fn gated_snare(m: &mut Mix, at: f32, gain: f32) {
    let mut lp = 0.0f32;
    m.voice(at, 0.3, gain * 0.34, 0.05, 0.25, move |t, mm| {
        let n = mm.rnd();
        lp += (n - lp) * 0.45;
        let crack = 0.8 * n * (-t * 25.0).exp() + 0.6 * (TAU * 190.0 * t).sin() * (-t * 20.0).exp();
        let gate = if t < 0.17 {
            1.0
        } else {
            (1.0 - (t - 0.17) / 0.015).max(0.0)
        };
        crack + 0.42 * lp * gate * (1.0 - 0.3 * t / 0.17)
    });
}

/// Six toms down the kit over the last two beats of a bar.
fn tom_fill(m: &mut Mix, t0: f32) {
    for k in 0..6 {
        let at = t0 + 2.0 * BEAT + k as f32 * BEAT / 3.0;
        let base = 210.0 - k as f32 * 22.0;
        let pan = 0.6 - k as f32 * 0.24;
        let mut phase = 0.0f32;
        m.voice(at, 0.4, 0.3, pan, 0.2, move |t, mm| {
            let f = base * (1.0 + 0.6 * (-t * 22.0).exp());
            phase += f / SR;
            (phase * TAU).sin() * (-t * 7.0).exp() + 0.15 * mm.rnd() * (-t * 40.0).exp()
        });
    }
}

/// White noise opening up over a bar, into the next section.
fn riser(m: &mut Mix, t0: f32) {
    let mut lp = 0.0f32;
    m.voice(t0, BAR, 0.12, 0.0, 0.4, move |t, mm| {
        let k = t / BAR;
        let n = mm.rnd();
        lp += (n - lp) * (0.01 + 0.5 * k * k);
        lp * k * k
    });
}

fn hats(m: &mut Mix, t0: f32, per_beat: usize) {
    let steps = 4 * per_beat;
    for k in 0..steps {
        if per_beat == 2 && k % 2 == 0 {
            continue;
        }
        let at = t0 + k as f32 * BEAT / per_beat as f32;
        // An open hat on every off-beat eighth of the 16th pattern.
        let open = per_beat == 4 && k % 4 == 2;
        let accent = if k % 2 == 1 { 0.7 } else { 1.0 };
        let decay = if open { 14.0 } else { 70.0 };
        let mut prev = 0.0f32;
        m.voice(
            at,
            if open { 0.25 } else { 0.06 },
            0.075 * accent,
            0.3,
            0.0,
            move |t, mm| {
                let n = mm.rnd();
                let hp = n - prev;
                prev = n;
                hp * (-t * decay).exp()
            },
        );
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Osc {
    /// Three detuned saws, filtered.
    Super,
    /// A pulse with slow width modulation, tracker style.
    Pulse,
    /// Two-operator FM brass.
    Brass,
}

/// A four-bar phrase as one monophonic voice that bends: legato notes glide
/// into each other, notes after a rest and held notes scoop up from two
/// semitones below, every note longer than an eighth grows a vibrato, and
/// with `dive` the last note falls an octave.
fn lead(m: &mut Mix, t0: f32, phrase: &[(u8, u8, u8)], up: f32, osc: Osc, gain: f32, dive: bool) {
    let eighth = BEAT / 2.0;
    let notes: Vec<(f32, f32, f32, bool)> = phrase
        .iter()
        .enumerate()
        .map(|(i, &(s, l, n))| {
            let start = s as f32 * eighth;
            let legato = i > 0 && {
                let (ps, pl, _) = phrase[i - 1];
                ps + pl == s
            };
            (start, l as f32 * eighth, n as f32 + up, legato)
        })
        .collect();
    let last = notes.len() - 1;
    let mut idx = usize::MAX;
    let mut pitch = notes[0].2 - 2.0;
    let mut k_glide = coeff(0.05);
    let mut phases = [0.0f32; 3];
    let mut mod_phase = 0.0f32;
    let mut lp = 0.0f32;
    let mut amp = 0.0f32;
    let (k_on, k_off) = (coeff(0.004), coeff(0.08));
    let (pan, send, level) = match osc {
        Osc::Super => (-0.12, 0.45, 0.075),
        Osc::Pulse => (0.18, 0.4, 0.06),
        Osc::Brass => (0.05, 0.5, 0.085),
    };
    let total = 16.0 * BEAT + if dive { 0.9 } else { 0.3 };
    m.voice(t0, total, level * gain, pan, send, move |t, _| {
        while idx.wrapping_add(1) <= last && notes[idx.wrapping_add(1)].0 <= t {
            idx = idx.wrapping_add(1);
            let (_, len, n, legato) = notes[idx];
            if legato && len < 3.0 * eighth {
                k_glide = coeff(0.045);
            } else {
                // Scoop up into the note.
                pitch = pitch.min(n - 2.0);
                if !legato {
                    pitch = n - 2.0;
                }
                k_glide = coeff(if len >= 3.0 * eighth { 0.09 } else { 0.05 });
            }
        }
        if idx == usize::MAX {
            return 0.0;
        }
        let (start, len, n, _) = notes[idx];
        let into = t - start;
        let next_legato = idx < last && notes[idx + 1].3;
        let mut gate = into < len - if next_legato { 0.0 } else { 0.02 };
        let mut target = n;
        if dive && idx == last {
            let from = len * 0.5;
            target -= 12.0 * smooth((into - from) / (len - from + 0.6));
            gate = into < len + 0.6;
        }
        pitch += (target - pitch) * k_glide;
        let vib = 0.3 * (TAU * 5.6 * into).sin() * smooth((into - 0.18) / 0.3);
        let f = hz(pitch + vib);
        amp += ((gate as i32 as f32) - amp) * if gate { k_on } else { k_off };
        let s = match osc {
            Osc::Super => {
                let mut sum = 0.0;
                for (j, d) in [1.0f32, 1.007, 0.993].iter().enumerate() {
                    phases[j] = (phases[j] + f * d / SR).fract();
                    sum += saw(phases[j]);
                }
                let bright = 0.12 + 0.2 * (-into * 4.0).exp();
                lp += (sum / 3.0 - lp) * bright;
                lp
            }
            Osc::Pulse => {
                phases[0] = (phases[0] + f / SR).fract();
                let duty = 0.3 + 0.15 * (TAU * 0.7 * (start + into)).sin();
                lp += (pulse(phases[0], duty) - lp) * 0.3;
                lp
            }
            Osc::Brass => {
                phases[0] = (phases[0] + f / SR).fract();
                mod_phase = (mod_phase + 2.0 * f / SR).fract();
                let index = 0.5 + 1.8 * (-into * 5.0).exp();
                (TAU * phases[0] + index * (TAU * mod_phase).sin()).sin()
            }
        };
        s * amp
    });
}

fn blip(m: &mut Mix, at: f32, f: f32, pan: f32) {
    m.voice(at, 0.14, 0.05, pan, 0.4, move |t, _| {
        (TAU * f * t).sin() * (-t * 30.0).exp()
    });
}

fn mixdown(m: Mix) -> Vec<[f32; 2]> {
    let len = m.dry.len();
    let mut out = m.dry;

    // The pad ducks under every kick.
    let mut kicks = m.kicks;
    kicks.sort_by(f32::total_cmp);
    let mut ki = 0;
    for (i, frame) in out.iter_mut().enumerate() {
        let t = i as f32 / SR;
        while ki + 1 < kicks.len() && kicks[ki + 1] <= t {
            ki += 1;
        }
        let duck = match kicks.get(ki) {
            Some(&k) if k <= t => 1.0 - 0.55 * (-(t - k) * 9.0).exp(),
            _ => 1.0,
        };
        frame[0] += m.pad[i][0] * duck;
        frame[1] += m.pad[i][1] * duck;
    }

    // Ping-pong delay on the send: a dotted eighth.
    let d = (BEAT * 0.75 * SR) as usize;
    let mut l = vec![0.0f32; len];
    let mut r = vec![0.0f32; len];
    for i in 0..len {
        let fb_l = if i >= d { r[i - d] * 0.42 } else { 0.0 };
        let fb_r = if i >= d { l[i - d] * 0.42 } else { 0.0 };
        l[i] = m.send[i] + fb_l;
        r[i] = fb_r;
    }
    for i in 0..len {
        out[i][0] += 0.32 * l[i];
        out[i][1] += 0.32 * r[i];
    }

    // Soft clip, then normalise to a fixed peak.
    let mut peak = 0.0f32;
    for frame in out.iter_mut() {
        for s in frame.iter_mut() {
            *s = (*s * 1.2).tanh();
            peak = peak.max(s.abs());
        }
    }
    let g = if peak > 0.0 { 0.89 / peak } else { 1.0 };
    for (i, frame) in out.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let fade = (t / 0.3).min(1.0) * ((DURATION - t) / 1.5).clamp(0.0, 1.0);
        frame[0] *= g * fade;
        frame[1] *= g * fade;
    }
    out
}
