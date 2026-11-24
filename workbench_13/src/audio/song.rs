// WORKBENCH_13 soundtrack: a new score for the factory week, in G minor
// with the raised seventh of the harmonic minor (D Hijaz over the dominant):
// archeofuturistic, ancient instruments played by a machine.
//
// One motif carries it: a bent chord tone held a beat and a half, then
// three notes falling through the chord, over Gm - Eb - Cm - D. The duduk
// states it, the guitar sings it when the machine starts, the twin guitar
// harmonises it in the repair, and the outro bends it home.
//
// The form follows the film's sections (cut.rs), one part each:
//   intro    gong, drone, a rising shimmer under the turning banner, a
//            taiko on each model panel, a roll into the first take;
//   orders   santur 16ths over the tanpura, frame drums in maqsum, the duduk;
//   week     the machine starts: four on the floor, the riff on supersaws,
//            the pulse arp, the guitar's motif and its answer;
//   issued   half time, the guitar's held bend coming home, a bell;
//   delay    the verse again, darker: the arp ticks like a clock, the santur
//            thins out, a riser into the powder;
//   powder   the alarm: a D pedal, a heartbeat kick, Eb grinding on D in the
//            choir and the bells;
//   call     Cordelia: concern on C minor; on the second bar a gong, the
//            boom and a wide bend (defiance); on the fourth the D major
//            chord, bright, and the decision;
//   replan   the build: taiko in eighths, santur tremolo, the duduk's motif;
//   repair   everything: tremolo picking with a twin a third below, then the
//            answer, twinned;
//   repaired half time, the guitar and the duduk trading calls, then tapped
//            arpeggios climbing;
//   outro    Eb - D - Gm, the gong, the guitar's last held bend and dive.
//
// The strings (santur, tanpura) are Karplus-Strong strings; the duduk is a
// reed through two formants with breath; frame drums and taiko are skins.
// Around them run the supersaw stabs, the square bass, the pulse arp, the
// gated snare and the sidechain pump. The lead is an electric guitar in the
// tracker style: one monophonic voice that bends, slides and dives.
//
// The music keeps one level from the first frame to the last; the narration
// is mixed over it in the composition, never by turning the music down.
use crate::cut::{DURATION, SECTIONS};
use crate::scenes::intro::PANEL_AT;
use crate::timeline::{smooth, BAR, BEAT};

pub const SAMPLE_RATE: u32 = 44_100;
const TAU: f32 = std::f32::consts::TAU;
const STEP: f32 = BEAT / 4.0;

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

fn bars(b: f32) -> f32 {
    b * BAR
}

/// One-pole coefficient for a time constant in seconds.
fn coeff(tau: f32) -> f32 {
    1.0 - (-1.0 / (tau * SAMPLE_RATE as f32)).exp()
}

// ── Harmony ──────────────────────────────────────────────────────────────────
const GM: usize = 0;
const EB: usize = 1;
const CM: usize = 2;
const DM: usize = 3;
const BB: usize = 4;
/// Bare triads on open fifths, low.
const PAD_VOICINGS: [[f32; 5]; 5] = [
    [43.0, 50.0, 55.0, 58.0, 62.0], // Gm
    [39.0, 46.0, 51.0, 55.0, 58.0], // Eb
    [36.0, 43.0, 48.0, 51.0, 55.0], // Cm
    [38.0, 45.0, 50.0, 54.0, 57.0], // D, with the raised F#
    [34.0, 41.0, 46.0, 50.0, 53.0], // Bb
];
const LOOP: [usize; 4] = [GM, EB, CM, DM];
/// The repair's own turn: the relative major's light before the dominant.
const LIFT: [usize; 4] = [BB, EB, CM, DM];
const INTRO_CHORDS: [usize; 7] = [GM, GM, GM, EB, CM, DM, DM];
const OUTRO_CHORDS: [usize; 6] = [EB, DM, GM, GM, GM, GM];
/// G harmonic minor with F natural for the Bb chord: every pitch the score
/// writes is one of these.
const SCALE: [i32; 8] = [7, 9, 10, 0, 2, 3, 5, 6];

// ── The lines ────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
enum Art {
    /// Picked plainly.
    N,
    /// Fretted a whole step low and bent up into pitch.
    B,
    /// Ends diving an octave.
    D,
    /// Re-picked on every 16th: tremolo picking.
    T,
    /// Slid into from a fourth below.
    S,
    /// Bent a step and a half up into pitch: the wide bend.
    W,
    /// Hammered or tapped: no pick, sliding straight from the last note.
    H,
}
use Art::{B, D, H, N, S, T, W};

/// A melodic line: (midi, beats, articulation), 0 for a rest.
type Line = [(f32, f32, Art)];

/// The motif over Gm - Eb - Cm - D: a bent chord tone, three falling; the
/// fourth bar's wide bend onto F# and the D under it, the dominant pulling
/// home. Every note on beats 1 and 3 is a tone of its bar's chord (tested).
const MOTIF: [(f32, f32, Art); 14] = [
    (74.0, 1.5, B),
    (72.0, 0.5, N),
    (70.0, 1.0, N),
    (67.0, 1.0, N),
    (75.0, 1.5, B),
    (74.0, 0.5, N),
    (70.0, 1.0, N),
    (67.0, 1.0, N),
    (72.0, 1.5, B),
    (70.0, 0.5, N),
    (67.0, 1.0, N),
    (63.0, 1.0, N),
    (78.0, 2.0, W),
    (74.0, 2.0, N),
];

/// The answer: the motif turned upside down, climbing where it fell, to the
/// high C, then down onto D and a dive.
const ANSWER: [(f32, f32, Art); 15] = [
    (67.0, 1.0, S),
    (70.0, 1.0, N),
    (74.0, 1.5, B),
    (79.0, 0.5, N),
    (79.0, 1.5, B),
    (75.0, 0.5, N),
    (82.0, 1.0, N),
    (79.0, 1.0, N),
    (84.0, 1.5, B),
    (82.0, 0.5, N),
    (79.0, 1.0, N),
    (75.0, 1.0, N),
    (81.0, 1.5, N),
    (78.0, 0.5, N),
    (74.0, 2.0, D),
];

/// The motif as the duduk states it: long chord tones.
const DUDUK: [(f32, f32, Art); 10] = [
    (67.0, 2.0, B),
    (70.0, 1.0, N),
    (69.0, 1.0, N),
    (67.0, 2.0, N),
    (63.0, 2.0, N),
    (72.0, 2.0, B),
    (67.0, 1.0, N),
    (70.0, 1.0, N),
    (66.0, 2.0, N),
    (62.0, 2.0, N),
];

/// The repair, low on the neck: the motif's chord tones as half notes,
/// tremolo-picked over Bb - Eb - Cm - D, a twin a third below.
const TREMOLO: [(f32, f32, Art); 8] = [
    (70.0, 2.0, T),
    (74.0, 2.0, T),
    (70.0, 2.0, T),
    (67.0, 2.0, T),
    (72.0, 2.0, T),
    (67.0, 2.0, T),
    (74.0, 2.0, T),
    (78.0, 2.0, T),
];

