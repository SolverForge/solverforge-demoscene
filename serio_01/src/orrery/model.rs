// ═══════════════════════════════════════════════════════════════
// ORRERY MODEL -- Classical 7-planet constraint problem
//
// The 7 classical celestial bodies (Greek/Roman cosmology):
//   Luna, Mercury, Venus, Sol (fixed), Mars, Jupiter, Saturn
//
// Each planet is assigned to one of SLOT_COUNT orbital radii.
// The solver optimises the assignment so that pairwise orbital
// radius ratios approximate Pythagorean harmonic intervals.
//
// Hard constraint: No two planets share the same orbital slot.
// Soft constraint: Penalise deviation from pure harmonic ratios.
// ═══════════════════════════════════════════════════════════════

use solverforge_core::score::HardSoftScore;
use solverforge_core::PlanningSolution;

use crate::palette;

/// Number of planets (the 7 classical bodies, Sol is fixed at center)
pub const PLANET_COUNT: usize = 7;

/// Number of orbital slot choices (more than planets for solver freedom)
pub const SLOT_COUNT: usize = 21;

/// Orbital radii in screen pixels from the center (sun).
/// Spaced so that some ratios are near-Pythagorean, others not.
/// Range: 70px to 300px. Non-uniform to allow harmonic choices.
pub const ORBITAL_RADII: [f64; SLOT_COUNT] = [
    70.0,  // slot 0
    78.0,  // slot 1
    84.0,  // slot 2  (84/70 = 1.200 = Minor Third)
    91.0,  // slot 3
    98.0,  // slot 4
    105.0, // slot 5  (105/70 = 1.500 = Perfect Fifth)
    112.0, // slot 6
    120.0, // slot 7
    126.0, // slot 8  (126/84 = 1.500, 126/70 = 1.800)
    133.0, // slot 9
    140.0, // slot 10 (140/105 = 1.333 = Fourth; 140/70 = 2.0 = Octave)
    150.0, // slot 11
    158.0, // slot 12
    168.0, // slot 13 (168/126 = 1.333 = Fourth)
    175.0, // slot 14 (175/140 = 1.250 = Major Third)
    187.0, // slot 15
    200.0, // slot 16
    210.0, // slot 17 (210/140 = 1.500; 210/105 = 2.0)
    224.0, // slot 18
    240.0, // slot 19
    252.0, // slot 20 (252/168 = 1.500; 252/126 = 2.0)
];

/// Maximum radius (used for Kepler frequency calculation)
pub const R_MAX: f64 = ORBITAL_RADII[SLOT_COUNT - 1];

/// One of the 7 classical planets.
#[derive(Clone, Debug)]
pub struct Planet {
    pub name: &'static str,
    /// The planning variable: which orbital slot this planet occupies.
    /// None = unassigned (won't happen after initialization).
    pub slot: usize,
    /// Display color
    pub color: u32,
    /// Visual radius in pixels
    pub visual_radius: f32,
    /// Orbital speed multiplier (Kepler: ω ∝ r^(-3/2), normalized)
    pub angular_speed: f64,
    /// Current angle in radians (updated each frame)
    pub angle: f64,
}

impl Planet {
    pub fn orbital_radius(&self) -> f64 {
        ORBITAL_RADII[self.slot]
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
    /// Build the initial orrery with planets in arbitrary (bad) slots.
    /// The solver will rearrange them to maximize harmonic consonance.
    pub fn initial() -> Self {
        // Starting slots — deliberately non-harmonic, spread out
        // so the solver has interesting work to do.
        let initial_slots = [0usize, 3, 7, 11, 15, 17, 20];

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
            let slot = initial_slots[i];
            let r: f64 = ORBITAL_RADII[slot];
            // Kepler: ω = k / r^(3/2). Normalize to outermost = 1.0 base unit.
            let angular_speed = R_MAX.powf(1.5) / r.powf(1.5);
            // Spread initial angles so planets aren't stacked
            let angle = i as f64 * std::f64::consts::TAU / PLANET_COUNT as f64;
            planets.push(Planet {
                name,
                slot,
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

    /// Planet screen position from center (cx, cy).
    pub fn planet_screen_pos(&self, planet: &Planet, cx: f64, cy: f64) -> (f64, f64) {
        let r = planet.orbital_radius();
        let x = cx + r * planet.angle.cos();
        let y = cy + r * planet.angle.sin();
        (x, y)
    }
}
