// WORKBENCH_12 soundtrack: the latest Planner123 theme (planner123_06), remixed.
//
// Kept from planner123_06: 125 BPM, the Cm - Ab - Bb - G loop and its pad
// voicings, the lead riff (as written and as the 3-3-2 stab), the solo's
// two phrases, the 16th square bass, the offbeat octave bass, the FM bell and
// the voices, filters, echo and reverb they are built from.
//
// New in the remix:
// - its own hook, an eight-bar statement and answer over the loop (HOOK),
//   which the verse states, the first drop trades bar for bar with the
//   Planner riff, the second drop lifts an octave and the final drop plays
//   together with the riff;
// - a form with an arc instead of a loop: taiko intro, verse, first drop,
//   a breakdown on its own progression (Fm - Ab - Bb - Cm) that quotes the
//   Planner solo, a build, a bigger second drop, a half-time section, the
//   final drop, and an outro cadence Ab - Bb - Cm;
// - an archeofuturistic palette, ancient instruments played by a machine:
//   a choir pad (the saws sung through two vowel formants), a plucked lyre
//   on swung 16ths, a glass lyre for the hook, taiko toms, a temple gong,
//   a gated snare with a clap, offbeat hats in the second drop;
// - fills at the end of every phrase, snare rolls and risers into each drop.
//
// The sections follow the cut (cut.rs): each starts on the bar nearest its
// chapter's first frame. Under the narration only the melodic parts step
// back, by about 4 dB; drums and bass keep their level.
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

// ── Harmony ──────────────────────────────────────────────────────────────────
// Planner123's loop (Cm, Ab, Bb, G) plus Fm for the breakdown.
const CM: usize = 0;
const AB: usize = 1;
const BB: usize = 2;
const G: usize = 3;
const FM: usize = 4;
const ROOTS: [f32; 5] = [48.0, 44.0, 46.0, 43.0, 41.0];
const PAD_VOICINGS: [[f32; 5]; 5] = [
    [48.0, 55.0, 62.0, 63.0, 70.0], // Cm9
    [44.0, 51.0, 60.0, 63.0, 67.0], // Abmaj7
    [46.0, 53.0, 60.0, 62.0, 65.0], // Bb add9
    [43.0, 50.0, 55.0, 62.0, 65.0], // G5 add b7
    [41.0, 48.0, 56.0, 60.0, 63.0], // Fm7
];
const LOOP: [usize; 4] = [CM, AB, BB, G];
const BREAK_LOOP: [usize; 4] = [FM, AB, BB, CM];
const INTRO_CHORDS: [usize; 7] = [CM, CM, AB, BB, CM, AB, G];
const OUTRO_CHORDS: [usize; 6] = [AB, BB, CM, CM, CM, CM];

/// The remix's hook: (midi, beats), eight bars over the loop. Bars 1-4 state
/// it and rest on the dominant; bars 5-8 answer an octave-wide arc higher and
/// come home to C. 0 is a rest.
const HOOK: [(f32, f32); 34] = [
    (67.0, 1.0),
    (75.0, 0.5),
    (74.0, 0.5),
    (72.0, 1.0),
    (67.0, 1.0),
    (68.0, 1.5),
    (72.0, 0.5),
    (75.0, 1.0),
    (77.0, 1.0),
    (74.0, 1.5),
    (77.0, 0.5),
    (70.0, 1.0),
    (74.0, 1.0),
    (74.0, 1.5),
    (75.0, 0.5),
    (74.0, 1.0),
    (0.0, 1.0),
    (79.0, 1.0),
    (77.0, 0.5),
    (75.0, 0.5),
    (74.0, 1.0),
    (72.0, 1.0),
    (75.0, 1.5),
    (77.0, 0.5),
    (79.0, 1.0),
    (80.0, 1.0),
    (77.0, 1.5),
    (74.0, 0.5),
    (70.0, 1.0),
    (74.0, 1.0),
    (74.0, 1.0),
    (75.0, 0.5),
    (74.0, 0.5),
    (72.0, 2.0),
];

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