/// The half time's calls over Gm - Eb: one slid in and pushed by a wide
/// bend, the other answered by the duduk over Cm - D.
const CALL_A: [(f32, f32, Art); 4] = [
    (67.0, 1.0, S),
    (70.0, 0.5, N),
    (72.0, 0.5, N),
    (74.0, 2.0, W),
];
const CALL_B: [(f32, f32, Art); 4] = [
    (75.0, 1.0, S),
    (74.0, 0.5, N),
    (70.0, 0.5, N),
    (67.0, 2.0, N),
];

/// The issued week: a held bend from D onto G, the dominant coming home.
const HOME: [(f32, f32, Art); 3] = [(74.0, 4.0, B), (78.0, 2.0, W), (79.0, 6.0, N)];

#[derive(Clone, Copy)]
struct LNote {
    start: f32,
    dur: f32,
    midi: f32,
    art: Art,
    vel: f32,
}

/// Lay a (midi, beats, articulation) line from `at` seconds, `shift`
/// semitones up and `stretch` times slower. 0 is a rest.
fn lay(out: &mut Vec<LNote>, at: f32, line: &Line, shift: f32, stretch: f32, vel: f32) -> f32 {
    let mut t = at;
    for &(m, beats, art) in line {
        let dur = beats * BEAT * stretch;
        if m > 0.0 {
            out.push(LNote {
                start: t,
                dur,
                midi: m + shift,
                art,
                vel,
            });
        }
        t += dur;
    }
    t
}

/// One monophonic voice that bends: legato notes glide, notes after a rest
/// scoop up from two semitones below, B notes are bent up a whole step into
/// pitch, held notes grow a vibrato, D notes dive an octave at their end.
struct Mono {
    notes: Vec<LNote>,
    idx: usize,
    pitch: f32,
    k_glide: f32,
    amp: f32,
}

impl Mono {
    fn new(mut notes: Vec<LNote>) -> Self {
        notes.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        Self {
            notes,
            idx: usize::MAX,
            pitch: 60.0,
            k_glide: coeff(0.05),
            amp: 0.0,
        }
    }

    /// (pitch in midi, amplitude, seconds into the note, the note), or None
    /// while silent.
    fn tick(&mut self, t: f32) -> Option<(f32, f32, f32, LNote)> {
        let next = self.idx.wrapping_add(1);
        if next < self.notes.len() && self.notes[next].start <= t {
            let n = self.notes[next];
            let legato = self.idx != usize::MAX && {
                let p = self.notes[self.idx];
                p.start + p.dur >= n.start - 0.01
            };
            // Only a phrase's first note scoops up from below; inside a
            // phrase the voice slides quickly from note to note.
            let phrase_start = self.idx == usize::MAX || {
                let p = self.notes[self.idx];
                n.start - (p.start + p.dur) > 0.25
            };
            if n.art == B {
                self.pitch = n.midi - 2.0;
                self.k_glide = coeff(0.02);
            } else if n.art == W {
                self.pitch = n.midi - 3.0;
                self.k_glide = coeff(0.02);
            } else if n.art == S {
                self.pitch = n.midi - 5.0;
                self.k_glide = coeff(0.05);
            } else if n.art == H {
                self.k_glide = coeff(0.004);
            } else if phrase_start {
                self.pitch = n.midi - 2.0;
                self.k_glide = coeff(0.03);
            } else {
                if !legato {
                    self.pitch = n.midi;
                }
                self.k_glide = coeff(0.012);
            }
            self.idx = next;
        }
        if self.idx == usize::MAX {
            return None;
        }
        let n = self.notes[self.idx];
        let into = t - n.start;
        let next_legato = self
            .notes
            .get(self.idx + 1)
            .is_some_and(|m| m.start <= n.start + n.dur + 0.01);
        let mut gate = into < n.dur - if next_legato { 0.0 } else { 0.03 };
        let mut target = n.midi;
        match n.art {
            // Fretted low, bent up over a fifth of the note after a short hold.
            B => target -= 2.0 * (1.0 - smooth((into - 0.06) / (0.18 * n.dur + 0.06))),
            W => target -= 3.0 * (1.0 - smooth((into - 0.08) / (0.25 * n.dur + 0.08))),
            D => {
                let from = n.dur * 0.45;
                target -= 12.0 * smooth((into - from) / (n.dur - from + 0.7));
                gate = into < n.dur + 0.7;
            }
            N | T | S | H => {}
        }
        self.pitch += (target - self.pitch) * self.k_glide;
        let vib = 0.32
            * (TAU * 5.6 * into).sin()
            * smooth((into - 0.22) / 0.3)
            * (n.art != T && n.art != H) as i32 as f32
            * (n.dur > 0.4) as i32 as f32;
        let k = if gate { coeff(0.004) } else { coeff(0.06) };
        self.amp += ((gate as i32 as f32) * n.vel - self.amp) * k;
        if !gate && self.amp < 1e-4 {
            return None;
        }
        Some((self.pitch + vib, self.amp, into, n))
    }
}

/// The electric guitar: two saws and a square, picked (re-picked every 16th
/// when tremolo-picked, not at all when hammered), driven through tanh into a
/// cabinet, with a feedback octave growing on held notes.
struct Guitar {
    line: Mono,
    phase: [f32; 3],
    lo: Svf,
    hi: Svf,
    peak: Svf,
    level: f32,
}

impl Guitar {
    fn new(notes: Vec<LNote>, level: f32) -> Self {
        Self {
            line: Mono::new(notes),
            phase: [0.0; 3],
            lo: Svf::default(),
            hi: Svf::default(),
            peak: Svf::default(),
            level,
        }
    }

    fn tick(&mut self, t: f32, noise: &mut Noise) -> f32 {
        let Some((pitch, amp, into, n)) = self.line.tick(t) else {
            return 0.0;
        };
        let sr = SAMPLE_RATE as f32;
        let fq = hz(pitch);
        self.phase[0] = wrap(self.phase[0] + fq / sr);
        self.phase[1] = wrap(self.phase[1] + fq * 1.004 / sr);
        self.phase[2] = wrap(self.phase[2] + fq * 2.0 / sr);
        let (pick_age, trem) = if n.art == T {
            let a = into % STEP;
            (a, 0.55 + 0.45 * (-a * 30.0).exp())
        } else {
            (into, 1.0)
        };
        let pick = if n.art == H {
            0.0
        } else {
            noise.next() * (-pick_age * 90.0).exp() * 0.5
        };
        let feed = sine(self.phase[2])
            * 0.6
            * smooth((into - 0.7) / 0.8)
            * (n.dur > 1.0 && n.art != T) as i32 as f32;
        let x = saw(self.phase[0]) * 0.55
            + saw(self.phase[1]) * 0.45
            + pulse(self.phase[0], 0.5) * 0.25
            + pick
            + feed;
        let driven = (x * 7.0).tanh();
        let lo = self.lo.run(driven, 2900.0, 0.8).0;
        let body = self.hi.run(lo, 140.0, 0.7).2;
        let (_, peak, _) = self.peak.run(body, 1500.0, 1.3);
        (body + 0.5 * peak) * amp * trem * self.level
    }
}

// ── Karplus-Strong strings ──────────────────────────────────────────────────
const KS_LEN: usize = 4096;

