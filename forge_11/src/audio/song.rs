// The forge_11 soundtrack: synthwave at 120 BPM in E harmonic minor (D sharp,
// never D natural), one chord per bar, the B major dominant pulling home to
// E minor. The series' palette -- detuned supersaw pads, gliding bass,
// gated snare, tom fills, noise risers, tracker-style bending leads -- with
// a new eight-bar question-and-answer hook.
//
// The arrangement follows the picture bar for bar (timeline.rs), and the
// capture's events play in the mix on the frames they are drawn
// (script.rs): every typed word clicks, every code excerpt lands on an
// anvil, score improvements ping up the scale as the score climbs, the
// feasible plan hits the crash, the disruption sounds the alarm on a
// D sharp / A tritone and drops the band out, every changed job whooshes
// to its new slot, and the healed plan brings the hook back.
use crate::timeline::{bars, now, Section, BAR, BEAT, DURATION, SECTIONS, TOTAL_BARS};

pub const SAMPLE_RATE: u32 = 44_100;
const SR: f32 = SAMPLE_RATE as f32;
const TAU: f32 = std::f32::consts::TAU;

/// What the picture does, on the demo clock.
pub struct Cues {
    pub typing: Vec<f32>,
    pub steps: Vec<f32>,
    pub anvils: Vec<f32>,
    pub drops: Vec<f32>,
    pub improvements: Vec<(f32, f32)>,
    pub solve_feasible: Option<f32>,
    pub event: f32,
    pub moves: Vec<f32>,
    pub repair_feasible: Option<f32>,
    pub reviews: Vec<f32>,
    /// Cordelia's beat: enter (charge), ignite, fire, cut (beams off), exit.
    pub cordelia: [f32; 5],
}

/// Cordelia's laser pitches: the charge glides B3 -> E5 -> B5, the zap
/// falls E6 -> E5, the beam hums on E2 + B2, the power-down falls B4 -> E4.
const LASER_NOTES: [f32; 8] = [59.0, 76.0, 83.0, 88.0, 76.0, 40.0, 47.0, 64.0];

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

fn coeff(tau: f32) -> f32 {
    1.0 - (-1.0 / (tau * SR)).exp()
}

fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Chord {
    Em,
    C,
    Am,
    B,
}

impl Chord {
    /// Bass root (MIDI, the bass voice adds an octave).
    fn root(self) -> f32 {
        match self {
            Chord::Em => 40.0,
            Chord::C => 36.0,
            Chord::Am => 33.0,
            Chord::B => 35.0,
        }
    }
    /// Pad voicing, voice-led around the common E.
    fn pad(self) -> [f32; 4] {
        match self {
            Chord::Em => [64.0, 67.0, 71.0, 78.0], // Em add9
            Chord::C => [64.0, 67.0, 72.0, 76.0],  // C/E
            Chord::Am => [64.0, 69.0, 72.0, 76.0], // Am/E
            Chord::B => [63.0, 66.0, 71.0, 75.0],  // B, D sharp leading home
        }
    }
}

use Chord::*;

/// One chord per bar, per section.
/// The intro: Em and C under the stars, the banner on the progression,
/// the hook's question stated on bar seven, the dominant at the end of the
/// build.
const BOOT: [Chord; 10] = [Em, C, Em, C, Am, B, Em, C, Am, B];
const DESCRIBE: [Chord; 10] = [Em, C, Am, B, Em, C, Am, B, C, B];
const GENERATE: [Chord; 16] = [Em, C, Am, B, Em, C, Am, B, Em, C, Am, B, Em, C, Am, B];
const SOLVE: [Chord; 15] = [Em, C, Am, B, Em, C, Am, B, Em, C, Am, B, Em, Am, B];
/// The opener and the live week, the alarm on Am, Cordelia's charge on
/// C - B, the fire resolving to Em, the dominant while the plan heals, the
/// hook's answer as it lands, then the reviews.
const REPAIR: [Chord; 17] = [Em, C, Am, B, Am, C, B, Em, B, Em, C, Am, B, Em, C, Am, B];
const CLOSE: [Chord; 13] = [Em, C, Am, B, Em, C, Am, B, Em, C, Am, B, Em];

fn chords(section: Section) -> &'static [Chord] {
    match section {
        Section::Boot => &BOOT,
        Section::Describe => &DESCRIBE,
        Section::Generate => &GENERATE,
        Section::Solve => &SOLVE,
        Section::Repair => &REPAIR,
        Section::Close => &CLOSE,
    }
}

