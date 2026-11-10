// ROUTING_10 — "Switchback", 120 BPM, F harmonic minor.
// Eight-bar sentence: a four-bar question, its recognisable repeated opening,
// then a shorter dominant answer resolving to F. The rhythm section shares a
// straight eighth-note grid; no octave bass jumps, scoops or pitch dives.
// Only the opening two bars are foreground; the rest is a restrained statement.
// The live section loops a speech bed and the outro recalls the opening before
// its dominant/tonic cadence. Section lengths come from the existing timeline.
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

// Close-position voices: each pad part moves by at most three semitones.
// Stable pitches are F G Ab Bb C Db E; roots stay in a single bass register.
const ROOTS: [f32; 4] = [41.0, 37.0, 34.0, 36.0];
const PADS: [[f32; 4]; 4] = [
    [60.0, 65.0, 68.0, 72.0], // Fm
    [61.0, 65.0, 68.0, 72.0], // Dbmaj7: three common tones
    [61.0, 65.0, 70.0, 73.0], // Bbm
    [60.0, 64.0, 67.0, 72.0], // C: E rises to F at the resolution
];
const STATEMENT: [usize; 8] = [0, 1, 2, 3, 0, 1, 3, 0];
const BED: [usize; 8] = [0, 1, 2, 3, 0, 1, 3, 0];

// (sixteenth onset, duration, MIDI). This is a composed eight-bar sentence,
// not shuffled hooks. A and B share their first two bars exactly; the second
// ending contracts the rhythm and resolves E to F instead of adding a fill.
const HOOK_A: [(u8, u8, u8); 15] = [
    (0, 4, 72),
    (4, 4, 77),
    (8, 2, 79),
    (10, 4, 80),
    (16, 6, 80),
    (22, 2, 79),
    (24, 6, 77),
    (32, 4, 77),
    (36, 4, 80),
    (40, 2, 79),
    (42, 4, 77),
    (48, 4, 76),
    (52, 4, 79),
    (56, 4, 76),
    (60, 4, 72),
];
const HOOK_B: [(u8, u8, u8); 12] = [
    (0, 4, 72),
    (4, 4, 77),
    (8, 2, 79),
    (10, 4, 80),
    (16, 6, 80),
    (22, 2, 79),
    (24, 6, 77),
    (32, 4, 76),
    (36, 4, 77),
    (40, 4, 79),
    (44, 4, 76),
    (48, 12, 77),
];
const CODA: [(u8, u8, u8); 4] = [(0, 4, 76), (4, 4, 79), (8, 6, 76), (16, 12, 77)];

