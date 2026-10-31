// ═══════════════════════════════════════════════════════════════
// SERIO CONSTRAINTS -- The heart of the demo
//
// This is where solverforge-scoring is actually used.
// Two IncrementalBiConstraints operate on the Orrery solution:
//
// 1. no_collision (HARD): No two planets share the same voice
//    (frequency index). Each world must sing its own note.
// 2. harmonic_ratio (SOFT): Penalise deviation from Pythagorean
//    harmonic intervals between all planet pairs' frequencies.
//
// The TypedScoreDirector orchestrates the retract/insert cycle:
//   before_variable_changed(0, planet_idx)  -> retract
//   planet.freq_idx = new_freq              -> mutate
//   after_variable_changed(0, planet_idx)   -> insert
//
// Only the changed planet's pairwise arcs are re-evaluated.
// The other 15 arc scores are untouched -- that's SERIO.
// ═══════════════════════════════════════════════════════════════

use solverforge_core::score::HardSoftScore;
use solverforge_core::{ConstraintRef, ImpactType};
use solverforge_scoring::IncrementalBiConstraint;

use crate::orrery::harmony::harmonic_deviation;
use crate::orrery::model::{Orrery, Planet, FREQUENCIES};

// ── Constraint type aliases ───────────────────────────────────

/// Hard constraint: no two planets share the same voice (frequency index).
/// Key = freq_idx. Pairs in the same group = collision penalty.
pub type NoCollision = IncrementalBiConstraint<
    Orrery,
    Planet,
    usize, // key type = frequency index
    fn(&Orrery) -> &[Planet],
    fn(&Orrery, &Planet, usize) -> usize,
    fn(&Orrery, &Planet, &Planet, usize, usize) -> bool,
    fn(&Orrery, usize, usize) -> HardSoftScore,
    HardSoftScore,
>;

/// Soft constraint: penalise harmonic deviation for every planet pair.
/// Key = 0 (all in one group -- every pair is evaluated).
pub type HarmonicRatio = IncrementalBiConstraint<
    Orrery,
    Planet,
    u8, // key type = constant 0 (all-pairs group)
    fn(&Orrery) -> &[Planet],
    fn(&Orrery, &Planet, usize) -> u8,
    fn(&Orrery, &Planet, &Planet, usize, usize) -> bool,
    fn(&Orrery, usize, usize) -> HardSoftScore,
    HardSoftScore,
>;

/// The constraint tuple. Implements ConstraintSet<Orrery, HardSoftScore>.
pub type OrreryConstraints = (NoCollision, HarmonicRatio);

// ── Constraint functions ─────────────────────────────────────

fn extract_planets(s: &Orrery) -> &[Planet] {
    s.planets.as_slice()
}

fn collision_key(_s: &Orrery, p: &Planet, _idx: usize) -> usize {
    p.freq_idx
}

fn collision_filter(_s: &Orrery, _a: &Planet, _b: &Planet, _ai: usize, _bi: usize) -> bool {
    true
}

fn collision_weight(_s: &Orrery, _ai: usize, _bi: usize) -> HardSoftScore {
    HardSoftScore::ONE_HARD
}

fn harmonic_key(_s: &Orrery, _p: &Planet, _idx: usize) -> u8 {
    0
}

fn harmonic_filter(_s: &Orrery, _a: &Planet, _b: &Planet, _ai: usize, _bi: usize) -> bool {
    true
}

fn harmonic_weight(s: &Orrery, ai: usize, bi: usize) -> HardSoftScore {
    HardSoftScore::of_soft(pair_penalty(s, ai, bi))
}

/// Orbit weight of a pair: adjacent orbits (distance 1) count 3x,
/// distance-2 pairs 2x. The solver can't just satisfy far-apart pairs and
/// ignore neighbours. This is the only way the orbits enter the score.
pub fn pair_weight(ai: usize, bi: usize) -> i64 {
    match ai.abs_diff(bi) {
        1 => 3,
        2 => 2,
        _ => 1,
    }
}

/// The soft penalty of one pair: its distance from a pure interval, times
/// its orbit weight. The panel sums the same numbers the constraint does.
pub fn pair_penalty(s: &Orrery, ai: usize, bi: usize) -> i64 {
    let f_a = FREQUENCIES[s.planets[ai].freq_idx];
    let f_b = FREQUENCIES[s.planets[bi].freq_idx];
    let (penalty, _, _) = harmonic_deviation(f_a, f_b);
    penalty * pair_weight(ai, bi)
}

/// Pairs of planets on the same note: the hard score is minus this.
pub fn collision_count(s: &Orrery) -> i64 {
    let p = &s.planets;
    (0..p.len())
        .flat_map(|i| ((i + 1)..p.len()).map(move |j| (i, j)))
        .filter(|&(i, j)| p[i].freq_idx == p[j].freq_idx)
        .count() as i64
}

// ── Constructor ───────────────────────────────────────────────

/// Build the orrery constraint tuple.
pub fn build_constraints() -> OrreryConstraints {
    let no_collision: NoCollision = IncrementalBiConstraint::new(
        ConstraintRef::new("serio_02", "No collision"),
        ImpactType::Penalty,
        extract_planets as fn(&Orrery) -> &[Planet],
        collision_key as fn(&Orrery, &Planet, usize) -> usize,
        collision_filter as fn(&Orrery, &Planet, &Planet, usize, usize) -> bool,
        collision_weight as fn(&Orrery, usize, usize) -> HardSoftScore,
        true,
    );

    let harmonic_ratio: HarmonicRatio = IncrementalBiConstraint::new(
        ConstraintRef::new("serio_02", "Harmonic ratio"),
        ImpactType::Penalty,
        extract_planets as fn(&Orrery) -> &[Planet],
        harmonic_key as fn(&Orrery, &Planet, usize) -> u8,
        harmonic_filter as fn(&Orrery, &Planet, &Planet, usize, usize) -> bool,
        harmonic_weight as fn(&Orrery, usize, usize) -> HardSoftScore,
        false,
    );

    (no_collision, harmonic_ratio)
}
