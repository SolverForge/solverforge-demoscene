// WORKBENCH_12 soundtrack: the latest Planner123 theme (planner123_06),
// remixed for the SolverForge Workbench.
//
// Kept from planner123_06: 125 BPM, the loop's shape (read in C Phrygian as
// Cm - Ab - Bbm - Db), the lead riff (on the supersaw stab), the 16th square
// bass and the offbeat octave bass. The guitar plays the remix's own solo.
//
// Rearranged: the form has an arc (cut.rs sets where each part starts, on the
// bar nearest its chapter):
//   intro    drone, frame-drum heartbeat, a taiko on each model panel, the
//            duduk's first call, a roll into the warp;
//   verse    hammered santur on 16ths over a tanpura drone, frame drums in
//            maqsum, the duduk answering every four bars;
//   drop 1   the machine starts: four on the floor, gated snare and clap,
//            the riff on supersaws, the square bass, and the electric solo;
//   break    drums out, Fm - Ab - Bbm - Cm under choir and santur tremolo,
//            the guitar's slow bends and a taiko build;
//   drop 2   up a whole step to D Phrygian: everything, plus a pulse arp,
//            the solo again a step higher; it pivots in and out on C minor;
//   half     half-time, guitar and duduk trading calls;
//   final    C minor, the solo's answer climbing to the high G;
//   outro    Ab - Db - Cm, the gong, the guitar's last held bend and dive.
//
// Archeofuturistic: ancient instruments played by a machine. The strings
// (santur, tanpura, the lyre's plucks) are Karplus-Strong strings, the duduk
// is a reed through two formants with breath, the frame drums and taiko are
// skins; around them run the supersaw stabs, the square bass, the pulse arp,
// the gated snare and the sidechain pump.
//
// The lead is an electric guitar in the tracker style of One Must Fall: one
// monophonic voice with a continuous phase and a smoothed pitch, so notes
// slide into each other; notes marked B are fretted a whole step low and
// bent up to pitch, held notes grow a vibrato and a feedback octave, and
// phrase ends marked D dive an octave. It is driven through tanh and a
// cabinet filter.
//
// The music keeps one level from the first frame to the last; the narration
// is mixed over it in the composition, never by turning the music down.
//
// The whole track is rendered once, sample by sample, into a buffer. Live
// playback and the WAV export read the same samples.
use crate::cut::{CHAPTERS, SPEECH};
use crate::timeline::{
    bars, smooth, BAR, BEAT, DURATION, INTRO_BARS, LIVE_FROM, OUTRO_BARS, PANEL_AT, STEP,
    TOTAL_BARS,
};

pub const SAMPLE_RATE: u32 = 44_100;
const TAU: f32 = std::f32::consts::TAU;

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

/// One-pole coefficient for a time constant in seconds.
fn coeff(tau: f32) -> f32 {
    1.0 - (-1.0 / (tau * SAMPLE_RATE as f32)).exp()
}

// ── Harmony ──────────────────────────────────────────────────────────────────
const CM: usize = 0;
const AB: usize = 1;
const BB: usize = 2;
const DB: usize = 3;
const FM: usize = 4;
const ROOTS: [f32; 5] = [48.0, 44.0, 46.0, 49.0, 41.0];
// The mode is C Phrygian (C Db Eb F G Ab Bb): natural minor with its second
// flattened, the darkest mode with a minor tonic. Planner123's loop keeps its
// shape, Cm - Ab - Bb - G, re-read in the mode: Cm - Ab - Bbm - Db, so every
// turn of the loop ends on the flat II falling a semitone home to C minor,
// the Phrygian cadence. Bare triads on fifths, nothing sweet.
const PAD_VOICINGS: [[f32; 5]; 5] = [
    [48.0, 55.0, 60.0, 63.0, 67.0], // Cm
    [44.0, 51.0, 56.0, 60.0, 63.0], // Ab
    [46.0, 53.0, 58.0, 61.0, 65.0], // Bbm
    [49.0, 56.0, 61.0, 65.0, 68.0], // Db
    [41.0, 48.0, 53.0, 56.0, 60.0], // Fm
];
const LOOP: [usize; 4] = [CM, AB, BB, DB];
const BREAK_LOOP: [usize; 4] = [FM, AB, BB, CM];
// Two bars of C minor under the drone, then the loop itself under the
// duduk's first statement of the motif (bars 3-6), and Db into the verse.
const INTRO_CHORDS: [usize; 7] = [CM, CM, CM, AB, BB, DB, DB];
const OUTRO_CHORDS: [usize; 6] = [AB, DB, CM, CM, CM, CM];