fn harmony(section: Section, local_bar: usize) -> usize {
    match section {
        Section::Intro => STATEMENT[local_bar % STATEMENT.len()],
        Section::Live => {
            if local_bar + 1 == section_bars(section) {
                3 // A single dominant bar prepares the returning F-minor hook.
            } else {
                BED[local_bar % BED.len()]
            }
        }
        Section::Outro => {
            let length = section_bars(section);
            if local_bar + 2 == length {
                3
            } else if local_bar + 1 == length {
                0
            } else {
                STATEMENT[local_bar % STATEMENT.len()]
            }
        }
    }
}

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
        let k = b - first_bar(section);
        let chord = harmony(section, k);
        let left = section_bars(section) - k;
        pad(&mut m, t0, chord, BAR + 0.30, b);
        match section {
            Section::Intro => {
                // Two-bar opening, then step back rather than escalating an intro.
                let foreground = k < 2;
                let gain = if foreground { 0.85 } else { 0.42 };
                bass(&mut m, t0, chord, Bass::Drive, gain);
                groove(&mut m, t0, gain);
                arp(&mut m, t0, chord, if foreground { 0.32 } else { 0.12 });
                if k.is_multiple_of(8) {
                    lead(&mut m, t0, &HOOK_A, 0.0, Osc::Brass, 0.65);
                }
                if k % 8 == 4 {
                    lead(&mut m, t0, &HOOK_B, 0.0, Osc::Brass, 0.48);
                }
            }
            Section::Live => {
                // Preserve the pulse below speech, without a competing melody.
                bass(&mut m, t0, chord, Bass::Drive, 0.38);
                hats(&mut m, t0, 0.12);
                arp(&mut m, t0, chord, 0.10);
                // A single restrained pickup, only at the return of the hook.
                if left == 1 {
                    tom_fill(&mut m, t0);
                    riser(&mut m, t0 + BAR * 0.5);
                }
            }
            Section::Outro => {
                if left > 2 {
                    bass(&mut m, t0, chord, Bass::Drive, 0.85);
                    groove(&mut m, t0, 0.85);
                    arp(&mut m, t0, chord, 0.25);
                    if k.is_multiple_of(8) {
                        lead(&mut m, t0, &HOOK_A, 0.0, Osc::Super, 0.85);
                    } else if k % 8 == 4 && left >= 4 {
                        lead(&mut m, t0, &HOOK_B, 0.0, Osc::Super, 0.85);
                    }
                } else {
                    bass(&mut m, t0, chord, Bass::Held, 0.65);
                    if left == 2 {
                        lead(&mut m, t0, &CODA, 0.0, Osc::Pulse, 0.85);
                    } else {
                        kick(&mut m, t0, 0.65);
                    }
                }
            }
        }
    }

    // Keep the API but do not turn arbitrary UI event times into random musical
    // accents: the workflow edit supplies its own clicks, not a second melody.
    let _ = landings;

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
    // Quiet two-note offbeat reply, in a fixed register below the lead.
    for (step, tone) in [(6, 1), (14, 2)] {
        let note = notes[tone];
        let f = hz(note);
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
                lp * env(t, 0.004, 0.14, 0.06) * (-t * 18.0).exp()
            },
        );
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Bass {
    Drive,
    Held,
}

// The bass reinforces each kick and the offbeat hats on the same eighth grid.
// Root-only articulation is intentional: harmonic movement is between bars,
// not a competing tune or a seven-event root/fifth/octave pattern.
fn bass(m: &mut Mix, t0: f32, chord: usize, kind: Bass, gain: f32) {
    let root = ROOTS[chord];
    let mut phase = 0.0f32;
    let mut lp = 0.0f32;
    m.voice(t0, BAR, 0.22 * gain, 0.0, 0.0, move |t, _| {
        let local = if kind == Bass::Held {
            t
        } else {
            t % (BEAT / 2.0)
        };
        phase = (phase + hz(root) / SR).fract();
        let pluck = (-local * 13.0).exp();
        let raw = 0.45 * saw(phase) + 0.55 * (TAU * phase).sin();
        lp += (raw - lp) * (0.045 + 0.12 * pluck);
        let duration = if kind == Bass::Held {
            BAR
        } else {
            BEAT / 2.0 - 0.03
        };
        lp * env(local, 0.008, duration - 0.035, 0.035) * (0.6 + 0.4 * pluck)
    });
}

fn kick(m: &mut Mix, at: f32, gain: f32) {
    m.kicks.push(at);
    let mut phase = 0.0f32;
    m.voice(at, 0.45, 0.45 * gain, 0.0, 0.0, move |t, _| {
        let f = 46.0 + 120.0 * (-t * 30.0).exp();
        phase += f / SR;
        (phase * TAU).sin() * env(t, 0.001, 0.35, 0.10) * (-t * 9.0).exp()
    });
}

