// The guitar part, written once and read twice: the song renders every note
// from this list, and the guitar on screen rings the string each note was
// played on. The key is E harmonic minor (E F# G A B C D#), one chord per
// bar: Em, C, Am, B7 -- the B7's D sharp pulls back to Em.
use crate::timeline::{bars, span, Section, BAR, BEAT, TOTAL_BARS};

/// Standard tuning, low E to high E (MIDI). String 0 is the low E.
pub const TUNING: [f32; 6] = [40.0, 45.0, 50.0, 55.0, 59.0, 64.0];

/// Open-position shapes per chord, low to high string; None = not played.
const SHAPES: [[Option<f32>; 6]; 4] = [
    [
        Some(40.0),
        Some(47.0),
        Some(52.0),
        Some(55.0),
        Some(59.0),
        Some(64.0),
    ], // Em
    [
        None,
        Some(48.0),
        Some(52.0),
        Some(55.0),
        Some(60.0),
        Some(64.0),
    ], // C
    [
        None,
        Some(45.0),
        Some(52.0),
        Some(57.0),
        Some(60.0),
        Some(64.0),
    ], // Am
    [
        None,
        Some(47.0),
        Some(51.0),
        Some(57.0),
        Some(59.0),
        Some(66.0),
    ], // B7
];

/// Power chords (root, fifth, octave) and the string the root is on.
const POWER: [([f32; 3], usize); 4] = [
    ([40.0, 47.0, 52.0], 0), // E5
    ([48.0, 55.0, 60.0], 1), // C5
    ([45.0, 52.0, 57.0], 1), // A5
    ([47.0, 54.0, 59.0], 1), // B5
];

/// Bass roots, an octave under the guitar's.
pub const BASS: [f32; 4] = [28.0, 36.0, 33.0, 35.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// Clean, ringing.
    Clean,
    /// Distorted, let ring.
    Power,
    /// Distorted and palm-muted: short and dark.
    Mute,
    /// The distorted lead, with bends.
    Lead,
}

#[derive(Clone, Copy, Debug)]
pub struct Note {
    pub at: f32,
    pub dur: f32,
    pub midi: f32,
    /// The pitch the note starts from and bends up into `midi`.
    pub bend_from: Option<f32>,
    pub string: usize,
    pub kind: Kind,
    pub vel: f32,
}

/// Two four-bar lead phrases on an eighth grid: (start, length, midi, bent).
/// Every pitch is in E harmonic minor; bent notes rise two semitones into
/// the written note.
const LEAD_A: [(u8, u8, u8, bool); 17] = [
    (0, 2, 71, true),
    (2, 1, 72, false),
    (3, 1, 71, false),
    (4, 2, 67, false),
    (6, 2, 64, false),
    (8, 3, 72, true),
    (11, 1, 71, false),
    (12, 2, 67, false),
    (14, 2, 64, false),
    (16, 2, 69, false),
    (18, 1, 72, false),
    (19, 1, 76, false),
    (20, 4, 81, true),
    (24, 2, 78, false),
    (26, 1, 75, false),
    (27, 1, 71, false),
    (28, 4, 75, true),
];
const LEAD_B: [(u8, u8, u8, bool); 15] = [
    (0, 4, 76, true),
    (4, 1, 79, false),
    (5, 1, 78, false),
    (6, 2, 76, false),
    (8, 2, 79, true),
    (10, 2, 76, false),
    (12, 2, 72, false),
    (14, 2, 76, false),
    (16, 3, 81, true),
    (19, 1, 79, false),
    (20, 2, 76, false),
    (22, 2, 72, false),
    (24, 2, 75, false),
    (26, 2, 78, false),
    (28, 4, 83, true),
];

/// The chord of bar b.
pub fn chord(b: usize) -> usize {
    b % 4
}

/// The highest string a pitch can be played on (for the picture).
fn string_for(midi: f32) -> usize {
    (0..6).rev().find(|&s| TUNING[s] <= midi).unwrap_or(0)
}

