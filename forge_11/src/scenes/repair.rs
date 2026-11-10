// 04 REPAIR: the plan is live and the solver is still running. The
// captured disruption strikes on the second bar, enters the running solve
// through the captured mechanism, and the plan heals in place: the changed
// jobs travel to their new slots, every other block stays exactly where it
// was, and the score climbs back. No restart.
use super::cordelia::{Cordelia, Targets};
use super::score::{self, Chart, PANEL};
use super::{Timed, AMBER, EMERALD, LIME, MINT, RED, RED_STOPS};
use crate::board::Board;
use crate::capture::Capture;
use crate::fx;
use crate::gfx::{clip, mix, size, smooth, wrap, Frame, Size, S1, S2, W};
use crate::script::Script;
use crate::timeline::Now;

pub struct Repair {
    event: Timed,
    size: Size,
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
        Self {
            event: Timed::new("unstable", &text, 1401, RED_STOPS, s.event, 0.7),
            size: sz,
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
    if t < s.intro_end(crate::timeline::Section::Repair) {
        return;
    }
    let perf = s.cordelia;

    // Before the strike: the week as it runs.
    if since < 0.0 {
        let pulse = 0.6 + 0.4 * (t * 6.0).sin();
        f.text(150, 140, "*", S2, LIME, pulse);
        f.text(150 + 2 * S2.w, 140, "LIVE PLAN", S2, MINT, 1.0);
    } else {
        // The breakdown's sentence, read alone; the mechanism is in the
        // README and the capture, not on screen (it is code, and a second
        // text under this one would compete with it).
        r.event.draw(f, t, 150, 124, r.size, 1.0);
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
    let (x0, _, x1, _) = PANEL;
    let chart = Chart {
        x0: x0 + 30,
        y0: 500,
        x1: x1 - 30,
        y1: 690,
    };
    score::chart(f, &c.repair.solve.trajectory, &s.repair, t, &chart, 1.0);

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
    // Focus: while the event is read, everything but the broken bus's work
    // steps back; while a change is reviewed, everything but that job.
    let ev_focus = smooth(since / 0.3) * (1.0 - smooth((t - perf.fire) / 0.6)) * alarm;
    let rev = s
        .recap
        .iter()
        .position(|r| (r.at..r.at + r.len).contains(&t));
    let rev_focus = match (s.recap.first(), s.recap.last()) {
        (Some(a), Some(z)) => {
            let end = z.at + z.len;
            smooth((t - a.at) / 0.3) * (1.0 - smooth((t - end + 0.3) / 0.3))
        }
        _ => 0.0,
    };
    for b in &board.blocks {
        let mine = rev.is_some() && b.change == rev;
        if mine {
            // The recap draws this one on its replayed path.
            continue;
        }
        let grounded_b = board.struck_bus.as_deref() == Some(b.bus.as_str()) || b.solved.is_none();
        let keep = (1.0 - 0.7 * ev_focus * (!grounded_b) as i32 as f32)
            * (1.0 - 0.7 * rev_focus * (!mine) as i32 as f32);
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
                Board::draw_block(f, &rect, b, rgb, keep);
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
                // Once healed the flashing stops: nothing blinks while the
                // recap and its lines are read.
                let calm = if feasible_age >= 0.0 {
                    1.0 - smooth(feasible_age / 0.6)
                } else {
                    1.0
                };
                let base = if grounded {
                    mix(EMERALD, RED, struck * (0.6 + 0.4 * calm * (t * 14.0).sin()))
                } else {
                    mix(EMERALD, 0x0b5a3a, 0.35 * struck)
                };
                let rgb = mix(base, LIME, shimmer * 0.8);
                Board::draw_block(f, &from, b, rgb, keep);
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
                    Board::draw_block(f, &r, b, mix(hot, AMBER, cool), smooth(age / 0.1) * keep);
                    fx::sparks(f, cx, to.y1, age - 0.25, 777, 50, 1.2);
                }
            }
            _ => {}
        }
    }
    recap(f, c, s, board, t);
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

