//! Below the weir. D minor, in long breaths against the film's 72 BPM clock.
//! The melody turns around a semitone (A–Bb–A), then falls through a fourth.
//! At the fire it is the lower player who finds the missing last note.
use crate::timeline::{span, Scene, BEAT, BPM, DURATION};
use std::f32::consts::TAU;
pub const SAMPLE_RATE: u32 = 44100;
const SR: f32 = SAMPLE_RATE as f32;
// Absolute score positions, never accumulated rounded durations.
fn frame_at(beat: f32) -> usize {
    (beat as f64 * 60.0 * SAMPLE_RATE as f64 / BPM as f64).round() as usize
}

#[derive(Clone, Copy)]
enum Voice {
    Felt,
    Wire,
    Bow,
    Glass,
    Bass,
    Bend,
}
use Voice::*;
struct Paper(Vec<[f32; 2]>);
impl Paper {
    // Time and gate in beats. Bend is a signed interval below the destination.
    fn note(
        &mut self,
        v: Voice,
        beat: f32,
        key: u8,
        gate: f32,
        (level, pan): (f32, f32),
        bend: f32,
    ) {
        let start = frame_at(beat);
        let hz = 440.0 * 2.0f32.powf((key as f32 - 69.0) / 12.0);
        let (attack, release) = match v {
            Bow => (0.45, 0.85),
            Bend => (0.028, 0.28),
            Glass => (0.005, 0.55),
            Bass => (0.018, 0.18),
            _ => (0.007, 0.48),
        };
        let hold = gate * BEAT;
        let mut phase = 0.0f32;
        let mut last = 0.0;
        let lr = [
            ((1.0 - pan) * 0.5).sqrt() * level,
            ((1.0 + pan) * 0.5).sqrt() * level,
        ];
        for (i, frame) in self
            .0
            .iter_mut()
            .skip(start)
            .take(((hold + release) * SR) as usize)
            .enumerate()
        {
            let t = i as f32 / SR;
            let a = (t / attack).min(1.0);
            let r = (1.0 - (t - hold).max(0.0) / release).max(0.0);
            let env = a * (2.0 - a) * r * r;
            let y = match v {
                Bend => {
                    let u = ((t - 0.075) / (hold * 0.42).clamp(0.12, 0.75)).clamp(0.0, 1.0);
                    let cents = -bend * (1.0 - u * u * (3.0 - 2.0 * u))
                        + 0.19 * (TAU * 4.9 * t).sin() * ((t - 0.32) * 2.5).clamp(0.0, 1.0);
                    phase = (phase + hz / SR * 2.0f32.powf(cents / 12.0)).fract();
                    let p = TAU * phase;
                    // A neck pickup answers a bridge pickup: the far-bank
                    // player frets low, with less pick and more fundamental.
                    let pick = (-8.0 * t).exp();
                    let bridge = pan < 0.0;
                    let bite = if bridge { 0.41 } else { 0.19 };
                    let string = p.sin()
                        + (bite + 0.19 * pick) * (2.0 * p).sin()
                        + (0.11 + 0.21 * pick) * (3.0 * p).sin()
                        + 0.12 * pick * (5.0 * p).sin();
                    let driven = (string * (1.25 + 0.8 * pick)).tanh();
                    last += (if bridge { 0.32 } else { 0.19 }) * (driven - last);
                    last * (0.82 + 0.18 * (-2.0 * t).exp())
                }
                _ => {
                    let p = TAU * hz * t;
                    match v {
                        Felt => {
                            (p + 1.2 * (-3.1 * t).exp() * (2.0 * p).sin()).sin() * (-0.94 * t).exp()
                        }
                        Wire => {
                            (p.sin()
                                + 0.45 * (2.0 * p).sin() * (-1.5 * t).exp()
                                + 0.23 * (3.0 * p).sin() * (-2.4 * t).exp())
                                * (0.82 + 0.18 * (TAU * 4.4 * t).cos())
                                * (-0.68 * t).exp()
                        }
                        Bow => {
                            let q = p + 0.036 * (TAU * 4.6 * t).sin();
                            0.54 * q.sin()
                                + 0.32 * (p * 1.002).sin()
                                + 0.18 * (2.0 * q).sin()
                                + 0.09 * (3.0 * q).sin()
                        }
                        Glass => {
                            (p.sin()
                                + 0.39 * (2.71 * p).sin() * (-2.2 * t).exp()
                                + 0.12 * (4.09 * p).sin() * (-4.1 * t).exp())
                                * (-1.48 * t).exp()
                        }
                        Bass => {
                            (p.sin() + 0.32 * (2.0 * p).sin() * (-2.0 * t).exp())
                                * (-0.62 * t).exp()
                        }
                        _ => unreachable!(),
                    }
                }
            } * env;
            frame[0] += y * lr[0];
            frame[1] += y * lr[1];
        }
    }
    fn line(&mut self, start: f32, v: Voice, notes: &[(f32, u8, f32)], gain: f32, pan: f32) {
        for (i, &(at, key, len)) in notes.iter().enumerate() {
            // Pickups are lighter; lean into an ascending arrival, then let
            // the last note trail off. Keep the scene clock untouched.
            let rising = i > 0 && notes[i - 1].1 < key;
            let accent = if len < 0.8 {
                0.71
            } else if i + 1 == notes.len() {
                0.81
            } else if rising {
                1.04
            } else {
                0.92
            };

            self.note(v, start + at, key, len, (gain * accent, pan), 0.0)
        }
    }
}

