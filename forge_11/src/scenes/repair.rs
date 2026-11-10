// 04 REPAIR: the plan is live and the solver is still running. The
// captured disruption strikes on the second bar, enters the running solve
// through the captured mechanism, and the plan heals in place: the changed
// jobs travel to their new slots, every other block stays exactly where it
// was, and the score climbs back. No restart.
use super::cordelia::{Cordelia, Targets};
use super::score::{self, Chart, PANEL};
use super::{typed, Timed, AMBER, DIM, EMERALD, LIME, MINT, PAPER, RED, RED_STOPS};
use crate::board::Board;
use crate::capture::Capture;
use crate::fx;
use crate::gfx::{clip, mix, size, smooth, wrap, Frame, Size, S1, S2, W};
use crate::script::Script;
use crate::timeline::Now;

pub struct Repair {
    event: Timed,
    size: Size,
    rows: i32,
    cordelia: Cordelia,
}

const MOVE: f32 = 0.6;

impl Repair {
    /// The event's sentence: one line at 2x if it fits, else up to two
    /// lines at a smaller size, else one clipped line at 1x.
    pub fn new(c: &Capture, s: &Script) -> Self {
        let width = W as i32 - 300;
        let label = &c.repair.label;
        let (sz, text) = if label.len() as i32 * S2.w <= width {
            (S2, label.clone())
        } else {
            let mid = size(16, 30);
            let lines = wrap(label, (width / mid.w) as usize);
            if lines.len() <= 2 {
                (mid, lines.join("\n"))
            } else {
                (S1, clip(label, (width / S1.w) as usize))
            }
        };
        let rows = text.lines().count() as i32;
        Self {
            event: Timed::new("unstable", &text, 1401, RED_STOPS, s.event, 0.7),
            size: sz,
            rows,
            cordelia: Cordelia::new(s),
        }
    }
}