/// The hook, or one half of it (bars 0-3 or 4-7), from `bar`, `shift`
/// semitones up.
fn push_hook(out: &mut Vec<Note>, bar: f32, half: Option<usize>, shift: f32, vel: f32) {
    let mut beat = 0.0f32;
    for &(m, beats) in &HOOK {
        let in_half = half.is_none_or(|h| (beat / 16.0).floor() as usize == h);
        if m > 0.0 && in_half {
            let from = if let Some(h) = half {
                h as f32 * 16.0
            } else {
                0.0
            };
            out.push(Note {
                start: bars(bar) + (beat - from) * BEAT,
                dur: beats * BEAT * 0.92,
                freq: hz(m + shift),
                vel,
            });
        }
        beat += beats;
    }
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

/// Per bar: its part, the bar's place in that part and the part's length.
#[derive(Clone, Copy)]
struct Bar {
    part: Part,
    pos: usize,
    len: usize,
    chord: usize,
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
            let chord = match p {
                Part::Intro => INTRO_CHORDS[pos.min(6)],
                Part::Break => BREAK_LOOP[pos % 4],
                Part::Outro => OUTRO_CHORDS[pos.min(5)],
                _ => LOOP[pos % 4],
            };
            out.push(Bar {
                part: p,
                pos,
                len,
                chord,
            });
        }
        b += len;
    }
    out
}

fn energy(p: Part) -> f32 {
    match p {
        Part::Intro | Part::Outro => 0.5,
        Part::Verse => 0.55,
        Part::Break => 0.3,
        Part::Half => 0.65,
        Part::Drop1 => 0.9,
        Part::Drop2 | Part::Final => 1.0,
    }
}

fn is_drop(p: Part) -> bool {
    matches!(p, Part::Drop1 | Part::Drop2 | Part::Final)
}