fn groove(m: &mut Mix, t0: f32, gain: f32) {
    for beat in 0..4 {
        kick(m, t0 + beat as f32 * BEAT, gain);
    }
    gated_snare(m, t0 + BEAT, 0.65 * gain);
    gated_snare(m, t0 + 3.0 * BEAT, 0.65 * gain);
    hats(m, t0, gain * 0.65);
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

/// One two-tom pickup, not a fill superimposed on each four-bar phrase.
fn tom_fill(m: &mut Mix, t0: f32) {
    for (beat, base, pan) in [(3.0, 147.0, -0.2), (3.5, 110.0, 0.2)] {
        let mut phase = 0.0f32;
        m.voice(t0 + beat * BEAT, 0.24, 0.12, pan, 0.12, move |t, _| {
            phase += base / SR;
            (phase * TAU).sin() * env(t, 0.003, 0.16, 0.08) * (-t * 15.0).exp()
        });
    }
}

/// White noise opening up over a bar, into the next section.
fn riser(m: &mut Mix, t0: f32) {
    let mut lp = 0.0f32;
    let duration = if now(t0).section == Section::Live {
        crate::timeline::LIVE_TO - t0
    } else {
        BAR
    };
    m.voice(t0, duration, 0.035, 0.0, 0.4, move |t, mm| {
        let k = t / duration;
        let n = mm.rnd();
        lp += (n - lp) * (0.01 + 0.5 * k * k);
        lp * k * k * env(t, 0.05, duration - 0.04, 0.04)
    });
}

fn hats(m: &mut Mix, t0: f32, gain: f32) {
    for k in 0..8 {
        let at = t0 + k as f32 * BEAT / 2.0;
        let accent = if k % 2 == 1 { 1.0 } else { 0.45 };
        let mut prev = 0.0f32;
        m.voice(at, 0.07, 0.045 * accent * gain, 0.25, 0.0, move |t, mm| {
            let n = mm.rnd();
            let hp = n - prev;
            prev = n;
            hp * env(t, 0.001, 0.04, 0.03) * (-t * 65.0).exp()
        });
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

/// Continuous oscillator phase, but stable note attacks: no scoop or dive.
/// Only adjacent legato scale steps receive a six-millisecond portamento;
/// delayed five-cent vibrato is colour, not a second pitch contour.
fn lead(m: &mut Mix, t0: f32, phrase: &[(u8, u8, u8)], up: f32, osc: Osc, gain: f32) {
    let eighth = BEAT / 4.0;
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
    let mut pitch = notes[0].2;
    let mut k_glide = coeff(0.006);
    let mut phases = [0.0f32; 3];
    let mut mod_phase = 0.0f32;
    let mut fm_index = 0.5f32;
    let mut lp = 0.0f32;
    let mut amp = 0.0f32;
    let (k_on, k_off) = (coeff(0.004), coeff(0.018));
    let (pan, send, level) = match osc {
        Osc::Super => (-0.12, 0.45, 0.075),
        Osc::Pulse => (0.18, 0.4, 0.06),
        Osc::Brass => (0.05, 0.5, 0.085),
    };
    let &(end_start, end_len, _) = phrase.last().expect("nonempty lead phrase");
    let phrase_duration = (end_start + end_len) as f32 * eighth + 0.15;
    let clock = now(t0);
    let total = phrase_duration.min(clock.len - clock.local);
    m.voice(t0, total, level * gain, pan, send, move |t, _| {
        while idx.wrapping_add(1) <= last && notes[idx.wrapping_add(1)].0 <= t {
            idx = idx.wrapping_add(1);
            let (_, _, n, legato) = notes[idx];
            if legato && (pitch - n).abs() <= 2.0 {
                k_glide = coeff(0.006);
            } else {
                pitch = n;
                k_glide = 1.0;
            }
        }
        if idx == usize::MAX {
            return 0.0;
        }
        let (start, len, n, _) = notes[idx];
        let into = t - start;
        let next_legato = idx < last && notes[idx + 1].3;
        let gate = into < len - if next_legato { 0.0 } else { 0.035 };
        let target = n;
        pitch += (target - pitch) * k_glide;
        let vib = 0.05 * (TAU * 5.6 * into).sin() * smooth((into - 0.18) / 0.3);
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
                fm_index += (index - fm_index) * coeff(0.004);
                (TAU * phases[0] + fm_index * (TAU * mod_phase).sin()).sin()
            }
        };
        s * amp * smooth((total - t) / 0.04)
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

    // DC blocker also removes residual low-frequency bias from finite saw notes.
    let mut previous_in = [0.0f32; 2];
    let mut previous_out = [0.0f32; 2];
    let pole = (-TAU * 15.0 / SR).exp();
    for frame in &mut out {
        for ch in 0..2 {
            let input = frame[ch];
            let high = input - previous_in[ch] + pole * previous_out[ch];
            previous_in[ch] = input;
            previous_out[ch] = high;
            frame[ch] = high;
        }
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
        // Explicit broadband speech duck also catches delay and transition tails.
        let speech = if (crate::timeline::LIVE_FROM..crate::timeline::LIVE_TO).contains(&t) {
            0.30
        } else if t < crate::timeline::LIVE_FROM {
            let end = crate::timeline::LIVE_FROM.min(2.0 * BAR);
            let start = (end - BAR).max(0.0);
            1.0 - 0.70 * smooth((t - start) / (end - start).max(0.001))
        } else {
            0.30 + 0.70 * smooth((t - crate::timeline::LIVE_TO) / (BEAT * 0.5))
        };
        frame[0] *= g * fade * speech;
        frame[1] *= g * fade * speech;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_has_legal_pitches_space_and_a_tonic_cadence() {
        let scale = [0, 1, 4, 5, 7, 8, 10]; // F harmonic minor, absolute pitch classes
        for note in ROOTS.iter().chain(PADS.iter().flatten()) {
            assert!(scale.contains(&(*note as u8 % 12)), "pitch {note}");
        }
        for phrase in [&HOOK_A[..], &HOOK_B[..], &CODA[..]] {
            for &(start, length, note) in phrase {
                assert!(length > 0 && start + length <= 64);
                assert!(scale.contains(&(note % 12)));
            }
            for pair in phrase.windows(2) {
                assert!(pair[0].0 + pair[0].1 <= pair[1].0);
                assert!((pair[0].2 as i16 - pair[1].2 as i16).abs() <= 5);
            }
            assert!(phrase.windows(2).any(|p| p[0].0 + p[0].1 < p[1].0));
        }
        assert_eq!(&HOOK_A[..7], &HOOK_B[..7]);
        assert_eq!(HOOK_B.last().unwrap().2 % 12, 5);
        assert_eq!(CODA.last().unwrap().2 % 12, 5);
        for progression in [&STATEMENT[..], &BED[..]] {
            for pair in progression.windows(2).chain(std::iter::once(&[3, 0][..])) {
                for voice in 0..4 {
                    assert!((PADS[pair[0]][voice] - PADS[pair[1]][voice]).abs() <= 3.0);
                }
                assert!((ROOTS[pair[0]] - ROOTS[pair[1]]).abs() <= 7.0);
            }
        }
        // Include phrase joins, not only intervals inside independent tables.
        let whole: Vec<_> = HOOK_A.iter().chain(HOOK_B.iter()).collect();
        for pair in whole.windows(2) {
            assert!((pair[0].2 as i16 - pair[1].2 as i16).abs() <= 5);
        }
        assert_eq!(harmony(Section::Outro, section_bars(Section::Outro) - 2), 3);
        assert_eq!(harmony(Section::Outro, section_bars(Section::Outro) - 1), 0);
    }

    #[test]
    fn complete_render_is_finite_repeatable_and_ducked() {
        let events = [1.0, 7.5, 15.5, 20.0, 45.0];
        let a = render(&events);
        let b = render(&events);
        assert_eq!(a, b);
        assert_eq!(a.len(), (DURATION * SR) as usize + 1);
        assert!(a
            .iter()
            .flatten()
            .all(|s| s.is_finite() && s.abs() <= 0.891));
        let rms = |start: f32, end: f32| {
            let frames = &a[(start * SR) as usize..(end * SR) as usize];
            (frames.iter().flatten().map(|s| s * s).sum::<f32>() / (frames.len() * 2) as f32).sqrt()
        };
        let live_start = crate::timeline::LIVE_FROM + BAR;
        let live_end = crate::timeline::LIVE_TO - BAR;
        if live_end > live_start && section_bars(Section::Outro) > 2 {
            let bed = rms(live_start, live_end);
            let hook = rms(crate::timeline::LIVE_TO + BEAT, DURATION - 2.0 * BAR);
            assert!(bed < hook * 0.50);
            assert!(bed > 0.0005);
        }
        assert!(a.last().unwrap().iter().all(|s| s.abs() < 0.0001));
    }
}