struct Str {
    buf: Vec<f32>,
    w: usize,
    delay: f32,
    damp: f32,
    prev: f32,
}

impl Str {
    fn new() -> Self {
        Self {
            buf: vec![0.0; KS_LEN],
            w: 0,
            delay: 100.0,
            damp: 0.0,
            prev: 0.0,
        }
    }

    /// Fill the string with a noise burst, darker for smaller `bright`.
    fn pluck(&mut self, freq: f32, vel: f32, bright: f32, damp: f32, noise: &mut Noise) {
        self.delay = (SAMPLE_RATE as f32 / freq - 0.5).clamp(2.0, (KS_LEN - 2) as f32);
        let n = self.delay.ceil() as usize + 2;
        let mut lp = 0.0;
        for k in 0..n {
            lp += (noise.next() - lp) * bright;
            self.buf[(self.w + KS_LEN - n + k) % KS_LEN] = lp * vel;
        }
        self.damp = damp;
        self.prev = 0.0;
    }

    fn tick(&mut self) -> f32 {
        let rp = self.w as f32 + KS_LEN as f32 - self.delay;
        let i0 = rp as usize % KS_LEN;
        let fr = rp.fract();
        let s = self.buf[i0] * (1.0 - fr) + self.buf[(i0 + 1) % KS_LEN] * fr;
        self.buf[self.w] = self.damp * 0.5 * (s + self.prev);
        self.prev = s;
        self.w = (self.w + 1) % KS_LEN;
        s
    }
}

#[derive(Clone, Copy)]
struct Pluck {
    t: f32,
    freq: f32,
    vel: f32,
    bright: f32,
    damp: f32,
}

/// Strings taken in turn, each pluck stopping the oldest.
struct Strings {
    strs: Vec<Str>,
    next: usize,
    plucks: Vec<Pluck>,
    cursor: usize,
}

impl Strings {
    fn new(n: usize, mut plucks: Vec<Pluck>) -> Self {
        plucks.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        Self {
            strs: (0..n).map(|_| Str::new()).collect(),
            next: 0,
            plucks,
            cursor: 0,
        }
    }

    /// (left, right): the strings alternate a little left and right of
    /// centre, so a run of plucks moves across the field.
    fn tick(&mut self, t: f32, noise: &mut Noise) -> (f32, f32) {
        while self.cursor < self.plucks.len() && self.plucks[self.cursor].t <= t {
            let p = self.plucks[self.cursor];
            self.strs[self.next].pluck(p.freq, p.vel, p.bright, p.damp, noise);
            self.next = (self.next + 1) % self.strs.len();
            self.cursor += 1;
        }
        let (mut l, mut r) = (0.0, 0.0);
        for (k, s) in self.strs.iter_mut().enumerate() {
            let y = s.tick();
            let (gl, gr) = if k % 2 == 0 {
                (1.15, 0.85)
            } else {
                (0.85, 1.15)
            };
            l += y * gl;
            r += y * gr;
        }
        (l, r)
    }
}

#[derive(Clone, Copy)]
struct Note {
    start: f32,
    dur: f32,
    freq: f32,
    vel: f32,
}

// ── The form ─────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Intro,
    Verse,
    Drop1,
    Home,
    Dark,
    Alarm,
    Call,
    Build,
    Drop2,
    Half,
    Outro,
}

/// The part each section of the cut plays, in order.
const PARTS: [Part; 11] = [
    Part::Intro,
    Part::Verse,
    Part::Drop1,
    Part::Home,
    Part::Dark,
    Part::Alarm,
    Part::Call,
    Part::Build,
    Part::Drop2,
    Part::Half,
    Part::Outro,
];

#[derive(Clone, Copy)]
struct Bar {
    part: Part,
    pos: usize,
    len: usize,
    chord: usize,
}

fn form() -> Vec<Bar> {
    let mut out = Vec::new();
    for (i, (kind, _, from, to)) in SECTIONS.iter().enumerate() {
        let part = PARTS[i];
        let _ = kind;
        let (b0, b1) = ((from / BAR).round() as usize, (to / BAR).round() as usize);
        let len = b1 - b0;
        for pos in 0..len {
            let last = pos + 1 == len;
            let chord = match part {
                Part::Intro => INTRO_CHORDS[pos.min(6)],
                Part::Outro => OUTRO_CHORDS[pos.min(5)],
                Part::Alarm => [DM, EB][pos % 2],
                Part::Call => [CM, EB, EB, DM, DM][pos.min(4)],
                Part::Home => [DM, DM, GM, GM][pos.min(3)],
                Part::Drop2 if !last => LIFT[pos % 4],
                // Every part turns round on D, the dominant, before the next.
                _ if last => DM,
                _ => LOOP[pos % 4],
            };
            out.push(Bar {
                part,
                pos,
                len,
                chord,
            });
        }
    }
    out
}

fn is_drop(p: Part) -> bool {
    matches!(p, Part::Drop1 | Part::Drop2)
}

// ── Events ───────────────────────────────────────────────────────────────────
#[derive(Default)]
struct Events {
    kick: Vec<(f32, f32)>,
    snare: Vec<(f32, f32)>,
    clap: Vec<(f32, f32)>,
    hat: Vec<(f32, f32)>,
    open_hat: Vec<(f32, f32)>,
    shaker: Vec<(f32, f32)>,
    doum: Vec<(f32, f32)>,
    tek: Vec<(f32, f32)>,
    crash: Vec<(f32, f32)>,
    boom: Vec<(f32, f32)>,
    gong: Vec<(f32, f32)>,
    /// (time, midi pitch)
    taiko: Vec<(f32, f32)>,
    risers: Vec<f32>,
    swells: Vec<f32>,
    santur: Vec<Pluck>,
    tanpura: Vec<Pluck>,
    stab: Vec<Note>,
    arp: Vec<Note>,
    bell: Vec<Note>,
    guitar: Vec<LNote>,
    harmony: Vec<LNote>,
    duduk: Vec<LNote>,
}

