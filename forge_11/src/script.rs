// The schedule: when every piece of the capture appears on the demo clock.
// Picture and soundtrack both read it, so a typed word clicks, a code
// excerpt lands on the anvil and feasibility hits the crash on the frame it
// is drawn. Section windows come from timeline.rs; the capture decides
// everything inside them.
use crate::capture::{Capture, Point, Solve};
use crate::gfx::{size, wrap, Size};
use crate::timeline::{snap_beat, span, Section, BAR, BEAT, STEP};

/// The prompt's terminal: text area in pixels.
pub const PROMPT_BOX: (i32, i32, i32, i32) = (200, 336, 1720, 860);

pub struct Typing {
    /// (time, first display char, end display char) per word.
    pub words: Vec<(f32, usize, usize)>,
    pub dur: f32,
}

impl Typing {
    /// Display characters visible at t.
    pub fn chars_at(&self, t: f32) -> usize {
        let mut n = 0;
        for &(at, a, b) in &self.words {
            if t < at {
                break;
            }
            let k = ((t - at) / self.dur).clamp(0.0, 1.0);
            n = a + ((b - a) as f32 * k).ceil() as usize;
        }
        n
    }
}

/// Maps solver wall time onto a window of the demo clock. Solvers improve
/// fastest at the start, so the replay runs on a log clock; the on-screen
/// timer always shows the real milliseconds, counted from `origin` (the
/// solve's start, or the first point after a change was pushed in).
#[derive(Clone, Copy, Debug)]
pub struct Replay {
    pub from: f32,
    pub len: f32,
    pub t_max: f64,
    origin: f64,
    t0: f64,
}

impl Replay {
    fn new(solve: &Solve, from: f32, len: f32, origin: f64) -> Self {
        let last = solve.trajectory.iter().map(|p| p.t_ms).fold(0.0, f64::max) - origin;
        let t_max = solve.wall_ms.max(last).max(1.0);
        Self {
            from,
            len,
            t_max,
            origin,
            t0: (t_max / 200.0).max(0.5),
        }
    }

    pub fn tau(&self, t_ms: f64) -> f32 {
        let rel = (t_ms - self.origin).max(0.0);
        ((1.0 + rel / self.t0).ln() / (1.0 + self.t_max / self.t0).ln()) as f32
    }

    pub fn at(&self, t_ms: f64) -> f32 {
        self.from + self.tau(t_ms) * self.len
    }

    /// Real solver milliseconds since the origin at demo time t (clamped
    /// to the run).
    pub fn ms_at(&self, t: f32) -> f64 {
        let tau = ((t - self.from) / self.len).clamp(0.0, 1.0) as f64;
        self.t0 * ((1.0 + self.t_max / self.t0).powf(tau) - 1.0)
    }

    /// Index of the latest trajectory point shown at t (None before the run).
    pub fn index_at(&self, traj: &[Point], t: f32) -> Option<usize> {
        if t < self.from {
            return None;
        }
        let ms = self.ms_at(t);
        traj.iter()
            .rposition(|p| p.t_ms - self.origin <= ms + 1e-9)
            .or(Some(0))
    }

    /// Stretch the window so that `t_ms` lands on a beat, within limits.
    fn snap(mut self, t_ms: f64, max_len: f32) -> Self {
        let tau = self.tau(t_ms);
        if tau < 0.05 {
            return self;
        }
        let mut target = snap_beat(self.from + tau * self.len);
        if (target - self.from) / tau > max_len {
            target = ((self.from + tau * max_len) / BEAT).floor() * BEAT;
        }
        if target > self.from + 0.25 {
            self.len = (target - self.from) / tau;
        }
        self
    }
}

pub struct Script {
    pub prompt_lines: Vec<String>,
    pub prompt_size: Size,
    pub typing: Typing,
    /// One time per agent step.
    pub steps: Vec<f32>,
    /// (start, end) per code excerpt; the excerpt lands on `start`.
    pub code: Vec<(f32, f32)>,
    pub solve: Replay,
    pub solve_feasible: Option<f32>,
    /// Drop time per block of the solved plan, in `board order`.
    pub drops: Vec<f32>,
    pub event: f32,
    pub repair: Replay,
    pub repair_feasible: Option<f32>,
    /// Start of each changed job's move.
    pub moves: Vec<f32>,
    /// When each change is reviewed after the heal (fewer than `moves` if
    /// the section runs out), and for how long.
    pub reviews: Vec<f32>,
    pub review_len: f32,
    /// Close lines' landings.
    pub close: [f32; 3],
    /// (time, 0..1 progress) for every score improvement, both runs.
    pub improvements: Vec<(f32, f32)>,
}