// Planner123's lead riff (its fourth bar's G read as Ab, the fifth of Db), 16th steps over 4 bars (0 = rest).
const PLANNER_RIFF: [u8; 64] = [
    72, 0, 70, 0, 72, 0, 72, 0, 70, 0, 72, 0, 72, 0, 70, 72, 68, 0, 68, 0, 67, 0, 68, 0, 68, 0, 68,
    0, 67, 0, 68, 0, 70, 0, 70, 0, 68, 0, 70, 0, 70, 0, 70, 0, 68, 0, 70, 0, 68, 0, 68, 0, 65, 0,
    68, 0, 68, 0, 70, 0, 70, 0, 70, 72,
];

// ── The guitar's lines ──────────────────────────────────────────────────────
/// Plain note, bent up a whole step into pitch, or diving an octave at its end.
#[derive(Clone, Copy, PartialEq)]
enum Art {
    N,
    B,
    D,
}
use Art::{B, D, N};

/// A melodic line: (midi, beats, articulation), 0 for a rest.
type Line = [(f32, f32, Art)];

/// The remix's solo over the loop (midi, beats, articulation), in C minor
/// pentatonic with the mode's Db where the chord holds it: seven bars here and the eighth in SOLO_END, so a part can end
/// on that cadence alone. It is built from one motif: a bent chord tone held
/// a beat and a half, then three notes falling through the chord (bar 1).
/// Bars 2-4 carry it down the loop, each bar falling onto that chord's root
/// or third; bar 4 ends on C over Db, the cadence in the melody. Bars 5-7
/// sing it an octave up, one bent note and one held note per bar. Bar 8
/// holds Db and dives from F, handing back to C minor. Every note on beats 1 and 3 is a tone of its bar's chord
/// (pinned by a test).
const SOLO: [(f32, f32, Art); 25] = [
    (79.0, 1.5, B),
    (77.0, 0.5, N),
    (75.0, 1.0, N),
    (72.0, 1.0, N),
    (75.0, 1.5, B),
    (77.0, 0.5, N),
    (72.0, 1.0, N),
    (68.0, 1.0, N),
    (77.0, 1.5, B),
    (73.0, 0.5, N),
    (70.0, 1.0, N),
    (73.0, 1.0, N),
    (80.0, 1.5, N),
    (77.0, 0.5, N),
    (73.0, 1.0, N),
    (72.0, 1.0, N),
    (84.0, 1.5, B),
    (82.0, 0.5, N),
    (87.0, 2.0, N),
    (84.0, 1.5, N),
    (82.0, 0.5, N),
    (80.0, 2.0, N),
    (89.0, 1.5, B),
    (85.0, 0.5, N),
    (82.0, 2.0, N),
];

const SOLO_END: [(f32, f32, Art); 3] = [(85.0, 1.5, N), (84.0, 0.5, N), (77.0, 2.0, D)];

/// The whole solo, eight bars.
fn solo() -> Vec<(f32, f32, Art)> {
    SOLO.iter().chain(&SOLO_END).copied().collect()
}

/// The answer, eight more bars: the motif turned upside down, climbing
/// through each chord where the solo fell, in the same two rhythms, to the
/// high G, then falling back onto Db and diving into C minor.
const ANSWER: [(f32, f32, Art); 28] = [
    (72.0, 1.5, B),
    (75.0, 0.5, N),
    (79.0, 1.0, N),
    (84.0, 1.0, N),
    (80.0, 1.5, B),
    (84.0, 0.5, N),
    (87.0, 1.0, N),
    (84.0, 1.0, N),
    (82.0, 1.5, B),
    (85.0, 0.5, N),
    (89.0, 1.0, N),
    (85.0, 1.0, N),
    (89.0, 1.5, N),
    (87.0, 0.5, N),
    (85.0, 1.0, N),
    (84.0, 1.0, N),
    (91.0, 1.5, B),
    (87.0, 0.5, N),
    (84.0, 2.0, N),
    (87.0, 1.5, B),
    (84.0, 0.5, N),
    (80.0, 2.0, N),
    (85.0, 1.5, N),
    (82.0, 0.5, N),
    (77.0, 2.0, N),
    (80.0, 1.5, N),
    (77.0, 0.5, N),
    (73.0, 2.0, D),
];