pub fn draw(f: &mut Frame, c: &Capture, s: &Script, board: &Board, r: &Repair, t: f32, _n: Now) {
    let since = t - s.event;
    let alarm = if since >= 0.0 { 1.0 } else { 0.0 };
    f.gradient(&[
        0x010403,
        mix(0x03120c, 0x1a0306, alarm * 0.8),
        0x05100c,
        0x010302,
    ]);
    let bars: &[u32] = if since >= 0.0 {
        &[RED, 0xa32000, 0x7a0010]
    } else {
        &[EMERALD, LIME, 0x0ea5a4]
    };
    fx::copper(
        f,
        t,
        140.0,
        1060.0,
        bars,
        4,
        0.16 + 0.25 * (-since.max(0.0) * 1.5).exp() * alarm,
    );
    fx::river(f, t, 1000, 4, &[0x02140c, 0x0b5a3a, EMERALD, 0xd1fae5], 0.5);

    // Before the strike: the solver keeps running on the published plan.
    if since < 0.0 {
        let pulse = 0.6 + 0.4 * (t * 6.0).sin();
        f.text(150, 140, "*", S2, LIME, pulse);
        f.text(
            150 + 2 * S2.w,
            140,
            "THE PLAN IS LIVE. THE SOLVER KEEPS RUNNING.",
            S2,
            MINT,
            1.0,
        );
    } else {
        r.event.draw(f, t, 150, 124, r.size, 1.0);
        let my = 124 + r.rows * r.size.h + 2;
        // The mechanism, verbatim, wrapped over the full width (at most two
        // lines), typed in over a second; the event kind sits at the right
        // end of the last line.
        let cols = ((W as i32 - 150 - 60) / S1.w) as usize;
        let mech = format!("pushed into the running solve: {}", c.repair.mechanism);
        let mut lines = wrap_code(&mech, cols);
        if lines.len() > 2 {
            let tail = lines[1..].join(" ");
            lines.truncate(1);
            lines.push(clip(&tail, cols));
        }
        let k = ((since - 0.6) / 1.0).clamp(0.0, 1.0);
        let total: usize = lines.iter().map(|l| l.len()).sum();
        let mut left = (k * total as f32).ceil() as usize;
        for (i, l) in lines.iter().enumerate() {
            let n = left.min(l.len());
            left -= n;
            f.text(150, my + i as i32 * S1.h, &typed(l, n), S1, AMBER, 1.0);
        }
        let kind = clip(&c.repair.kind, 40);
        let last = lines.last().map_or(0, |l| l.len());
        let ky = if last + kind.len() + 3 <= cols {
            my + (lines.len() as i32 - 1) * S1.h
        } else {
            my - S1.h
        };
        f.text(
            W as i32 - 60 - kind.len() as i32 * S1.w,
            ky,
            &kind,
            S1,
            DIM,
            0.9,
        );
    }

    // The score: the solved plan's final score until the change replays.
    let last_solve = c.solve.trajectory.last();
    score::panel(
        f,
        &c.repair.solve.trajectory,
        &s.repair,
        last_solve,
        t,
        1.0,
        if since >= 0.0 { RED } else { EMERALD },
    );
    let (x0, _, x1, y1) = PANEL;
    let chart = Chart {
        x0: x0 + 30,
        y0: 500,
        x1: x1 - 30,
        y1: 690,
    };
    score::chart(f, &c.repair.solve.trajectory, &s.repair, t, &chart, 1.0);
    changes(f, c, s, t, x0 + 24, 740, y1 - 16);

    let struck = smooth(since / 0.5) * alarm;
    board.draw_grid(
        f,
        1.0,
        if since >= 0.0 {
            mix(EMERALD, RED, 0.35)
        } else {
            EMERALD
        },
        struck,
        t,
    );
    let feasible_age = s.repair_feasible.map_or(-1.0, |at| t - at);
    for b in &board.blocks {
        match (b.solved, b.repaired, b.change) {
            (Some(from), Some(to), Some(k)) => {
                let p = smooth((t - s.moves[k]) / MOVE);
                let mut rect = from.lerp(&to, p);
                let lift = (p * std::f32::consts::PI).sin() * 46.0;
                rect.y0 -= lift;
                rect.y1 -= lift;
                let threatened = struck * (1.0 - p);
                let flash = 0.5 + 0.5 * (t * 14.0).sin();
                let rgb = if p > 0.0 {
                    AMBER
                } else {
                    mix(EMERALD, RED, threatened * (0.6 + 0.4 * flash))
                };
                if p > 0.0 && p < 1.0 {
                    let (cx, cy) = rect.centre();
                    f.glow(cx, cy, 70.0, AMBER, 0.35);
                }
                Board::draw_block(f, &rect, b, rgb, 1.0);
                let landed = t - (s.moves[k] + MOVE);
                let (cx, cy) = to.centre();
                fx::sparks(f, cx, cy, landed, 400 + k as i32, 24, 1.0);
            }
            (Some(from), _, None) => {
                // Untouched: exactly where it was.
                let shimmer = if feasible_age >= 0.0 {
                    let d = (from.x0 - 760.0) / 1100.0;
                    (-((feasible_age * 2.2 - d) * 6.0).powi(2)).exp()
                } else {
                    0.0
                };
                // A grounded bus's blocks flash; the rest dim under the alarm.
                let grounded = board.struck_bus.as_deref() == Some(b.bus.as_str());
                let base = if grounded {
                    mix(EMERALD, RED, struck * (0.6 + 0.4 * (t * 14.0).sin()))
                } else {
                    mix(EMERALD, 0x0b5a3a, 0.35 * struck)
                };
                let rgb = mix(base, LIME, shimmer * 0.8);
                Board::draw_block(f, &from, b, rgb, 1.0);
            }
            (None, Some(to), _) => {
                // The disruption's own job: it drops in red-hot with the
                // strike and cools to amber once the plan has healed.
                let age = t - (s.event + 0.25);
                if age >= 0.0 {
                    let mut r = to;
                    let fall = crate::gfx::ease_out(age / 0.25);
                    r.y0 -= 60.0 * (1.0 - fall);
                    r.y1 -= 60.0 * (1.0 - fall);
                    let cool = if feasible_age >= 0.0 {
                        smooth(feasible_age / 0.6)
                    } else {
                        0.0
                    };
                    let hot = mix(RED, 0xffffff, 0.5 * (-age * 5.0).exp());
                    let (cx, cy) = r.centre();
                    f.glow(cx, cy, 90.0, RED, 0.4 * (1.0 - cool));
                    Board::draw_block(f, &r, b, mix(hot, AMBER, cool), smooth(age / 0.1));
                    fx::sparks(f, cx, to.y1, age - 0.25, 777, 50, 1.2);
                }
            }
            _ => {}
        }
    }
    counters(f, c, s, t, board.added(), board.bottom() + 16);
    review(f, c, s, board, t);
    if r.cordelia.active(t) {
        let targets = beam_targets(s, board, t);
        r.cordelia.draw(f, t, &targets);
    }

    // The strike: a red flash and a tear across the frame; the heal: lime.
    if (0.0..0.6).contains(&since) {
        fx::flash(f, RED, 0.35 * (-since * 6.0).exp());
        fx::glitch(f, t, 0.8 * (1.0 - since / 0.6), 77);
    }
    stamp(f, "PLAN HEALED", LIME, feasible_age);
}

