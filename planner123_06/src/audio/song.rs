// PLANNER123_06 soundtrack: the Planner123 theme, rebuilt.
//
// From planner123_01: 125 BPM, the Cm - Ab - Bb - G loop, the C5/Bb4 lead
// riff, the 16th square bass, the arp shape and the first two phrases of the
// solo. The synth voices, drums and mix are the offline engine from hammer_05
// (copied, not shared). New: key clicks for every key the demo presses, a bell
// for each task the planner lands, and an arrangement that follows the
// picture's sections bar for bar (see timeline.rs).
//
// The whole track is rendered once, sample by sample, into a buffer. Live
// playback and the WAV export read the same samples.
use crate::timeline::{bars, landing, smooth, BAR, BEAT, DURATION, KEYS, STEP, TASK_COUNT};

pub const SAMPLE_RATE: u32 = 44_100;
const TAU: f32 = std::f32::consts::TAU;

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

// ── Harmony ──────────────────────────────────────────────────────────────────
// Planner123's loop, one chord per bar: Cm, Ab, Bb, G.
const ROOTS: [f32; 4] = [48.0, 44.0, 46.0, 43.0];
const PAD_VOICINGS: [[f32; 5]; 4] = [
    [48.0, 55.0, 62.0, 63.0, 70.0], // Cm9
    [44.0, 51.0, 60.0, 63.0, 67.0], // Abmaj7
    [46.0, 53.0, 60.0, 62.0, 65.0], // Bb add9
    [43.0, 50.0, 55.0, 62.0, 65.0], // G5 add b7
];

fn chord(t: f32) -> usize {
    ((t / BAR).floor() as usize) % 4
}

// Planner123's lead riff, 16th steps over 4 bars (0 = rest).
const PLANNER_RIFF: [u8; 64] = [
    72, 0, 70, 0, 72, 0, 72, 0, 70, 0, 72, 0, 72, 0, 70, 72, 68, 0, 68, 0, 67, 0, 68, 0, 68, 0, 68,
    0, 67, 0, 68, 0, 70, 0, 70, 0, 68, 0, 70, 0, 70, 0, 70, 0, 68, 0, 70, 0, 67, 0, 67, 0, 65, 0,
    67, 0, 67, 0, 70, 0, 70, 0, 70, 72,
];

// The first two phrases of the Planner123 solo (Hz, beats), from
// planner123_01 -- quoted over the agenda.
const PLANNER_SOLO: [(f32, f32); 52] = [
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
];

#[derive(Clone, Copy)]
struct Note {
    start: f32,
    dur: f32,
    freq: f32,
    vel: f32,
}

/// Planner riff re-cut to a 3-3-2 stab: hits on 16ths 0,3,6,8,11,14 of each
/// bar, each taking the riff note sounding at that step.
fn push_riff(out: &mut Vec<Note>, from_bar: f32, to_bar: f32, vel: f32) {
    let mut b = from_bar;
    while b < to_bar {
        for s in [0usize, 3, 6, 8, 11, 14] {
            let step = ((b as usize % 4) * 16) + s;
            let mut k = step;
            while PLANNER_RIFF[k] == 0 {
                k -= 1;
            }
            out.push(Note {
                start: bars(b) + s as f32 * STEP,
                dur: STEP * 1.6,
                freq: hz(PLANNER_RIFF[k] as f32),
                vel,
            });
        }
        b += 1.0;
    }
}

/// The Planner riff as written: every sounding 16th of the 4-bar line.
fn push_riff_full(out: &mut Vec<Note>, from_bar: f32, to_bar: f32, vel: f32) {
    let mut b = from_bar;
    while b < to_bar {
        for s in 0..16 {
            let m = PLANNER_RIFF[(b as usize % 4) * 16 + s];
            if m != 0 {
                out.push(Note {
                    start: bars(b) + s as f32 * STEP,
                    dur: STEP * 1.7,
                    freq: hz(m as f32),
                    vel,
                });
            }
        }
        b += 1.0;
    }
}

fn push_solo(out: &mut Vec<Note>, bar: f32, vel: f32) {
    let mut cursor = bars(bar);
    for &(f, beats) in &PLANNER_SOLO {
        out.push(Note {
            start: cursor,
            dur: beats * BEAT,
            freq: f,
            vel,
        });
        cursor += beats * BEAT;
    }
}