fn beat(s: Scene) -> f32 {
    (span(s).0 / BEAT).round()
}
// Bass, tenor, alto, upper string. Inversions keep inner voices nearly still
// while the ground descends D, C#, C, B, Bb, A. No triumphant major cadence.
const CH: [[u8; 4]; 10] = [
    [38, 53, 57, 62],
    [37, 52, 57, 61],
    [36, 53, 57, 62],
    [35, 50, 55, 62],
    [34, 53, 57, 62],
    [33, 52, 55, 61],
    [31, 50, 55, 58],
    [39, 51, 55, 58],
    [33, 50, 53, 57],
    [38, 50, 57, 62],
];
const CALL: &[(f32, u8, f32)] = &[
    (1.5, 57, 0.85),
    (2.5, 58, 0.48),
    (3.0, 57, 3.6),
    (7.0, 52, 0.73),
    (8.0, 53, 3.2),
    (12.5, 50, 2.6),
];
const ANSWER: &[(f32, u8, f32)] = &[
    (0.5, 62, 1.25),
    (2.0, 65, 2.6),
    (5.5, 64, 0.85),
    (6.5, 62, 1.2),
    (8.5, 58, 2.6),
    (12.0, 57, 2.75),
];
// The last eighteen bars share a sequencer clock: embers, ignition, solo,
// half-time sunrise, then one last downbeat with the completed seal.
fn finale(m: &mut Paper, f: f32) {
    // Phase-continuous stereo lead. Slides connect adjacent notes; rests let
    // the amplifier release. Held notes grow a fifth harmonic into feedback.
    let notes: &[(f32, f32, u8, f32)] = &[
        (22.5, 0.5, 62, 0.),
        (23., 0.5, 65, 0.),
        (23.5, 0.5, 67, 0.),
        (24., 1.85, 69, 2.),
        (26., 0.5, 70, 0.),
        (26.5, 2.3, 69, 0.),
        (29., 1., 64, 0.),
        (30., 1.8, 61, 1.),
        (32., 2.5, 74, 2.),
        (34.5, 0.5, 72, 0.),
        (35., 0.8, 69, 0.),
        (36., 1.5, 65, 0.),
        (37.5, 0.5, 67, 0.),
        (38., 1.8, 69, 0.),
        (40., 2., 70, 2.),
        (42., 1., 74, 0.),
        (43., 0.75, 77, 0.),
        (44., 3.4, 76, 3.),
        (48., 2.8, 77, 2.),
        (51., 0.5, 76, 0.),
        (51.5, 0.5, 74, 0.),
        (52., 2., 79, 2.),
        (54., 1., 77, 0.),
        (55., 1., 74, 0.),
        (56., 2.5, 81, 3.),
        (58.5, 0.5, 79, 0.),
        (59., 0.9, 77, 0.),
        (60., 1.45, 76, 2.),
        (61.5, 0.5, 73, 0.),
        (62., 0.6, 69, 0.),
        (64., 5.7, 74, 2.),
    ];
    let mut phase = [0.0f32, 0.29, 0.63];
    let mut lp = [0.0f32; 2];
    let mut pitch = 67.0;
    let mut env = 0.0;
    let mut n = 0;
    for (i, fr) in m.0.iter_mut().skip(frame_at(f)).enumerate() {
        let b = i as f32 / (SR * BEAT);
        if b < notes[0].0 {
            continue;
        }
        while n + 1 < notes.len() && b >= notes[n + 1].0 {
            n += 1
        }
        let (o, d, k, bend) = notes[n];
        let age = (b - o) * BEAT;
        let u = (age / (0.13 + d * 0.09)).clamp(0.0, 1.0);
        let aim = k as f32 - bend * (1.0 - u * u * (3.0 - 2.0 * u))
            + 0.17 * (age * TAU * 5.1).sin() * (age - 0.22).clamp(0.0, 1.0);
        pitch += (aim - pitch) * 0.0013;
        let gate = if b < o + d { 1.0 } else { 0.0 };
        env += (gate - env) * if gate > 0.0 { 0.003 } else { 0.00038 };
        let step = 440.0 * 2.0f32.powf((pitch - 69.0) / 12.0) / SR;
        for (ch, det) in [1.0, 0.997, 1.003].iter().enumerate() {
            phase[ch] = (phase[ch] + step * det).fract()
        }
        for ch in 0..2 {
            let p = TAU * phase[0];
            let q = TAU * phase[ch + 1];
            let raw = p.sin()
                + 0.48 * q.sin()
                + 0.44 * (2.0 * q).sin()
                + 0.24 * (3.0 * p).sin()
                + 0.20 * age.min(1.0) * (5.0 * q).sin();
            lp[ch] += 0.27 * ((raw * 1.8).tanh() - lp[ch]);
            fr[ch] += lp[ch] * env * 0.207;
        }
    }
}
// One rhythm section across all 56 bars. Slots are sixteenths counted
// from the head of the record. Pickups belong to the following phrase.
const FORM: [usize; 56] = [
    9, 9, 7, 8, 0, 1, 2, 3, 4, 6, 5, 0, 1, 2, 3, 6, 8, 4, 6, 7, 8, 5, 0, 7, 6, 8, 4, 6, 5, 9, 9, 7,
    7, 8, 6, 2, 4, 5, 0, 1, 2, 3, 4, 6, 8, 5, 0, 2, 6, 8, 4, 6, 8, 5, 0, 9,
];
#[derive(Clone, Copy)]
enum Skin {
    Foot,
    Side,
    Snare,
}
fn skin(m: &mut Paper, b: f32, v: Skin, g: f32) {
    let end = match v {
        Skin::Foot => 0.44,
        Skin::Side => 0.16,
        Skin::Snare => 0.32,
    };
    let at = frame_at(b);
    let mut rng = 0x832719a5u32 ^ at as u32;
    let mut lp = 0.0;
    let mut body = 0.0;
    for (i, f) in
        m.0.iter_mut()
            .skip(at)
            .take((end * SR) as usize)
            .enumerate()
    {
        let x = i as f32 / SR;
        rng ^= rng << 13;
        rng ^= rng >> 17;
        rng ^= rng << 5;
        let n = rng as i32 as f32 / i32::MAX as f32;
        lp += 0.20 * (n - lp);
        body += 0.034 * (lp - body);
        let y = match v {
            Skin::Foot => {
                (TAU * (46.0 * x + 1.52 * (1.0 - (-40.0 * x).exp()))).sin() * (-9.5 * x).exp()
            }
            Skin::Side => {
                (0.55 * (TAU * 188.0 * x).sin() + 0.6 * lp)
                    * (1.0 - (-85.0 * x).exp())
                    * (-27.0 * x).exp()
            }
            Skin::Snare => {
                // Muted head plus band-limited wire, with a continuous decay.
                let head = (TAU * 166.0 * x).sin() + 0.29 * (TAU * 247.0 * x).sin();
                0.43 * head * (-26.0 * x).exp() + 1.15 * (lp - body) * (-17.0 * x).exp()
            }
        };
        let y = y * g * (x * 1100.0).min(1.0) * ((end - x) * 180.0).min(1.0);
        f[0] += y;
        f[1] += y * if matches!(v, Skin::Side) { 0.74 } else { 1.0 };
    }
}
fn arrange(m: &mut Paper) {
    use Skin::*;
    // Strings tie across cuts, not merely across identical bars of one scene.
    for j in 1..=3 {
        let mut i = 0;
        while i < 56 {
            let level = |b: usize| match b {
                0..=2 | 29..=32 if j != 2 => 0,
                38..=43 if j != 1 => 0,
                44..=53 => 2,
                54..=55 if j == 3 => 0,
                _ => 1,
            };
            let k = CH[FORM[i]][j];
            let l = level(i);
            let mut z = i + 1;
            while z < 56 && level(z) == l && CH[FORM[z]][j] == k {
                z += 1
            }
            if l > 0 {
                m.note(
                    Bow,
                    4.0 * i as f32,
                    k,
                    4.0 * (z - i) as f32 - 0.21,
                    (
                        if l == 2 { 0.049 } else { 0.055 },
                        [-0.4, 0.16, 0.42][j - 1],
                    ),
                    0.0,
                )
            }
            i = z;
        }
    }
    for b in 0..56 {
        let h = CH[FORM[b]];
        let t = 4.0 * b as f32;
        let next = CH[FORM[(b + 1).min(55)]][0];
        // Rhythm is a two-bar sentence: the bass answers the foot rather than doubling it all night.
        let bass: &[(u8, u8, f32)] = match b {
            0..=3 | 29..=33 | 54..=55 => &[(0, 0, 3.78)],
            4..=10 | 34..=37 => {
                if b % 2 == 0 {
                    &[(0, 0, 2.4), (12, 7, 0.62)]
                } else {
                    &[(0, 0, 1.7), (10, 12, 0.7), (14, 99, 0.45)]
                }
            }
            11..=21 | 38..=42 => {
                if b % 2 == 0 {
                    &[(0, 0, 1.25), (6, 7, 0.7), (10, 12, 0.78), (14, 99, 0.38)]
                } else {
                    &[(0, 0, 1.82), (8, 7, 0.64), (12, 0, 0.78)]
                }
            }
            22..=27 => &[(0, 0, 1.75), (8, 7, 1.55)],
            28 => &[(0, 0, 2.8)],
            43 | 53 => &[(0, 0, 1.3), (6, 7, 0.6)],
            _ => &[
                (0, 0, 0.84),
                (4, 12, 0.36),
                (6, 0, 0.43),
                (8, 7, 0.84),
                (12, 0, 0.4),
                (14, 99, 0.42),
            ],
        };
        for &(slot, d, len) in bass {
            let k = if d == 99 {
                if next >= h[0] {
                    next.saturating_sub(1)
                } else {
                    next + 1
                }
            } else {
                h[0] + d
            };
            let quiet = matches!(b,0..=2|29..=32|55);
            m.note(
                if quiet { Bow } else { Bass },
                t + slot as f32 / 4.0,
                k,
                len,
                (
                    if quiet {
                        0.11
                    } else if slot == 0 {
                        0.227
                    } else {
                        0.148
                    },
                    0.0,
                ),
                0.0,
            )
        }
        // Count in quarters: the kick marks one and the brushed head answers
        // on three. Keep that pocket through the falls; widen to a backbeat
        // only when the campfire solo has the floor.
        let hits: &[(u8, Skin, f32)] = match b {
            0..=2 | 29..=36 | 55 => &[],
            3..=10 => &[(0, Foot, 0.105), (2, Side, 0.073)],
            11..=21 => &[(0, Foot, 0.13), (2, Side, 0.091)],
            22..=27 => &[(0, Foot, 0.16), (2, Snare, 0.12)],
            28 | 37 | 43 | 53 => &[(0, Foot, 0.13)],
            38..=42 => &[(0, Foot, 0.135), (2, Side, 0.095)],
            44..=52 => &[
                (0, Foot, 0.17),
                (1, Snare, 0.095),
                (2, Foot, 0.13),
                (3, Snare, 0.107),
            ],
            54 => &[(0, Foot, 0.18)],
            _ => unreachable!(),
        };
        for &(quarter, v, g) in hits {
            skin(m, t + quarter as f32, v, g)
        }
        // The piano answers the singer with one held dyad. Both hammers
        // land together. The bar after each answer is left to the bass.
        if matches!(
            b,
            4 | 6 | 8 | 12 | 14 | 16 | 22 | 24 | 26 | 34 | 36 | 38 | 40
        ) {
            for (j, gain) in [(1, 0.104), (3, 0.078)] {
                m.note(Felt, t + 2.0, h[j], 2.75, (gain, 0.16), 0.0);
            }
        }
        // The sequenced string is a straight eighth-note ostinato, not a piano fill.
        if (41..=53).contains(&b) {
            for (i, j) in [1, 2, 3, 2, 1, 2, 3, 2].into_iter().enumerate() {
                if (b == 43 || b == 53) && i >= 4 {
                    break;
                }
                m.note(
                    Wire,
                    t + i as f32 * 0.5,
                    h[j],
                    0.23,
                    (0.070, if i % 2 == 0 { -0.3 } else { 0.3 }),
                    0.0,
                );
            }
        }
    }
    m.line(
        33.0 * 4.0,
        Bow,
        &[(1.5, 57, 1.35), (3.0, 53, 0.8), (4.0, 50, 2.7)],
        0.11,
        -0.23,
    );
}
pub fn render() -> Vec<[f32; 2]> {
    use Scene::*;
    let mut m = Paper(vec![[0.0; 2]; (DURATION * SR).round() as usize]);
    arrange(&mut m);
    // Bells are an answering voice: beat three, then beat one. Interleave
    // them with the firefly piano instead of doubling its hammer attack.
    // All entries are whole quarter notes, including the gallery cadence.
    const BELLS: &[(u8, u8, u8, f32)] = &[
        (2, 2, 67, 0.060),
        (3, 0, 62, 0.052),
        (18, 0, 67, 0.064),
        (20, 0, 65, 0.058),
        (28, 2, 64, 0.072),
        (29, 0, 62, 0.065),
        (31, 2, 58, 0.084),
        (32, 0, 63, 0.077),
        (33, 0, 57, 0.061),
    ];
    for &(bar, quarter, key, gain) in BELLS {
        m.note(
            Glass,
            (4 * bar + quarter) as f32,
            key,
            1.25,
            (gain, 0.30),
            0.0,
        );
    }
    m.line(
        beat(Owl),
        Wire,
        &[(2.0, 45, 2.7), (7.0, 46, 1.6), (10.0, 45, 3.5)],
        0.215,
        -0.28,
    );
    m.line(beat(Village), Wire, CALL, 0.265, -0.25);
    m.line(beat(Forest), Wire, &CALL[..4], 0.22, -0.25);
    m.line(beat(Forest) + 8.0, Bow, ANSWER, 0.172, 0.29);
    // Same rhythm, now in the foreground: A, its upper neighbour, back to A.
    m.line(
        beat(Fireflies),
        Felt,
        &[
            (2.0, 69, 3.25),
            (6.0, 70, 3.25),
            (10.0, 69, 3.25),
            (14.0, 65, 3.25),
        ],
        0.17,
        0.16,
    );
    // One sustained argument across the falls. Lower answers occupy the
    // silences; the players meet only at the crest, then separate in contrary motion.
    let t = beat(Falls);
    for &(o, k, d, b) in &[
        (0.5, 69, 1.3, 2.0),
        (2.0, 70, 0.7, 0.0),
        (3.0, 69, 2.8, 0.0),
        (8.0, 67, 2.3, 2.0),
        (10.5, 65, 0.85, 0.0),
        (12.0, 62, 2.65, 0.0),
        (16.0, 65, 1.35, 0.0),
        (17.5, 69, 1.85, 2.0),
        (20.0, 70, 3.65, 3.0),
        (24.0, 69, 1.85, 0.0),
        (26.0, 64, 1.7, 0.0),
    ] {
        m.note(Bend, t + o, k, d, (0.231, -0.34), b)
    }
    for &(o, k, d, b) in &[
        (5.75, 58, 1.7, 1.0),
        (14.75, 57, 1.1, 0.0),
        (18.0, 62, 1.7, 2.0),
        (20.0, 55, 3.2, 0.0),
        (24.0, 57, 2.6, 1.0),
    ] {
        m.note(Bend, t + o, k, d, (0.185, 0.32), b)
    }
    m.line(
        beat(Lake),
        Wire,
        &[
            (1.0, 50, 3.6),
            (5.0, 53, 2.5),
            (8.25, 57, 2.3),
            (11.5, 58, 0.8),
            (12.5, 57, 2.6),
        ],
        0.165,
        -0.19,
    );
    let f = beat(Campfire);
    m.line(
        f,
        Bow,
        &[
            (1.5, 57, 0.9),
            (2.5, 58, 0.55),
            (3.0, 57, 4.0),
            (8.0, 53, 2.6),
            (12.0, 55, 2.9),
        ],
        0.145,
        0.23,
    );
    m.line(
        f + 16.0,
        Wire,
        &[
            (0.0, 53, 1.8),
            (2.0, 57, 1.35),
            (4.0, 58, 1.4),
            (5.5, 57, 0.52),
        ],
        0.18,
        -0.25,
    );
    // The lower answer waits until the foreground has taken a breath.
    m.line(
        f + 30.0,
        Bow,
        &[(1.0, 49, 0.72), (2.0, 50, 2.8), (7.0, 53, 1.75)],
        0.10,
        0.3,
    );
    m.line(
        beat(Crossing) + 4.0,
        Wire,
        &[(1.5, 49, 0.8), (3.0, 52, 0.7)],
        0.09,
        -0.35,
    );
    m.line(
        beat(Fireflies) + 16.0,
        Wire,
        &[(0.0, 52, 0.6), (1.5, 55, 0.6), (3.0, 61, 0.75)],
        0.135,
        0.21,
    );
    finale(&mut m, f);
    // Early reflections off the bank feed four mutually coupled, damped tails.
    // Keep the direct instrument close, especially the quiet fire duet.
    let lengths = [1559usize, 2081, 2671, 3191];
    let mut delays = lengths.map(|n| vec![0.0f32; n]);
    let mut pos = [0usize; 4];
    let mut air = [0.0f32; 4];
    for frame in &mut m.0 {
        let dry = *frame;
        for j in 0..4 {
            air[j] += 0.18 * (delays[j][pos[j]] - air[j]);
        }
        let sum = air.iter().sum::<f32>() * 0.5;
        for j in 0..4 {
            delays[j][pos[j]] = dry[j % 2] * 0.19 + (sum - air[j]) * 0.82;
            pos[j] = (pos[j] + 1) % lengths[j];
        }
        frame[0] += 0.48 * (air[0] - air[2]) + 0.18 * air[3];
        frame[1] += 0.48 * (air[1] - air[3]) + 0.18 * air[2];
    }
    for (i, frame) in m.0.iter_mut().enumerate() {
        let t = i as f32 / SR;
        let fade = (t / 1.3).min(1.0) * ((DURATION - t) / 3.4).clamp(0.0, 1.0);
        for x in frame {
            *x = (*x * 1.85).tanh() * 0.92 * fade
        }
    }
    m.0
}