// ── Drum and effect events ─────────────────────────────────────────────────
#[derive(Default)]
struct Events {
    kick: Vec<(f32, f32)>,
    snare: Vec<(f32, f32)>,
    clap: Vec<(f32, f32)>,
    hat: Vec<(f32, f32)>,
    open_hat: Vec<(f32, f32)>,
    crash: Vec<(f32, f32)>,
    boom: Vec<(f32, f32)>,
    gong: Vec<(f32, f32)>,
    /// (time, midi pitch)
    tom: Vec<(f32, f32)>,
    /// Bars where a riser lands.
    risers: Vec<f32>,
    lyre: Vec<Note>,
    hook: Vec<Note>,
    lead: Vec<Note>,
    bell: Vec<Note>,
    stab: Vec<Note>,
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
        let swing = if s % 2 == 1 { STEP * 0.14 } else { 0.0 };
        let last = pos + 1 == len;
        let next_drop = fm
            .get(b + 1)
            .is_some_and(|n| is_drop(n.part) && n.part != part);
        // The bar before a drop is a roll; the last bar of each eight a fill.
        let roll = last && next_drop;
        let fill = !roll && is_drop(part) && pos % 8 == 7;
        match part {
            Part::Verse => {
                if matches!(s, 0 | 6 | 8) {
                    e.kick.push((t, if s == 0 { 0.9 } else { 0.6 }));
                }
                if s == 4 || s == 12 {
                    e.snare.push((t, 0.55));
                }
                e.hat.push((
                    t + swing,
                    if s % 2 == 0 {
                        [0.5, 0.8][(s / 2) % 2]
                    } else {
                        0.15
                    },
                ));
                if s == 14 && pos % 2 == 1 {
                    e.open_hat.push((t, 0.5));
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
                    0 => 0.5,
                    2 => 0.85,
                    _ => 0.3,
                };
                e.hat.push((t + swing, v));
                if part == Part::Drop1 {
                    if s == 10 {
                        e.open_hat.push((t, 0.6));
                    }
                } else if s % 4 == 2 {
                    e.open_hat.push((t, 0.4));
                }
                if fill && s >= 12 {
                    e.snare.push((t, 0.5 + 0.1 * (s - 12) as f32));
                    e.tom
                        .push((t + STEP * 0.5, [48.0, 46.0, 43.0, 41.0][s - 12]));
                }
            }
            Part::Break => {
                // Taiko on 1 and 3 while the drums rest; the kit comes back
                // four bars before the drop.
                if len >= 6 && pos + 4 >= len {
                    if s % 4 == 0 {
                        e.kick.push((t, 0.8));
                    }
                    e.hat.push((t + swing, if s % 2 == 0 { 0.4 } else { 0.15 }));
                } else if s == 0 || s == 8 {
                    e.tom.push((t, if s == 0 { 36.0 } else { 39.0 }));
                }
            }
            Part::Half => {
                if s == 0 || s == 10 {
                    e.kick.push((t, 0.9));
                }
                if s == 8 {
                    e.snare.push((t, 0.85));
                    e.clap.push((t, 0.6));
                }
                if s % 2 == 0 {
                    e.hat.push((t + swing, if s % 4 == 2 { 0.7 } else { 0.4 }));
                }
            }
            Part::Intro | Part::Outro => {}
        }
        if roll {
            // 16ths rising through the bar, 32nds on the last beat.
            let k = s as f32 / 16.0;
            e.snare.push((t, 0.25 + 0.6 * k));
            if s >= 12 {
                e.snare.push((t + STEP * 0.5, 0.3 + 0.6 * k));
            }
        }
        // The lyre arp: chord tones an octave over the pad, swung 16ths where
        // the music runs, 8ths where it rests.
        let busy = matches!(part, Part::Verse | Part::Drop1 | Part::Drop2 | Part::Final);
        if b >= 1 && (busy || s % 2 == 0) {
            let order = [1usize, 2, 3, 4, 3, 2, 1, 2];
            let note = PAD_VOICINGS[chord][order[s % 8]] + 12.0;
            let vel = match part {
                Part::Intro => 0.4 + 0.5 * (b as f32 - 1.0) / 6.0,
                Part::Outro => 0.7 * (1.0 - pos as f32 / 6.0),
                Part::Break => 0.55,
                _ => 0.6 + 0.4 * energy(part),
            };
            e.lyre.push(Note {
                start: t + swing,
                dur: STEP * 0.9,
                freq: hz(note),
                vel,
            });
        }
    }

    // Section starts: crash, and the gong with the boom on the big ones.
    let mut prev = Part::Intro;
    for (b, bar) in fm.iter().enumerate() {
        if bar.pos != 0 || b == 0 {
            continue;
        }
        let at = bars(b as f32);
        e.crash
            .push((at, if bar.part == Part::Break { 0.5 } else { 1.0 }));
        if is_drop(bar.part) || bar.part == Part::Outro || prev == Part::Intro {
            e.gong.push((at, 1.0));
            e.boom.push((at, 1.0));
        }
        if is_drop(bar.part) {
            e.risers.push(b as f32);
        }
        prev = bar.part;
    }

    // Intro: the gong opens; a low taiko pulse on the first bar; one taiko on
    // each model panel as it lands, climbing the C minor chord; 8ths, then a
    // 32nd roll down the scale into the warp.
    e.gong.push((0.0, 0.8));
    for beat in [0.0, 2.0] {
        e.tom.push((beat * BEAT, 36.0));
    }
    for (at, m) in PANEL_AT.iter().zip([36.0, 39.0, 43.0, 46.0, 48.0]) {
        e.tom.push((*at, m));
    }
    let last = INTRO_BARS - 1.0;
    for k in 0..6 {
        e.tom.push((
            bars(last) + k as f32 * BEAT * 0.5,
            if k % 2 == 0 { 43.0 } else { 39.0 },
        ));
    }
    for (k, m) in [48.0, 46.0, 44.0, 43.0, 41.0, 39.0, 38.0, 36.0]
        .iter()
        .enumerate()
    {
        e.tom
            .push((bars(last) + 3.0 * BEAT + k as f32 * STEP * 0.5, *m));
    }
    e.risers.push(INTRO_BARS);
    // The hook's first half, slow and soft on the glass lyre, under the panels.
    push_hook(&mut e.hook, 3.0, Some(0), 0.0, 0.45);

    // Melodic layers per part, phrased in eights from the part's first bar.
    let mut b = 0;
    while b < fm.len() {
        let Bar { part, len, .. } = fm[b];
        let (from, to) = (b as f32, (b + len) as f32);
        let mut phrase = from;
        match part {
            Part::Verse => {
                // The hook stated in full; on its second pass the riff joins
                // underneath as a quiet 3-3-2 stab.
                while phrase + 8.0 <= to + 0.01 {
                    push_hook(&mut e.hook, phrase, None, 0.0, 0.85);
                    if phrase > from {
                        push_riff(&mut e.stab, phrase, phrase + 8.0, 0.45);
                    }
                    phrase += 8.0;
                }
                if phrase < to {
                    push_hook(&mut e.hook, phrase, Some(0), 0.0, 0.85);
                }
            }
            Part::Drop1 => {
                // The riff, and the hook answering it on bars 5-8 of each eight.
                push_riff_full(&mut e.stab, from, to, 0.9);
                push_riff_full(&mut e.lead, from, to, 0.6);
                while phrase + 8.0 <= to + 0.01 {
                    push_hook(&mut e.hook, phrase + 4.0, Some(1), 0.0, 0.9);
                    phrase += 8.0;
                }
            }
            Part::Break => {
                // The Planner solo, soft on the glass lyre, over Fm - Ab - Bb - Cm.
                push_solo(&mut e.hook, from + 1.0, 0.55);
            }
            Part::Drop2 => {
                // Bigger: the riff on both voices, the hook an octave up on
                // bars 5-8, and the solo's climb over the last eight.
                push_riff_full(&mut e.stab, from, to, 1.0);
                push_riff_full(&mut e.lead, from, to, 0.85);
                while phrase + 8.0 <= to + 0.01 {
                    push_hook(&mut e.hook, phrase + 4.0, Some(1), 12.0, 0.8);
                    phrase += 8.0;
                }
            }
            Part::Half => {
                push_riff(&mut e.stab, from, to, 0.8);
                push_hook(&mut e.hook, from + 4.0, Some(0), 0.0, 0.8);
                if len >= 12 {
                    push_hook(&mut e.hook, from + 8.0, Some(1), 0.0, 0.8);
                }
            }
            Part::Final => {
                // Everything together: the riff and the whole hook.
                push_riff_full(&mut e.stab, from, to, 1.0);
                push_riff_full(&mut e.lead, from, to, 0.8);
                push_hook(&mut e.hook, from, None, 12.0, 0.75);
            }
            Part::Outro => {
                // The hook's answer once more over Ab - Bb - Cm, home on C.
                push_hook(&mut e.hook, from, Some(1), 0.0, 0.7);
            }
            Part::Intro => {}
        }
        b += len;
    }

    // A bell climbs the C minor chord each time a plan lands on screen.
    for &(line, start, _) in SPEECH.iter() {
        if matches!(line, "solve2" | "replan2" | "compare" | "chat2") {
            for (k, m) in [72.0, 75.0, 79.0].iter().enumerate() {
                e.bell.push(Note {
                    start: LIVE_FROM + start - 0.3 + k as f32 * STEP * 2.0,
                    dur: BEAT,
                    freq: hz(*m),
                    vel: 0.7,
                });
            }
        }
    }
    let outro = TOTAL_BARS - OUTRO_BARS;
    for (b, m) in [
        (outro + 2.0, 72.0),
        (outro + 2.0, 79.0),
        (outro + 2.0, 84.0),
    ] {
        e.bell.push(Note {
            start: bars(b),
            dur: BAR * 2.0,
            freq: hz(m),
            vel: 0.6,
        });
    }
    for v in [
        &mut e.kick,
        &mut e.snare,
        &mut e.clap,
        &mut e.hat,
        &mut e.open_hat,
        &mut e.crash,
        &mut e.boom,
        &mut e.gong,
        &mut e.tom,
    ] {
        v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }
    for v in [
        &mut e.lyre,
        &mut e.hook,
        &mut e.lead,
        &mut e.bell,
        &mut e.stab,
    ] {
        v.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
    }
    e
}