fn score(fm: &[Bar]) -> Events {
    let mut e = Events::default();
    let steps = (DURATION / STEP).round() as usize;
    for i in 0..steps {
        let t = i as f32 * STEP;
        let b = (i / 16).min(fm.len() - 1);
        let Bar {
            part,
            pos,
            len,
            chord,
        } = fm[b];
        let s = i % 16;
        let swing = if s % 2 == 1 { STEP * 0.08 } else { 0.0 };
        let last = pos + 1 == len;
        let next = fm.get(b + 1).map(|n| n.part);
        let roll = last && next.is_some_and(|n| is_drop(n) && n != part);
        let fill = !roll && is_drop(part) && pos % 4 == 3;
        let voicing = PAD_VOICINGS[chord];

        // The tanpura: G and D strings in turn on every beat, but in the drops.
        if s % 4 == 0 && !is_drop(part) && part != Part::Alarm {
            let m = [50.0, 55.0, 55.0, 43.0][(s / 4) % 4];
            e.tanpura.push(Pluck {
                t,
                freq: hz(m),
                vel: 0.6,
                bright: 0.25,
                damp: 0.9995,
            });
        }
        match part {
            Part::Intro => {
                if b >= 1 && (s == 0 || s == 3) {
                    e.doum.push((t, if s == 0 { 0.7 } else { 0.45 }));
                }
            }
            Part::Verse | Part::Dark | Part::Half => {
                // Maqsum on the frame drums: D T . T D . T . in eighths.
                match s {
                    0 | 8 => e.doum.push((t, if s == 0 { 1.0 } else { 0.8 })),
                    2 | 6 | 12 => e.tek.push((t + swing, 0.8)),
                    14 if pos % 2 == 1 => e.tek.push((t, 0.5)),
                    _ => {}
                }
                e.shaker.push((
                    t + swing,
                    if s % 4 == 2 {
                        0.8
                    } else if s % 2 == 0 {
                        0.5
                    } else {
                        0.3
                    },
                ));
                if part != Part::Half && (s == 0 || s == 8) {
                    e.kick.push((t, 0.6));
                }
                if part == Part::Half {
                    if s == 0 || s == 10 {
                        e.kick.push((t, 0.9));
                    }
                    if s == 8 {
                        e.snare.push((t, 0.85));
                        e.clap.push((t, 0.7));
                    }
                }
                // In the delay the arp ticks every 8th like a clock.
                if part == Part::Dark && s % 2 == 0 {
                    e.arp.push(Note {
                        start: t,
                        dur: STEP * 0.5,
                        freq: hz(if s % 8 == 0 { 86.0 } else { 79.0 }),
                        vel: if s % 8 == 0 { 0.9 } else { 0.5 },
                    });
                }
            }
            Part::Drop1 | Part::Drop2 => {
                if s % 4 == 0 && !(roll && s >= 8) {
                    e.kick.push((t, 1.0));
                }
                if s == 14 && pos % 4 == 3 {
                    e.kick.push((t, 0.45));
                }
                if (s == 4 || s == 12) && !(fill && s == 12) {
                    e.snare.push((t, 0.9));
                    e.clap
                        .push((t, if part == Part::Drop1 { 0.5 } else { 0.8 }));
                }
                e.hat.push((
                    t + swing,
                    match s % 4 {
                        0 => 0.45,
                        2 => 0.85,
                        _ => 0.3,
                    },
                ));
                if s % 4 == 2 {
                    e.open_hat.push((t, 0.45));
                }
                if matches!(s, 6 | 14) {
                    e.tek.push((t + swing, 0.5));
                }
                if fill && s >= 12 {
                    e.snare.push((t, 0.5 + 0.1 * (s - 12) as f32));
                    e.taiko
                        .push((t + STEP * 0.5, [55.0, 50.0, 46.0, 43.0][s - 12]));
                }
                let order = [0usize, 2, 4, 2, 1, 3, 4, 3];
                e.arp.push(Note {
                    start: t,
                    dur: STEP * 0.7,
                    freq: hz(voicing[order[s % 8]] + 24.0),
                    vel: if s % 4 == 0 { 1.0 } else { 0.7 },
                });
            }
            Part::Home => {
                if s == 0 {
                    e.kick.push((t, 0.8));
                }
                if s == 8 {
                    e.snare.push((t, 0.6));
                }
                if s % 4 == 2 {
                    e.shaker.push((t, 0.5));
                }
            }
            Part::Alarm => {
                // A heartbeat: lub-dub on 1 and the 16th after, every beat
                // pair; a tick on every 8th; a taiko on each bar.
                if s == 0 || s == 3 || s == 8 || s == 11 {
                    e.kick.push((t, if s % 8 == 0 { 0.95 } else { 0.6 }));
                }
                if s % 2 == 0 {
                    e.tek.push((t, if s % 4 == 0 { 0.6 } else { 0.35 }));
                }
                if s == 0 {
                    e.taiko.push((t, 38.0));
                }
                // The alarm: Eb grinding onto D, high, every other bar.
                if pos % 2 == 0 && (s == 0 || s == 2) {
                    e.bell.push(Note {
                        start: t,
                        dur: BEAT,
                        freq: hz(if s == 0 { 87.0 } else { 86.0 }),
                        vel: 0.8,
                    });
                }
            }
            Part::Call => {
                if s == 0 {
                    e.taiko.push((t, 38.0));
                }
                if pos >= 1 && (s == 8 || s == 12) {
                    e.doum.push((t, 0.7));
                }
            }
            Part::Build => {
                let build = pos + 4 >= len;
                if build && s % 2 == 0 {
                    let k = (pos + 4 - len) as f32 / 4.0 + s as f32 / 64.0;
                    e.taiko.push((t, if s % 4 == 0 { 43.0 } else { 50.0 }));
                    e.doum.push((t, 0.3 + 0.6 * k));
                } else if !build && (s == 0 || s == 8) {
                    e.taiko.push((t, if s == 0 { 43.0 } else { 46.0 }));
                }
                if s % 4 == 2 {
                    e.shaker.push((t, 0.6));
                }
            }
            Part::Outro => {
                if pos < 3 && (s == 0 || s == 3) {
                    e.doum.push((t, 0.6));
                }
            }
        }
        if roll {
            let k = s as f32 / 16.0;
            e.snare.push((t, 0.25 + 0.6 * k));
            if s >= 12 {
                e.snare.push((t + STEP * 0.5, 0.3 + 0.6 * k));
            }
        }
        // Santur: hammered 16ths on chord tones, eighths in the dark verse
        // and the half time, a tremolo in the build, none in the alarm.
        let (play, vel, bright) = match part {
            Part::Verse => (true, 0.55 + 0.25 * (s % 4 == 0) as i32 as f32, 0.6),
            Part::Drop1 | Part::Drop2 => (true, 0.45 + 0.2 * (s % 4 == 0) as i32 as f32, 0.55),
            Part::Dark | Part::Half => (s % 2 == 0, 0.45, 0.5),
            Part::Build => (true, 0.3 + 0.3 * pos as f32 / len as f32, 0.4),
            Part::Call => (pos >= 3 && s % 2 == 0, 0.35, 0.45),
            Part::Home => (s % 4 == 0, 0.4, 0.5),
            Part::Intro => (b >= 3 && s % 2 == 0, 0.2 + 0.08 * (b - 2) as f32, 0.45),
            Part::Outro => (pos < 4 && s % 2 == 0, 0.4 * (1.0 - pos as f32 / 4.0), 0.4),
            Part::Alarm => (false, 0.0, 0.0),
        };
        if play {
            let order = [1usize, 3, 2, 4, 3, 2, 4, 1];
            // The last 16th of each eight leans on Eb over the G minor and D
            // bars: the flat sixth that gives the mode its colour.
            let m = if s % 8 == 7 && (chord == GM || chord == DM) {
                75.0
            } else {
                voicing[order[s % 8]] + 12.0
            };
            e.santur.push(Pluck {
                t: t + swing,
                freq: hz(m),
                vel,
                bright,
                damp: 0.998,
            });
        }
    }

    // Section downbeats: crash; the gong and the boom on the drops and the turns.
    for (b, bar) in fm.iter().enumerate() {
        if bar.pos != 0 || b == 0 {
            continue;
        }
        let at = bars(b as f32);
        e.crash.push((
            at,
            if matches!(bar.part, Part::Alarm | Part::Call) {
                0.6
            } else {
                1.0
            },
        ));
        if is_drop(bar.part) || matches!(bar.part, Part::Outro | Part::Verse | Part::Alarm) {
            e.gong.push((at, 1.0));
            e.boom.push((at, 1.0));
        }
        if is_drop(bar.part) || matches!(bar.part, Part::Verse | Part::Alarm) {
            e.risers.push(b as f32);
            e.swells.push(at);
        }
    }

    // Intro: the gong opens; a shimmer of santur climbs under the banner; a
    // taiko on each model panel, climbing the G minor chord; a roll into the
    // first take.
    e.gong.push((0.0, 0.8));
    for (at, m) in PANEL_AT.iter().zip([43.0, 46.0, 50.0, 55.0, 58.0]) {
        e.taiko.push((*at, m));
    }
    let intro_bars = (SECTIONS[0].3 / BAR).round();
    let last = intro_bars - 1.0;
    for (k, m) in [55.0, 54.0, 51.0, 50.0, 48.0, 46.0, 45.0, 43.0]
        .iter()
        .enumerate()
    {
        e.taiko
            .push((bars(last) + 2.0 * BEAT + k as f32 * STEP, *m));
    }
    for k in 0..12 {
        e.santur.push(Pluck {
            t: 0.4 + k as f32 * STEP * 2.0,
            freq: hz([62.0, 67.0, 70.0, 74.0][k % 4] + 12.0 * (k / 4) as f32),
            vel: 0.35,
            bright: 0.5,
            damp: 0.999,
        });
    }

    // The melodic parts, phrased from each part's first bar.
    let mut b = 0;
    while b < fm.len() {
        let Bar { part, len, .. } = fm[b];
        let (from, to) = (b as f32, (b + len) as f32);
        match part {
            Part::Verse | Part::Dark => {
                // The duduk calls every other four bars; the riff joins
                // quietly in the last four of the first verse.
                let mut p = from + if part == Part::Dark { 4.0 } else { 0.0 };
                while p + 4.0 <= to - 1.0 + 0.01 {
                    lay(
                        &mut e.duduk,
                        bars(p),
                        &DUDUK,
                        0.0,
                        1.0,
                        if part == Part::Dark { 0.65 } else { 0.75 },
                    );
                    p += 8.0;
                }
                if part == Part::Verse {
                    riff(&mut e.stab, fm, (to - 4.0).max(from), to, 0.35);
                }
            }
            Part::Drop1 => {
                riff(&mut e.stab, fm, from, to, 0.9);
                let t = lay(&mut e.guitar, bars(from), &MOTIF, 0.0, 1.0, 1.0);
                lay(&mut e.guitar, t, &ANSWER, 0.0, 1.0, 0.95);
            }
            Part::Home => {
                lay(&mut e.guitar, bars(from), &HOME, 0.0, 1.0, 0.9);
                for (k, m) in [67.0, 70.0, 74.0, 79.0].iter().enumerate() {
                    e.bell.push(Note {
                        start: bars(from + 2.0) + k as f32 * STEP * 2.0,
                        dur: BEAT,
                        freq: hz(*m + 12.0),
                        vel: 0.7,
                    });
                }
            }
            Part::Alarm => {
                // The duduk's motif, its first two bars only, low and slow.
                lay(&mut e.duduk, bars(from + 1.0), &DUDUK[..5], -12.0, 2.0, 0.7);
            }
            Part::Call => {
                // Concern: a held G on the duduk. Defiance: the guitar's wide
                // bend on the second bar. The decision: F# up to A on D.
                lay(&mut e.duduk, bars(from), &[(67.0, 3.5, B)], 0.0, 1.0, 0.7);
                lay(
                    &mut e.guitar,
                    bars(from + 1.0),
                    &[
                        (79.0, 3.0, W),
                        (75.0, 1.0, N),
                        (75.0, 4.0, B),
                        (78.0, 2.0, S),
                        (81.0, 6.0, B),
                    ],
                    0.0,
                    1.0,
                    0.95,
                );
            }
            Part::Build => {
                let mut p = from;
                while p + 4.0 <= to - 2.0 + 0.01 {
                    lay(&mut e.duduk, bars(p), &DUDUK, 0.0, 1.0, 0.8);
                    p += 4.0;
                }
                riff(&mut e.stab, fm, to - 4.0, to, 0.6);
            }
            Part::Drop2 => {
                riff(&mut e.stab, fm, from, to - 1.0, 1.0);
                let mut lead = Vec::new();
                let t = lay(&mut lead, bars(from), &TREMOLO, 0.0, 1.0, 1.0);
                e.harmony.extend(twin(&lead));
                let mut answer = Vec::new();
                lay(&mut answer, t, &ANSWER, 0.0, 1.0, 0.95);
                e.harmony.extend(twin(&answer));
                lead.extend(answer);
                e.guitar
                    .extend(lead.into_iter().filter(|n| n.start < bars(to) - 0.05));
                e.harmony.retain(|n| n.start < bars(to) - 0.05);
            }
            Part::Half => {
                riff(&mut e.stab, fm, from, to - 4.0, 0.55);
                let mut p = from;
                let mut k = 0;
                while p + 2.0 <= to - 4.0 + 0.01 {
                    match k % 2 {
                        0 => _ = lay(&mut e.guitar, bars(p), &CALL_A, 0.0, 1.0, 0.9),
                        _ => _ = lay(&mut e.duduk, bars(p), &CALL_B, 0.0, 1.0, 0.85),
                    }
                    p += 2.0;
                    k += 1;
                }
                taps(&mut e.guitar, fm, to - 4.0, 3, 0.8);
            }
            Part::Outro => {
                lay(
                    &mut e.guitar,
                    bars(from),
                    &[(74.0, 4.0, B), (78.0, 4.0, W), (79.0, 10.0, D)],
                    0.0,
                    1.0,
                    0.9,
                );
                lay(
                    &mut e.duduk,
                    bars(from + 2.0),
                    &[(62.0, 4.0, B), (67.0, 8.0, N)],
                    0.0,
                    1.0,
                    0.6,
                );
            }
            Part::Intro => {}
        }
        b += len;
    }

    for v in [
        &mut e.kick,
        &mut e.snare,
        &mut e.clap,
        &mut e.hat,
        &mut e.open_hat,
        &mut e.shaker,
        &mut e.doum,
        &mut e.tek,
        &mut e.crash,
        &mut e.boom,
        &mut e.gong,
        &mut e.taiko,
    ] {
        v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }
    for v in [&mut e.stab, &mut e.arp, &mut e.bell] {
        v.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
    }
    e
}