/// The breakdown's line over Fm - Ab - Bbm - Cm: one bent chord tone per
/// bar, held and singing, a step up each bar, and the last one diving.
const BREAK_LINE: [(f32, f32, Art); 7] = [
    (72.0, 3.0, B),
    (68.0, 1.0, N),
    (75.0, 3.0, B),
    (72.0, 1.0, N),
    (77.0, 3.0, B),
    (73.0, 1.0, N),
    (79.0, 4.0, D),
];

/// The motif as the duduk states it, before the guitar takes it: long chord
/// tones, the mode's Db turning onto C each time.
const DUDUK: [(f32, f32, Art); 14] = [
    (67.0, 2.0, B),
    (63.0, 1.0, N),
    (61.0, 0.5, N),
    (60.0, 0.5, N),
    (68.0, 2.0, N),
    (63.0, 0.5, N),
    (61.0, 0.5, N),
    (60.0, 1.0, N),
    (70.0, 2.0, N),
    (65.0, 1.0, N),
    (61.0, 1.0, N),
    (73.0, 1.5, N),
    (72.0, 0.5, N),
    (68.0, 2.0, N),
];

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
            D => {
                let from = n.dur * 0.45;
                target -= 12.0 * smooth((into - from) / (n.dur - from + 0.7));
                gate = into < n.dur + 0.7;
            }
            N => {}
        }
        self.pitch += (target - self.pitch) * self.k_glide;
        let vib = 0.32
            * (TAU * 5.6 * into).sin()
            * smooth((into - 0.22) / 0.3)
            * (n.dur > 0.4) as i32 as f32;
        let k = if gate { coeff(0.004) } else { coeff(0.06) };
        self.amp += ((gate as i32 as f32) * n.vel - self.amp) * k;
        if !gate && self.amp < 1e-4 {
            return None;
        }
        Some((self.pitch + vib, self.amp, into, n))
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
    Break,
    Drop2,
    Half,
    Final,
    Outro,
}

fn part_of(chapter: &str) -> Part {
    match chapter {
        "home" | "fsr" | "data" => Part::Verse,
        "solve" | "focus" | "explain" | "fill" => Part::Drop1,
        "replan" => Part::Break,
        "profiles" | "compare" | "trends" => Part::Drop2,
        "chat" => Part::Half,
        _ => Part::Final,
    }
}

#[derive(Clone, Copy)]
struct Bar {
    part: Part,
    pos: usize,
    len: usize,
    chord: usize,
    /// Semitones above C minor: 2 in the D minor drop.
    key: f32,
}

fn form() -> Vec<Bar> {
    let n = TOTAL_BARS as usize;
    let intro = INTRO_BARS as usize;
    let outro = n - OUTRO_BARS as usize;
    let mut parts = vec![Part::Intro; n];
    let mut starts: Vec<(Part, usize)> = Vec::new();
    for &(name, start, _) in CHAPTERS.iter() {
        let b = (((LIVE_FROM + start) / BAR).round() as usize).clamp(intro, outro);
        let p = part_of(name);
        if starts.last().is_none_or(|(q, _)| *q != p) {
            starts.push((p, b));
        }
    }
    for (k, &(p, b)) in starts.iter().enumerate() {
        let end = starts.get(k + 1).map_or(outro, |(_, e)| *e);
        for slot in &mut parts[b..end] {
            *slot = p;
        }
    }
    for slot in &mut parts[outro..] {
        *slot = Part::Outro;
    }
    let mut out = Vec::with_capacity(n);
    let mut b = 0;
    while b < n {
        let p = parts[b];
        let len = parts[b..].iter().take_while(|q| **q == p).count();
        for pos in 0..len {
            let last = pos + 1 == len;
            let next = parts.get(b + len).copied();
            let chord = match p {
                Part::Intro => INTRO_CHORDS[pos.min(6)],
                Part::Outro => OUTRO_CHORDS[pos.min(5)],
                // The key changes pivot on C minor, common to C and D Phrygian.
                Part::Break | Part::Drop2 if last => CM,
                // Every other part turns round on Db, falling into C minor.
                _ if last && next.is_some_and(|q| q != Part::Break && q != Part::Outro) => DB,
                Part::Break => BREAK_LOOP[pos % 4],
                _ => LOOP[pos % 4],
            };
            // In D Phrygian the loop's chords move up; the pivot Cm stays put.
            let key = if p == Part::Drop2 && !last { 2.0 } else { 0.0 };
            out.push(Bar {
                part: p,
                pos,
                len,
                chord,
                key,
            });
        }
        b += len;
    }
    out
}