/// A large word over the board, glyph cells shaking in; it lands, holds
/// for the viewer, then lifts so the board shows through again.
pub fn stamp(f: &mut Frame, word: &str, rgb: u32, age: f32) {
    use crate::script::STAMP_HOLD;
    if !(0.0..STAMP_HOLD).contains(&age) {
        return;
    }
    let size = crate::gfx::size(60, 108);
    let w = word.len() as i32 * size.w;
    let (cx, cy) = (1310, 520);
    let k = smooth(age / 0.08) * (1.0 - smooth((age - STAMP_HOLD + 0.5) / 0.5));
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

/// After the heal, each change replayed so the eye can follow it: the card
/// lifts off its new slot, reappears at its old one (outlined red) with its
/// label, travels along the path to its new slot and lands there.
fn recap(f: &mut Frame, c: &Capture, s: &Script, board: &Board, t: f32) {
    use crate::script::Recap;
    for (k, r) in s.recap.iter().enumerate() {
        let age = t - r.at;
        if !(0.0..r.len).contains(&age) {
            continue;
        }
        let Some(b) = board.blocks.iter().find(|b| b.change == Some(k)) else {
            continue;
        };
        let (Some(from), Some(to)) = (b.solved, b.repaired) else {
            continue;
        };
        let out = 1.0 - smooth((age - r.len + 0.3) / 0.3);
        let (fx0, fy0) = from.centre();
        let (tx, ty) = to.centre();
        let go = Recap::LIFT + Recap::WAIT;
        // The card: lifting off, back at the old slot, travelling, landed.
        let (rect, alpha, rgb) = if age < Recap::LIFT {
            let p = smooth(age / Recap::LIFT);
            let mut q = to;
            q.y0 -= 40.0 * p;
            q.y1 -= 40.0 * p;
            (q, 1.0 - p, AMBER)
        } else if age < go {
            let p = smooth((age - Recap::LIFT) / 0.25);
            (from, p, mix(RED, AMBER, 0.4))
        } else {
            let p = smooth((age - go) / r.travel);
            let mut q = from.lerp(&to, p);
            let lift = (p * std::f32::consts::PI).sin() * 60.0;
            q.y0 -= lift;
            q.y1 -= lift;
            (q, 1.0, AMBER)
        };
        let shown = age >= Recap::LIFT;
        if shown {
            // The old slot, outlined red, and the path to the new one.
            let k_in = smooth((age - Recap::LIFT) / 0.2) * out;
            f.rect(
                from.x0 as i32,
                from.y0 as i32,
                from.x1 as i32,
                from.y1 as i32,
                0x200004,
                0.5 * k_in,
            );
            f.frame_rect(
                from.x0 as i32,
                from.y0 as i32,
                from.x1 as i32,
                from.y1 as i32,
                RED,
                k_in,
            );
            let dots = 24;
            for i in 0..=dots {
                let u = i as f32 / dots as f32;
                let lift = (u * std::f32::consts::PI).sin() * 60.0;
                let (x, y) = (fx0 + (tx - fx0) * u, fy0 + (ty - fy0) * u - lift);
                f.glow(x, y, 4.0, AMBER, 0.5 * k_in);
            }
            f.frame_rect(
                to.x0 as i32 - 3,
                to.y0 as i32 - 3,
                to.x1 as i32 + 3,
                to.y1 as i32 + 3,
                0xffffff,
                0.6 * k_in,
            );
        }
        let (cx, cy) = rect.centre();
        let moving = age >= go && age < go + r.travel;
        if moving {
            f.glow(cx, cy, 70.0, AMBER, 0.4);
        }
        Board::draw_block(f, &rect, b, rgb, alpha);
        let landed = age - go - r.travel;
        fx::sparks(f, tx, ty, landed, 500 + k as i32, 30, 1.0);
        // The label rides with the card.
        if shown {
            let ch = &c.repair.changed[k];
            let label = format!(" {} ", crate::script::recap_label(ch));
            let w = label.len() as i32 * S1.w;
            let lx = ((cx as i32) - w / 2).clamp(crate::board::AREA.0, crate::board::AREA.2 - w);
            let ly = if rect.y0 as i32 - 34 > crate::board::AREA.1 {
                rect.y0 as i32 - 30
            } else {
                rect.y1 as i32 + 8
            };
            let k_in = smooth((age - Recap::LIFT) / 0.15) * out;
            f.rect(lx, ly, lx + w, ly + 24, 0x140c02, 0.92 * k_in);
            f.frame_rect(lx, ly, lx + w, ly + 24, AMBER, k_in);
            f.text(lx, ly + 3, &label, S1, 0xffffff, k_in);
        }
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
