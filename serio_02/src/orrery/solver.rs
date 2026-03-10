// ═══════════════════════════════════════════════════════════════
// SOLVER -- Seeded local search with incremental SERIO scoring
//
// Each move retunes one planet's voice (frequency index).
// The SERIO engine retracts only that planet's 6 pairwise arcs,
// applies the mutation, then inserts the 6 new arc scores.
// The other 15 arcs are untouched.
//
// Fixed seed -> deterministic visual output.
// Late-acceptance hill climbing for steady convergence.
// ═══════════════════════════════════════════════════════════════

use std::time::Instant;

use solverforge_core::score::HardSoftScore;
use solverforge_core::PlanningSolution;
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
    /// New frequency index (after the move, if accepted)
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

    /// Human-readable old frequency name (e.g. "D3")
    pub fn old_freq_name(&self) -> &'static str {
        freq_name(self.old_freq)
    }

    /// Human-readable new frequency name
    pub fn new_freq_name(&self) -> &'static str {
        freq_name(self.new_freq)
    }
}

/// Note names for the 48-pitch microtonal palette.
/// Even indices = equal-tempered semitones, odd = quarter-tones/traps.
const NOTE_NAMES: [&str; 48] = [
    "A2", "A2+", "A#2", "~B2", "B2", "B2+", // 0-5
    "C3", "C3+", "C#3", "C#3+", "D3", "~D#3", // 6-11
    "D#3", "D#3+", "E3", "E3p", "F3", "F3+", // 12-17
    "F#3", "F#3+", "G3", "~G#3", "G#3", "G#3+", // 18-23
    "A3", "A3+", "A#3", "~B3", "B3", "B3+", // 24-29
    "C4", "C4+", "C#4", "C#4+", "D4", "~D#4", // 30-35
    "D#4", "D#4+", "E4", "E4p", "F4", "F4+", // 36-41
    "F#4", "F#4+", "G4", "~G#4", "G#4", "G#4+", // 42-47
];

fn freq_name(idx: usize) -> &'static str {
    if idx < NOTE_NAMES.len() {
        NOTE_NAMES[idx]
    } else {
        "?"
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
    pub total_moves: u64,
    pub accepted_moves: u64,

    /// Moves completed in the last step/step_timed call (for per-frame reporting)
    pub last_step_moves: u64,

    /// Cumulative wall-clock time spent inside step_timed (seconds)
    solver_elapsed_secs: f64,

    rng: Rng,
    late_queue: Vec<HardSoftScore>,
    late_head: usize,
}

const LATE_SIZE: usize = 400;

/// Maximum history entries kept (ring buffer style — we only need the recent ones for viz)
const MAX_HISTORY: usize = 2048;

impl SolverState {
    pub fn new(orrery: Orrery) -> Self {
        let constraints = build_constraints();
        let mut director = TypedScoreDirector::new(orrery, constraints);
        let initial_score = director.calculate_score();

        let late_queue = vec![initial_score; LATE_SIZE];

        SolverState {
            director,
            history: Vec::with_capacity(MAX_HISTORY),
            best_score: initial_score,
            total_moves: 0,
            accepted_moves: 0,
            last_step_moves: 0,
            solver_elapsed_secs: 0.0,
            rng: Rng::new(0x6A09E667F3BCC908),
            late_queue,
            late_head: 0,
        }
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

        let score_before = self
            .director
            .working_solution()
            .score()
            .unwrap_or(HardSoftScore::ZERO);

        // SERIO incremental move: retract -> mutate -> insert
        let score_after = self.director.do_change(0, planet_idx, |orrery| {
            orrery.planets[planet_idx].freq_idx = new_freq;
        });

        self.total_moves += 1;

        // Late acceptance
        let late_ref = self.late_queue[self.late_head];
        let accepted = score_after >= late_ref || score_after >= score_before;

        if !accepted {
            // Reject: revert
            self.director.do_change(0, planet_idx, |orrery| {
                orrery.planets[planet_idx].freq_idx = old_freq;
            });
        } else {
            self.accepted_moves += 1;
            if score_after > self.best_score {
                self.best_score = score_after;
            }
            self.late_queue[self.late_head] = score_after;
            self.late_head = (self.late_head + 1) % LATE_SIZE;
        }

        // Keep history bounded
        if self.history.len() >= MAX_HISTORY {
            self.history.drain(0..MAX_HISTORY / 2);
        }

        self.history.push(SolverMove {
            planet_idx,
            old_freq,
            new_freq: if accepted { new_freq } else { old_freq },
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

    /// Get the last accepted move (for commentary display)
    pub fn last_accepted_move(&self) -> Option<&SolverMove> {
        self.history.iter().rev().find(|m| m.accepted)
    }

    /// Current frequency assignments as human-readable strings
    pub fn freq_assignment_summary(&self) -> Vec<(&'static str, &'static str, f64)> {
        self.director
            .working_solution()
            .planets
            .iter()
            .map(|p| (p.name, freq_name(p.freq_idx), FREQUENCIES[p.freq_idx]))
            .collect()
    }
}
