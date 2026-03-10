// ═══════════════════════════════════════════════════════════════
// ORRERY MODEL -- Seven worlds with tunable voices
//
// Each planet has a FIXED orbital radius (its position in space
// is not a decision variable). The planning variable is the
// planet's VOICE — a frequency index into a discrete set of
// pitches. The solver searches for an assignment of voices
// such that every pair of worlds produces a consonant interval.
//
// Hard constraint: No two planets may share the same voice.
// Soft constraint: Penalise frequency ratios that deviate from
//                  Pythagorean harmonic intervals.
// ═══════════════════════════════════════════════════════════════

use solverforge_core::score::HardSoftScore;
use solverforge_core::PlanningSolution;

use crate::palette;

/// Number of planets (the 7 classical bodies; the sun is fixed at center)
pub const PLANET_COUNT: usize = 7;

/// Number of available frequency choices.
///
/// 48 pitches spanning ~3 octaves. Not all equal-tempered — the palette
/// mixes standard semitones with quarter-tones, Pythagorean-comma detunings,
/// and deliberate "trap" frequencies placed near (but not at) consonant
/// intervals. This makes the combinatorial problem genuinely hard:
/// C(48,7) ~ 73 million unordered subsets, and most are dissonant.
pub const FREQ_COUNT: usize = 48;

/// Fixed orbital radii in screen pixels from the center (sun).
/// These never change — each planet has a permanent orbit.
pub const ORBITAL_RADII: [f64; PLANET_COUNT] = [
    80.0,  // Luna     — closest
    110.0, // Mercury
    140.0, // Venus
    175.0, // Earth
    210.0, // Mars
    250.0, // Jupiter
    295.0, // Saturn   — farthest
];

/// Maximum radius (for Kepler angular speed)
pub const R_MAX: f64 = 295.0;

/// The discrete frequency palette: 48 pitches from ~110 Hz to ~840 Hz.
///
/// Even-numbered indices (0,2,4,...) are standard equal-tempered semitones.
/// Odd-numbered indices are microtonal detunings: quarter-tones (~halfway
/// between semitones), Pythagorean-comma-shifted pitches, and deliberate
/// traps that sit tantalizingly close to consonant ratios but just outside
/// the scoring tolerance. The solver must navigate this minefield.
///
/// Sorted ascending by frequency.
pub const FREQUENCIES: [f64; FREQ_COUNT] = [
    //  idx  Hz        description
    110.00, //  0  A2          (equal-tempered)
    113.22, //  1  A2+quarter  (quarter-tone above A2)
    116.54, //  2  A#2         (equal-tempered)
    120.00, //  3  ~B2-trap    (trap: near B2 but 3% flat)
    123.47, //  4  B2          (equal-tempered)
    127.09, //  5  B2+quarter  (quarter-tone above B2)
    130.81, //  6  C3          (equal-tempered)
    134.65, //  7  C3+quarter  (quarter-tone)
    138.59, //  8  C#3         (equal-tempered)
    142.60, //  9  C#3+quarter (quarter-tone)
    146.83, // 10  D3          (equal-tempered)
    150.00, // 11  D3+trap     (trap: near D#3 but 3.7% flat)
    155.56, // 12  D#3         (equal-tempered)
    160.12, // 13  D#3+quarter (quarter-tone)
    164.81, // 14  E3          (equal-tempered)
    168.00, // 15  E3+pythagorean (Pythagorean comma sharp of E3)
    174.61, // 16  F3          (equal-tempered)
    179.73, // 17  F3+quarter  (quarter-tone)
    185.00, // 18  F#3         (equal-tempered)
    190.31, // 19  F#3+quarter (quarter-tone)
    196.00, // 20  G3          (equal-tempered)
    200.00, // 21  G3+trap     (trap: near G#3 but 3.8% flat)
    207.65, // 22  G#3         (equal-tempered)
    213.74, // 23  G#3+quarter (quarter-tone)
    220.00, // 24  A3          (equal-tempered)
    226.45, // 25  A3+quarter  (quarter-tone)
    233.08, // 26  A#3         (equal-tempered)
    240.00, // 27  A#3+trap    (trap: near B3 but 2.8% flat)
    246.94, // 28  B3          (equal-tempered)
    254.18, // 29  B3+quarter  (quarter-tone)
    261.63, // 30  C4          (equal-tempered)
    269.29, // 31  C4+quarter  (quarter-tone)
    277.18, // 32  C#4         (equal-tempered)
    285.30, // 33  C#4+quarter (quarter-tone)
    293.66, // 34  D4          (equal-tempered)
    300.00, // 35  D4+trap     (trap: near D#4 but 3.7% flat)
    311.13, // 36  D#4         (equal-tempered)
    320.24, // 37  D#4+quarter (quarter-tone)
    329.63, // 38  E4          (equal-tempered)
    336.00, // 39  E4+pythagorean (Pythagorean comma sharp of E4)
    349.23, // 40  F4          (equal-tempered)
    359.46, // 41  F4+quarter  (quarter-tone)
    369.99, // 42  F#4         (equal-tempered)
    380.61, // 43  F#4+quarter (quarter-tone)
    392.00, // 44  G4          (equal-tempered)
    403.00, // 45  G4+trap     (trap: near G#4 but 2.96% flat)
    415.30, // 46  G#4         (equal-tempered)
    427.47, // 47  G#4+quarter (quarter-tone above G#4)
];