/// The riff: a 3-3-2 stab on each bar's chord, falling through its tones.
fn riff(out: &mut Vec<Note>, fm: &[Bar], from: f32, to: f32, vel: f32) {
    let mut b = from;
    while b < to - 0.01 {
        let v = PAD_VOICINGS[fm[(b as usize).min(fm.len() - 1)].chord];
        for (s, k) in [(0usize, 4usize), (3, 3), (6, 2), (8, 4), (11, 3), (14, 1)] {
            out.push(Note {
                start: bars(b) + s as f32 * STEP,
                dur: STEP * 1.6,
                freq: hz(v[k] + 12.0),
                vel,
            });
        }
        b += 1.0;
    }
}

/// Tapped arpeggios: 16ths up and down the chord two octaves over the pad.
fn taps(out: &mut Vec<LNote>, fm: &[Bar], bar: f32, count: usize, vel: f32) {
    for k in 0..count {
        let chord = fm[((bar as usize) + k).min(fm.len() - 1)].chord;
        let tones = PAD_VOICINGS[chord].map(|m| m + 24.0);
        for (i, idx) in [0usize, 1, 2, 3, 4, 3, 2, 1, 0, 1, 2, 3, 4, 3, 2, 1]
            .iter()
            .enumerate()
        {
            out.push(LNote {
                start: bars(bar + k as f32) + i as f32 * STEP,
                dur: STEP,
                midi: tones[*idx],
                art: H,
                vel,
            });
        }
    }
}