fn is_drop(p: Part) -> bool {
    matches!(p, Part::Drop1 | Part::Drop2 | Part::Final)
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
    /// Downbeats a reverse cymbal swells into.
    swells: Vec<f32>,
    santur: Vec<Pluck>,
    tanpura: Vec<Pluck>,
    stab: Vec<Note>,
    arp: Vec<Note>,
    bell: Vec<Note>,
    guitar: Vec<LNote>,
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
            key,
        } = fm[b];
        let s = i % 16;
        // Swing only in the hats, shaker and rim; the melodic parts play straight.
        let swing = if s % 2 == 1 { STEP * 0.08 } else { 0.0 };
        let last = pos + 1 == len;
        let next_drop = fm
            .get(b + 1)
            .is_some_and(|n| is_drop(n.part) && n.part != part);
        let roll = last && next_drop;
        let fill = !roll && is_drop(part) && pos % 8 == 7;
        let voicing = PAD_VOICINGS[chord].map(|m| m + key);

        // Tanpura: the drone's four strings plucked in turn on every beat
        // (fifth, tonic, tonic, low tonic), everywhere but the drops.
        if s % 4 == 0 && !is_drop(part) {
            let k = (s / 4) % 4;
            let m = [43.0, 48.0, 48.0, 36.0][k] + key;
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
                // A heartbeat on the frame drum: doum-doum, from the second bar.
                if b >= 1 && (s == 0 || s == 3) {
                    e.doum.push((t, if s == 0 { 0.7 } else { 0.45 }));
                }
            }
            Part::Verse | Part::Half => {
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
                if part == Part::Verse && (s == 0 || s == 8) {
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
            }
            Part::Drop1 | Part::Drop2 | Part::Final => {
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
                let v = match s % 4 {
                    0 => 0.45,
                    2 => 0.85,
                    _ => 0.3,
                };
                e.hat.push((t + swing, v));
                if part == Part::Drop1 && s == 10 || part != Part::Drop1 && s % 4 == 2 {
                    e.open_hat.push((t, 0.45));
                }
                // The frame drum keeps playing under the kit: tek on the offbeats.
                if matches!(s, 6 | 14) {
                    e.tek.push((t + swing, 0.5));
                }
                if fill && s >= 12 {
                    e.snare.push((t, 0.5 + 0.1 * (s - 12) as f32));
                    e.taiko
                        .push((t + STEP * 0.5, [48.0, 46.0, 43.0, 41.0][s - 12] + key));
                }
                // The pulse arp: chord tones two octaves up, in the second drop
                // and the final.
                if part != Part::Drop1 {
                    let order = [0usize, 2, 4, 2, 1, 3, 4, 3];
                    e.arp.push(Note {
                        start: t,
                        dur: STEP * 0.7,
                        freq: hz(voicing[order[s % 8]] + 24.0),
                        vel: if s % 4 == 0 { 1.0 } else { 0.7 },
                    });
                }
            }
            Part::Break => {
                // Taiko on 1 and 3 while the kit rests; over the last four bars
                // they run in eighths, growing.
                let build = len >= 6 && pos + 4 >= len;
                if build && s % 2 == 0 {
                    let k = (pos + 4 - len) as f32 / 4.0 + s as f32 / 64.0;
                    e.taiko.push((t, if s % 4 == 0 { 36.0 } else { 43.0 }));
                    e.doum.push((t, 0.3 + 0.6 * k));
                } else if !build && (s == 0 || s == 8) {
                    e.taiko.push((t, if s == 0 { 36.0 } else { 39.0 }));
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
        // Santur: hammered 16ths on chord tones in the verse and the drops,
        // a soft tremolo on the chord in the break, eighths in the half-time.
        let (play, vel, bright) = match part {
            Part::Verse => (true, 0.55 + 0.25 * (s % 4 == 0) as i32 as f32, 0.6),
            Part::Drop1 | Part::Drop2 | Part::Final => {
                (true, 0.45 + 0.2 * (s % 4 == 0) as i32 as f32, 0.55)
            }
            Part::Break => (true, 0.25, 0.35),
            Part::Half => (s % 2 == 0, 0.5, 0.55),
            Part::Intro => (b >= 4 && s % 2 == 0, 0.25 + 0.1 * (b - 3) as f32, 0.45),
            Part::Outro => (pos < 4 && s % 2 == 0, 0.4 * (1.0 - pos as f32 / 4.0), 0.4),
        };
        if play {
            let order = [1usize, 3, 2, 4, 3, 2, 4, 1];
            // The last 16th of each group of eight turns on the mode's flat
            // second over the C minor and Ab bars.
            let m = if s % 8 == 7 && (chord == CM || chord == AB) && part != Part::Break {
                73.0 + key
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
            if part == Part::Break {
                e.santur.push(Pluck {
                    t: t + STEP * 0.5,
                    freq: hz(m),
                    vel: vel * 0.7,
                    bright,
                    damp: 0.998,
                });
            }
        }
    }

    // Section downbeats: crash; the gong and the boom on the drops and the outro.
    for (b, bar) in fm.iter().enumerate() {
        if bar.pos != 0 || b == 0 {
            continue;
        }
        let at = bars(b as f32);
        e.crash
            .push((at, if bar.part == Part::Break { 0.5 } else { 1.0 }));
        if is_drop(bar.part) || matches!(bar.part, Part::Outro | Part::Verse) {
            e.gong.push((at, 1.0));
            e.boom.push((at, 1.0));
        }
        if is_drop(bar.part) || bar.part == Part::Verse {
            e.risers.push(b as f32);
            e.swells.push(at);
        }
    }

    // Intro: the gong opens; a taiko on each model panel as it lands,
    // climbing the C minor chord; eighths, then a roll down into the warp.
    e.gong.push((0.0, 0.8));
    for (at, m) in PANEL_AT.iter().zip([36.0, 39.0, 43.0, 46.0, 48.0]) {
        e.taiko.push((*at, m));
    }
    let last = INTRO_BARS - 1.0;
    for k in 0..6 {
        e.taiko.push((
            bars(last) + k as f32 * BEAT * 0.5,
            if k % 2 == 0 { 43.0 } else { 39.0 },
        ));
    }
    for (k, m) in [48.0, 46.0, 44.0, 43.0, 41.0, 39.0, 38.0, 36.0]
        .iter()
        .enumerate()
    {
        e.taiko
            .push((bars(last) + 3.0 * BEAT + k as f32 * STEP * 0.5, *m));
    }
    lay(&mut e.duduk, bars(2.0), &DUDUK, 0.0, 1.0, 0.8);

    // The melodic parts, phrased from each part's first bar.
    let mut b = 0;
    while b < fm.len() {
        let Bar { part, len, .. } = fm[b];
        let (from, to) = (b as f32, (b + len) as f32);
        match part {
            Part::Verse => {
                // The duduk calls every other four bars; the riff stabs join
                // quietly in the last eight.
                let mut p = from;
                while p + 4.0 <= to + 0.01 {
                    lay(&mut e.duduk, bars(p), &DUDUK, 0.0, 1.0, 0.75);
                    p += 8.0;
                }
                riff_stab(&mut e.stab, (to - 8.0).max(from), to, 0.0, 0.4);
            }
            Part::Drop1 => {
                riff_full(&mut e.stab, from, to, 0.0, 0.9);
                // The solo, then its answer if the part runs sixteen bars.
                let t = lay(&mut e.guitar, bars(from), &solo(), 0.0, 1.0, 1.0);
                if t + bars(8.0) <= bars(to) + 0.01 {
                    lay(&mut e.guitar, t, &ANSWER, 0.0, 1.0, 0.95);
                }
            }
            Part::Break => {
                // Four bars of choir alone, then the guitar's held bends.
                if from + 8.0 <= to {
                    lay(&mut e.guitar, bars(from + 4.0), &BREAK_LINE, 0.0, 1.0, 0.85);
                }
            }
            Part::Drop2 => {
                riff_full(&mut e.stab, from, to - 1.0, 2.0, 1.0);
                let t = lay(&mut e.guitar, bars(from), &solo(), 2.0, 1.0, 1.0);
                if t + bars(8.0) <= bars(to - 1.0) + 0.01 {
                    lay(&mut e.guitar, t, &ANSWER, 2.0, 1.0, 0.95);
                }
            }
            Part::Half => {
                // Call and answer: the guitar plays two bars of the solo, the
                // duduk echoes them an octave lower, and on to the next two.
                riff_stab(&mut e.stab, from, to, 0.0, 0.7);
                let whole = solo();
                let mut p = from;
                let mut k = 0;
                while p + 2.0 <= to + 0.01 {
                    let pair = &whole[(k / 2 % 2) * 8..(k / 2 % 2) * 8 + 8];
                    if k % 2 == 0 {
                        lay(&mut e.guitar, bars(p), pair, 0.0, 1.0, 0.9);
                    } else {
                        lay(&mut e.duduk, bars(p), pair, -12.0, 1.0, 0.85);
                    }
                    p += 2.0;
                    k += 1;
                }
            }
            Part::Final => {
                riff_full(&mut e.stab, from, to, 0.0, 1.0);
                lay(&mut e.guitar, bars(from), &ANSWER, 0.0, 1.0, 1.0);
            }
            Part::Outro => {
                // The last held bend, singing over the cadence, and the dive.
                lay(
                    &mut e.guitar,
                    bars(from),
                    &[(80.0, 4.0, B), (82.0, 4.0, B), (84.0, 10.0, D)],
                    0.0,
                    1.0,
                    0.9,
                );
                lay(
                    &mut e.duduk,
                    bars(from + 2.0),
                    &[(67.0, 4.0, B), (72.0, 8.0, N)],
                    0.0,
                    1.0,
                    0.6,
                );
            }
            Part::Intro => {}
        }
        b += len;
    }

    // A bell climbs the chord each time a plan lands on screen.
    for &(line, start, _) in SPEECH.iter() {
        if matches!(line, "solve2" | "replan2" | "compare" | "chat2") {
            let at = LIVE_FROM + start - 0.3;
            let key = fm[((at / BAR) as usize).min(fm.len() - 1)].key;
            for (k, m) in [72.0, 75.0, 79.0].iter().enumerate() {
                e.bell.push(Note {
                    start: at + k as f32 * STEP * 2.0,
                    dur: BEAT,
                    freq: hz(m + key),
                    vel: 0.7,
                });
            }
        }
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

/// Planner riff re-cut to a 3-3-2 stab: hits on 16ths 0,3,6,8,11,14 of each
/// bar, each taking the riff note sounding at that step.
fn riff_stab(out: &mut Vec<Note>, from: f32, to: f32, key: f32, vel: f32) {
    let mut b = from;
    while b < to {
        for s in [0usize, 3, 6, 8, 11, 14] {
            let mut k = ((b as usize % 4) * 16) + s;
            while PLANNER_RIFF[k] == 0 {
                k -= 1;
            }
            out.push(Note {
                start: bars(b) + s as f32 * STEP,
                dur: STEP * 1.6,
                freq: hz(PLANNER_RIFF[k] as f32 + key),
                vel,
            });
        }
        b += 1.0;
    }
}

/// The Planner riff as written: every sounding 16th of the 4-bar line.
fn riff_full(out: &mut Vec<Note>, from: f32, to: f32, key: f32, vel: f32) {
    let mut b = from;
    while b < to {
        for s in 0..16 {
            let m = PLANNER_RIFF[(b as usize % 4) * 16 + s];
            if m != 0 {
                out.push(Note {
                    start: bars(b) + s as f32 * STEP,
                    dur: STEP * 1.7,
                    freq: hz(m as f32 + key),
                    vel,
                });
            }
        }
        b += 1.0;
    }
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
    let mut f = [(); 16].map(|_| Svf::default());
    let mut pad_phase = [[0.0f32; 2]; 5];
    let mut bass_phase = 0.0f32;
    let mut gtr_phase = [0.0f32; 3];
    let mut duduk_phase = 0.0f32;
    let mut echo_l = Delay::new(BEAT * 0.75);
    let mut echo_r = Delay::new(BEAT * 0.75);
    let (mut verb_l, mut verb_r) = (Reverb::new(1.0), Reverb::new(1.07));

    let mut santur = Strings::new(12, std::mem::take(&mut ev.santur));
    let mut tanpura = Strings::new(4, std::mem::take(&mut ev.tanpura));
    let mut guitar = Mono::new(std::mem::take(&mut ev.guitar));
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
            key,
        } = fm[bi];
        let bar = t / BAR;
        let step_age = t % STEP;
        let step = (t / STEP) as usize % 16;
        let [c_kick, c_snare, c_clap, c_hat, c_oh, c_shaker, c_doum, c_tek, c_crash, c_boom, c_gong, c_taiko] =
            &mut cur;
        let [snare_f, clap_f, hat_f, shaker_f, tek_f, crash_f, taiko_f, riser_f, stab_f, bass_f, g_lo, g_hi, g_peak, d_f1, d_f2, d_br] =
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
            Part::Break => 1.0,
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
        let open = if part == Part::Break {
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
        let root = hz(ROOTS[chord] + key - 12.0);
        match part {
            Part::Verse | Part::Half => {
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
            Part::Drop1 | Part::Drop2 | Part::Final => {
                let accent = if step.is_multiple_of(2) { 1.0 } else { 0.6 };
                bass_phase = wrap(bass_phase + root * 2.0 / sr);
                let slide = (1.0 - step_age / STEP * 0.7).max(0.1);
                bass = (pulse(bass_phase, 0.5) * 0.55 + sine(bass_phase * 0.5) * 0.4)
                    * accent
                    * slide
                    * 0.36;
            }
            Part::Break | Part::Intro | Part::Outro if bar >= 2.0 => {
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

        // ── The electric guitar: two saws and a square, picked, driven through
        // tanh into a cabinet, with a feedback octave growing on held notes.
        let mut gtr = 0.0;
        if let Some((pitch, amp, into, n)) = guitar.tick(t) {
            let fq = hz(pitch);
            gtr_phase[0] = wrap(gtr_phase[0] + fq / sr);
            gtr_phase[1] = wrap(gtr_phase[1] + fq * 1.004 / sr);
            gtr_phase[2] = wrap(gtr_phase[2] + fq * 2.0 / sr);
            let pick = noise_l.next() * (-into * 90.0).exp() * 0.5;
            let feed =
                sine(gtr_phase[2]) * 0.6 * smooth((into - 0.7) / 0.8) * (n.dur > 1.0) as i32 as f32;
            let x = saw(gtr_phase[0]) * 0.55
                + saw(gtr_phase[1]) * 0.45
                + pulse(gtr_phase[0], 0.5) * 0.25
                + pick
                + feed;
            let driven = (x * 7.0).tanh();
            let lo = g_lo.run(driven, 2900.0, 0.8).0;
            let body = g_hi.run(lo, 140.0, 0.7).2;
            let (_, peak, _) = g_peak.run(body, 1500.0, 1.3);
            gtr = (body + 0.5 * peak) * amp * 0.16;
        }
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
        echo_l.push((gtr * 0.8 + reed * 0.5 + arp + sant * 0.3) * pump + echo_r.tap() * 0.35);
        let el = echo_l.tap();
        echo_r.push(el * 0.8);
        let er = echo_r.tap();
        let send = choir[0] * 0.3
            + bell
            + gtr * 0.35
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
            + (sl + al) * pump
            + choir[0] * pump
            + el * 0.45
            + rl * 0.55
            + crash
            + bell * 0.6;
        let mut rgt = center
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

    const C_PHRYGIAN: [i32; 7] = [0, 1, 3, 5, 7, 8, 10];
    const D_PHRYGIAN: [i32; 7] = [2, 3, 5, 7, 9, 10, 0];

    /// The mode of the bar sounding at `t`.
    fn scale_at(fm: &[Bar], t: f32) -> &'static [i32; 7] {
        let b = &fm[((t / BAR) as usize).min(fm.len() - 1)];
        if b.key > 0.0 {
            &D_PHRYGIAN
        } else {
            &C_PHRYGIAN
        }
    }

    /// Every written pitch sits in its section's mode: C Phrygian, D
    /// Phrygian in the second drop. (The bends only pass through others.)
    #[test]
    fn every_pitch_is_in_the_mode() {
        let fm = form();
        let e = score(&fm);
        let check = |t: f32, m: f32, what: &str| {
            let pc = (m.round() as i32).rem_euclid(12);
            assert!(
                scale_at(&fm, t).contains(&pc),
                "{what} {m} at {t:.2}s is outside the mode"
            );
        };
        for n in e.guitar.iter().chain(&e.duduk) {
            check(n.start + 0.01, n.midi, "lead");
        }
        for n in e.stab.iter().chain(&e.arp).chain(&e.bell) {
            check(n.start + 0.01, midi_of(n.freq), "note");
        }
        for p in e.santur.iter().chain(&e.tanpura) {
            check(p.t + 0.01, midi_of(p.freq), "string");
        }
        for v in PAD_VOICINGS.iter().flatten() {
            assert!(C_PHRYGIAN.contains(&(*v as i32).rem_euclid(12)), "pad {v}");
        }
    }

    #[test]
    fn the_solo_fills_eight_bars_and_each_note_stays_in_its_bar() {
        let solo = solo();
        let total: f32 = solo.iter().map(|(_, b, _)| b).sum();
        assert_eq!(total, 32.0);
        let mut beat = 0.0f32;
        for &(_, b, _) in &solo {
            let bar_end = (beat / 4.0).floor() * 4.0 + 4.0;
            assert!(
                beat + b <= bar_end + 1e-3,
                "a solo note crosses a bar line at beat {beat}"
            );
            beat += b;
        }
        assert_eq!(DUDUK.iter().map(|(_, b, _)| b).sum::<f32>(), 16.0);
        assert_eq!(ANSWER.iter().map(|(_, b, _)| b).sum::<f32>(), 32.0);
        assert_eq!(BREAK_LINE.iter().map(|(_, b, _)| b).sum::<f32>(), 16.0);
    }

    /// The duduk's first statement in the intro (bars 3-6) sits on the loop
    /// it was written over.
    #[test]
    fn the_intro_plays_the_loop_under_the_duduk() {
        assert_eq!(&INTRO_CHORDS[2..6], &LOOP);
        assert_eq!(INTRO_CHORDS[6], DB);
    }

    /// No flurries: the solo lines move in eighths at the quickest.
    #[test]
    fn the_solos_have_no_quick_figures() {
        for line in [&solo()[..], &ANSWER[..], &BREAK_LINE[..]] {
            assert!(
                line.iter().all(|(_, b, _)| *b >= 0.5),
                "a solo note shorter than an eighth"
            );
        }
    }

    /// The solo is built on the harmony: whatever sounds on beats 1 and 3 of
    /// a bar is a tone of that bar's chord. The duduk's motif too.
    #[test]
    fn the_solo_lands_on_chord_tones() {
        let solo = solo();
        let lines: [(&Line, &[usize; 4]); 4] = [
            (&solo, &LOOP),
            (&ANSWER, &LOOP),
            (&DUDUK, &LOOP),
            (&BREAK_LINE, &BREAK_LOOP),
        ];
        for (line, chords) in lines {
            let mut beat = 0.0f32;
            for &(m, b, _) in line {
                for strong in [0.0f32, 2.0] {
                    let bar = (beat / 4.0).floor();
                    let at = bar * 4.0 + strong;
                    if m > 0.0 && at >= beat - 1e-3 && at < beat + b - 1e-3 {
                        let chord = chords[bar as usize % 4];
                        let tones: Vec<i32> = PAD_VOICINGS[chord]
                            .iter()
                            .map(|v| (*v as i32).rem_euclid(12))
                            .collect();
                        assert!(
                            tones.contains(&((m as i32).rem_euclid(12))),
                            "{m} on beat {} of bar {} is not in its chord",
                            strong + 1.0,
                            bar + 1.0
                        );
                    }
                }
                beat += b;
            }
        }
    }

    #[test]
    fn the_form_follows_the_chapters_pivots_on_c_minor_and_cadences_from_db() {
        let fm = form();
        assert_eq!(fm.len(), TOTAL_BARS as usize);
        for &(name, start, _) in CHAPTERS.iter() {
            let b = ((LIVE_FROM + start) / BAR).round() as usize;
            assert_eq!(fm[b].part, part_of(name), "{name} at bar {b}");
        }
        for w in fm.windows(2) {
            if w[0].key != w[1].key {
                let pivot = if w[0].key == 0.0 { &w[0] } else { &w[1] };
                assert_eq!(pivot.chord, CM, "a key change not on the C minor pivot");
            }
            // Every part that starts on C minor is entered from Db or the pivot.
            if w[1].pos == 0 && w[1].chord == CM && w[0].part != Part::Intro {
                assert!(
                    w[0].chord == DB || w[0].chord == CM,
                    "{:?} entered from chord {}",
                    w[1].part,
                    w[0].chord
                );
            }
        }
        assert_eq!(fm.last().unwrap().chord, CM);
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