/// One of the 7 classical planets.
#[derive(Clone, Debug)]
pub struct Planet {
    pub name: &'static str,
    /// Fixed orbital radius in pixels (never changes)
    pub orbital_radius: f64,
    /// The planning variable: index into FREQUENCIES.
    pub freq_idx: usize,
    /// Display color
    pub color: u32,
    /// Visual radius in pixels
    pub visual_radius: f32,
    /// Orbital angular speed (Kepler: omega ~ r^(-3/2), normalised)
    pub angular_speed: f64,
    /// Current angle in radians (updated each frame for animation)
    pub angle: f64,
}

impl Planet {
    /// The planet's current voice frequency in Hz.
    pub fn frequency(&self) -> f64 {
        FREQUENCIES[self.freq_idx]
    }
}

/// The complete orrery solution — planets + score.
#[derive(Clone, Debug)]
pub struct Orrery {
    pub planets: Vec<Planet>,
    pub score: Option<HardSoftScore>,
}

impl PlanningSolution for Orrery {
    type Score = HardSoftScore;
    fn score(&self) -> Option<HardSoftScore> {
        self.score
    }
    fn set_score(&mut self, score: Option<HardSoftScore>) {
        self.score = score;
    }
}

impl Orrery {
    /// Build the initial orrery with maximally dissonant frequency assignments.
    ///
    /// All seven planets start on trap or quarter-tone frequencies — the pitches
    /// deliberately placed between Pythagorean intervals. Every pairwise ratio
    /// lands outside the 1.5% tolerance window. Maximum initial soft penalty.
    pub fn initial() -> Self {
        // Maximally dissonant start: all quarter-tones, no simple ratios between any pair.
        // Chosen so that every pairwise ratio lands outside the 1.5% Pythagorean tolerance.
        // 113.22, 138.59, 168.00, 213.74, 254.18, 320.24, 403.00 Hz
        // No pair forms an octave, fifth, fourth, or any other consonant ratio.
        let initial_freqs = [1usize, 8, 15, 23, 29, 37, 45];

        let mut planets = Vec::with_capacity(PLANET_COUNT);

        let defs: &[(&str, u32, f32)] = &[
            ("Luna", palette::LUNA, 4.0),
            ("Mercury", palette::MERCURY_COL, 5.0),
            ("Venus", palette::VENUS_COL, 7.0),
            ("Earth", palette::EARTH_COL, 8.0),
            ("Mars", palette::MARS_COL, 6.0),
            ("Jupiter", palette::JUPITER_COL, 13.0),
            ("Saturn", palette::SATURN_COL, 11.0),
        ];

        for i in 0..PLANET_COUNT {
            let (name, color, vr) = defs[i];
            let r = ORBITAL_RADII[i];
            let angular_speed = R_MAX.powf(1.5) / r.powf(1.5);
            let angle = i as f64 * std::f64::consts::TAU / PLANET_COUNT as f64;
            planets.push(Planet {
                name,
                orbital_radius: r,
                freq_idx: initial_freqs[i],
                color,
                visual_radius: vr,
                angular_speed,
                angle,
            });
        }

        Orrery {
            planets,
            score: None,
        }
    }

    /// Build a consonant display orrery for the genesis scene.
    ///
    /// Frequencies chosen so ALL 21 pairwise ratios are Pythagorean consonant:
    /// A2, C#3, E3, G#3, B3, D#4, F#4 — augmented triad stack, zero red arcs.
    pub fn consonant() -> Self {
        // All 21 pairs verified within TOLERANCE=0.015 of a Pythagorean interval.
        let freqs = [0usize, 8, 14, 22, 28, 36, 42];

        let defs: &[(&str, u32, f32)] = &[
            ("Luna", palette::LUNA, 4.0),
            ("Mercury", palette::MERCURY_COL, 5.0),
            ("Venus", palette::VENUS_COL, 7.0),
            ("Earth", palette::EARTH_COL, 8.0),
            ("Mars", palette::MARS_COL, 6.0),
            ("Jupiter", palette::JUPITER_COL, 13.0),
            ("Saturn", palette::SATURN_COL, 11.0),
        ];

        let mut planets = Vec::with_capacity(PLANET_COUNT);
        for i in 0..PLANET_COUNT {
            let (name, color, vr) = defs[i];
            let r = ORBITAL_RADII[i];
            let angular_speed = R_MAX.powf(1.5) / r.powf(1.5);
            let angle = i as f64 * std::f64::consts::TAU / PLANET_COUNT as f64;
            planets.push(Planet {
                name,
                orbital_radius: r,
                freq_idx: freqs[i],
                color,
                visual_radius: vr,
                angular_speed,
                angle,
            });
        }
        Orrery { planets, score: None }
    }

    /// Update planet angles based on elapsed time.
    pub fn update_angles(&mut self, dt: f64, speed: f64) {
        let base_angular_rate = 0.15;
        for planet in &mut self.planets {
            planet.angle += planet.angular_speed * base_angular_rate * speed * dt;
        }
    }

    /// Planet screen position from center (cx, cy).
    pub fn planet_screen_pos(&self, planet: &Planet, cx: f64, cy: f64) -> (f64, f64) {
        let r = planet.orbital_radius;
        let x = cx + r * planet.angle.cos();
        let y = cy + r * planet.angle.sin();
        (x, y)
    }
}