/// 1 while the narrator speaks (from a little before to a little after each
/// line), 0 otherwise.
fn speaking(t: f32) -> f32 {
    let lt = t - LIVE_FROM;
    if SPEECH
        .iter()
        .any(|&(_, a, b)| lt >= a - 0.25 && lt < b + 0.35)
    {
        1.0
    } else {
        0.0
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
    let ev = score(&fm);
    let sr = SAMPLE_RATE as f32;
    let total = (DURATION * sr) as usize;
    let mut out = Vec::with_capacity(total);

    let mut noise_l = Noise(0x1234_5678);
    let mut noise_r = Noise(0x9E37_79B9);
    let mut noise_d = Noise(0x0BAD_F00D);
    let mut noise_c = Noise(0x5EED_CAFE);
    let mut vowel = [
        [Svf::default(), Svf::default(), Svf::default()],
        [Svf::default(), Svf::default(), Svf::default()],
    ];
    let mut lead_f = Svf::default();
    let mut stab_f = Svf::default();
    let mut bass_f = Svf::default();
    let mut riser_f = Svf::default();
    let mut lyre_f = Svf::default();
    let (mut hat_f, mut snare_f, mut crash_f, mut tom_f, mut clap_f) = (
        Svf::default(),
        Svf::default(),
        Svf::default(),
        Svf::default(),
        Svf::default(),
    );
    let mut pad_phase = [[0.0f32; 2]; 5];
    let mut bass_phase = 0.0f32;
    let mut echo_l = Delay::new(BEAT * 0.75);
    let mut echo_r = Delay::new(BEAT * 0.75);
    let (mut verb_l, mut verb_r) = (Reverb::new(1.0), Reverb::new(1.07));
    let mut energy_s = 0.5f32;
    let mut duck_s = 1.0f32;

    let mut cur = [
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
        Cursor(0),
    ];
    let (mut n_lyre, mut n_hook, mut n_lead, mut n_bell, mut n_stab) = (
        NoteCursor(0),
        NoteCursor(0),
        NoteCursor(0),
        NoteCursor(0),
        NoteCursor(0),
    );

    for i in 0..total {
        let t = i as f32 / sr;
        let bi = ((t / BAR) as usize).min(fm.len() - 1);
        let Bar {
            part,
            pos,
            len,
            chord,
        } = fm[bi];
        let bar = t / BAR;
        let step_age = t % STEP;
        let step = (t / STEP) as usize % 16;
        energy_s += (energy(part) - energy_s) * (1.0 / (0.6 * sr));
        // Under the narration the melodic parts step back about 4 dB.
        duck_s += (1.0 - 0.37 * speaking(t) - duck_s) * (1.0 / (0.15 * sr));

        // Drums.
        let [c_kick, c_snare, c_clap, c_hat, c_oh, c_crash, c_boom, c_gong, c_tom] = &mut cur;
        let mut drums = 0.0;
        let mut pump = 1.0;
        if let Some((age, v)) = c_kick.last(&ev.kick, t) {
            if age < 0.5 {
                let ph = 45.0 * age + 140.0 * (1.0 - (-age * 28.0).exp()) / 28.0;
                drums += sine(ph) * v * ((-age * 9.0).exp() + 0.3 * (-age * 90.0).exp()) * 1.1;
                pump = 1.0 - 0.45 * v * (-age / 0.09).exp();
            }
        }
        // Gated snare: noise and a body tuned to G3 and D4, cut hard at 180 ms.
        if let Some((age, v)) = c_snare.last(&ev.snare, t) {
            if age < 0.18 {
                let (_, nb, _) = snare_f.run(noise_l.next(), 2400.0, 0.7);
                let body = sine(196.0 * age) * 0.5 + sine(293.7 * age) * 0.25;
                let gate = 1.0 - smooth((age - 0.15) / 0.03);
                drums +=
                    (nb * 1.3 + body * (-age * 30.0).exp()) * v * (-age * 5.0).exp() * gate * 0.6;
            }
        }
        // Clap: three quick bursts of band-passed noise, then a short tail.
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
                drums += cb * burst * v * 0.9;
            }
        }
        if let Some((age, v)) = c_hat.last(&ev.hat, t) {
            let (_, _, hh) = hat_f.run(noise_r.next(), 9000.0, 0.8);
            drums += hh * v * (-age * 80.0).exp() * 0.3;
            if let Some((oage, ov)) = c_oh.last(&ev.open_hat, t) {
                if oage < 0.4 {
                    drums += hh * ov * (-oage * 9.0).exp() * 0.22;
                }
            }
        }
        let mut crash = 0.0;
        if let Some((age, v)) = c_crash.last(&ev.crash, t) {
            if age < 2.5 {
                let (_, _, ch) = crash_f.run(noise_d.next(), 5000.0, 0.5);
                crash = ch * v * (-age * 1.6).exp() * 0.3;
            }
        }
        if let Some((age, v)) = c_boom.last(&ev.boom, t) {
            if age < 2.0 {
                let ph = 38.0 * age + 60.0 * (1.0 - (-age * 6.0).exp()) / 6.0;
                drums += sine(ph) * v * (-age * 2.2).exp() * 0.9;
            }
        }
        // Taiko: a big low skin, its pitch sagging, with the stick's slap.
        let mut tom = 0.0;
        if let Some((age, m)) = c_tom.last(&ev.tom, t) {
            if age < 0.6 {
                let f = hz(m);
                let ph = f * 0.8 * age + f * 0.2 * (1.0 - (-age * 25.0).exp()) / 25.0;
                let (_, slap, _) = tom_f.run(noise_d.next(), 1200.0, 1.0);
                tom = (sine(ph) * (-age * 6.0).exp() + slap * 0.5 * (-age * 60.0).exp()) * 0.75;
            }
        }
        // Temple gong: inharmonic partials over C2, swelling in and ringing long.
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
                gong *= v * (1.0 - (-age * 30.0).exp()) * 0.22;
            }
        }
        // Risers: a sweeping noise band over the bar before each drop.
        let mut riser = 0.0;
        if let Some(&end) = ev.risers.iter().find(|&&e| bar >= e - 1.0 && bar < e) {
            let p = bar - (end - 1.0);
            let (_, rb, _) = riser_f.run(noise_r.next(), 300.0 * 20f32.powf(p), 2.5);
            riser = rb * p * 0.45;
        }

        // Choir pad: the detuned saws sung through an "ah" (730 and 1090 Hz)
        // over a low body that opens with the energy; the breakdown's body
        // opens over its last four bars into the drop.
        let pad_gain = match part {
            Part::Intro => smooth(bar / 2.0),
            Part::Outro => 1.0 - smooth((pos as f32 + (bar - bi as f32) - 3.0) / 3.0),
            _ => 0.8,
        };
        let mut pad = [0.0f32; 2];
        for (v, note) in PAD_VOICINGS[chord].iter().enumerate() {
            let f = hz(*note);
            for (side, det) in [0.996f32, 1.004].iter().enumerate() {
                pad_phase[v][side] = wrap(pad_phase[v][side] + f * det / sr);
                pad[side] += saw(pad_phase[v][side]) * 0.12 + sine(pad_phase[v][side]) * 0.05;
            }
        }
        let open = if part == Part::Break {
            smooth((pos as f32 + (bar - bi as f32) - (len as f32 - 4.0)) / 4.0)
        } else {
            0.0
        };
        let body_cut = 450.0 + 900.0 * energy_s + 2400.0 * open;
        let shimmer = 0.85 + 0.15 * sine(t * 0.8);
        let mut choir = [0.0f32; 2];
        for (side, [lo, f1, f2]) in vowel.iter_mut().enumerate() {
            let x = pad[side];
            let body = lo.run(x, body_cut * (1.0 + 0.05 * side as f32), 0.8).0;
            let (_, a, _) = f1.run(x, 730.0, 4.0);
            let (_, o, _) = f2.run(x, 1090.0, 5.0);
            choir[side] = (body * 0.7 + a * 1.1 + o * 0.7) * pad_gain * shimmer;
        }

        // Lyre: a plucked FM string, bright at the pluck and darkening.
        let mut lyre = 0.0;
        for n in n_lyre.active(&ev.lyre, t, 0.4) {
            let age = t - n.start;
            let index = 1.6 * (-age * 14.0).exp();
            lyre += sine(n.freq * age + index * sine(n.freq * 2.0 * age) / TAU)
                * (-age * 7.0).exp()
                * (age / 0.002).min(1.0)
                * n.vel;
        }
        let lyre = lyre_f.run(lyre, 2200.0 + 1600.0 * energy_s, 0.7).0 * 0.14;

        // Glass lyre for the hook: FM with a held body and vibrato on long notes.
        let mut hook = 0.0;
        for n in n_hook.active(&ev.hook, t, 0.35) {
            let age = t - n.start;
            let vib = if n.dur > BEAT {
                0.005 * (age * 5.5 * TAU).sin() * ((age - 0.2) * 3.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let ph = n.freq * age * (1.0 + vib);
            let index = 0.9 + 1.4 * (-age * 9.0).exp();
            let env = adsr(age, n.dur, 0.006, 0.35) * (0.55 + 0.45 * (-age * 4.0).exp());
            hook += (sine(ph + index * sine(ph * 3.0) / TAU) + 0.25 * sine(ph * 2.0)) * env * n.vel;
        }
        let hook = hook * 0.16;

        // Bass by part: offbeat octaves in the verse and the half-time,
        // Planner123's 16th square bass in the drops, a held sub in the break.
        let mut bass = 0.0;
        let root = hz(ROOTS[chord] - 12.0);
        match part {
            Part::Verse | Part::Half => {
                let off = matches!(step, 2 | 6 | 10 | 14);
                let f = if off { root * 2.0 } else { root };
                bass_phase = wrap(bass_phase + f / sr);
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
            Part::Break | Part::Intro if bar >= 2.0 => {
                bass_phase = wrap(bass_phase + root / sr);
                bass = sine(bass_phase) * 0.3;
            }
            _ => {}
        }
        let bass = bass_f.run(bass, 900.0, 0.9).0;

        // Lead: the riff doubled. Two detuned pulses with slow PWM, plus a sub.
        let mut lead = 0.0;
        for n in n_lead.active(&ev.lead, t, 0.12) {
            let age = t - n.start;
            let ph = n.freq * age;
            let pwm = 0.5 + 0.2 * (t * 0.9).sin();
            let osc = (pulse(ph, pwm) - (1.0 - 2.0 * pwm)) * 0.5
                + (pulse(ph * 1.006, 1.0 - pwm) - (2.0 * pwm - 1.0)) * 0.4
                + sine(ph * 0.5) * 0.3;
            lead += osc * adsr(age, n.dur, 0.008, 0.12) * n.vel;
        }
        let lead_cut = 1800.0 + 2200.0 * (0.5 + 0.5 * (t * 0.25).sin());
        let lead = lead_f.run(lead, lead_cut, 1.2).0 * 0.13;

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
        let stab = stab_f.run(stab, 1600.0 + 1400.0 * energy_s, 1.4).0 * 0.12;

        // Mix buses: drums and bass stay put; the melodic bus ducks under the
        // narration and pumps with the kick.
        let melodic = (lyre + hook + lead + stab) * pump * duck_s;
        echo_l.push(melodic * 0.45 + echo_r.tap() * 0.35);
        let el = echo_l.tap();
        echo_r.push(el * 0.8);
        let er = echo_r.tap();

        let send = choir[0] * 0.3
            + bell
            + lead * 0.4
            + hook * 0.6
            + lyre * 0.5
            + gong * 0.4
            + tom * 0.2
            + drums * 0.08;
        let rl = verb_l.run(send * duck_s, 0.82);
        let rr = verb_r.run(send * duck_s, 0.82);

        let center = drums + tom + gong + bass * pump + melodic + riser;
        let pads = duck_s * pump;
        let mut lft = center + choir[0] * pads + el * 0.5 + rl * 0.5 + crash + bell * 0.6;
        let mut rgt = center + choir[1] * pads + er * 0.5 + rr * 0.5 + crash * 0.9 + bell * 0.6;

        // Master: gentle drive, a soft ceiling, the very end faded.
        let fade = ((DURATION - t) / 1.5).clamp(0.0, 1.0);
        lft = soft_clip(lft * 0.62) * fade;
        rgt = soft_clip(rgt * 0.62) * fade;
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

    /// Every pitch the remix plays sits in C natural minor, as Planner123's does.
    #[test]
    fn every_pitch_is_in_c_minor() {
        let minor = [0, 2, 3, 5, 7, 8, 10];
        let e = score(&form());
        for n in e
            .lyre
            .iter()
            .chain(&e.hook)
            .chain(&e.lead)
            .chain(&e.bell)
            .chain(&e.stab)
        {
            let midi = (69.0 + 12.0 * (n.freq / 440.0).log2()).round() as i32;
            assert!(
                minor.contains(&midi.rem_euclid(12)),
                "{} Hz is outside C minor",
                n.freq
            );
        }
        for &(_, m) in &e.tom {
            assert!(
                minor.contains(&(m as i32).rem_euclid(12)),
                "tom {m} outside C minor"
            );
        }
        for v in PAD_VOICINGS.iter().flatten() {
            assert!(minor.contains(&(*v as i32).rem_euclid(12)));
        }
    }

    #[test]
    fn every_hook_bar_lasts_four_beats() {
        let total: f32 = HOOK.iter().map(|(_, b)| b).sum();
        assert_eq!(total, 32.0);
        let mut beat = 0.0f32;
        for &(_, b) in &HOOK {
            let bar_end = (beat / 4.0).floor() * 4.0 + 4.0;
            assert!(
                beat + b <= bar_end + 1e-3,
                "a hook note crosses a bar line at beat {beat}"
            );
            beat += b;
        }
    }

    #[test]
    fn the_form_follows_the_chapters() {
        let fm = form();
        assert_eq!(fm.len(), TOTAL_BARS as usize);
        assert!(fm[..INTRO_BARS as usize]
            .iter()
            .all(|b| b.part == Part::Intro));
        let outro = (TOTAL_BARS - OUTRO_BARS) as usize;
        assert!(fm[outro..].iter().all(|b| b.part == Part::Outro));
        for &(name, start, _) in CHAPTERS.iter() {
            let b = ((LIVE_FROM + start) / BAR).round() as usize;
            assert_eq!(fm[b].part, part_of(name), "{name} at bar {b}");
        }
        // The outro cadence ends on C minor.
        assert_eq!(fm.last().unwrap().chord, CM);
    }
}
