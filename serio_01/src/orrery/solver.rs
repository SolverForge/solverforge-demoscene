// ═══════════════════════════════════════════════════════════════
// SOLVER -- Seeded local search with incremental SERIO scoring
//
// A simple hill-climbing solver with late-acceptance.
// Fixed seed → deterministic visual output.
//
// Each SolverMove records:
//   - which planet moved
//   - old and new slot
//   - the score delta from SERIO's retract/insert cycle
//   - which planet pairs had arcs affected (for visualization)
//
// The solver drives the visual — each frame it executes N moves
// and the renderer replays the recorded deltas.
// ═══════════════════════════════════════════════════════════════

use solverforge_core::score::HardSoftScore;
use solverforge_core::PlanningSolution;
use solverforge_scoring::TypedScoreDirector;

use crate::orrery::constraints::{build_constraints, OrreryConstraints};
use crate::orrery::model::{Orrery, PLANET_COUNT, SLOT_COUNT};

/// A single solver move with visualization metadata.
#[derive(Clone, Debug)]
pub struct SolverMove {
    /// Which planet was moved (index into Orrery::planets)
    pub planet_idx: usize,
    /// Score before the move
    pub score_before: HardSoftScore,
    /// Score after the move (= score_before + retract_delta + insert_delta)
    pub score_after: HardSoftScore,
    /// Whether the move was accepted
    pub accepted: bool,
}

impl SolverMove {
    pub fn soft_delta(&self) -> i64 {
        self.score_after.soft() - self.score_before.soft()
    }
}

/// Deterministic PRNG — no external dependency.
/// Xorshift64 with a fixed seed.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1) // avoid 0
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn usize_range(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// The live solver state. Holds the TypedScoreDirector and move history.
pub struct SolverState {
    pub director: TypedScoreDirector<Orrery, OrreryConstraints>,
    /// Every move attempted (accepted or rejected), in order.
    pub history: Vec<SolverMove>,
    /// Current best score
    pub best_score: HardSoftScore,
    /// Total moves evaluated
    pub total_moves: u64,
    /// Accepted moves
    pub accepted_moves: u64,

    rng: Rng,
    /// Late-acceptance queue length
    late_queue: Vec<HardSoftScore>,
    late_head: usize,
}

const LATE_SIZE: usize = 400;

impl SolverState {
    pub fn new(orrery: Orrery) -> Self {
        let constraints = build_constraints();
        let mut director = TypedScoreDirector::new(orrery, constraints);
        let initial_score = director.calculate_score();

        let late_queue = vec![initial_score; LATE_SIZE];

        SolverState {
            director,
            history: Vec::with_capacity(16384),
            best_score: initial_score,
            total_moves: 0,
            accepted_moves: 0,
            rng: Rng::new(0x6A09E667F3BCC908), // fixed seed — SHA-256 first constant
            late_queue,
            late_head: 0,
        }
    }

    /// Execute `n` solver moves. Returns a slice of the new history entries.
    /// `n` should be small (1-10) per frame for smooth animation.
    pub fn step(&mut self, n: usize) {
        for _ in 0..n {
            self.do_one_move();
        }
    }

    fn do_one_move(&mut self) {
        // Choose a random planet and a random new slot (different from current)
        let planet_idx = self.rng.usize_range(PLANET_COUNT);
        let old_slot = self.director.working_solution().planets[planet_idx].slot;

        // Pick a slot that isn't occupied by another planet (to reduce wasted moves)
        // and isn't the current slot
        let mut new_slot = self.rng.usize_range(SLOT_COUNT);
        // Try up to 8 times to find a non-occupied different slot
        for _ in 0..8 {
            if new_slot != old_slot && !self.slot_occupied(new_slot, planet_idx) {
                break;
            }
            new_slot = self.rng.usize_range(SLOT_COUNT);
        }
        if new_slot == old_slot {
            return;
        }

        let score_before = self
            .director
            .working_solution()
            .score()
            .unwrap_or(HardSoftScore::ZERO);

        // SERIO incremental move: retract → mutate → insert
        let score_after = self.director.do_change(0, planet_idx, |orrery| {
            orrery.planets[planet_idx].slot = new_slot;
            // Update angular speed for new orbital radius
            use crate::orrery::model::{ORBITAL_RADII, R_MAX};
            let r = ORBITAL_RADII[new_slot];
            orrery.planets[planet_idx].angular_speed = R_MAX.powf(1.5) / r.powf(1.5);
        });

        self.total_moves += 1;

        // Late acceptance: accept if score_after >= late_queue[head]
        let late_ref = self.late_queue[self.late_head];
        let accepted = score_after >= late_ref || score_after >= score_before;

        if !accepted {
            // Reject: revert the move
            self.director.do_change(0, planet_idx, |orrery| {
                orrery.planets[planet_idx].slot = old_slot;
                use crate::orrery::model::{ORBITAL_RADII, R_MAX};
                let r = ORBITAL_RADII[old_slot];
                orrery.planets[planet_idx].angular_speed = R_MAX.powf(1.5) / r.powf(1.5);
            });
        } else {
            self.accepted_moves += 1;
            if score_after > self.best_score {
                self.best_score = score_after;
            }
            // Update late-acceptance queue with accepted score
            self.late_queue[self.late_head] = score_after;
            self.late_head = (self.late_head + 1) % LATE_SIZE;
        }

        self.history.push(SolverMove {
            planet_idx,
            score_before,
            score_after: if accepted { score_after } else { score_before },
            accepted,
        });
    }

    /// Check if a slot is occupied by any planet other than `exclude_idx`.
    fn slot_occupied(&self, slot: usize, exclude_idx: usize) -> bool {
        self.director
            .working_solution()
            .planets
            .iter()
            .enumerate()
            .any(|(i, p)| i != exclude_idx && p.slot == slot)
    }

    /// Current score from the director
    pub fn current_score(&self) -> HardSoftScore {
        self.director
            .working_solution()
            .score()
            .unwrap_or(HardSoftScore::ZERO)
    }

    /// Get the solution for rendering
    pub fn solution(&self) -> &Orrery {
        self.director.working_solution()
    }

    /// Get moves/sec over the last 100 moves
    pub fn moves_per_sec(&self, elapsed_secs: f64) -> f64 {
        if elapsed_secs > 0.0 {
            self.total_moves as f64 / elapsed_secs
        } else {
            0.0
        }
    }
}