pub fn score() -> Vec<Note> {
    let mut notes = Vec::new();
    let eighth = BEAT / 2.0;
    for b in 0..TOTAL_BARS as usize {
        let t0 = bars(b as f32);
        let c = chord(b);
        let sec = crate::timeline::now(t0 + 0.01).section;
        let (s0, _) = span(sec);
        let k = b - s0;
        match sec {
            Section::Guitar => {
                // The first bar is one slow strum; then the arpeggio.
                if b == 0 {
                    strum(&mut notes, t0 + 0.6, c, 0.06, 3.2, 0.9);
                } else {
                    arpeggio(&mut notes, t0, c, if b < 4 { 0.75 } else { 0.9 });
                }
            }
            Section::Worlds => {
                arpeggio(&mut notes, t0, c, 0.7);
                if k >= 4 {
                    power(&mut notes, t0, c, BAR * 0.98, Kind::Power, 0.55);
                }
            }
            Section::Challenge if k < 8 => {
                // Chugs under the giant, an accent on the one and the three.
                for e in 0..8 {
                    let accent = e == 0 || e == 4;
                    let kind = if accent { Kind::Power } else { Kind::Mute };
                    let len = if accent { eighth * 0.95 } else { eighth * 0.7 };
                    power(&mut notes, t0 + e as f32 * eighth, c, len, kind, 0.85);
                }
            }
            Section::Challenge => {
                // The hall: single clean notes, E F E D#, one a beat, ringing.
                let line = [64.0, 65.0, 64.0, 63.0];
                for (i, m) in line.iter().enumerate() {
                    let vel = if k == 3 && i == 3 { 0.9 } else { 0.6 };
                    notes.push(Note {
                        at: t0 + i as f32 * BEAT,
                        dur: BEAT * 1.6,
                        midi: *m - if k % 2 == 1 { 12.0 } else { 0.0 },
                        bend_from: None,
                        string: string_for(*m),
                        kind: Kind::Clean,
                        vel,
                    });
                }
            }
            Section::Machine | Section::Work => {
                // Rhythm: hit on 1, chug the and-of-1, hit on the and-of-2,
                // chug through 3 and 4.
                let hits: [(u8, Kind); 7] = [
                    (0, Kind::Power),
                    (1, Kind::Mute),
                    (3, Kind::Power),
                    (4, Kind::Mute),
                    (5, Kind::Mute),
                    (6, Kind::Mute),
                    (7, Kind::Mute),
                ];
                for (i, &(e, kind)) in hits.iter().enumerate() {
                    let next = hits.get(i + 1).map(|h| h.0).unwrap_or(8);
                    let len =
                        (next - e) as f32 * eighth * if kind == Kind::Mute { 0.7 } else { 0.97 };
                    power(&mut notes, t0 + e as f32 * eighth, c, len, kind, 0.7);
                }
                let lead_from = if sec == Section::Machine { 2 } else { 1 };
                if k >= lead_from && (k - lead_from) % 4 == 0 && k + 4 <= section_bars(sec) {
                    let phrase: &[(u8, u8, u8, bool)] = if ((k - lead_from) / 4) % 2 == 0 {
                        &LEAD_A
                    } else {
                        &LEAD_B
                    };
                    lead(&mut notes, t0, phrase);
                }
            }
            Section::Reading => {
                arpeggio(&mut notes, t0, c, 0.65);
                power(&mut notes, t0, c, BAR * 0.98, Kind::Power, 0.45);
            }
            Section::Home => arpeggio(&mut notes, t0, c, 0.6),
            Section::Name => {
                if k == 0 {
                    power(&mut notes, t0, 0, BAR * 3.0, Kind::Power, 1.0);
                } else if (3..5).contains(&k) {
                    arpeggio(&mut notes, t0, 0, 0.5);
                } else if k == 5 {
                    strum(&mut notes, t0, 0, 0.08, 3.0, 0.7);
                }
            }
        }
    }
    notes.sort_by(|a, b| a.at.total_cmp(&b.at));
    notes
}

fn section_bars(sec: Section) -> usize {
    let (b0, b1) = span(sec);
    b1 - b0
}

/// One bar of eighth-note arpeggio over the chord's shape.
fn arpeggio(notes: &mut Vec<Note>, t0: f32, c: usize, vel: f32) {
    let shape = SHAPES[c];
    let low = (0..6).find(|&s| shape[s].is_some()).unwrap_or(0);
    let order = [low, 2, 3, 4, 5, 4, 3, 2];
    for (e, &s) in order.iter().enumerate() {
        let Some(midi) = shape[s] else { continue };
        notes.push(Note {
            at: t0 + e as f32 * BEAT / 2.0,
            dur: BEAT * 1.5,
            midi,
            bend_from: None,
            string: s,
            kind: Kind::Clean,
            vel: vel * if e == 0 { 1.0 } else { 0.8 },
        });
    }
}

/// A slow downstroke across the shape, `gap` seconds between strings.
fn strum(notes: &mut Vec<Note>, at: f32, c: usize, gap: f32, dur: f32, vel: f32) {
    for (s, midi) in SHAPES[c].iter().enumerate() {
        if let Some(midi) = midi {
            notes.push(Note {
                at: at + s as f32 * gap,
                dur,
                midi: *midi,
                bend_from: None,
                string: s,
                kind: Kind::Clean,
                vel,
            });
        }
    }
}

fn power(notes: &mut Vec<Note>, at: f32, c: usize, dur: f32, kind: Kind, vel: f32) {
    let (pitches, root_string) = POWER[c];
    for (i, midi) in pitches.iter().enumerate() {
        notes.push(Note {
            at: at + i as f32 * 0.004,
            dur,
            midi: *midi,
            bend_from: None,
            string: root_string + i,
            kind,
            vel,
        });
    }
}

fn lead(notes: &mut Vec<Note>, t0: f32, phrase: &[(u8, u8, u8, bool)]) {
    let eighth = BEAT / 2.0;
    for &(s, l, m, bent) in phrase {
        let midi = m as f32;
        notes.push(Note {
            at: t0 + s as f32 * eighth,
            dur: l as f32 * eighth,
            midi,
            bend_from: bent.then_some(midi - 2.0),
            string: string_for(midi),
            kind: Kind::Lead,
            vel: 0.9,
        });
    }
}

/// How hard each string is ringing at time t (0..1), and the pitch it was
/// last played at: the guitar on screen draws this.
pub fn strings_at(notes: &[Note], t: f32) -> [(f32, f32); 6] {
    let mut out = [(0.0f32, 0.0f32); 6];
    let end = notes.partition_point(|n| n.at <= t);
    for n in notes[..end].iter().rev() {
        let s = n.string.min(5);
        if out[s].1 > 0.0 {
            continue;
        }
        let since = t - n.at;
        out[s] = (n.vel * decay(n, since), n.midi);
    }
    out
}

/// How much of a note is still ringing `since` seconds after it was played.
fn decay(n: &Note, since: f32) -> f32 {
    let rate = match n.kind {
        Kind::Mute => 14.0,
        Kind::Clean => 1.6,
        _ => 2.4,
    };
    let held = if since > n.dur {
        (-(since - n.dur) * 9.0).exp()
    } else {
        1.0
    };
    (-since * rate).exp() * held
}