/// A large word over the board for a moment, glyph cells shaking in.
pub fn stamp(f: &mut Frame, word: &str, rgb: u32, age: f32) {
    if !(0.0..1.6).contains(&age) {
        return;
    }
    let size = crate::gfx::size(60, 108);
    let w = word.len() as i32 * size.w;
    let (cx, cy) = (1310, 520);
    let k = smooth(age / 0.08) * (1.0 - smooth((age - 1.1) / 0.5));
    let jitter = if age < 0.25 {
        ((age * 60.0) as i32 % 3 - 1) * 6
    } else {
        0
    };
    f.dim(
        cx - w / 2 - 30,
        cy - 20,
        cx + w / 2 + 30,
        cy + size.h + 20,
        1.0 - 0.7 * k,
    );
    f.text_shadow(
        cx - w / 2 + jitter,
        cy,
        word,
        size,
        mix(rgb, 0xffffff, (-age * 6.0).exp()),
        k,
    );
}

/// The changes so far, one line each: job, from, to.
fn changes(f: &mut Frame, c: &Capture, s: &Script, t: f32, x: i32, y0: i32, y1: i32) {
    let shown: Vec<usize> = (0..c.repair.changed.len())
        .filter(|&k| t >= s.moves[k])
        .collect();
    if shown.is_empty() {
        return;
    }
    f.text(x, y0, "CHANGED ASSIGNMENTS", S1, AMBER, 1.0);
    let pitch = S1.h + 4;
    let fit = ((y1 - y0 - 30) / pitch).max(1) as usize;
    let cols = ((PANEL.2 - PANEL.0 - 48) / S1.w) as usize;
    let skip = shown.len().saturating_sub(fit);
    for (row, &k) in shown.iter().skip(skip).enumerate() {
        let ch = &c.repair.changed[k];
        let line = format!(
            "{}: {} d{} -> {} d{}",
            ch.job, ch.from.technician, ch.from.day, ch.to.technician, ch.to.day
        );
        let age = t - s.moves[k];
        let mut col = mix(0xffffff, PAPER, smooth(age / 0.4));
        let y = y0 + 30 + row as i32 * pitch;
        if reviewing(s, k, t) {
            f.rect(x - 6, y - 3, PANEL.2 - 20, y + S1.h + 3, AMBER, 0.3);
            col = 0xffffff;
        }
        f.text(
            x,
            y0 + 30 + row as i32 * pitch,
            &clip(&line, cols),
            S1,
            col,
            1.0,
        );
    }
    if skip > 0 {
        f.text(x, y1 - S1.h, &format!("(+{skip} earlier)"), S1, DIM, 0.8);
    }
}

fn reviewing(s: &Script, k: usize, t: f32) -> bool {
    s.reviews
        .get(k)
        .is_some_and(|&at| (at..at + s.review_len).contains(&t))
}