/// The repair bar the hook's answer returns on, once the plan has healed.
const HEAL_BAR: usize = 9;

/// E harmonic minor, E4 up two octaves: the improvement pings climb it.
const SCALE: [f32; 15] = [
    64.0, 66.0, 67.0, 69.0, 71.0, 72.0, 75.0, 76.0, 78.0, 79.0, 81.0, 83.0, 84.0, 87.0, 88.0,
];

/// The question, over Em - C - Am - B, ending open on the dominant
/// (start, length, midi) on an eighth-note grid.
const HOOK_Q: [(u8, u8, u8); 17] = [
    (0, 2, 71),
    (2, 2, 76),
    (4, 1, 79),
    (5, 1, 78),
    (6, 2, 76),
    (8, 3, 79),
    (11, 1, 84),
    (12, 2, 83),
    (14, 2, 79),
    (16, 2, 81),
    (18, 2, 84),
    (20, 3, 88),
    (23, 1, 84),
    (24, 4, 83),
    (28, 2, 87),
    (30, 1, 83),
    (31, 1, 81),
];
/// The answer, over the same bars, closing on E through the D sharp.
const HOOK_A: [(u8, u8, u8); 16] = [
    (0, 2, 83),
    (2, 2, 79),
    (4, 1, 76),
    (5, 1, 78),
    (6, 2, 79),
    (8, 3, 88),
    (11, 1, 84),
    (12, 2, 79),
    (14, 2, 76),
    (16, 3, 81),
    (19, 1, 84),
    (20, 2, 83),
    (22, 2, 81),
    (24, 2, 78),
    (26, 2, 75),
    (28, 4, 76),
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
        if at < 0.0 {
            return;
        }
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

fn section_first_bar(section: Section) -> usize {
    SECTIONS
        .iter()
        .find(|(s, _, _)| *s == section)
        .map_or(0, |(_, b0, _)| *b0 as usize)
}

pub fn render(cues: &Cues) -> Vec<[f32; 2]> {
    let len = (DURATION * SR) as usize + 1;
    let mut m = Mix::new(len);
    let total = TOTAL_BARS as usize;
    let event_bar = (cues.event / BAR).round() as usize;
    // The healed plan brings the answer back on the first bar line after
    // feasibility that starts an Em - C - Am - B run in the repair table.
    let heal_bar = section_first_bar(Section::Repair) + HEAL_BAR;
    let heal_ok = cues
        .repair_feasible
        .is_some_and(|t| t <= bars(heal_bar as f32) + 0.01);

    let mut prev = Em;
    for b in 0..total {
        let t0 = bars(b as f32);
        let section = now(t0 + 0.01).section;
        let next = now(t0 + BAR + 0.01).section;
        let k = b - section_first_bar(section);
        let table = chords(section);
        let chord = table[k.min(table.len() - 1)];
        let last_bar = b + 1 == total;
        let left = table.len() - k;

        let pad_len = if last_bar { 3.8 } else { BAR + 0.35 };
        let mut pad_gain = 1.0;
        match section {
            Section::Boot => {
                // Bars 1-2: the pad alone, swelling under the stars, a
                // filtered arp creeping in on bar two.
                pad_gain = [0.5, 0.7, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0][k];
                if k == 1 {
                    arp(&mut m, t0, chord, 0.2);
                }
                // Bar 3: the banner lands; the bass enters held.
                if k == 2 {
                    crash(&mut m, t0, 0.8);
                    boom(&mut m, t0, 0.7);
                }
                if (2..6).contains(&k) {
                    bass(&mut m, t0, prev.root(), chord.root(), Bass::Held, 0.7);
                    arp(&mut m, t0, chord, 0.5);
                    kick(&mut m, t0);
                    if k >= 4 {
                        kick(&mut m, t0 + 2.5 * BEAT);
                        hats(&mut m, t0, 2, 0.6);
                    }
                }
                // Bars 7-10: the hook's question over the full groove,
                // the snare joining on bar nine, a roll into the cut.
                if k >= 6 {
                    arp(&mut m, t0, chord, 0.8);
                    bass(&mut m, t0, prev.root(), chord.root(), Bass::Octaves, 0.9);
                    four_on_floor(&mut m, t0);
                    hats(&mut m, t0, if k >= 8 { 4 } else { 2 }, 0.8);
                    if k == 8 {
                        gated_snare(&mut m, t0 + BEAT, 0.8);
                        gated_snare(&mut m, t0 + 3.0 * BEAT, 0.8);
                    }
                    if k == 9 {
                        snare_roll(&mut m, t0);
                    }
                }
            }
            Section::Describe => {
                // Reading: the band stays light under the prompt.
                arp(&mut m, t0, chord, 0.45);
                bass(&mut m, t0, prev.root(), chord.root(), Bass::Held, 0.55);
                if k >= 2 {
                    kick(&mut m, t0);
                    kick(&mut m, t0 + 2.5 * BEAT);
                }
                if k >= 4 {
                    hats(&mut m, t0, 2, 0.8);
                    gated_snare(&mut m, t0 + 3.0 * BEAT, 0.5);
                }
            }
            Section::Generate => {
                arp(&mut m, t0, chord, 0.8);
                bass(&mut m, t0, prev.root(), chord.root(), Bass::Octaves, 1.0);
                four_on_floor(&mut m, t0);
                gated_snare(&mut m, t0 + BEAT, 1.0);
                gated_snare(&mut m, t0 + 3.0 * BEAT, 1.0);
                hats(&mut m, t0, 4, 1.0);
            }
            Section::Solve => {
                arp(&mut m, t0, chord, 1.0);
                bass(&mut m, t0, prev.root(), chord.root(), Bass::Sixteenths, 0.9);
                four_on_floor(&mut m, t0);
                gated_snare(&mut m, t0 + BEAT, 0.9);
                gated_snare(&mut m, t0 + 3.0 * BEAT, 0.9);
                hats(&mut m, t0, 4, 0.9);
            }
            Section::Repair => {
                let rel = b as i32 - event_bar as i32;
                if rel < 0 {
                    // The plan is live: the groove runs on, lighter.
                    arp(&mut m, t0, chord, 0.6);
                    bass(&mut m, t0, prev.root(), chord.root(), Bass::Eighths, 0.8);
                    four_on_floor(&mut m, t0);
                    hats(&mut m, t0, 2, 0.8);
                } else if rel < 3 {
                    // The break: a heartbeat under the alarm, held through
                    // Cordelia's charge.
                    pad_gain = 0.7;
                    bass(&mut m, t0, prev.root(), chord.root(), Bass::Held, 0.6);
                    kick(&mut m, t0);
                    kick(&mut m, t0 + 0.75 * BEAT);
                    kick(&mut m, t0 + 2.0 * BEAT);
                    kick(&mut m, t0 + 2.75 * BEAT);
                } else {
                    // Building back while the plan heals.
                    let build = (rel - 3) as f32;
                    arp(&mut m, t0, chord, (0.4 + 0.15 * build).min(1.0));
                    bass(&mut m, t0, prev.root(), chord.root(), Bass::Eighths, 0.9);
                    four_on_floor(&mut m, t0);
                    hats(&mut m, t0, if rel >= 5 { 4 } else { 2 }, 0.9);
                    if rel >= 4 {
                        gated_snare(&mut m, t0 + BEAT, 0.9);
                        gated_snare(&mut m, t0 + 3.0 * BEAT, 0.9);
                    }
                }
            }
            Section::Close => {
                if left > 2 {
                    arp(&mut m, t0, chord, 1.0);
                    bass(&mut m, t0, prev.root(), chord.root(), Bass::Octaves, 1.0);
                    four_on_floor(&mut m, t0);
                    gated_snare(&mut m, t0 + BEAT, 1.0);
                    gated_snare(&mut m, t0 + 3.0 * BEAT, 1.0);
                    hats(&mut m, t0, 4, 1.0);
                } else if left == 2 {
                    // The dominant, held, the band out after the downbeat.
                    kick(&mut m, t0);
                    crash(&mut m, t0, 0.7);
                    bass(&mut m, t0, prev.root(), chord.root(), Bass::Held, 0.9);
                    arp(&mut m, t0, chord, 0.5);
                } else {
                    // Home: one last hit on E minor.
                    kick(&mut m, t0);
                    crash(&mut m, t0, 1.0);
                    bass(&mut m, t0, prev.root(), chord.root(), Bass::Held, 1.0);
                }
            }
        }
        pad(&mut m, t0, chord, pad_len, b, pad_gain);

        // Toms and a noise riser into every new section.
        if next != section && !last_bar {
            tom_fill(&mut m, t0);
            riser(&mut m, t0);
        }
        if k == 0 && section != Section::Boot {
            crash(&mut m, t0, 0.8);
        }

        // Leads.
        match (section, k) {
            // The intro states the question; the answer comes back softly
            // under the prompt, then in full in the forge.
            (Section::Boot, 6) => lead(&mut m, t0, &HOOK_Q, 0.0, Osc::Super, 0.8, false),
            (Section::Describe, 4) => lead(&mut m, t0, &HOOK_A, -12.0, Osc::Pulse, 0.45, false),
            (Section::Generate, 4) => lead(&mut m, t0, &HOOK_Q, 0.0, Osc::Super, 0.9, false),
            (Section::Generate, 8) => {
                lead(&mut m, t0, &HOOK_A, 0.0, Osc::Super, 0.9, false);
                lead(&mut m, t0, &HOOK_A, -12.0, Osc::Pulse, 0.45, false);
            }
            (Section::Generate, 12) => lead(&mut m, t0, &HOOK_Q, -12.0, Osc::Brass, 0.7, true),
            (Section::Solve, 8) => lead(&mut m, t0, &HOOK_Q, -12.0, Osc::Brass, 0.7, true),
            (Section::Repair, HEAL_BAR) if heal_ok => {
                lead(&mut m, t0, &HOOK_A, 0.0, Osc::Super, 0.8, false);
            }
            (Section::Close, 0) => lead(&mut m, t0, &HOOK_Q, -12.0, Osc::Pulse, 0.5, false),
            (Section::Close, 4) => {
                lead(&mut m, t0, &HOOK_A, 0.0, Osc::Super, 1.0, false);
                lead(&mut m, t0, &HOOK_A, -12.0, Osc::Pulse, 0.5, false);
            }
            _ => {}
        }
        prev = chord;
    }

    // The capture's events.
    for (i, &at) in cues.typing.iter().enumerate() {
        key_click(&mut m, at, i);
    }
    for (i, &at) in cues.steps.iter().enumerate() {
        let note = SCALE[7 + (i * 3) % 7];
        blip(
            &mut m,
            at,
            hz(note + 12.0),
            if i % 2 == 0 { -0.6 } else { 0.6 },
            0.035,
        );
    }
    for (i, &at) in cues.anvils.iter().enumerate() {
        anvil(&mut m, at, i);
    }
    for (i, &at) in thin(&cues.drops, 0.06).iter().enumerate() {
        woodblock(&mut m, at, if i % 2 == 0 { -0.4 } else { 0.4 });
    }
    for &(at, p) in thin_pairs(&cues.improvements, 0.07).iter() {
        let note = SCALE[((p * (SCALE.len() - 1) as f32).round() as usize).min(SCALE.len() - 1)];
        blip(&mut m, at, hz(note + 12.0), 0.2, 0.05);
    }
    for at in [cues.solve_feasible, cues.repair_feasible]
        .into_iter()
        .flatten()
    {
        crash(&mut m, at, 1.0);
        stab(&mut m, at);
        boom(&mut m, at, 0.6);
    }
    alarm(&mut m, cues.event);
    boom(&mut m, cues.event, 1.0);
    for (k, &at) in cues.moves.iter().enumerate() {
        whoosh(&mut m, at, 0.6, if k % 2 == 0 { -0.5 } else { 0.5 });
        woodblock(&mut m, at + 0.6, 0.0);
    }

    laser(&mut m, cues.cordelia);

    // Each reviewed change rings a bell up the tonic chord.
    for (k, &at) in cues.reviews.iter().enumerate() {
        let note = [76.0, 79.0, 83.0, 88.0][k % 4];
        bell(&mut m, at, hz(note), if k % 2 == 0 { -0.3 } else { 0.3 });
    }

    mixdown(m)
}

/// A soft FM bell.
fn bell(m: &mut Mix, at: f32, f: f32, pan: f32) {
    m.voice(at, 1.2, 0.05, pan, 0.6, move |t, _| {
        let idx = 1.2 * (-t * 3.0).exp();
        (TAU * f * t + idx * (TAU * f * 3.5 * t).sin()).sin() * (-t * 3.5).exp()
    });
}

/// Keep events at least `gap` seconds apart (dense bursts stay readable).
fn thin(times: &[f32], gap: f32) -> Vec<f32> {
    let mut out: Vec<f32> = Vec::new();
    for &t in times {
        if out.last().is_none_or(|&l| t - l >= gap) {
            out.push(t);
        }
    }
    out
}

fn thin_pairs(events: &[(f32, f32)], gap: f32) -> Vec<(f32, f32)> {
    let mut sorted = events.to_vec();
    sorted.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out: Vec<(f32, f32)> = Vec::new();
    for e in sorted {
        match out.last_mut() {
            Some(l) if e.0 - l.0 < gap => l.1 = l.1.max(e.1),
            _ => out.push(e),
        }
    }
    out
}

/// Supersaw pad: three detuned saws per chord tone through a lowpass whose
/// cutoff breathes over eight bars.
fn pad(m: &mut Mix, t0: f32, chord: Chord, dur: f32, bar: usize, gain: f32) {
    let open = 0.03 + 0.035 * (0.5 - 0.5 * (bar as f32 * TAU / 8.0).cos());
    for (k, &note) in chord.pad().iter().enumerate() {
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
                let v = lp2 * e * 0.026 * gain;
                m.pad[i][0] += v * (1.0 - pan) * 0.5 * 1.4;
                m.pad[i][1] += v * (1.0 + pan) * 0.5 * 1.4;
            }
        }
    }
}