/// A twin guitar's line: each note two steps of the scale below.
fn twin(notes: &[LNote]) -> Vec<LNote> {
    notes
        .iter()
        .map(|n| {
            let mut x = n.midi as i32;
            let mut k = 0;
            while k < 2 {
                x -= 1;
                if SCALE.contains(&x.rem_euclid(12)) && x.rem_euclid(12) != 5 {
                    k += 1;
                }
            }
            LNote {
                midi: x as f32,
                vel: n.vel * 0.75,
                ..*n
            }
        })
        .collect()
}

// ── DSP building blocks ─────────────────────────────────────────────────────
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

// ── The render ───────────────────────────────────────────────────────────────
pub fn render() -> Vec<[f32; 2]> {
    let fm = form();
    let mut ev = score(&fm);
    let sr = SAMPLE_RATE as f32;
    let total = (DURATION * sr) as usize;
    let mut out = Vec::with_capacity(total);

    let mut noise_l = Noise(0x1234_5678);
    let mut noise_r = Noise(0x9E37_79B9);
    let mut noise_d = Noise(0x0BAD_F00D);
    let mut noise_c = Noise(0x5EED_CAFE);
    let mut noise_k = Noise(0xC0FF_EE11);
    let mut vowel = [
        [Svf::default(), Svf::default(), Svf::default()],
        [Svf::default(), Svf::default(), Svf::default()],
    ];
    let mut f = [(); 13].map(|_| Svf::default());
    let mut pad_phase = [[0.0f32; 2]; 5];
    let mut bass_phase = 0.0f32;
    let mut duduk_phase = 0.0f32;
    let mut echo_l = Delay::new(BEAT * 0.75);
    let mut echo_r = Delay::new(BEAT * 0.75);
    let (mut verb_l, mut verb_r) = (Reverb::new(1.0), Reverb::new(1.07));

    let mut santur = Strings::new(12, std::mem::take(&mut ev.santur));
    let mut tanpura = Strings::new(4, std::mem::take(&mut ev.tanpura));
    let mut guitar = Guitar::new(std::mem::take(&mut ev.guitar), 0.16);
    let mut harmony = Guitar::new(std::mem::take(&mut ev.harmony), 0.12);
    let mut duduk = Mono::new(std::mem::take(&mut ev.duduk));
    let mut cur = [(); 12].map(|_| Cursor(0));
    let (mut n_stab, mut n_arp, mut n_bell) = (NoteCursor(0), NoteCursor(0), NoteCursor(0));

    for i in 0..total {
        let t = i as f32 / sr;
        let bi = ((t / BAR) as usize).min(fm.len() - 1);
        let Bar {
            part,
            pos,
            len,
            chord,
        } = fm[bi];
        let key = 0.0;
        let bar = t / BAR;
        let step_age = t % STEP;
        let step = (t / STEP) as usize % 16;
        let [c_kick, c_snare, c_clap, c_hat, c_oh, c_shaker, c_doum, c_tek, c_crash, c_boom, c_gong, c_taiko] =
            &mut cur;
        let [snare_f, clap_f, hat_f, shaker_f, tek_f, crash_f, taiko_f, riser_f, stab_f, bass_f, d_f1, d_f2, d_br] =
            &mut f;

        // ── Drums and skins.
        let mut drums = 0.0;
        let mut pump = 1.0;
        if let Some((age, v)) = c_kick.last(&ev.kick, t) {
            if age < 0.5 {
                let ph = 45.0 * age + 140.0 * (1.0 - (-age * 28.0).exp()) / 28.0;
                drums += sine(ph) * v * ((-age * 9.0).exp() + 0.3 * (-age * 90.0).exp()) * 1.1;
                pump = 1.0 - 0.4 * v * (-age / 0.1).exp();
            }
        }
        if let Some((age, v)) = c_snare.last(&ev.snare, t) {
            if age < 0.18 {
                let (_, nb, _) = snare_f.run(noise_l.next(), 2400.0, 0.7);
                let body = sine(196.0 * age) * 0.5 + sine(293.7 * age) * 0.25;
                let gate = 1.0 - smooth((age - 0.15) / 0.03);
                drums +=
                    (nb * 1.3 + body * (-age * 30.0).exp()) * v * (-age * 5.0).exp() * gate * 0.6;
            }
        }
        if let Some((age, v)) = c_clap.last(&ev.clap, t) {
            if age < 0.25 {
                let (_, cb, _) = clap_f.run(noise_c.next(), 1300.0, 1.4);
                let burst = [0.0f32, 0.011, 0.022]
                    .iter()
                    .map(|o| {
                        if age >= *o {
                            (-(age - o) * 120.0).exp()
                        } else {
                            0.0
                        }
                    })
                    .fold(0.0, f32::max)
                    + 0.35 * (-age * 18.0).exp();
                drums += cb * burst * v * 0.85;
            }
        }
        let hiss = noise_r.next();
        if let Some((age, v)) = c_hat.last(&ev.hat, t) {
            let (_, _, hh) = hat_f.run(hiss, 9000.0, 0.8);
            drums += hh * v * (-age * 80.0).exp() * 0.28;
            if let Some((oage, ov)) = c_oh.last(&ev.open_hat, t) {
                if oage < 0.4 {
                    drums += hh * ov * (-oage * 9.0).exp() * 0.2;
                }
            }
        }
        // Shaker: seeds in a gourd, a soft high hiss with a slow attack.
        if let Some((age, v)) = c_shaker.last(&ev.shaker, t) {
            if age < 0.12 {
                let (_, sb, _) = shaker_f.run(hiss, 6500.0, 1.2);
                drums += sb * v * (age / 0.012).min(1.0) * (-age * 40.0).exp() * 0.22;
            }
        }
        // Frame drum: the doum, a low skin, and the tek, a slap at the rim.
        let mut skins = 0.0;
        if let Some((age, v)) = c_doum.last(&ev.doum, t) {
            if age < 0.6 {
                let ph = 70.0 * age + 50.0 * (1.0 - (-age * 18.0).exp()) / 18.0;
                skins += sine(ph) * (-age * 7.0).exp() * v * 0.75;
            }
        }
        if let Some((age, v)) = c_tek.last(&ev.tek, t) {
            if age < 0.15 {
                let (_, tb, _) = tek_f.run(noise_d.next(), 3400.0, 2.0);
                skins += (tb * 1.1 + sine(1750.0 * age) * 0.3) * (-age * 45.0).exp() * v * 0.5;
            }
        }
        if let Some((age, m)) = c_taiko.last(&ev.taiko, t) {
            if age < 0.6 {
                let fq = hz(m);
                let ph = fq * 0.8 * age + fq * 0.2 * (1.0 - (-age * 25.0).exp()) / 25.0;
                let (_, slap, _) = taiko_f.run(noise_k.next(), 1200.0, 1.0);
                skins += (sine(ph) * (-age * 6.0).exp() + slap * 0.5 * (-age * 60.0).exp()) * 0.75;
            }
        }
        let mut crash = 0.0;
        if let Some((age, v)) = c_crash.last(&ev.crash, t) {
            if age < 2.5 {
                let (_, _, ch) = crash_f.run(noise_d.next(), 5000.0, 0.5);
                crash = ch * v * (-age * 1.6).exp() * 0.28;
            }
        }
        if let Some((age, v)) = c_boom.last(&ev.boom, t) {
            if age < 2.0 {
                let ph = 38.0 * age + 60.0 * (1.0 - (-age * 6.0).exp()) / 6.0;
                drums += sine(ph) * v * (-age * 2.2).exp() * 0.9;
            }
        }
        let mut gong = 0.0;
        if let Some((age, v)) = c_gong.last(&ev.gong, t) {
            if age < 6.0 {
                let f0 = hz(36.0);
                for (ratio, amp, decay) in [
                    (1.0, 1.0, 0.5),
                    (1.483, 0.6, 0.7),
                    (2.11, 0.45, 0.9),
                    (2.74, 0.3, 1.2),
                    (3.9, 0.2, 1.6),
                ] {
                    gong += sine(f0 * ratio * age) * amp * (-age * decay).exp();
                }
                gong *= v * (1.0 - (-age * 30.0).exp()) * 0.2;
            }
        }
        let mut riser = 0.0;
        if let Some(&end) = ev.risers.iter().find(|&&e| bar >= e - 1.0 && bar < e) {
            let p = bar - (end - 1.0);
            let (_, rb, _) = riser_f.run(hiss, 300.0 * 20f32.powf(p), 2.5);
            riser = rb * p * 0.35;
        }
        // A reverse cymbal: the crash's hiss swelling over two beats into the
        // downbeat, cut dead on it.
        if let Some(&at) = ev.swells.iter().find(|&&a| t >= a - 2.0 * BEAT && t < a) {
            let k = (t - (at - 2.0 * BEAT)) / (2.0 * BEAT);
            let (_, _, ch) = crash_f.run(noise_d.next(), 5000.0, 0.5);
            riser += ch * k * k * k * 0.45;
        }

        // ── Strings: santur and tanpura. The tanpura's bridge buzzes (jawari).
        let (sl, sr_) = santur.tick(t, &mut noise_c);
        let (sl, sr_) = (sl * 0.26, sr_ * 0.26);
        let sant = 0.5 * (sl + sr_);
        let (tl, tr) = tanpura.tick(t, &mut noise_k);
        let tp = 0.5 * (tl + tr);
        let tanp = (tp + 0.35 * (tp * 5.0).tanh()) * 0.2;

        // ── Choir pad: the detuned saws sung through an "ah".
        let pad_gain = match part {
            Part::Intro => smooth(bar / 2.0),
            Part::Outro => 1.0 - smooth((pos as f32 + bar.fract() - 3.0) / 3.0),
            Part::Call | Part::Build | Part::Alarm => 1.0,
            _ => 0.65,
        };
        let mut pad = [0.0f32; 2];
        for (v, note) in PAD_VOICINGS[chord].iter().enumerate() {
            let fq = hz(*note + key);
            for (side, det) in [0.996f32, 1.004].iter().enumerate() {
                pad_phase[v][side] = wrap(pad_phase[v][side] + fq * det / sr);
                pad[side] += saw(pad_phase[v][side]) * 0.12 + sine(pad_phase[v][side]) * 0.05;
            }
        }
        let open = if part == Part::Build {
            smooth((pos as f32 + bar.fract() - (len as f32 - 4.0)) / 4.0)
        } else {
            0.0
        };
        let body_cut = 700.0 + 2600.0 * open;
        let shimmer = 0.85 + 0.15 * sine(t * 0.8);
        let mut choir = [0.0f32; 2];
        for (side, [lo, f1, f2]) in vowel.iter_mut().enumerate() {
            let x = pad[side];
            let body = lo.run(x, body_cut * (1.0 + 0.05 * side as f32), 0.8).0;
            let (_, a, _) = f1.run(x, 730.0, 4.0);
            let (_, o, _) = f2.run(x, 1090.0, 5.0);
            choir[side] = (body * 0.7 + a * 1.1 + o * 0.7) * pad_gain * shimmer;
        }

        // ── Bass.
        let mut bass = 0.0;
        let root = hz(PAD_VOICINGS[chord][0] + key);
        match part {
            Part::Verse | Part::Half | Part::Dark => {
                let off = matches!(step, 2 | 6 | 10 | 14);
                let fq = if off { root * 2.0 } else { root };
                bass_phase = wrap(bass_phase + fq / sr);
                let gate = if off || step.is_multiple_of(4) {
                    (-step_age * 9.0).exp()
                } else {
                    0.0
                };
                bass = (pulse(bass_phase, 0.5) * 0.6 + sine(bass_phase) * 0.6) * gate * 0.36;
            }
            Part::Drop1 | Part::Drop2 => {
                let accent = if step.is_multiple_of(2) { 1.0 } else { 0.6 };
                bass_phase = wrap(bass_phase + root * 2.0 / sr);
                let slide = (1.0 - step_age / STEP * 0.7).max(0.1);
                bass = (pulse(bass_phase, 0.5) * 0.55 + sine(bass_phase * 0.5) * 0.4)
                    * accent
                    * slide
                    * 0.36;
            }
            Part::Alarm => {
                // The pedal: D, pulsing with the heartbeat.
                bass_phase = wrap(bass_phase + hz(38.0) / sr);
                let gate = if matches!(step, 0 | 3 | 8 | 11) {
                    (-step_age * 6.0).exp()
                } else {
                    0.15
                };
                bass = (sine(bass_phase) * 0.8 + pulse(bass_phase, 0.5) * 0.2) * gate * 0.4;
            }
            Part::Call | Part::Build | Part::Home | Part::Intro | Part::Outro if bar >= 2.0 => {
                bass_phase = wrap(bass_phase + root / sr);
                bass = sine(bass_phase) * 0.28;
            }
            _ => {}
        }
        let bass = bass_f.run(bass, 900.0, 0.9).0;

        // ── Supersaw stab on the riff, and the pulse arp.
        let mut stab = 0.0;
        for n in n_stab.active(&ev.stab, t, 0.06) {
            let age = t - n.start;
            let mut s = 0.0;
            for d in [0.991f32, 0.997, 1.0, 1.003, 1.009] {
                s += saw(n.freq * d * age + d * 7.0);
            }
            stab += s * 0.3 * adsr(age, n.dur, 0.003, 0.06) * n.vel;
        }
        let stab = stab_f.run(stab, 2400.0, 1.2).0 * 0.11;
        // The arp ping-pongs, every other 16th on the other side.
        let (mut al, mut ar) = (0.0, 0.0);
        for n in n_arp.active(&ev.arp, t, 0.02) {
            let age = t - n.start;
            let y = (pulse(n.freq * age, 0.25) + 0.5) * (-age * 18.0).exp() * n.vel * 0.045;
            if ((n.start / STEP).round() as i32) % 2 == 0 {
                al += y * 1.4;
                ar += y * 0.6;
            } else {
                al += y * 0.6;
                ar += y * 1.4;
            }
        }
        let arp = 0.5 * (al + ar);

        // ── Bell.
        let mut bell = 0.0;
        for n in n_bell.active(&ev.bell, t, 1.6) {
            let age = t - n.start;
            let index = 2.2 * (-age * 3.0).exp();
            bell += sine(n.freq * age + index * sine(n.freq * 3.5 * age) / TAU)
                * (-age * 1.4).exp()
                * n.vel
                * (age / 0.004).min(1.0);
        }
        let bell = bell * 0.15;

        // ── The electric guitars: the lead, and the twin a little to the left.
        let gtr = guitar.tick(t, &mut noise_l);
        let twin_g = harmony.tick(t, &mut noise_c);
        // ── The duduk: a reed through two formants, with breath.
        let mut reed = 0.0;
        if let Some((pitch, amp, _, _)) = duduk.tick(t) {
            let fq = hz(pitch);
            duduk_phase = wrap(duduk_phase + fq / sr);
            let x = pulse(duduk_phase, 0.32) + 0.36 + saw(duduk_phase) * 0.4;
            let (_, a, _) = d_f1.run(x, 620.0, 3.0);
            let (_, b2, _) = d_f2.run(x, 1150.0, 4.0);
            let (_, br, _) = d_br.run(noise_k.next(), 1400.0, 1.5);
            reed = (a * 0.9 + b2 * 0.5 + br * 0.15) * amp * 0.3;
        }

        // ── Mix: one level throughout. The melodic bus pumps with the kick.
        let melodic = (stab + gtr + reed) * pump;
        echo_l.push(
            (gtr * 0.8 + twin_g * 0.6 + reed * 0.5 + arp + sant * 0.3) * pump + echo_r.tap() * 0.35,
        );
        let el = echo_l.tap();
        echo_r.push(el * 0.8);
        let er = echo_r.tap();
        let send = choir[0] * 0.3
            + bell
            + gtr * 0.35
            + twin_g * 0.3
            + reed * 0.6
            + sant * 0.5
            + tanp * 0.4
            + gong * 0.4
            + skins * 0.25
            + drums * 0.06;
        let rl = verb_l.run(send, 0.84);
        let rr = verb_r.run(send, 0.84);
        let center = drums + skins + gong + bass * pump + melodic + riser + tanp;
        let mut lft = center
            + twin_g * 0.75 * pump
            + (sl + al) * pump
            + choir[0] * pump
            + el * 0.45
            + rl * 0.55
            + crash
            + bell * 0.6;
        let mut rgt = center
            + twin_g * 0.35 * pump
            + (sr_ + ar) * pump
            + choir[1] * pump
            + er * 0.45
            + rr * 0.55
            + crash * 0.9
            + bell * 0.6;
        let fade = ((DURATION - t) / 1.5).clamp(0.0, 1.0);
        lft = soft_clip(lft * 0.6) * fade;
        rgt = soft_clip(rgt * 0.6) * fade;
        out.push([lft, rgt]);
    }
    out
}