// ── Drum and effect events ─────────────────────────────────────────────────
#[derive(Default)]
struct Events {
    kick: Vec<(f32, f32)>,
    click: Vec<(f32, f32)>,
    snare: Vec<(f32, f32)>,
    hat: Vec<(f32, f32)>,
    open_hat: Vec<(f32, f32)>,
    crash: Vec<(f32, f32)>,
    boom: Vec<(f32, f32)>,
    lead: Vec<Note>,
    bell: Vec<Note>,
    stab: Vec<Note>,
    solo: Vec<Note>,
}

fn score() -> Events {
    let mut e = Events::default();
    let steps = (DURATION / STEP) as usize;
    for i in 0..steps {
        let t = i as f32 * STEP;
        let bar = t / BAR;
        let s = i % 16;
        let in_bars = |a: f32, b: f32| bar >= a && bar < b;

        // Kick: half time while the week loads, four-on-the-floor from the
        // month on; the inbox is a breakdown without it.
        if in_bars(8.0, 12.0) && (s == 0 || s == 8) {
            e.kick.push((t, 0.7));
        }
        if (in_bars(12.0, 20.0) || in_bars(24.0, 52.0)) && s % 4 == 0 {
            e.kick.push((t, 1.0));
        }
        // Drive 16ths while the planner works, like the Planner123 groove.
        if in_bars(24.0, 34.0) && matches!(s, 2 | 6 | 10 | 14) {
            e.kick.push((t, 0.32));
        }
        // Snare on 2 and 4.
        if (in_bars(12.0, 20.0) || in_bars(24.0, 52.0)) && (s == 4 || s == 12) {
            e.snare.push((t, 0.9));
        }
        // INBOX roll: 16ths rising, then 32nds into `o`.
        if in_bars(22.0, 24.0) {
            let rise = (bar - 22.0) / 2.0;
            e.snare.push((t, 0.25 + rise * 0.6));
            if bar >= 23.0 {
                e.snare.push((t + STEP * 0.5, 0.25 + rise * 0.6));
            }
        }
        // Hats: soft 8ths in the breakdown, full otherwise.
        if in_bars(8.0, 20.0) || in_bars(24.0, 52.0) {
            let v = match s % 4 {
                0 => 0.5,
                2 => 0.8,
                _ => 0.3,
            };
            e.hat.push((t, v));
            if s == 10 {
                e.open_hat.push((t, 0.6));
            }
        } else if in_bars(20.0, 22.0) && s % 2 == 0 {
            e.hat.push((t, 0.3));
        }
        // Crashes and booms on the big downbeats.
        let on = |b: f32| s == 0 && (bar - b).abs() < 1e-3;
        if [12.0, 24.0, 34.0, 38.0, 48.0].iter().any(|b| on(*b)) {
            e.crash.push((t, 1.0));
        }
        if [12.0, 24.0, 34.0].iter().any(|b| on(*b)) {
            e.boom.push((t, 1.0));
        }
    }

    // Every key the demo presses clicks: the command typed at the shell,
    // Enter, the view and planner keys, the j presses, and `/credits`.
    for k in 0..10 {
        e.click.push((bars(0.5 + k as f32 / 8.0), 0.7));
    }
    e.click.push((bars(2.0), 1.0));
    for &(b, _) in KEYS.iter() {
        e.click.push((bars(b), 1.0));
    }
    for k in 0..8 {
        e.click.push((bars(22.0 + k as f32 / 4.0), 0.6));
        e.click.push((bars(49.0 + k as f32 / 8.0), 0.6));
    }
    e.click.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // A bell for each task the planner lands, climbing the C minor chord.
    for (i, m) in [72.0, 75.0, 79.0, 82.0, 84.0]
        .iter()
        .enumerate()
        .take(TASK_COUNT)
    {
        e.bell.push(Note {
            start: landing(i),
            dur: BEAT,
            freq: hz(*m),
            vel: 0.9,
        });
    }
    // Enter, and the app ready.
    for (b, m) in [(2.0, 79.0), (4.0, 84.0)] {
        e.bell.push(Note {
            start: bars(b),
            dur: BEAT,
            freq: hz(m),
            vel: 0.5,
        });
    }

    // Melodic layers.
    push_riff_full(&mut e.stab, 12.0, 24.0, 0.8);
    push_riff_full(&mut e.stab, 24.0, 34.0, 1.0);
    push_riff(&mut e.stab, 34.0, 38.0, 0.9);
    push_riff_full(&mut e.stab, 48.0, 52.0, 0.9);
    push_riff_full(&mut e.lead, 28.0, 34.0, 0.8);
    push_riff_full(&mut e.lead, 48.0, 52.0, 0.8);
    push_solo(&mut e.solo, 39.0, 1.0);
    for (b, m) in [(52.0, 72.0), (54.0, 72.0)] {
        e.bell.push(Note {
            start: bars(b),
            dur: BAR * 2.0,
            freq: hz(m),
            vel: 0.8,
        });
    }
    e.bell
        .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
    e
}

