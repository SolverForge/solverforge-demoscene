// ═══════════════════════════════════════════════════════════════
// SOLVER -- Seeded local search with incremental SERIO scoring
//
// Each move retunes one planet's voice (frequency index).
// The SERIO engine retracts only that planet's 6 pairwise arcs,
// applies the mutation, then inserts the 6 new arc scores.
// The other 15 arcs are untouched.
//
// Fixed seed -> deterministic visual output.
// Hill climbing: a change is kept when it does not make the score worse.
// ═══════════════════════════════════════════════════════════════

use std::time::Instant;

use solverforge_core::score::HardSoftScore;
use solverforge_scoring::TypedScoreDirector;

use crate::orrery::constraints::{build_constraints, OrreryConstraints};
use crate::orrery::model::{Orrery, FREQUENCIES, FREQ_COUNT, PLANET_COUNT};

/// A single solver move with visualization metadata.
#[derive(Clone, Debug)]
pub struct SolverMove {
    /// Which planet was retuned (index into Orrery::planets)
    pub planet_idx: usize,
    /// Old frequency index (before the move)
    pub old_freq: usize,
    /// The frequency index the move tried (kept only if `accepted`)
    pub new_freq: usize,
    /// Score before the move
    pub score_before: HardSoftScore,
    /// Score after the move
    pub score_after: HardSoftScore,
    /// Whether the move was accepted
    pub accepted: bool,
}

impl SolverMove {
    pub fn soft_delta(&self) -> i64 {
        self.score_after.soft() - self.score_before.soft()
    }

    /// The note before the move, in Hz.
    pub fn old_hz(&self) -> f64 {
        FREQUENCIES[self.old_freq]
    }

    /// The note the move tried, in Hz.
    pub fn tried_hz(&self) -> f64 {
        FREQUENCIES[self.new_freq]
    }
}

/// Deterministic PRNG -- Xorshift64 with a fixed seed.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1)
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

/// The live solver state.
pub struct SolverState {
    pub director: TypedScoreDirector<Orrery, OrreryConstraints>,
    pub history: Vec<SolverMove>,
    pub best_score: HardSoftScore,
    /// The voices of the best tuning found so far.
    best_voices: Vec<usize>,
    pub total_moves: u64,
    pub accepted_moves: u64,

    /// Moves completed in the last step/step_timed call (for per-frame reporting)
    pub last_step_moves: u64,

    /// Cumulative wall-clock time spent inside step_timed (seconds)
    solver_elapsed_secs: f64,

    rng: Rng,
}

/// Maximum history entries kept (ring buffer style — we only need the recent ones for viz)
const MAX_HISTORY: usize = 2048;

impl SolverState {
    pub fn new(orrery: Orrery) -> Self {
        let constraints = build_constraints();
        let mut director = TypedScoreDirector::new(orrery, constraints);
        let initial_score = director.calculate_score();

        let best_voices = director_voices(&director);

        SolverState {
            director,
            history: Vec::with_capacity(MAX_HISTORY),
            best_score: initial_score,
            best_voices,
            total_moves: 0,
            accepted_moves: 0,
            last_step_moves: 0,
            solver_elapsed_secs: 0.0,
            rng: Rng::new(0x6A09E667F3BCC908),
        }
    }

    /// Run exactly `n` moves (deterministic: the slow-motion part of the search).
    pub fn step(&mut self, n: u64) {
        for _ in 0..n {
            self.do_one_move();
        }
        self.last_step_moves = n;
    }

    /// Run moves for up to `budget_ms` milliseconds of wall-clock time.
    /// Returns the number of moves completed.
    pub fn step_timed(&mut self, budget_ms: f64) -> u64 {
        let start = Instant::now();
        let budget = std::time::Duration::from_secs_f64(budget_ms / 1000.0);
        let mut count: u64 = 0;

        // Check time every 64 moves to avoid syscall overhead
        loop {
            for _ in 0..64 {
                self.do_one_move();
                count += 1;
            }
            if start.elapsed() >= budget {
                break;
            }
        }

        self.solver_elapsed_secs += start.elapsed().as_secs_f64();
        self.last_step_moves = count;
        count
    }

    fn do_one_move(&mut self) {
        // Choose a random planet and a random new frequency
        let planet_idx = self.rng.usize_range(PLANET_COUNT);
        let old_freq = self.director.working_solution().planets[planet_idx].freq_idx;

        // Pick a different frequency not already used by another planet
        let mut new_freq = self.rng.usize_range(FREQ_COUNT);
        for _ in 0..12 {
            if new_freq != old_freq && !self.freq_occupied(new_freq, planet_idx) {
                break;
            }
            new_freq = self.rng.usize_range(FREQ_COUNT);
        }
        if new_freq == old_freq {
            return;
        }

        let score_before = self.director.get_score();

        // SERIO incremental move: retract -> mutate -> insert
        let score_after = self.director.do_change(0, planet_idx, |orrery| {
            orrery.planets[planet_idx].freq_idx = new_freq;
        });

        self.total_moves += 1;

        // Keep the change unless it makes the score worse
        let accepted = score_after >= score_before;

        if !accepted {
            // Reject: revert
            self.director.do_change(0, planet_idx, |orrery| {
                orrery.planets[planet_idx].freq_idx = old_freq;
            });
        } else {
            self.accepted_moves += 1;
            if score_after > self.best_score {
                self.best_score = score_after;
                self.best_voices = director_voices(&self.director);
            }
        }

        // Keep history bounded
        if self.history.len() >= MAX_HISTORY {
            self.history.drain(0..MAX_HISTORY / 2);
        }

        self.history.push(SolverMove {
            planet_idx,
            old_freq,
            new_freq,
            score_before,
            score_after: if accepted { score_after } else { score_before },
            accepted,
        });
    }

    fn freq_occupied(&self, freq_idx: usize, exclude_idx: usize) -> bool {
        self.director
            .working_solution()
            .planets
            .iter()
            .enumerate()
            .any(|(i, p)| i != exclude_idx && p.freq_idx == freq_idx)
    }

    /// Put the best tuning found back on the orrery, one scored change per
    /// planet that differs.
    pub fn restore_best(&mut self) {
        for (idx, &voice) in self.best_voices.clone().iter().enumerate() {
            if self.director.working_solution().planets[idx].freq_idx != voice {
                self.director.do_change(0, idx, |orrery| {
                    orrery.planets[idx].freq_idx = voice;
                });
            }
        }
    }

    pub fn current_score(&self) -> HardSoftScore {
        self.director.get_score()
    }

    pub fn solution(&self) -> &Orrery {
        self.director.working_solution()
    }

    pub fn moves_per_sec(&self) -> f64 {
        if self.solver_elapsed_secs > 0.0 {
            self.total_moves as f64 / self.solver_elapsed_secs
        } else {
            0.0
        }
    }
}

fn director_voices(director: &TypedScoreDirector<Orrery, OrreryConstraints>) -> Vec<usize> {
    director
        .working_solution()
        .planets
        .iter()
        .map(|p| p.freq_idx)
        .collect()
}
