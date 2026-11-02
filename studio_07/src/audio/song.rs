// STUDIO_07 soundtrack: 120 BPM in A minor, Am - F - C - G, one chord per
// bar. The arrangement follows the picture's sections bar for bar (see
// timeline.rs): a pad and a soft arpeggio for the name, the kick arrives
// over the city, the full groove under the route search -- where every kept
// move of the search is also a blip in the mix -- a bell for the advisory
// blueprint, a saw lead for the agents, claps and a square lead for Redona,
// everything for the scroller, and back to the pad for the address.
//
// Every note is rendered once into buffers at startup; live playback and
// the WAV export read the same samples.
use crate::timeline::{bars, now, Section, BAR, BEAT, DURATION, TOTAL_BARS};

pub const SAMPLE_RATE: u32 = 44_100;
const SR: f32 = SAMPLE_RATE as f32;
const TAU: f32 = std::f32::consts::TAU;

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

// Am, F, C, G: bass roots and pad voicings.
const ROOTS: [f32; 4] = [33.0, 29.0, 36.0, 31.0];
const PADS: [[f32; 4]; 4] = [
    [57.0, 60.0, 64.0, 67.0], // Am7
    [53.0, 57.0, 60.0, 64.0], // Fmaj7
    [55.0, 60.0, 64.0, 67.0], // C/G
    [55.0, 59.0, 62.0, 67.0], // G
];