/// Sixteenth arpeggio climbing the voicing, an octave up in the second half.
fn arp(m: &mut Mix, t0: f32, chord: Chord, gain: f32) {
    let notes = chord.pad();
    let order = [0, 2, 1, 3, 2, 0, 3, 1];
    for step in 0..16 {
        let note = notes[order[step % 8]] + if step >= 8 { 12.0 } else { 0.0 };
        let f = hz(note + 12.0);
        let at = t0 + step as f32 * BEAT / 4.0;
        let mut lp = 0.0f32;
        m.voice(
            at,
            0.2,
            0.04 * gain,
            if step % 2 == 0 { -0.45 } else { 0.45 },
            0.6,
            move |t, _| {
                let raw = pulse(f * t, 0.25);
                lp += (raw - lp) * 0.22;
                lp * (-t * 18.0).exp()
            },
        );
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Bass {
    Eighths,
    Sixteenths,
    Octaves,
    Held,
}

/// One bar of bass as a single gliding voice: it slides in from the last
/// bar's root; Octaves jumps to the octave on every off-beat eighth.
fn bass(m: &mut Mix, t0: f32, from: f32, root: f32, kind: Bass, gain: f32) {
    let unit = if kind == Bass::Sixteenths {
        BEAT / 4.0
    } else {
        BEAT / 2.0
    };
    let mut pitch = from + 12.0;
    let mut phase = 0.0f32;
    let mut lp = 0.0f32;
    let k_bar = coeff(0.035);
    let k_oct = coeff(0.012);
    m.voice(t0, BAR + 0.02, 0.17 * gain, 0.0, 0.0, move |t, _| {
        let step = (t / unit) as usize;
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
            t - step as f32 * unit
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
            env(local, 0.003, unit - 0.02, 0.015)
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

fn hats(m: &mut Mix, t0: f32, per_beat: usize, gain: f32) {
    let steps = 4 * per_beat;
    for k in 0..steps {
        if per_beat == 2 && k % 2 == 0 {
            continue;
        }
        let at = t0 + k as f32 * BEAT / per_beat as f32;
        let open = per_beat == 4 && k % 4 == 2;
        let accent = if k % 2 == 1 { 0.7 } else { 1.0 };
        let decay = if open { 14.0 } else { 70.0 };
        let mut prev = 0.0f32;
        m.voice(
            at,
            if open { 0.25 } else { 0.06 },
            0.07 * accent * gain,
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

/// A long, bright noise wash.
fn crash(m: &mut Mix, at: f32, gain: f32) {
    let mut prev = 0.0f32;
    let mut lp = 0.0f32;
    m.voice(at, 2.2, 0.16 * gain, -0.2, 0.35, move |t, mm| {
        let n = mm.rnd();
        let hp = n - prev;
        prev = n;
        lp += (hp - lp) * 0.6;
        lp * (-t * 2.2).exp() * (1.0 + 0.6 * (-t * 30.0).exp())
    });
}

#[derive(Clone, Copy, PartialEq)]
enum Osc {
    Super,
    Pulse,
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

fn blip(m: &mut Mix, at: f32, f: f32, pan: f32, gain: f32) {
    m.voice(at, 0.16, gain, pan, 0.45, move |t, _| {
        (TAU * f * t).sin() * (-t * 26.0).exp()
    });
}

/// A typed word: a short bright tick with a pitched body.
fn key_click(m: &mut Mix, at: f32, i: usize) {
    let f = 1800.0 + 400.0 * ((i * 7) % 5) as f32;
    let pan = if i.is_multiple_of(2) { -0.35 } else { 0.35 };
    let mut prev = 0.0f32;
    m.voice(at, 0.05, 0.05, pan, 0.1, move |t, mm| {
        let n = mm.rnd();
        let hp = n - prev;
        prev = n;
        (hp * 0.7 + 0.5 * (TAU * f * t).sin()) * (-t * 90.0).exp()
    });
}

/// The anvil: an inharmonic FM ring over a thud, as each excerpt lands.
fn anvil(m: &mut Mix, at: f32, i: usize) {
    let base = [1244.5, 1318.5, 987.8][i % 3]; // D#6, E6, B5 ring
    let mut pc = 0.0f32;
    let mut pm = 0.0f32;
    m.voice(at, 1.4, 0.13, 0.15, 0.5, move |t, mm| {
        pc = (pc + base / SR).fract();
        pm = (pm + base * 1.414 / SR).fract();
        let idx = 2.5 * (-t * 6.0).exp();
        let ring = (TAU * pc + idx * (TAU * pm).sin()).sin() * (-t * 3.2).exp();
        ring + 0.6 * mm.rnd() * (-t * 60.0).exp()
    });
    boom(m, at, 0.35);
}

fn woodblock(m: &mut Mix, at: f32, pan: f32) {
    m.voice(at, 0.08, 0.06, pan, 0.2, move |t, _| {
        (TAU * 1650.0 * t).sin() * (-t * 55.0).exp()
    });
}

/// A sub drop: a sine sweeping down under an impact.
fn boom(m: &mut Mix, at: f32, gain: f32) {
    let mut phase = 0.0f32;
    m.voice(at, 1.2, 0.5 * gain, 0.0, 0.0, move |t, mm| {
        let f = 32.0 + 90.0 * (-t * 5.0).exp();
        phase += f / SR;
        (phase * TAU).sin() * (-t * 2.4).exp() + 0.25 * mm.rnd() * (-t * 25.0).exp()
    });
}

/// A supersaw stab of the tonic chord.
fn stab(m: &mut Mix, at: f32) {
    for (k, &note) in Em.pad().iter().enumerate() {
        let f = hz(note + 12.0);
        let mut lp = 0.0f32;
        m.voice(at, 0.7, 0.05, (k as f32 - 1.5) * 0.3, 0.6, move |t, _| {
            let s = saw(f * t) + saw(f * 1.006 * t) + saw(f * 0.994 * t);
            lp += (s / 3.0 - lp) * (0.08 + 0.3 * (-t * 8.0).exp());
            lp * (-t * 4.0).exp()
        });
    }
}

/// The alarm: a square siren on the D sharp / A tritone, four times.
fn alarm(m: &mut Mix, at: f32) {
    for k in 0..8 {
        let f = hz(if k % 2 == 0 { 87.0 } else { 81.0 });
        let mut lp = 0.0f32;
        m.voice(
            at + k as f32 * BEAT / 2.0,
            BEAT / 2.0,
            0.07 * (1.0 - k as f32 * 0.08),
            0.0,
            0.5,
            move |t, _| {
                lp += (pulse(f * t, 0.5) - lp) * 0.2;
                lp * env(t, 0.005, BEAT / 2.0 - 0.04, 0.03)
            },
        );
    }
    let mut prev = 0.0f32;
    m.voice(at, 0.6, 0.25, 0.0, 0.3, move |t, mm| {
        let n = mm.rnd();
        let hp = n - prev;
        prev = n;
        hp * (-t * 6.0).exp()
    });
}

/// Cordelia's eye-lasers, in key and on the grid: a charge whine gliding
/// up the dominant to the tonic from her entrance to the ignite beat and on
/// to the fifth into the fire; on the downbeat an FM zap falling an octave
/// on E with a noise crack and a sub hit; a beam hum on E and B pulsing in
/// sixteenths while the beams hold; a fall on the exit as the hook returns.
fn laser(m: &mut Mix, [enter, ignite, fire, cut, exit]: [f32; 5]) {
    let [b3, e5, b5, e6, e5z, e2, b2, e4] = LASER_NOTES;
    // Charge: one voice, continuous phase, pitch gliding on a curve.
    let len = fire - enter;
    let mut ph = 0.0f32;
    let mut ph2 = 0.0f32;
    let mut lp = 0.0f32;
    m.voice(enter, len, 0.05, -0.2, 0.5, move |t, _| {
        let at = enter + t;
        let midi = if at < ignite {
            b3 + (e5 - b3) * smooth((at - enter) / (ignite - enter)).powf(1.6)
        } else {
            e5 + (b5 - e5) * smooth((at - ignite) / (fire - ignite))
        };
        let f = hz(midi) * (1.0 + 0.004 * (t * TAU * 7.0).sin());
        ph = (ph + f / SR).fract();
        ph2 = (ph2 + f * 1.007 / SR).fract();
        let s = 0.6 * saw(ph) + 0.4 * saw(ph2);
        lp += (s - lp) * (0.05 + 0.25 * t / len);
        let trem = 0.75 + 0.25 * (t * TAU * 16.0 * (1.0 + t / len)).sin();
        lp * trem * smooth(t / 0.15) * (0.3 + 0.7 * t / len)
    });
    // Zap: FM sine falling E6 -> E5, a noise crack, a sub hit.
    let mut pc = 0.0f32;
    let mut pm = 0.0f32;
    m.voice(fire, 0.5, 0.16, 0.2, 0.6, move |t, mm| {
        let midi = e5z + (e6 - e5z) * (-t * 14.0).exp();
        let f = hz(midi);
        pc = (pc + f / SR).fract();
        pm = (pm + f * 2.0 / SR).fract();
        let idx = 3.0 * (-t * 9.0).exp();
        let tone = (TAU * pc + idx * (TAU * pm).sin()).sin() * (-t * 5.0).exp();
        tone + 0.5 * mm.rnd() * (-t * 40.0).exp()
    });
    boom(m, fire, 0.8);
    // Beam hum: E2 + B2 pulses, gated in sixteenths, while the beams hold;
    // a smaller zap on every beat the beams surge.
    let hold = cut - fire;
    let mut beat_at = fire + BEAT;
    while beat_at < cut - 0.05 {
        let mut pc = 0.0f32;
        m.voice(beat_at, 0.2, 0.05, -0.1, 0.4, move |t, _| {
            let midi = e5z + 12.0 * (-t * 30.0).exp();
            pc = (pc + hz(midi) / SR).fract();
            (TAU * pc).sin() * (-t * 14.0).exp()
        });
        beat_at += BEAT;
    }
    for (k, note) in [e2, b2].into_iter().enumerate() {
        let f = hz(note);
        let mut lp = 0.0f32;
        let pan = if k == 0 { -0.3 } else { 0.3 };
        m.voice(fire, hold, 0.05, pan, 0.2, move |t, _| {
            lp += (pulse(f * t, 0.3) - lp) * 0.08;
            let gate = 0.55 + 0.45 * (1.0 - (t / (BEAT / 4.0)).fract());
            lp * gate * smooth(t / 0.05) * smooth((hold - t) / 0.08)
        });
    }
    // Power-down as the beams cut; a glitch tick as she tears out.
    let mut ph = 0.0f32;
    m.voice(cut, 0.4, 0.05, 0.0, 0.5, move |t, _| {
        let midi = e4 + (b5 - 12.0 - e4) * (-t * 10.0).exp();
        ph = (ph + hz(midi) / SR).fract();
        saw(ph) * (1.0 - t / 0.4)
    });
    let mut prev = 0.0f32;
    m.voice(exit, 0.35, 0.08, 0.3, 0.3, move |t, mm| {
        let n = mm.rnd();
        let hp = n - prev;
        prev = n;
        let gate = if ((t * 40.0) as i32) % 2 == 0 {
            1.0
        } else {
            0.2
        };
        hp * gate * (1.0 - t / 0.35)
    });
}

/// A snare roll over a bar: eighths, then sixteenths, then thirty-seconds,
/// rising in level into the downbeat.
fn snare_roll(m: &mut Mix, t0: f32) {
    let mut at = t0;
    while at < t0 + BAR - 0.01 {
        let k = (at - t0) / BAR;
        gated_snare(m, at, 0.25 + 0.6 * k);
        at += if k < 0.5 {
            BEAT / 2.0
        } else if k < 0.75 {
            BEAT / 4.0
        } else {
            BEAT / 8.0
        };
    }
}

/// Filtered noise sweeping up and away over a move.
fn whoosh(m: &mut Mix, at: f32, dur: f32, pan: f32) {
    let mut lp = 0.0f32;
    m.voice(at, dur, 0.09, pan, 0.4, move |t, mm| {
        let k = t / dur;
        lp += (mm.rnd() - lp) * (0.02 + 0.3 * k);
        lp * (std::f32::consts::PI * k).sin()
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
            Some(&k) if k <= t => 1.0 - 0.5 * (-(t - k) * 9.0).exp(),
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
        let fb_l = if i >= d { r[i - d] * 0.4 } else { 0.0 };
        let fb_r = if i >= d { l[i - d] * 0.4 } else { 0.0 };
        l[i] = m.send[i] + fb_l;
        r[i] = fb_r;
    }
    for i in 0..len {
        out[i][0] += 0.3 * l[i];
        out[i][1] += 0.3 * r[i];
    }

    // Soft clip, then normalise to a fixed peak below full scale.
    let mut peak = 0.0f32;
    for frame in out.iter_mut() {
        for s in frame.iter_mut() {
            *s = (*s * 1.2).tanh();
            peak = peak.max(s.abs());
        }
    }
    let g = if peak > 0.0 { 0.8 / peak } else { 1.0 };
    for (i, frame) in out.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let fade = (t / 0.3).min(1.0) * ((DURATION - t) / 1.5).clamp(0.0, 1.0);
        frame[0] *= g * fade;
        frame[1] *= g * fade;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// E harmonic minor: E F# G A B C D#. Every pitch the song writes.
    #[test]
    fn every_pitch_is_in_e_harmonic_minor() {
        let scale = [4, 6, 7, 9, 11, 0, 3];
        let ok = |n: f32| scale.contains(&((n as i32).rem_euclid(12)));
        for c in [Em, C, Am, B] {
            assert!(ok(c.root()), "{c:?} root");
            for n in c.pad() {
                assert!(ok(n), "{c:?} pad {n}");
            }
        }
        for &(_, _, n) in HOOK_Q.iter().chain(HOOK_A.iter()) {
            assert!(ok(n as f32), "hook note {n}");
        }
        for n in SCALE {
            assert!(ok(n));
        }
        // The alarm's tritone and the anvil ring's D sharp / E.
        assert!(ok(87.0) && ok(81.0));
        // Cordelia's lasers: every glide endpoint and drone.
        for n in LASER_NOTES {
            assert!(ok(n), "laser note {n}");
        }
        assert!(ok(83.0 - 12.0));
    }

    /// On the real capture the alarm lands on Am, the fire resolves the
    /// charge's dominant on Em, and the healed plan is in time for the hook.
    #[test]
    fn the_repair_lands_on_its_chords() {
        let real = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");
        if !std::path::Path::new(real).exists() {
            return;
        }
        let c = crate::capture::Capture::load(real).unwrap();
        let s = crate::script::Script::new(&c);
        let r0 = section_first_bar(Section::Repair);
        let chord = |t: f32| REPAIR[(t / BAR).round() as usize - r0];
        assert_eq!(chord(s.event), Am);
        assert_eq!(chord(s.cordelia.fire), Em);
        assert_eq!(chord(s.cordelia.fire - BAR), B);
        assert!(s.repair_feasible.unwrap() <= bars((r0 + HEAL_BAR) as f32));
    }

    #[test]
    fn chord_tables_match_the_sections() {
        for &(s, b0, b1) in SECTIONS.iter() {
            assert_eq!(chords(s).len(), (b1 - b0) as usize, "{s:?}");
        }
        // The hook's bars run Em - C - Am - B wherever it plays.
        for (s, k) in [
            (Section::Boot, 6),
            (Section::Describe, 4),
            (Section::Generate, 4),
            (Section::Generate, 8),
            (Section::Generate, 12),
            (Section::Solve, 8),
            (Section::Repair, HEAL_BAR),
            (Section::Close, 0),
            (Section::Close, 4),
        ] {
            assert_eq!(&chords(s)[k..k + 4], &[Em, C, Am, B], "{s:?} bar {k}");
        }
    }
}