/// After the heal, each change in turn: a ghost where the job was, an
/// arrow to where it went, and the change spelled out beside it.
fn review(f: &mut Frame, c: &Capture, s: &Script, board: &Board, t: f32) {
    for (k, &at) in s.reviews.iter().enumerate() {
        let age = t - at;
        if !(0.0..s.review_len).contains(&age) {
            continue;
        }
        let Some(b) = board.blocks.iter().find(|b| b.change == Some(k)) else {
            continue;
        };
        let (Some(from), Some(to)) = (b.solved, b.repaired) else {
            continue;
        };
        let k_in = smooth(age / 0.2) * (1.0 - smooth((age - s.review_len + 0.25) / 0.25));
        let (x0, y0, x1, y1) = (
            from.x0 as i32,
            from.y0 as i32,
            from.x1 as i32,
            from.y1 as i32,
        );
        f.rect(x0, y0, x1, y1, 0x200004, 0.6 * k_in);
        f.frame_rect(x0, y0, x1, y1, RED, k_in);
        let (fx0, fy0) = from.centre();
        let (tx, ty) = to.centre();
        let p = smooth(age / 0.35);
        let (hx, hy) = (fx0 + (tx - fx0) * p, fy0 + (ty - fy0) * p);
        f.line((fx0, fy0), (hx, hy), 3.0, AMBER, k_in);
        f.glow(hx, hy, 40.0, AMBER, 0.6 * k_in);
        f.frame_rect(
            to.x0 as i32 - 3,
            to.y0 as i32 - 3,
            to.x1 as i32 + 3,
            to.y1 as i32 + 3,
            0xffffff,
            k_in,
        );
        let ch = &c.repair.changed[k];
        let label = format!(
            " {}: {} d{} -> {} d{} ",
            ch.job, ch.from.technician, ch.from.day, ch.to.technician, ch.to.day
        );
        let w = label.len() as i32 * S1.w;
        let lx = ((tx as i32) - w / 2).clamp(crate::board::AREA.0, crate::board::AREA.2 - w);
        let ly = if to.y0 as i32 - 34 > crate::board::AREA.1 + 30 {
            to.y0 as i32 - 30
        } else {
            to.y1 as i32 + 8
        };
        f.rect(lx, ly, lx + w, ly + 24, 0x140c02, 0.9 * k_in);
        f.frame_rect(lx, ly, lx + w, ly + 24, AMBER, k_in);
        f.text(lx, ly + 3, &label, S1, 0xffffff, k_in);
    }
}

/// Under the board: changed and untouched counts, and no restart.
fn counters(f: &mut Frame, c: &Capture, s: &Script, t: f32, added: usize, y: i32) {
    if t < s.repair.from {
        return;
    }
    let moved = s.moves.iter().filter(|&&m| t >= m).count();
    let done = s.repair_feasible.is_some_and(|at| t >= at);
    let items = [
        (format!("CHANGED {moved}"), AMBER, true),
        (
            format!("UNTOUCHED {}", c.repair.unchanged_count),
            LIME,
            true,
        ),
        (format!("ADDED {added}"), RED, added > 0),
        ("NO RESTART".to_string(), MINT, done),
    ];
    let mut x = 770;
    for (txt, col, on) in items {
        if !on {
            continue;
        }
        let w = (txt.len() as i32 + 2) * S2.w;
        f.rect(x, y, x + w, y + 46, 0x020806, 0.85);
        f.frame_rect(x, y, x + w, y + 46, col, 0.9);
        f.text(x + S2.w, y + 5, &txt, S2, col, 1.0);
        x += w + 18;
    }
}

/// Where Cordelia's beams strike: the left eye holds the breakdown's own
/// job, the right eye rides the latest changed job (lifted as it moves).
fn beam_targets(s: &Script, board: &Board, t: f32) -> Targets {
    let mut left = None;
    let mut right = None;
    let mut latest = f32::MIN;
    for b in &board.blocks {
        match (b.solved, b.repaired, b.change) {
            (Some(from), Some(to), Some(k)) if s.moves[k] <= t.max(s.moves[0]) => {
                if s.moves[k] > latest {
                    latest = s.moves[k];
                    let p = smooth((t - s.moves[k]) / MOVE);
                    let mut rect = from.lerp(&to, p);
                    let lift = (p * std::f32::consts::PI).sin() * 46.0;
                    rect.y0 -= lift;
                    rect.y1 -= lift;
                    right = Some(rect.centre());
                }
            }
            (None, Some(to), _) => left = Some(to.centre()),
            _ => {}
        }
    }
    Targets { left, right }
}

/// Wrap a code-ish line at spaces, or hard-break a token longer than a line.
fn wrap_code(s: &str, cols: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in s.split(' ') {
        let mut word = word.to_string();
        while word.len() > cols {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            let rest = word.split_off(cols);
            out.push(word);
            word = rest;
        }
        if !cur.is_empty() && cur.len() + 1 + word.len() > cols {
            out.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(&word);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