// ── DSP building blocks ─────────────────────────────────────────────────────
struct Noise(u32);
impl Noise {
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Topology-preserving state-variable filter.
#[derive(Default)]
struct Svf {
    ic1: f32,
    ic2: f32,
}
impl Svf {
    /// Returns (low, band, high).
    fn run(&mut self, x: f32, cutoff: f32, q: f32) -> (f32, f32, f32) {
        let g = (std::f32::consts::PI * cutoff.clamp(20.0, 18_000.0) / SAMPLE_RATE as f32).tan();
        let k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let v3 = x - self.ic2;
        let v1 = a1 * self.ic1 + g * a1 * v3;
        let v2 = self.ic2 + g * v1;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        (v2, v1, x - k * v1 - v2)
    }
}

struct Delay {
    buf: Vec<f32>,
    pos: usize,
}
impl Delay {
    fn new(seconds: f32) -> Self {
        Self {
            buf: vec![0.0; (seconds * SAMPLE_RATE as f32) as usize + 1],
            pos: 0,
        }
    }
    fn tap(&self) -> f32 {
        self.buf[self.pos]
    }
    fn push(&mut self, x: f32) {
        self.buf[self.pos] = x;
        self.pos = (self.pos + 1) % self.buf.len();
    }
}

/// Small Schroeder reverb: four combs into two allpasses per channel.
struct Reverb {
    combs: Vec<(Delay, f32)>,
    allpass: Vec<Delay>,
}
impl Reverb {
    fn new(spread: f32) -> Self {
        let combs = [0.0297, 0.0371, 0.0411, 0.0437]
            .iter()
            .map(|s| (Delay::new(s * spread), 0.0))
            .collect();
        let allpass = [0.005, 0.0017]
            .iter()
            .map(|s| Delay::new(s * spread))
            .collect();
        Self { combs, allpass }
    }
    fn run(&mut self, x: f32, feedback: f32) -> f32 {
        let mut sum = 0.0;
        for (d, lp) in self.combs.iter_mut() {
            let y = d.tap();
            *lp += 0.35 * (y - *lp);
            d.push(x + *lp * feedback);
            sum += y;
        }
        let mut y = sum * 0.25;
        for d in self.allpass.iter_mut() {
            let b = d.tap();
            d.push(y + b * 0.5);
            y = b - y * 0.5;
        }
        y
    }
}

/// Keep an oscillator phase in [0, 2). Free-running f32 phases lose their
/// fractional precision as they grow, which plays every note off pitch;
/// 2 rather than 1 so `phase * 0.5` sub-octaves stay continuous.
fn wrap(phase: f32) -> f32 {
    if phase >= 2.0 {
        phase - 2.0
    } else {
        phase
    }
}

fn saw(phase: f32) -> f32 {
    2.0 * (phase - phase.floor()) - 1.0
}

fn pulse(phase: f32, width: f32) -> f32 {
    if phase - phase.floor() < width {
        1.0
    } else {
        -1.0
    }
}

fn sine(phase: f32) -> f32 {
    (phase * TAU).sin()
}

/// Most recent event at or before t, tracked with a moving cursor.
struct Cursor(usize);
impl Cursor {
    fn last(&mut self, events: &[(f32, f32)], t: f32) -> Option<(f32, f32)> {
        while self.0 + 1 < events.len() && events[self.0 + 1].0 <= t {
            self.0 += 1;
        }
        events
            .get(self.0)
            .copied()
            .filter(|(et, _)| *et <= t)
            .map(|(et, v)| (t - et, v))
    }
}

/// Notes currently sounding (start <= t < end + release).
struct NoteCursor(usize);
impl NoteCursor {
    fn active<'a>(
        &mut self,
        notes: &'a [Note],
        t: f32,
        release: f32,
    ) -> impl Iterator<Item = &'a Note> {
        while self.0 < notes.len() && notes[self.0].start + notes[self.0].dur + release < t {
            self.0 += 1;
        }
        notes[self.0..]
            .iter()
            .take_while(move |n| n.start <= t)
            .filter(move |n| t < n.start + n.dur + release)
    }
}