/// The hook, four bars on an eighth-note grid: (start, length, midi).
const HOOK: [(u8, u8, u8); 19] = [
    (0, 2, 76),
    (2, 1, 79),
    (3, 1, 81),
    (4, 2, 79),
    (6, 2, 76),
    (8, 2, 77),
    (10, 1, 76),
    (11, 1, 72),
    (12, 4, 69),
    (16, 2, 72),
    (18, 1, 76),
    (19, 1, 79),
    (20, 2, 84),
    (22, 2, 79),
    (24, 2, 79),
    (26, 1, 77),
    (27, 1, 74),
    (28, 3, 71),
    (31, 1, 74),
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

pub fn render(move_times: &[f32]) -> Vec<[f32; 2]> {
    let len = (DURATION * SR) as usize + 1;
    let mut m = Mix::new(len);

    for b in 0..TOTAL_BARS as usize {
        let t0 = bars(b as f32);
        let section = now(t0 + 0.01).section;
        let next = now(t0 + BAR + 0.01).section;
        let chord = b % 4;
        let last_bar = b + 1 == TOTAL_BARS as usize;

        pad(&mut m, t0, chord, if last_bar { 3.5 } else { BAR + 0.35 });
        let groove = !matches!(section, Section::Boot | Section::Contact);
        if section != Section::Boot || b >= 2 {
            arp(&mut m, t0, chord, if groove { 1.0 } else { 0.6 });
        }
        match section {
            Section::Boot => {}
            Section::City => {
                bass(&mut m, t0, chord, false);
                for beat in [0.0, 2.0] {
                    kick(&mut m, t0 + beat * BEAT);
                }
                if b >= 10 {
                    hats(&mut m, t0, 2);
                }
            }
            Section::Plan | Section::Agents | Section::Work => {
                bass(&mut m, t0, chord, true);
                four_on_floor(&mut m, t0);
                snares(&mut m, t0);
                hats(&mut m, t0, 4);
            }
            Section::Advice => {
                bass(&mut m, t0, chord, true);
                four_on_floor(&mut m, t0);
                hats(&mut m, t0, 2);
            }
            Section::Redona => {
                bass_octaves(&mut m, t0, chord);
                four_on_floor(&mut m, t0);
                claps(&mut m, t0);
                hats(&mut m, t0, 4);
            }
            Section::Contact => {
                if b < 64 {
                    bass(&mut m, t0, chord, false);
                }
            }
        }
        // A snare roll into the big sections.
        if next != section && matches!(next, Section::Plan | Section::Agents | Section::Work) {
            for k in 0..8 {
                let at = t0 + 2.0 * BEAT + k as f32 * BEAT / 4.0;
                snare(&mut m, at, 0.35 + 0.08 * k as f32);
            }
        }

        // Melody, four-bar phrases aligned to the section.
        let phrase_bar = (b - first_bar(section)) % 4;
        if phrase_bar == 0 {
            match section {
                Section::Advice => hook(&mut m, t0, Lead::Bell, 12.0),
                Section::Agents => hook(&mut m, t0, Lead::Saw, 0.0),
                Section::Redona => hook(&mut m, t0, Lead::Square, 0.0),
                Section::Work => {
                    hook(&mut m, t0, Lead::Saw, 0.0);
                    hook(&mut m, t0, Lead::Bell, 12.0);
                }
                Section::Contact if b < 64 => hook(&mut m, t0, Lead::Bell, 0.0),
                _ => {}
            }
        }
    }

    // A blip for every kept move of the route search.
    let mut last = -1.0f32;
    for (i, &at) in move_times.iter().enumerate() {
        if at - last < 0.07 {
            continue;
        }
        last = at;
        let chord = ((at / BAR) as usize) % 4;
        let note = PADS[chord][i % 4] + 24.0;
        blip(&mut m, at, hz(note), if i % 2 == 0 { -0.5 } else { 0.5 });
    }

    mixdown(m)
}

fn first_bar(section: Section) -> usize {
    crate::timeline::SECTIONS
        .iter()
        .find(|(s, _, _)| *s == section)
        .map(|(_, b0, _)| *b0 as usize)
        .unwrap_or(0)
}

fn pad(m: &mut Mix, t0: f32, chord: usize, dur: f32) {
    for (k, &note) in PADS[chord].iter().enumerate() {
        for (d, det) in [-0.07f32, 0.0, 0.07].iter().enumerate() {
            let f = hz(note + det);
            let pan = (k as f32 - 1.5) * 0.25 + (d as f32 - 1.0) * 0.2;
            let i0 = (t0 * SR) as usize;
            let n = (dur * SR) as usize;
            let mut lp = 0.0f32;
            let phase0 = (k * 3 + d) as f32 * 0.21;
            for s in 0..n {
                let i = i0 + s;
                if i >= m.pad.len() {
                    break;
                }
                let t = s as f32 / SR;
                let raw = saw(phase0 + f * t);
                lp += (raw - lp) * 0.08;
                let e = env(t, 0.35, dur - 0.45, 0.45);
                let v = lp * e * 0.022;
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
            0.05 * gain,
            if step % 2 == 0 { -0.35 } else { 0.35 },
            0.6,
            move |t, _| {
                let raw = pulse(f * t, 0.25);
                lp += (raw - lp) * 0.25;
                lp * (-t * 18.0).exp()
            },
        );
    }
}

fn bass(m: &mut Mix, t0: f32, chord: usize, eighths: bool) {
    let root = ROOTS[chord];
    let steps: &[(f32, f32, f32)] = if eighths {
        &[
            (0.0, 0.45, 0.0),
            (0.5, 0.45, 12.0),
            (1.0, 0.45, 0.0),
            (1.5, 0.45, 12.0),
            (2.0, 0.45, 0.0),
            (2.5, 0.45, 12.0),
            (3.0, 0.45, 0.0),
            (3.5, 0.45, 7.0),
        ]
    } else {
        &[(0.0, 3.8, 0.0)]
    };
    for &(beat, len, up) in steps {
        let f = hz(root + up + 12.0);
        let mut lp = 0.0f32;
        let dur = len * BEAT;
        m.voice(t0 + beat * BEAT, dur + 0.05, 0.16, 0.0, 0.0, move |t, _| {
            let raw = 0.6 * saw(f * t) + 0.4 * pulse(f * t, 0.5);
            lp += (raw - lp) * 0.06;
            lp * env(t, 0.004, dur - 0.04, 0.05)
        });
    }
}

fn bass_octaves(m: &mut Mix, t0: f32, chord: usize) {
    let root = ROOTS[chord];
    for k in 0..8 {
        let up = if k % 2 == 1 { 24.0 } else { 12.0 };
        let f = hz(root + up);
        let mut lp = 0.0f32;
        m.voice(
            t0 + k as f32 * BEAT / 2.0,
            0.24,
            0.15,
            0.0,
            0.0,
            move |t, _| {
                let raw = pulse(f * t, 0.5);
                lp += (raw - lp) * 0.09;
                lp * env(t, 0.003, 0.16, 0.06)
            },
        );
    }
}

fn kick(m: &mut Mix, at: f32) {
    m.kicks.push(at);
    let mut phase = 0.0f32;
    m.voice(at, 0.45, 0.55, 0.0, 0.0, move |t, _| {
        let f = 48.0 + 110.0 * (-t * 32.0).exp();
        phase += f / SR;
        (phase * TAU).sin() * (-t * 7.0).exp() + 0.3 * (-t * 300.0).exp()
    });
}

fn four_on_floor(m: &mut Mix, t0: f32) {
    for beat in 0..4 {
        kick(m, t0 + beat as f32 * BEAT);
    }
}

fn snare(m: &mut Mix, at: f32, gain: f32) {
    m.voice(at, 0.3, gain * 0.32, 0.05, 0.15, move |t, mm| {
        let n = mm.rnd();
        0.7 * n * (-t * 16.0).exp() + 0.5 * (TAU * 185.0 * t).sin() * (-t * 22.0).exp()
    });
}

fn snares(m: &mut Mix, t0: f32) {
    for beat in [1.0, 3.0] {
        snare(m, t0 + beat * BEAT, 1.0);
    }
}

fn claps(m: &mut Mix, t0: f32) {
    for beat in [1.0, 3.0] {
        m.voice(t0 + beat * BEAT, 0.25, 0.3, -0.1, 0.2, move |t, mm| {
            let n = mm.rnd();
            let burst = [0.0, 0.011, 0.022]
                .iter()
                .map(|o| {
                    if t >= *o {
                        (-(t - o) * 90.0).exp()
                    } else {
                        0.0
                    }
                })
                .sum::<f32>();
            n * (burst * 0.6 + 0.5 * (-t * 14.0).exp())
        });
    }
}

fn hats(m: &mut Mix, t0: f32, per_beat: usize) {
    let steps = 4 * per_beat;
    for k in 0..steps {
        // Eighth-note hats sit on the off-beats.
        if per_beat == 2 && k % 2 == 0 {
            continue;
        }
        let at = t0 + k as f32 * BEAT / per_beat as f32;
        let accent = if k % 2 == 1 { 1.0 } else { 0.6 };
        let mut prev = 0.0f32;
        m.voice(at, 0.06, 0.08 * accent, 0.3, 0.0, move |t, mm| {
            let n = mm.rnd();
            let hp = n - prev;
            prev = n;
            hp * (-t * 70.0).exp()
        });
    }
}

#[derive(Clone, Copy)]
enum Lead {
    Saw,
    Square,
    Bell,
}

fn hook(m: &mut Mix, t0: f32, lead: Lead, up: f32) {
    let eighth = BEAT / 2.0;
    for &(start, len, note) in HOOK.iter() {
        let f = hz(note as f32 + up);
        let at = t0 + start as f32 * eighth;
        let dur = len as f32 * eighth;
        match lead {
            Lead::Saw => {
                let mut lp = 0.0f32;
                m.voice(at, dur + 0.12, 0.075, -0.15, 0.45, move |t, _| {
                    let vib =
                        1.0 + 0.004 * (TAU * 5.5 * t).sin() * ((t - 0.12) * 4.0).clamp(0.0, 1.0);
                    let raw = 0.5 * saw(f * vib * t) + 0.5 * saw(f * vib * 1.004 * t + 0.3);
                    lp += (raw - lp) * 0.2;
                    lp * env(t, 0.01, dur - 0.02, 0.12)
                });
            }
            Lead::Square => {
                let mut lp = 0.0f32;
                m.voice(at, dur + 0.08, 0.065, 0.15, 0.35, move |t, _| {
                    let raw = pulse(f * t, 0.5);
                    lp += (raw - lp) * 0.3;
                    lp * env(t, 0.005, dur - 0.04, 0.08)
                });
            }
            Lead::Bell => {
                m.voice(at, 1.4, 0.07, 0.25, 0.55, move |t, _| {
                    let idx = 2.2 * (-t * 5.0).exp();
                    let modu = (TAU * f * 3.5 * t).sin() * idx;
                    (TAU * f * t + modu).sin() * (-t * 3.2).exp() * env(t, 0.003, 10.0, 1.0)
                });
            }
        }
    }
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
