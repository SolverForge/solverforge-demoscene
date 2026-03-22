// ═══════════════════════════════════════════════════════════════
// SERIO CONSTRAINTS -- The heart of the demo
//
// This is where solverforge-scoring is actually used.
// Two IncrementalBiConstraints operate on the Orrery solution:
//
// 1. no_collision (HARD): No two planets in the same orbital slot.
// 2. harmonic_ratio (SOFT): Penalise deviation from Pythagorean
//    harmonic intervals between all planet pairs.
//
// The TypedScoreDirector orchestrates the retract/insert cycle:
//   before_variable_changed(0, planet_idx)  → retract
//   planet.slot = new_slot                   → mutate
//   after_variable_changed(0, planet_idx)   → insert
//
// Only the changed planet's pairwise arcs are re-evaluated.
// The other 15 arc scores are untouched — that's SERIO.
// ═══════════════════════════════════════════════════════════════

use solverforge_core::score::HardSoftScore;
use solverforge_core::{ConstraintRef, ImpactType};
use solverforge_scoring::IncrementalBiConstraint;

use crate::orrery::harmony::harmonic_deviation;
use crate::orrery::model::{Orrery, Planet, ORBITAL_RADII};

// ── Constraint type aliases ───────────────────────────────────
// These are complex generic types — we use type aliases to keep
// the code readable while preserving zero-erasure monomorphization.

/// Hard constraint: no two planets in the same orbital slot.
/// Key = slot index. Pairs in the same slot = collision penalty.
pub type NoCollision = IncrementalBiConstraint<
    Orrery,
    Planet,
    usize, // key type = slot index
    fn(&Orrery) -> &[Planet],
    fn(&Orrery, &Planet, usize) -> usize,
    fn(&Orrery, &Planet, &Planet, usize, usize) -> bool,
    fn(&Orrery, usize, usize) -> HardSoftScore,
    HardSoftScore,
>;

/// Soft constraint: penalise harmonic deviation for every planet pair.
/// Key = 0 (all in one group — every pair is evaluated).
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
    p.slot
}

fn collision_filter(_s: &Orrery, _a: &Planet, _b: &Planet, ai: usize, bi: usize) -> bool {
    // Only unique ordered pairs (ai < bi handled by IncrementalBiConstraint)
    // Filter: match when slots are equal (collision)
    // Note: the bi-constraint groups by key; pairs in the same group all collide.
    // We return true to score ALL pairs in the same slot group.
    // (The key_extractor already groups by slot — every pair in the same group fires.)
    let _ = (ai, bi);
    true
}

fn collision_weight(_s: &Orrery, _ai: usize, _bi: usize) -> HardSoftScore {
    HardSoftScore::ONE_HARD
}

fn harmonic_key(_s: &Orrery, _p: &Planet, _idx: usize) -> u8 {
    // All planets in one group → every pair is evaluated
    0
}

fn harmonic_filter(_s: &Orrery, _a: &Planet, _b: &Planet, _ai: usize, _bi: usize) -> bool {
    true
}

fn harmonic_weight(s: &Orrery, ai: usize, bi: usize) -> HardSoftScore {
    let r_a = ORBITAL_RADII[s.planets[ai].slot];
    let r_b = ORBITAL_RADII[s.planets[bi].slot];
    let (penalty, _, _) = harmonic_deviation(r_a, r_b);
    HardSoftScore::of_soft(penalty)
}

// ── Constructor ───────────────────────────────────────────────

/// Build the orrery constraint tuple.
/// Returns (NoCollision, HarmonicRatio) which implements ConstraintSet.
pub fn build_constraints() -> OrreryConstraints {
    let no_collision: NoCollision = IncrementalBiConstraint::new(
        ConstraintRef::new("serio_01", "No collision"),
        ImpactType::Penalty,
        extract_planets as fn(&Orrery) -> &[Planet],
        collision_key as fn(&Orrery, &Planet, usize) -> usize,
        collision_filter as fn(&Orrery, &Planet, &Planet, usize, usize) -> bool,
        collision_weight as fn(&Orrery, usize, usize) -> HardSoftScore,
        true, // is_hard
    );

    let harmonic_ratio: HarmonicRatio = IncrementalBiConstraint::new(
        ConstraintRef::new("serio_01", "Harmonic ratio"),
        ImpactType::Penalty,
        extract_planets as fn(&Orrery) -> &[Planet],
        harmonic_key as fn(&Orrery, &Planet, usize) -> u8,
        harmonic_filter as fn(&Orrery, &Planet, &Planet, usize, usize) -> bool,
        harmonic_weight as fn(&Orrery, usize, usize) -> HardSoftScore,
        false, // is_soft
    );

    (no_collision, harmonic_ratio)
}