fn adsr(age: f32, dur: f32, attack: f32, release: f32) -> f32 {
    if age < 0.0 {
        return 0.0;
    }
    let a = (age / attack).min(1.0);
    if age < dur {
        a
    } else {
        a * (1.0 - (age - dur) / release).max(0.0)
    }
}

fn gain_in(t: f32, ranges: &[(f32, f32, f32)]) -> f32 {
    // (from bar, to bar, gain) with 1/4-bar fades at each edge.
    let b = t / BAR;
    ranges
        .iter()
        .map(|&(a, z, g)| {
            let fade_in = ((b - a) * 4.0).clamp(0.0, 1.0);
            let fade_out = ((z - b) * 4.0).clamp(0.0, 1.0);
            g * fade_in.min(fade_out)
        })
        .fold(0.0, f32::max)
}

// ── The render ───────────────────────────────────────────────────────────────
pub fn render() -> Vec<[f32; 2]> {
    let ev = score();
    let sr = SAMPLE_RATE as f32;
    let total = (DURATION * sr) as usize;
    let mut out = Vec::with_capacity(total);

    let mut noise_l = Noise(0x1234_5678);
    let mut noise_r = Noise(0x9E37_79B9);
    let mut noise_d = Noise(0x0BAD_F00D);
    let (mut pad_f_l, mut pad_f_r) = (Svf::default(), Svf::default());
    let mut lead_f = Svf::default();
    let mut stab_f = Svf::default();
    let mut bass_f = Svf::default();
    let mut riser_f = Svf::default();
    let (mut hat_f, mut snare_f, mut crash_f, mut click_f) = (
        Svf::default(),
        Svf::default(),
        Svf::default(),
        Svf::default(),
    );
    let mut pad_phase = [[0.0f32; 2]; 5];
    let mut arp_phase = 0.0f32;
    let mut bass_phase = 0.0f32;
    let mut echo_l = Delay::new(BEAT * 0.75);
    let mut echo_r = Delay::new(BEAT * 0.75);
    let (mut verb_l, mut verb_r) = (Reverb::new(1.0), Reverb::new(1.07));

    let (mut c_kick, mut c_click, mut c_snare, mut c_hat, mut c_oh, mut c_crash, mut c_boom) = (
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
    );
    let (mut n_lead, mut n_bell, mut n_stab, mut n_solo) =
        (NoteCursor(0), NoteCursor(0), NoteCursor(0), NoteCursor(0));

    for i in 0..total {
        let t = i as f32 / sr;
        let bar = t / BAR;
        let ci = chord(t);
        let step_age = t % STEP;
        let step = (t / STEP) as usize % 16;

        // Drums.
        let mut drums = 0.0;
        let mut duck = 1.0;
        if let Some((age, v)) = c_kick.last(&ev.kick, t) {
            if age < 0.5 {
                let ph = 45.0 * age + 140.0 * (1.0 - (-age * 28.0).exp()) / 28.0;
                drums += sine(ph) * v * ((-age * 9.0).exp() + 0.3 * (-age * 90.0).exp()) * 1.1;
                duck = 1.0 - 0.5 * v * (-age / 0.09).exp();
            }
        }
        // Key clicks: a short filtered tick with a 2 kHz blip.
        let mut click = 0.0;
        if let Some((age, v)) = c_click.last(&ev.click, t) {
            if age < 0.05 {
                let (_, cb, _) = click_f.run(noise_d.next(), 3200.0, 1.2);
                click = (cb * 0.8 + sine(2000.0 * age) * 0.35) * v * (-age * 140.0).exp();
            }
        }
        if let Some((age, v)) = c_snare.last(&ev.snare, t) {
            if age < 0.4 {
                let (_, nb, _) = snare_f.run(noise_l.next(), 2200.0, 0.7);
                // Body tuned to G3 and D4, inside C minor: the inbox roll repeats it
                // fast enough to be heard as a pitch.
                let body = sine(196.0 * age) * 0.5 + sine(293.7 * age) * 0.25;
                drums += (nb * 1.4 + body * (-age * 30.0).exp()) * v * (-age * 16.0).exp() * 0.7;
            }
        }
        if let Some((age, v)) = c_hat.last(&ev.hat, t) {
            let (_, _, hh) = hat_f.run(noise_r.next(), 8000.0, 0.8);
            drums += hh * v * (-age * 70.0).exp() * 0.28;
            if let Some((oage, ov)) = c_oh.last(&ev.open_hat, t) {
                if oage < 0.4 {
                    drums += hh * ov * (-oage * 9.0).exp() * 0.2;
                }
            }
        }
        let mut crash = 0.0;
        if let Some((age, v)) = c_crash.last(&ev.crash, t) {
            if age < 2.5 {
                let (_, _, ch) = crash_f.run(noise_d.next(), 5000.0, 0.5);
                crash = ch * v * (-age * 1.6).exp() * 0.35;
            }
        }
        if let Some((age, v)) = c_boom.last(&ev.boom, t) {
            if age < 2.0 {
                let ph = 38.0 * age + 60.0 * (1.0 - (-age * 6.0).exp()) / 6.0;
                drums += sine(ph) * v * (-age * 2.2).exp() * 0.9;
            }
        }
        // Risers into the month and into `o`.
        let riser_amt = gain_in(t, &[(10.0, 12.0, 1.0), (22.0, 24.0, 1.0)]);
        let riser_pos = if bar < 16.0 {
            (bar - 10.0) / 2.0
        } else {
            (bar - 22.0) / 2.0
        };
        let (_, rb, _) = riser_f.run(
            noise_r.next(),
            300.0 * 20f32.powf(riser_pos.clamp(0.0, 1.0)),
            2.5,
        );
        let riser = rb * riser_amt * riser_pos.clamp(0.0, 1.0) * 0.5;

        // Pad: detuned saws; the filter opens as the app comes up and
        // closes for the inbox breakdown.
        let pad_gain = gain_in(
            t,
            &[
                (0.0, 4.0, 1.0),
                (4.0, 20.0, 0.8),
                (20.0, 24.0, 1.0),
                (24.0, 52.0, 0.8),
                (52.0, 56.0, 1.0),
            ],
        ) * if bar >= 54.0 {
            1.0 - ((bar - 54.0) / 2.0).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let pad_chord = if bar < 4.0 { 0 } else { ci };
        let mut pad = [0.0f32; 2];
        for (v, note) in PAD_VOICINGS[pad_chord].iter().enumerate() {
            let f = hz(*note);
            for (side, det) in [0.996f32, 1.004].iter().enumerate() {
                pad_phase[v][side] = wrap(pad_phase[v][side] + f * det / sr);
                pad[side] += saw(pad_phase[v][side]) * 0.12 + sine(pad_phase[v][side]) * 0.05;
            }
        }
        let pad_cut = 650.0
            + 2200.0
                * smooth((bar - 2.0) / 4.0)
                * if (20.0..24.0).contains(&bar) {
                    1.0 - 0.7 * smooth((bar - 20.0) / 1.0) * (1.0 - smooth(bar - 23.0))
                } else {
                    1.0
                };
        let shimmer = 0.85 + 0.15 * sine(t * 0.8);
        let pad = [
            pad_f_l.run(pad[0], pad_cut, 0.8).0 * pad_gain * shimmer,
            pad_f_r.run(pad[1], pad_cut * 1.05, 0.8).0 * pad_gain * shimmer,
        ];

        // Arp: Planner123's root/fifth/octave shape, now up-and-down chord tones.
        let arp_gain = gain_in(t, &[(4.0, 52.0, 0.42)]);
        let mut arp = 0.0;
        if arp_gain > 0.0 {
            let order = [1usize, 2, 3, 4, 3, 2, 1, 2];
            let f = hz(PAD_VOICINGS[ci][order[step % 8]] + 12.0);
            arp_phase = wrap(arp_phase + f / sr);
            arp = pulse(arp_phase, 0.3) * (-step_age * 22.0).exp() * arp_gain * 0.16;
        }

        // Bass.
        let mut bass = 0.0;
        let root = hz(ROOTS[ci] - 12.0);
        if (8.0..20.0).contains(&bar) || (34.0..52.0).contains(&bar) {
            // Offbeat octave bass.
            let off = matches!(step, 2 | 6 | 10 | 14);
            let f = if off { root * 2.0 } else { root };
            bass_phase = wrap(bass_phase + f / sr);
            let gate = if off || step.is_multiple_of(4) {
                (-step_age * 9.0).exp()
            } else {
                0.0
            };
            bass = (pulse(bass_phase, 0.5) * 0.6 + sine(bass_phase) * 0.6) * gate * 0.35;
        } else if (24.0..34.0).contains(&bar) {
            // The Planner123 16th square bass, driven.
            let accent = if step.is_multiple_of(2) { 1.0 } else { 0.6 };
            bass_phase = wrap(bass_phase + root * 2.0 / sr);
            let slide = (1.0 - step_age / STEP * 0.7).max(0.1);
            bass = (pulse(bass_phase, 0.5) * 0.55 + sine(bass_phase * 0.5) * 0.4)
                * accent
                * slide
                * 0.38;
        }
        let bass = bass_f.run(bass, 900.0, 0.9).0;

        // Lead: the riff doubled. Two detuned pulses with slow PWM, plus a sub.
        let mut lead = 0.0;
        for n in n_lead.active(&ev.lead, t, 0.12) {
            let age = t - n.start;
            let vib = if n.dur > BEAT {
                0.004 * (age * 5.5 * TAU).sin() * ((age - 0.25) * 3.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let ph = n.freq * age * (1.0 + vib);
            let pwm = 0.5 + 0.2 * (t * 0.9).sin();
            let osc =
                pulse(ph, pwm) * 0.5 + pulse(ph * 1.006, 1.0 - pwm) * 0.4 + sine(ph * 0.5) * 0.3;
            lead += osc * adsr(age, n.dur, 0.008, 0.12) * n.vel;
        }
        let lead_env_cut = 1800.0 + 2200.0 * (0.5 + 0.5 * (t * 0.25).sin());
        let lead = lead_f.run(lead, lead_env_cut, 1.2).0 * 0.2;

        // Bell: FM, long tails.
        let mut bell = 0.0;
        for n in n_bell.active(&ev.bell, t, 1.6) {
            let age = t - n.start;
            let index = 2.2 * (-age * 3.0).exp();
            bell += sine(n.freq * age + index * sine(n.freq * 3.5 * age) / TAU)
                * (-age * 1.4).exp()
                * n.vel
                * (age / 0.004).min(1.0);
        }
        let bell = bell * 0.16;

        // Stab: Planner123's riff timbre (saw + 3x sine), gated short.
        let mut stab = 0.0;
        for n in n_stab.active(&ev.stab, t, 0.05) {
            let age = t - n.start;
            stab += (saw(n.freq * age) + sine(n.freq * 3.0 * age) * 0.35)
                * adsr(age, n.dur, 0.003, 0.05)
                * n.vel;
        }
        // The riff is muffled for the inbox breakdown and opens into `o`.
        let stab_cut = if (20.0..24.0).contains(&bar) {
            600.0 + 3600.0 * smooth((bar - 22.5) / 1.5)
        } else {
            4200.0
        };
        let stab = stab_f.run(stab, stab_cut, 1.4).0 * 0.16;

        // Solo quote: soft saws + sine with Planner123's tremolo.
        let mut solo = 0.0;
        for n in n_solo.active(&ev.solo, t, 0.2) {
            let age = t - n.start;
            let trem = 0.75 + 0.25 * (t * 7.0 * TAU).sin();
            let ph = n.freq * age * (1.0 + 0.006 * (age * 5.0 * TAU).sin());
            solo += (saw(ph * 1.002) * 0.3 + saw(ph * 0.998) * 0.3 + sine(ph) * 0.5)
                * adsr(age, n.dur, 0.03, 0.2)
                * trem
                * n.vel;
        }
        let solo = solo * 0.2;

        // Mix buses.
        let melodic_mono = (arp + lead + stab + solo) * duck;
        echo_l.push(melodic_mono * 0.45 + echo_r.tap() * 0.35);
        let el = echo_l.tap();
        echo_r.push(el * 0.8);
        let er = echo_r.tap();

        let send = pad[0] * 0.3 + bell + lead * 0.4 + solo * 0.5 + drums * 0.08 + click * 0.2;
        let rl = verb_l.run(send, 0.8);
        let rr = verb_r.run(send, 0.8);

        let center = drums + bass * duck + click * 0.6 + melodic_mono + riser;
        let mut l = center + pad[0] * duck + el * 0.5 + rl * 0.5 + crash + bell * 0.6;
        let mut r = center + pad[1] * duck + er * 0.5 + rr * 0.5 + crash * 0.9 + bell * 0.6;

        // Master: gentle drive, then a soft ceiling. Fade the very end.
        let fade = ((DURATION - t) / 1.5).clamp(0.0, 1.0);
        l = soft_clip(l * 0.66) * fade;
        r = soft_clip(r * 0.66) * fade;
        out.push([l, r]);
    }
    out
}

fn soft_clip(x: f32) -> f32 {
    x.tanh() * 0.95
}