fn soft_clip(x: f32) -> f32 {
    x.tanh() * 0.95
}

#[cfg(test)]
mod tests {
    use super::*;

    fn midi_of(freq: f32) -> f32 {
        (69.0 + 12.0 * (freq / 440.0).log2()).round()
    }

    /// Every written pitch is in G minor (harmonic, with F for the Bb chord).
    #[test]
    fn every_pitch_is_in_the_key() {
        let fm = form();
        let e = score(&fm);
        let ok = |m: f32| SCALE.contains(&(m.round() as i32).rem_euclid(12));
        for n in e.guitar.iter().chain(&e.harmony).chain(&e.duduk) {
            assert!(ok(n.midi), "lead {} at {:.2}", n.midi, n.start);
        }
        for n in e.stab.iter().chain(&e.arp).chain(&e.bell) {
            assert!(
                ok(midi_of(n.freq)),
                "note {} at {:.2}",
                midi_of(n.freq),
                n.start
            );
        }
        for p in e.santur.iter().chain(&e.tanpura) {
            assert!(
                ok(midi_of(p.freq)),
                "string {} at {:.2}",
                midi_of(p.freq),
                p.t
            );
        }
    }

    /// The motif and its answers sit on the harmony: what sounds on beats 1
    /// and 3 of a bar is a tone of that bar's chord.
    #[test]
    fn the_lines_land_on_chord_tones() {
        let lines: [(&Line, &[usize]); 4] = [
            (&MOTIF, &LOOP),
            (&ANSWER, &LOOP),
            (&DUDUK, &LOOP),
            (&TREMOLO, &LIFT),
        ];
        for (line, chords) in lines {
            let mut beat = 0.0f32;
            for &(m, b, _) in line {
                for strong in [0.0f32, 2.0] {
                    let bar = (beat / 4.0).floor();
                    let at = bar * 4.0 + strong;
                    if m > 0.0 && at >= beat - 1e-3 && at < beat + b - 1e-3 {
                        let chord = chords[bar as usize % chords.len()];
                        let tones: Vec<i32> = PAD_VOICINGS[chord]
                            .iter()
                            .map(|v| (*v as i32).rem_euclid(12))
                            .collect();
                        assert!(
                            tones.contains(&((m as i32).rem_euclid(12))),
                            "{m} on beat {} of bar {}",
                            strong + 1.0,
                            bar + 1.0
                        );
                    }
                }
                beat += b;
            }
            assert_eq!(
                beat,
                if line.len() == TREMOLO.len() {
                    16.0
                } else {
                    16.0
                }
            );
        }
    }

    #[test]
    fn the_form_covers_the_film_and_turns_on_the_dominant() {
        let fm = form();
        assert_eq!(fm.len() as f32 * BAR, DURATION);
        for w in fm.windows(2) {
            if w[1].pos == 0 && matches!(w[1].part, Part::Drop1 | Part::Build | Part::Half) {
                assert_eq!(
                    w[0].chord, DM,
                    "{:?} entered from chord {}",
                    w[1].part, w[0].chord
                );
            }
        }
        assert_eq!(fm.last().unwrap().chord, GM);
    }

    /// The guitar is one voice: its notes never overlap.
    #[test]
    fn the_guitar_plays_one_note_at_a_time() {
        let e = score(&form());
        let mut g = e.guitar.clone();
        g.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        for w in g.windows(2) {
            assert!(
                w[0].start + w[0].dur <= w[1].start + 0.02,
                "guitar notes overlap at {:.2}s",
                w[1].start
            );
        }
    }
}