/// The prompt's cell size: the largest that fits the terminal, else a
/// still-readable size that scrolls.
pub fn prompt_layout(prompt: &str) -> (Size, Vec<String>) {
    let (x0, y0, x1, y1) = PROMPT_BOX;
    let (w, h) = (x1 - x0, y1 - y0);
    for s in [size(28, 50), size(24, 44), size(20, 36), size(16, 30)] {
        let cols = (w / s.w) as usize - 2;
        let lines = wrap(prompt, cols);
        if lines.len() as i32 * (s.h + 4) <= h {
            return (s, lines);
        }
    }
    // Longer prompts stay readable: the terminal scrolls as they type.
    let s = size(16, 30);
    (s, wrap(prompt, (w / s.w) as usize - 2))
}

impl Script {
    pub fn new(c: &Capture) -> Self {
        // DESCRIBE: words type on the sixteenth-note grid.
        let (d0, d1) = span(Section::Describe);
        let (prompt_size, prompt_lines) = prompt_layout(&c.prompt);
        let mut tokens = Vec::new();
        let mut idx = 0;
        for line in &prompt_lines {
            let mut start = None;
            for (i, ch) in line.chars().enumerate() {
                match (ch == ' ', start) {
                    (false, None) => start = Some(i),
                    (true, Some(s)) => {
                        tokens.push((idx + s, idx + i + 1));
                        start = None;
                    }
                    _ => {}
                }
            }
            let n = line.chars().count();
            if let Some(s) = start {
                tokens.push((idx + s, idx + n));
            }
            idx += n;
        }
        let (ty0, ty1) = (d0 + 1.0, d1 - 3.0);
        let slots = ((ty1 - ty0) / STEP) as usize;
        let per = slots as f32 / tokens.len().max(1) as f32;
        let words = tokens
            .iter()
            .enumerate()
            .map(|(i, &(a, b))| {
                let at = ty0 + (i as f32 * per).floor() * STEP;
                // Each word starts where the previous one ended.
                let a = if i == 0 { 0 } else { a.min(tokens[i - 1].1) };
                (at, a, b)
            })
            .collect();
        let typing = Typing {
            words,
            dur: (per * STEP).clamp(0.03, 0.2),
        };

        // GENERATE: agent steps in transcript order, code on the beat.
        let (g0, g1) = span(Section::Generate);
        let n = c.steps.len();
        let t_last = c.steps.iter().map(|s| s.t_ms).fold(0.0, f64::max).max(1.0);
        let steps = c
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let real = (s.t_ms / t_last) as f32;
                let even = if n > 1 {
                    i as f32 / (n - 1) as f32
                } else {
                    0.0
                };
                g0 + 0.4 + (0.5 * real + 0.5 * even) * (g1 - g0 - 2.4)
            })
            .collect();
        // Half-bar slots shared by reading weight (lines + a base), at least
        // a bar each; the spare halves go to the largest remainders. With
        // more excerpts than bars, one beat each.
        let k = c.code.len().max(1);
        let half = BAR / 2.0;
        let units = ((g1 - g0) / half).round() as usize;
        let lens: Vec<usize> = if 2 * k <= units {
            let w: Vec<f32> = c.code.iter().map(|x| x.lines.len() as f32 + 4.0).collect();
            let total: f32 = w.iter().sum();
            let share: Vec<f32> = w.iter().map(|v| v / total * units as f32).collect();
            let mut lens: Vec<usize> = share.iter().map(|s| (*s as usize).max(2)).collect();
            while lens.iter().sum::<usize>() > units {
                let i = (0..k)
                    .filter(|&i| lens[i] > 2)
                    .max_by_key(|&i| lens[i])
                    .unwrap();
                lens[i] -= 1;
            }
            while lens.iter().sum::<usize>() < units {
                let i = (0..k)
                    .max_by(|&a, &b| {
                        (share[a] - lens[a] as f32).total_cmp(&(share[b] - lens[b] as f32))
                    })
                    .unwrap();
                lens[i] += 1;
            }
            lens
        } else {
            vec![1; k]
        };
        let unit = if 2 * k <= units { half } else { BEAT };
        let mut at = g0;
        let code = lens
            .iter()
            .map(|&n| {
                let slot = (at, (at + n as f32 * unit).min(g1));
                at += n as f32 * unit;
                slot
            })
            .collect();

        // SOLVE: the run replays from two seconds in, feasibility on a beat.
        let (s0, s1) = span(Section::Solve);
        let mut solve = Replay::new(&c.solve, s0 + 2.0, 8.0, 0.0);
        let first_ok = |s: &Solve| {
            s.trajectory
                .iter()
                .find(|p| p.score.feasible())
                .map(|p| p.t_ms)
        };
        if let Some(ms) = first_ok(&c.solve) {
            solve = solve.snap(ms, s1 - s0 - 4.0);
        }
        let solve_feasible = first_ok(&c.solve).map(|ms| solve.at(ms));
        let drop_end = solve_feasible.unwrap_or(solve.from + solve.len * 0.85);
        let drops = spread(c.solve.plan.len(), solve.from + 0.2, drop_end - 0.15, STEP);

        // REPAIR: the event strikes on the second bar; the change replays.
        let (r0, r1) = span(Section::Repair);
        let event = r0 + BAR;
        // The repair's clock starts at its first point: the change went in.
        let origin = c
            .repair
            .solve
            .trajectory
            .iter()
            .map(|p| p.t_ms)
            .fold(f64::MAX, f64::min);
        let mut repair = Replay::new(&c.repair.solve, r0 + 2.0 * BAR, 8.0, origin);
        if let Some(ms) = first_ok(&c.repair.solve) {
            repair = repair.snap(ms, r1 - r0 - 12.0);
        }
        let repair_feasible = first_ok(&c.repair.solve).map(|ms| repair.at(ms));
        let move_end = repair_feasible.unwrap_or(repair.from + repair.len * 0.85);
        let moves = spread(
            c.repair.changed.len(),
            repair.from,
            move_end - 0.5,
            BEAT / 2.0,
        );
        // Then each change is reviewed in turn, old slot to new, on the beat.
        let n_rev = c.repair.changed.len();
        let rev0 = ((move_end + BEAT) / BEAT).ceil() * BEAT;
        let rev_step = if n_rev > 0 {
            (((r1 - 1.0 - rev0) / n_rev as f32 / BEAT).floor() * BEAT).clamp(BEAT, BAR)
        } else {
            BAR
        };
        let reviews = (0..n_rev)
            .map(|k| rev0 + k as f32 * rev_step)
            .filter(|&t| t < r1 - 0.5)
            .collect();

        let (c0, _) = span(Section::Close);
        let close = [c0, c0 + BAR, c0 + 2.0 * BAR];

        let mut improvements = progress(&c.solve.trajectory, &solve);
        improvements.extend(progress(&c.repair.solve.trajectory, &repair));

        Self {
            prompt_lines,
            prompt_size,
            typing,
            steps,
            code,
            solve,
            solve_feasible,
            drops,
            event,
            repair,
            repair_feasible,
            moves,
            reviews,
            review_len: rev_step,
            close,
            improvements,
        }
    }
}

