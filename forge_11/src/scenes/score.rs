// The score panel shared by SOLVE and REPAIR: the current score in large
// cells, the feasibility state, the solver's own clock, and the trajectory
// drawn as it replays. Every value is a trajectory point from the capture.
use super::{AMBER, DIM, LIME, PAPER, RED};
use crate::capture::{num, Point};
use crate::fx;
use crate::gfx::{mix, size, Frame, S1, S2};
use crate::script::Replay;

pub const PANEL: (i32, i32, i32, i32) = (40, 220, 720, 970);

/// Reads one score level from a trajectory point.
type Level = fn(&Point) -> Option<f64>;

pub struct Chart {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

/// The score text, its feasibility and the moment it last changed.
pub fn score_at<'a>(traj: &'a [Point], replay: &Replay, t: f32) -> Option<(&'a Point, f32)> {
    let i = replay.index_at(traj, t)?;
    let p = &traj[i];
    let since = t - replay.at(p.t_ms);
    Some((p, since))
}

/// Draw the score block. `traj` replays on `replay`; before it starts the
/// panel shows `idle` (the previous run's final score) if given.
pub fn panel(
    f: &mut Frame,
    traj: &[Point],
    replay: &Replay,
    idle: Option<&Point>,
    t: f32,
    a: f32,
    accent: u32,
) {
    let (x0, y0, x1, y1) = PANEL;
    fx::panel(f, x0, y0, x1, y1, accent, a);
    if a < 0.5 {
        return;
    }
    f.text(x0 + 24, y0 + 22, "SCORE", S1, accent, 1.0);
    let shown = score_at(traj, replay, t).or(idle.map(|p| (p, 9.0)));
    let Some((p, since)) = shown else {
        f.text(x0 + 24, y0 + 60, "waiting for the solver", S2, DIM, 0.8);
        return;
    };
    let label = p.score.label();
    let feasible = p.score.feasible();
    // Large cells, as wide as the label allows.
    let cw = ((x1 - x0 - 48) / label.len().max(1) as i32).clamp(16, 44);
    let big = size(cw, cw * 9 / 5);
    let hit = (-since.max(0.0) * 7.0).exp();
    let base = if feasible { LIME } else { RED };
    let col = mix(base, 0xffffff, 0.6 * hit);
    f.text_shadow(x0 + 24, y0 + 56, &label, big, col, 1.0);
    let state = if feasible { "FEASIBLE" } else { "INFEASIBLE" };
    f.text(x0 + 24, y0 + 72 + big.h, state, S2, base, 1.0);

    let ms = replay.ms_at(t).min(replay.t_max);
    let clock = if t < replay.from {
        format!("solver time      0 ms of {} ms", num(replay.t_max.round()))
    } else {
        format!(
            "solver time {:>6} ms of {} ms",
            num(ms.round()),
            num(replay.t_max.round())
        )
    };
    f.text(x0 + 24, y0 + 124 + big.h, &clock, S1, PAPER, 0.85);
}

/// Where one level of the score label sits in the panel (x0, x1, y, h):
/// "hard" lights the part before the slash, "soft" the part after it.
pub fn label_span(
    traj: &[Point],
    replay: &Replay,
    idle: Option<&Point>,
    t: f32,
    part: &str,
) -> Option<(f32, f32, f32, f32)> {
    let (x0, y0, x1, _) = PANEL;
    let (p, _) = score_at(traj, replay, t).or(idle.map(|p| (p, 9.0)))?;
    let label = p.score.label();
    let cw = ((x1 - x0 - 48) / label.len().max(1) as i32).clamp(16, 44);
    let h = cw * 9 / 5;
    let slash = label.find('/').unwrap_or(label.len());
    let (a, b) = if part == "hard" {
        (0, slash)
    } else {
        ((slash + 1).min(label.len()), label.len())
    };
    let lx = (x0 + 24) as f32;
    Some((
        lx + (a as i32 * cw) as f32,
        lx + (b as i32 * cw) as f32,
        (y0 + 56) as f32,
        h as f32,
    ))
}

/// A light running along one level of the score, once a beat, while the
/// line about it is read.
pub fn run_light(f: &mut Frame, span: (f32, f32, f32, f32), rgb: u32, t: f32, k: f32) {
    if k <= 0.0 {
        return;
    }
    let (x0, x1, y, h) = span;
    let u = (t / crate::timeline::BEAT).fract();
    let x = x0 + (x1 - x0) * u;
    f.glow(x, y + h * 0.5, h * 1.4, rgb, 0.5 * k);
    f.rect(
        x0 as i32,
        (y + h + 6.0) as i32,
        x as i32,
        (y + h + 10.0) as i32,
        rgb,
        0.9 * k,
    );
    let pulse = crate::timeline::beat_pulse(t, 5.0);
    f.glow(
        (x0 + x1) * 0.5,
        y + h * 0.5,
        (x1 - x0) * 0.6,
        rgb,
        0.2 * k * pulse,
    );
}

/// The trajectory so far: hard in red turning lime at zero, soft in amber,
/// each level normalised to its own range, on the replay's log clock.
pub fn chart(f: &mut Frame, traj: &[Point], replay: &Replay, t: f32, c: &Chart, a: f32) {
    if a <= 0.0 {
        return;
    }
    f.rect(c.x0, c.y1, c.x1, c.y1 + 1, DIM, 0.6 * a);
    f.rect(c.x0, c.y0, c.x0 + 1, c.y1, DIM, 0.6 * a);
    f.text(c.x1 - 8 * S1.w, c.y1 + 8, "log time", S1, DIM, 0.8 * a);
    let Some(last) = replay.index_at(traj, t) else {
        return;
    };
    let head = replay.head(t);
    let levels: [(Level, u32, &str); 2] = [
        (|p| p.score.hard, RED, "hard"),
        (|p| p.score.soft, AMBER, "soft"),
    ];
    let mut legend_x = c.x0 + 10;
    for (get, rgb, name) in levels {
        let vals: Vec<f64> = traj.iter().filter_map(get).collect();
        if vals.is_empty() {
            continue;
        }
        let lo = vals
            .iter()
            .cloned()
            .fold(f64::MAX, f64::min)
            .min(0.0_f64.min(vals[0]));
        let hi = vals.iter().cloned().fold(f64::MIN, f64::max);
        let span = (hi - lo).max(1e-9);
        let y_of =
            |v: f64| c.y1 as f32 - 8.0 - ((v - lo) / span) as f32 * (c.y1 - c.y0 - 24) as f32;
        let x_of = |ms: f64| c.x0 as f32 + replay.tau(ms) * (c.x1 - c.x0) as f32;
        let mut prev: Option<(f32, f32)> = None;
        for p in traj.iter().take(last + 1) {
            let Some(v) = get(p) else { continue };
            let (x, y) = (x_of(p.t_ms), y_of(v));
            let col = if name == "hard" && v >= 0.0 {
                LIME
            } else {
                rgb
            };
            if let Some((px, py)) = prev {
                f.line((px, py), (x, py), 2.0, col, a);
                f.line((x, py), (x, y), 2.0, col, a);
            }
            prev = Some((x, y));
        }
        if let Some((px, py)) = prev {
            let hx = c.x0 as f32 + head * (c.x1 - c.x0) as f32;
            f.line((px, py), (hx, py), 2.0, rgb, a * 0.8);
            f.glow(hx, py, 14.0, rgb, 0.8 * a);
        }
        f.text(legend_x, c.y0 - 4, name, S1, rgb, a);
        legend_x += 7 * S1.w;
    }
}