/// `n` times from `a` to `b`, each on a `grid` step after `a`.
fn spread(n: usize, a: f32, b: f32, grid: f32) -> Vec<f32> {
    let slots = (((b - a) / grid).floor() as usize).max(1);
    (0..n)
        .map(|i| {
            let k = if n > 1 {
                (i as f32 * slots as f32 / (n - 1) as f32).round()
            } else {
                0.0
            };
            a + k.min(slots as f32) * grid
        })
        .collect()
}

/// Score-improvement events and their progress towards the best score.
fn progress(traj: &[Point], replay: &Replay) -> Vec<(f32, f32)> {
    let hard_min = traj.iter().filter_map(|p| p.score.hard).fold(0.0, f64::min);
    let soft: Vec<f64> = traj.iter().filter_map(|p| p.score.soft).collect();
    let (s_lo, s_hi) = soft
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let mut out = Vec::new();
    for (i, p) in traj.iter().enumerate() {
        if i > 0 && p.score == traj[i - 1].score {
            continue;
        }
        let ph = match p.score.hard {
            Some(h) if hard_min < 0.0 => 1.0 - h / hard_min,
            _ => 1.0,
        };
        let ps = match p.score.soft {
            Some(s) if s_hi > s_lo => (s - s_lo) / (s_hi - s_lo),
            _ => 1.0,
        };
        out.push((
            replay.at(p.t_ms),
            (0.7 * ph + 0.3 * ps).clamp(0.0, 1.0) as f32,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture() -> Capture {
        Capture::parse(include_str!("../assets/capture.placeholder.json")).unwrap()
    }

    /// The placeholder, and the real capture once it has been copied in.
    fn all() -> Vec<Capture> {
        let mut v = vec![capture()];
        let real = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");
        if std::path::Path::new(real).exists() {
            v.push(Capture::load(real).unwrap());
        }
        v
    }

    #[test]
    fn typing_finishes_inside_describe() {
        for c in all() {
            let s = Script::new(&c);
            let total: usize = s.prompt_lines.iter().map(|l| l.chars().count()).sum();
            let (_, d1) = span(Section::Describe);
            assert_eq!(s.typing.chars_at(d1 - 2.5), total);
            assert_eq!(s.typing.chars_at(0.0), 0);
        }
    }

    #[test]
    fn events_stay_in_their_sections() {
        for c in all() {
            let s = Script::new(&c);
            let inside = |t: f32, sec: Section| {
                let (a, b) = span(sec);
                assert!(t >= a && t < b, "{t} outside {sec:?}");
            };
            for &t in &s.steps {
                inside(t, Section::Generate);
            }
            for &(a, b) in &s.code {
                inside(a, Section::Generate);
                assert!(b > a + 0.9, "an excerpt gets at least a bar-ish slot");
            }
            for &t in &s.drops {
                inside(t, Section::Solve);
            }
            inside(s.solve_feasible.unwrap(), Section::Solve);
            inside(s.event, Section::Repair);
            for &t in s.moves.iter().chain(&s.reviews) {
                inside(t, Section::Repair);
            }
            inside(s.repair_feasible.unwrap(), Section::Repair);
            let (_, s1) = span(Section::Solve);
            assert!(s.solve.from + s.solve.len <= s1);
            let (_, r1) = span(Section::Repair);
            assert!(s.repair.from + s.repair.len <= r1 - 4.0);
            for &t in &s.reviews {
                assert!(t + s.review_len <= r1);
            }
        }
    }

    #[test]
    fn feasibility_lands_on_a_beat() {
        for c in all() {
            let s = Script::new(&c);
            for t in [s.solve_feasible.unwrap(), s.repair_feasible.unwrap()] {
                let off = (t / BEAT - (t / BEAT).round()).abs();
                assert!(off < 1e-3, "{t} is off the beat");
            }
        }
    }

    #[test]
    fn replay_clock_round_trips() {
        let c = capture();
        let s = Script::new(&c);
        for ms in [0.0, 10.0, 1234.0, s.solve.t_max] {
            let t = s.solve.at(ms);
            assert!((s.solve.ms_at(t) - ms).abs() < 1e-3 * ms.max(1.0));
        }
    }

    #[test]
    fn prompt_fits_its_terminal() {
        let c = capture();
        let (sz, lines) = prompt_layout(&c.prompt);
        let (x0, y0, x1, y1) = PROMPT_BOX;
        assert!(lines.len() as i32 * (sz.h + 4) <= y1 - y0);
        for l in &lines {
            assert!(l.chars().count() as i32 * sz.w <= x1 - x0);
        }
        // A prompt four times longer scrolls at a readable size, inside
        // the terminal's width.
        let long = c.prompt.repeat(4);
        let (sz, lines) = prompt_layout(&long);
        assert!(sz.w >= 16);
        for l in &lines {
            assert!(l.chars().count() as i32 * sz.w <= x1 - x0);
        }
    }
}
