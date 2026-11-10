// 04 REPAIR: the plan is live and the solver is still running. The
// captured breakdown strikes on a bar line and the next twelve seconds
// never stand still: the strike punches in on the broken bus's work and an
// alarm sweeps its mechanic's row while "Bus 05 breaks down in service" is
// read; the urgent job materialises white-hot over the board with "It
// needs urgent electrical repair by day 2", the deadline brackets its days,
// and it drops at two slots inside them and bounces off (the jobs there
// jolt); it hangs over the board while the jobs that will have to move
// start to tremble; with "The depot sends it to the running app" the
// change flies off the board into the app's panel and Cordelia tears in on
// its arrival, charging on its heat. On the downbeat she fires, the urgent
// job slams into its slot, the changed jobs travel, and the score climbs
// back. No restart.
use super::cordelia::{Cordelia, Targets};
use super::score::{self, Chart, PANEL};
use super::{Timed, AMBER, EMERALD, LIME, MINT, RED, RED_STOPS};
use crate::board::{Board, Rect, AREA};
use crate::capture::Capture;
use crate::fx;
use crate::gfx::{ease_out, mix, smooth, Frame, Size, S1, S2};
use crate::script::{Breakdown, Script};
use crate::story::Cap;
use crate::timeline::{beat_pulse, Now, BAR, BEAT};

pub struct Repair {
    /// Each breakdown line: its clip, size, and read window.
    lines: Vec<(Timed, Size, f32, f32)>,
    cordelia: Cordelia,
}

const MOVE: f32 = 0.6;
/// Where the app's panel takes the change in (by the score).
const INBOX: (f32, f32) = (380.0, 300.0);

impl Repair {
    pub fn new(_c: &Capture, s: &Script) -> Self {
        let lines = s
            .breakdown
            .lines
            .iter()
            .enumerate()
            .map(|(i, (text, from, to))| {
                let sz = if text.len() as i32 * S2.w <= 1500 {
                    S2
                } else {
                    S1
                };
                (
                    Timed::new("unstable", text, 1401 + i as u64, RED_STOPS, *from, 0.5),
                    sz,
                    *from,
                    *to,
                )
            })
            .collect();
        Self {
            lines,
            cordelia: Cordelia::new(s),
        }
    }
}

/// The breakdown's cast on the board.
struct Cast {
    /// The urgent job (it only exists in the repaired plan).
    hot: Option<usize>,
    /// The broken bus's solved work.
    grounded: Vec<usize>,
    /// The rows that work sits in.
    rows: Vec<usize>,
    /// The two slots the urgent job tries.
    tries: [Rect; 2],
    /// Where it hangs over the board.
    hover: Rect,
    /// The day columns those slots sit in.
    days: [Rect; 2],
    /// Day columns inside the deadline (first, last).
    window: Option<(usize, usize)>,
}

fn row_of(board: &Board, r: &Rect) -> usize {
    let (_, cy) = r.centre();
    (0..board.techs.len())
        .find(|&i| {
            let rr = board.row_rect(i);
            cy >= rr.y0 && cy < rr.y1
        })
        .unwrap_or(0)
}

fn col_of(board: &Board, r: &Rect) -> usize {
    let (cx, _) = r.centre();
    (0..board.days.len())
        .find(|&i| {
            let c = board.col_rect(i);
            cx >= c.x0 && cx < c.x1
        })
        .unwrap_or(0)
}

fn cast(board: &Board, b: &Breakdown) -> Cast {
    let hot = board.blocks.iter().position(|x| x.solved.is_none());
    let grounded: Vec<usize> = board
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, x)| x.solved.is_some() && board.struck_bus.as_deref() == Some(x.bus.as_str()))
        .map(|(i, _)| i)
        .collect();
    let mut rows: Vec<usize> = grounded
        .iter()
        .filter_map(|&i| board.blocks[i].solved)
        .map(|r| row_of(board, &r))
        .collect();
    if rows.is_empty() {
        rows.extend(board.struck_tech);
    }
    rows.dedup();
    let to = hot.and_then(|i| board.blocks[i].repaired).unwrap_or(Rect {
        x0: 1000.0,
        y0: 400.0,
        x1: 1100.0,
        y1: 440.0,
    });
    let col = col_of(board, &to);
    let next = if col + 1 < board.days.len() {
        col + 1
    } else {
        col.saturating_sub(1)
    };
    let dx = board.col_rect(next).x0 - board.col_rect(col).x0;
    let try2 = Rect {
        x0: to.x0 + dx,
        x1: to.x1 + dx,
        ..to
    };
    let window = b.due_day.and_then(|due| {
        let last = board.days.iter().rposition(|&d| d <= due)?;
        Some((0, last))
    });
    let h = (to.y1 - to.y0).min(56.0);
    let cx = match window {
        Some((a, z)) => (board.col_rect(a).x0 + board.col_rect(z).x1) * 0.5,
        None => to.centre().0,
    };
    let w = to.x1 - to.x0;
    let hover = Rect {
        x0: cx - w * 0.5,
        x1: cx + w * 0.5,
        y0: AREA.1 as f32 - 8.0 - h,
        y1: AREA.1 as f32 - 8.0,
    };
    let day = |r: &Rect| {
        let c = board.col_rect(col_of(board, r));
        Rect {
            y0: AREA.1 as f32 + 34.0,
            ..c
        }
    };
    Cast {
        hot,
        grounded,
        rows,
        days: [day(&to), day(&try2)],
        tries: [to, try2],
        hover,
        window,
    }
}

/// Where the urgent job is at t before the fire, and how hot it glows.
fn hot_rect(c: &Cast, b: &Breakdown, t: f32) -> (Rect, f32) {
    let mut r = c.hover;
    for (k, &(drop, hit)) in b.tries.iter().enumerate() {
        let target = c.tries[k];
        let h = target.y1 - target.y0;
        // It crashes into the top of the slot's cell and bounces off.
        let crash = Rect {
            y0: target.y0 - 0.45 * h,
            y1: target.y1 - 0.45 * h,
            ..target
        };
        if t >= drop && t < hit {
            let p = ((t - drop) / (hit - drop)).powi(2);
            r = c.hover.lerp(&crash, p);
        } else if t >= hit && t < hit + 0.45 {
            let p = ease_out((t - hit) / 0.45);
            r = crash.lerp(&c.hover, p);
        }
    }
    (r, 1.0)
}

#[allow(clippy::too_many_arguments)]
pub fn draw(f: &mut Frame, c: &Capture, s: &Script, board: &Board, r: &Repair, t: f32, _n: Now) {
    let since = t - s.event;
    let alarm = if since >= 0.0 { 1.0 } else { 0.0 };
    let perf = s.cordelia;
    let bd = &s.breakdown;
    let firing = (perf.fire..perf.cut).contains(&t);
    f.gradient(&[
        0x010403,
        mix(
            0x03120c,
            0x1a0306,
            alarm * 0.8 * (1.0 - smooth((t - perf.fire) / 1.0)),
        ),
        0x05100c,
        0x010302,
    ]);
    let bars: &[u32] = if firing {
        &[MINT, EMERALD, LIME]
    } else if since >= 0.0 && t < perf.fire {
        &[RED, 0xa32000, 0x7a0010]
    } else {
        &[EMERALD, LIME, 0x0ea5a4]
    };
    // The build tightens the copper; the fire flares it on every beat.
    let build = if since >= 0.0 && t < perf.fire {
        smooth(since / (perf.fire - s.event))
    } else {
        0.0
    };
    let flare = if firing {
        0.22 + 0.25 * beat_pulse(t, 6.0)
    } else {
        0.0
    };
    fx::copper(
        f,
        t * (1.0 + 0.8 * build),
        140.0,
        1060.0,
        bars,
        4,
        0.16 + 0.25 * (-since.max(0.0) * 1.5).exp() * alarm + 0.12 * build + flare,
    );
    fx::river(f, t, 1000, 4, &[0x02140c, 0x0b5a3a, EMERALD, 0xd1fae5], 0.5);
    if t < s.intro_end(crate::timeline::Section::Repair) {
        return;
    }
    let ca = cast(board, bd);

    // The header: LIVE PLAN, then each breakdown line while it is read;
    // the last one stays, dimmed, as the situation.
    if since < 0.0 {
        let pulse = 0.6 + 0.4 * (t * 6.0).sin();
        f.text(150, 140, "*", S2, LIME, pulse);
        f.text(150 + 2 * S2.w, 140, "LIVE PLAN", S2, MINT, 1.0);
    } else {
        let last = r.lines.len().saturating_sub(1);
        for (i, (clip, sz, from, to)) in r.lines.iter().enumerate() {
            if t < *from {
                continue;
            }
            let gain = if i == last {
                1.0 - 0.45 * smooth((t - to) / 0.4)
            } else {
                1.0 - smooth((t - to + 0.25) / 0.25)
            };
            clip.draw(f, t, 150, 124, *sz, gain);
        }
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
        if since >= 0.0 && t < perf.fire + 1.0 {
            mix(EMERALD, RED, 0.35 * (1.0 - smooth((t - perf.fire) / 1.0)))
        } else {
            EMERALD
        },
        struck,
        t,
    );
    let pre_fire = since >= 0.0 && t < perf.fire;
    if pre_fire || (t - perf.fire) < 0.6 {
        breakdown_under(f, board, &ca, bd, s, t);
    }

    let feasible_age = s.repair_feasible.map_or(-1.0, |at| t - at);
    // Focus: through the breakdown the board steps back, except the broken
    // bus's work, the urgent job and (as the tension builds) the jobs that
    // will move; while a change is reviewed, everything but that job.
    let focus = smooth(since / 0.3) * (1.0 - smooth((t - perf.fire) / 0.6)) * alarm;
    let tension = if pre_fire {
        smooth((t - bd.hover) / (perf.fire - bd.hover).max(0.1))
    } else {
        0.0
    };
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
    // Each try's impact jolts the jobs already booked that day: the day
    // is full (the workshop takes so many buses a day).
    let jolt = |rect: &Rect| -> (f32, f32) {
        let mut dx = 0.0;
        let mut flash = 0.0f32;
        for (k, &(_, hit)) in bd.tries.iter().enumerate() {
            let a = t - hit;
            if (0.0..0.7).contains(&a) {
                let day = board.col_rect(col_of(board, &ca.tries[k]));
                let (cx, _) = rect.centre();
                if cx >= day.x0 && cx < day.x1 {
                    dx += (a * 70.0).sin() * 9.0 * (-a * 7.0).exp();
                    flash = flash.max((-a * 5.0).exp());
                }
            }
        }
        (dx, flash)
    };
    for (bi, b) in board.blocks.iter().enumerate() {
        let mine = rev.is_some() && b.change == rev;
        if mine {
            // The recap draws this one on its replayed path.
            continue;
        }
        let grounded = ca.grounded.contains(&bi);
        let rel = if grounded {
            1.0
        } else if b.change.is_some() {
            tension
        } else {
            0.0
        };
        let keep = (1.0 - 0.6 * focus * (1.0 - rel)) * (1.0 - 0.7 * rev_focus);
        match (b.solved, b.repaired, b.change) {
            (Some(from), Some(to), Some(k)) => {
                let p = smooth((t - s.moves[k]) / MOVE);
                let mut rect = from.lerp(&to, p);
                let lift = (p * std::f32::consts::PI).sin() * 46.0;
                rect.y0 -= lift;
                rect.y1 -= lift;
                if p <= 0.0 {
                    // Trembling as the fire nears: these are the ones
                    // that will have to move.
                    let (jx, jf) = jolt(&rect);
                    let tremble = (fx::h2(bi as i32, (t * 30.0) as i32) - 0.5) * 7.0 * tension;
                    rect.x0 += jx + tremble;
                    rect.x1 += jx + tremble;
                    if grounded {
                        draw_grounded(f, board, &ca, bd, &rect, b, t, keep);
                    } else {
                        let rgb = mix(EMERALD, AMBER, jf * 0.8);
                        Board::draw_block(f, &rect, b, rgb, keep);
                    }
                    if tension > 0.0 {
                        let pulse = 0.35 + 0.65 * beat_pulse(t, 5.0);
                        f.frame_rect(
                            rect.x0 as i32 - 3,
                            rect.y0 as i32 - 3,
                            rect.x1 as i32 + 3,
                            rect.y1 as i32 + 3,
                            AMBER,
                            tension * pulse,
                        );
                        let (cx, cy) = rect.centre();
                        f.glow(cx, cy, 50.0, AMBER, 0.25 * tension * pulse);
                    }
                } else {
                    if p < 1.0 {
                        let (cx, cy) = rect.centre();
                        f.glow(cx, cy, 70.0, AMBER, 0.35);
                    }
                    Board::draw_block(f, &rect, b, AMBER, keep);
                }
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
                let mut rect = from;
                let (jx, jf) = jolt(&rect);
                rect.x0 += jx;
                rect.x1 += jx;
                if grounded && t < perf.fire + 0.6 {
                    draw_grounded(f, board, &ca, bd, &rect, b, t, keep);
                } else {
                    let base = mix(
                        EMERALD,
                        0x0b5a3a,
                        0.35 * struck * (1.0 - smooth(feasible_age.max(0.0) / 0.6)),
                    );
                    let rgb = mix(mix(base, AMBER, jf * 0.8), LIME, shimmer * 0.8);
                    Board::draw_block(f, &rect, b, rgb, keep);
                }
            }
            (None, Some(to), _) => {
                hot_job(f, s, &ca, bd, &to, b, t, feasible_age, rev_focus);
            }
            _ => {}
        }
    }
    recap(f, c, s, board, t);
    caption_motion(f, s, board, t);
    if pre_fire {
        signal(f, &ca, bd, t);
    }
    if r.cordelia.active(t) {
        let targets = beam_targets(s, board, t);
        r.cordelia.draw(f, t, &targets);
        // Over her picture, so the heat visibly reaches her eyes.
        if (bd.arrive..perf.fire).contains(&t) {
            charge_stream(f, &r.cordelia, &ca, bd, s, t);
        }
    }

    // The strike: a red flash and a tear across the frame; the fire: a
    // mint raster on every bar the beams hold.
    if (0.0..0.6).contains(&since) {
        fx::flash(f, RED, 0.35 * (-since * 6.0).exp());
        fx::glitch(f, t, 0.8 * (1.0 - since / 0.6), 77);
    }
    if firing {
        let bar_age = (t - perf.fire).rem_euclid(BAR);
        if bar_age < 0.45 {
            fx::raster_sweep(f, bar_age / 0.45, MINT);
        }
        fx::flash(f, MINT, 0.05 * beat_pulse(t, 9.0));
    }
    stamp(f, "PLAN HEALED", LIME, feasible_age);
    camera(f, board, &ca, s, t);
}

/// Under the blocks: the alarm sweeping the broken bus's row, the row's
/// red wash, and the deadline bracketing its days.
fn breakdown_under(f: &mut Frame, board: &Board, ca: &Cast, bd: &Breakdown, s: &Script, t: f32) {
    let perf = s.cordelia;
    let gone = 1.0 - smooth((t - perf.fire) / 0.5);
    let (ax0, _, ax1, _) = AREA;
    let sweep = (t - bd.sweep) / Breakdown::SWEEP;
    for &row in &ca.rows {
        let rr = board.row_rect(row);
        let (y0, y1) = (rr.y0 as i32 + 1, rr.y1 as i32);
        if sweep >= 0.0 {
            let x = ax0 as f32 + (ax1 - ax0) as f32 * smooth(sweep.min(1.0));
            // The wash the sweep leaves behind, beating with the alarm.
            let wash = if sweep < 1.0 {
                0.35
            } else {
                0.14 + 0.12 * beat_pulse(t, 4.0)
            };
            f.rect(ax0, y0, x as i32, y1, 0x400008, wash * gone);
            if sweep < 1.15 {
                let k = 1.0 - smooth((sweep - 1.0) / 0.15);
                f.rect(x as i32 - 4, y0, x as i32 + 4, y1, 0xffd0d4, 0.9 * k);
                f.glow(x, (rr.y0 + rr.y1) * 0.5, 90.0, RED, 0.7 * k);
                for i in 0..6 {
                    f.glow(
                        x - 30.0 * i as f32,
                        (rr.y0 + rr.y1) * 0.5,
                        40.0,
                        RED,
                        0.18 * k * (1.0 - i as f32 / 6.0),
                    );
                }
            }
        }
    }
    // The deadline: a red bracket under the days the repair must fit in,
    // drawn left to right as its line is read; later days step back.
    if let Some((a, z)) = ca.window {
        let k = smooth((t - bd.spawn - 0.1) / 0.6);
        if k > 0.0 {
            let (x0, x1) = (board.col_rect(a).x0, board.col_rect(z).x1);
            let xe = x0 + (x1 - x0) * k;
            let y = AREA.1 + 30;
            let a_ = gone;
            f.rect(x0 as i32, y, xe as i32, y + 4, RED, a_);
            f.rect(x0 as i32, y - 10, x0 as i32 + 4, y + 4, RED, a_);
            if k > 0.98 {
                f.rect(x1 as i32 - 4, y - 10, x1 as i32, y + 4, RED, a_);
            }
            f.glow(
                xe,
                y as f32 + 2.0,
                30.0,
                RED,
                0.6 * a_ * (1.0 - smooth((k - 0.9) / 0.1)),
            );
            let bottom = board.bottom();
            f.rect(x0 as i32, y + 4, x1 as i32, bottom, RED, 0.06 * k * a_);
            f.dim(x1 as i32 + 1, AREA.1 + 34, ax1, bottom, 1.0 - 0.35 * k * a_);
        }
    }
}

/// The broken bus's own work: it jolts on the strike, flips red as the
/// alarm passes, then goes dark, hatched, its edge beating red.
#[allow(clippy::too_many_arguments)]
fn draw_grounded(
    f: &mut Frame,
    _board: &Board,
    _ca: &Cast,
    bd: &Breakdown,
    rect: &Rect,
    b: &crate::board::Block,
    t: f32,
    keep: f32,
) {
    let (ax0, _, ax1, _) = AREA;
    let hit_at = bd.sweep
        + Breakdown::SWEEP * ((rect.centre().0 - ax0 as f32) / (ax1 - ax0) as f32).clamp(0.0, 1.0);
    let strike = (t - bd.sweep + 0.15).max(0.0);
    let mut r = *rect;
    let shake = (strike * 80.0).sin() * 10.0 * (-strike * 5.0).exp();
    r.x0 += shake;
    r.x1 += shake;
    let age = t - hit_at;
    if age < 0.0 {
        Board::draw_block(f, &r, b, mix(EMERALD, RED, 0.5), keep);
        return;
    }
    let hot = (-age * 4.0).exp();
    let dark = smooth(age / 0.8);
    let rgb = mix(mix(RED, 0xffffff, 0.6 * hot), 0x5a0010, dark * 0.7);
    Board::draw_block(f, &r, b, rgb, keep);
    // The hatch scrolls across the dark card.
    let (x0, y0, x1, y1) = (r.x0 as i32, r.y0 as i32, r.x1 as i32, r.y1 as i32);
    let step = 14;
    let off = ((t * 40.0) as i32).rem_euclid(step);
    let mut x = x0 - (y1 - y0) + off;
    while x < x1 {
        let (a, bb) = ((x.max(x0)) as f32, (x + (y1 - y0)).min(x1) as f32);
        if bb > a {
            let ya = y1 as f32 - (a - x as f32);
            let yb = y1 as f32 - (bb - x as f32);
            f.line((a, ya), (bb, yb), 2.0, RED, 0.5 * dark * keep);
        }
        x += step;
    }
    let pulse = 0.4 + 0.6 * beat_pulse(t, 5.0);
    f.frame_rect(x0 - 2, y0 - 2, x1 + 2, y1 + 2, RED, pulse * keep);
    // A red heartbeat off the dead card on every beat, with the kick,
    // until the urgent job takes the eye.
    if t < bd.spawn {
        let b = (t / BEAT).fract() * BEAT;
        let (cx, cy) = r.centre();
        fx::ring(
            f,
            cx,
            cy,
            30.0 + 200.0 * ease_out(b / 0.45),
            2.0,
            RED,
            0.7 * (1.0 - smooth(b / 0.45)) * dark,
        );
    }
    if age < 0.5 {
        let (cx, cy) = r.centre();
        f.glow(cx, cy, 120.0, RED, 0.6 * hot);
        fx::sparks(f, cx, cy, age, 4040, 40, 1.4);
    }
}

/// The urgent job: born white-hot over the board with its line, drops at
/// two slots inside its deadline and bounces off, hangs there hotter as the
/// fire nears; on the fire it slams into its slot; it cools to amber once
/// the plan has healed.
#[allow(clippy::too_many_arguments)]
fn hot_job(
    f: &mut Frame,
    s: &Script,
    ca: &Cast,
    bd: &Breakdown,
    to: &Rect,
    b: &crate::board::Block,
    t: f32,
    feasible_age: f32,
    rev_focus: f32,
) {
    let perf = s.cordelia;
    let age = t - bd.spawn;
    if age < 0.0 {
        return;
    }
    let keep = 1.0 - 0.7 * rev_focus;
    if t < perf.fire {
        let (mut r, _) = hot_rect(ca, bd, t);
        // Materialise: grows from a white point, a shockwave, sparks.
        let grow = ease_out(age / 0.35);
        let (cx, cy) = r.centre();
        let (hw, hh) = ((r.x1 - r.x0) * 0.5 * grow, (r.y1 - r.y0) * 0.5 * grow);
        // Hanging there it shakes harder as the fire nears.
        let tension = smooth((t - bd.hover) / (perf.fire - bd.hover).max(0.1));
        let j = (fx::h2(77, (t * 40.0) as i32) - 0.5) * 8.0 * tension;
        r = Rect {
            x0: cx - hw + j,
            x1: cx + hw + j,
            y0: cy - hh,
            y1: cy + hh,
        };
        let beat = beat_pulse(t, 5.0);
        let white = (-age * 3.0).exp().max(0.25 * beat * (0.5 + tension));
        let rgb = mix(RED, 0xffffff, white);
        let (cx, cy) = r.centre();
        f.glow(
            cx,
            cy,
            90.0 + 60.0 * tension + 30.0 * beat,
            RED,
            0.45 + 0.3 * tension,
        );
        f.glow(cx, cy, 30.0, 0xffffff, 0.3 * white);
        Board::draw_block(f, &r, b, rgb, smooth(age / 0.08));
        fx::ring(
            f,
            cx,
            cy,
            20.0 + 420.0 * ease_out(age / 0.6),
            3.0,
            RED,
            0.8 * (1.0 - smooth(age / 0.6)),
        );
        fx::sparks(f, cx, cy, age, 777, 70, 1.6);
        // Every slot it tries: a crash, sparks, a red cross on the cell.
        for (k, &(_, hit)) in bd.tries.iter().enumerate() {
            let a = t - hit;
            if (0.0..0.9).contains(&a) {
                let tr = ca.tries[k];
                let (tx, ty) = (tr.centre().0, tr.y0);
                fx::sparks(f, tx, ty, a, 880 + k as i32, 60, 1.5);
                fx::ring(
                    f,
                    tx,
                    ty,
                    20.0 + 160.0 * ease_out(a / 0.4),
                    2.0,
                    RED,
                    0.7 * (1.0 - smooth(a / 0.4)),
                );
                // The whole day lights red: it is full.
                let x = 1.0 - smooth((a - 0.4) / 0.5);
                let day = ca.days[k];
                f.rect(
                    day.x0 as i32,
                    day.y0 as i32,
                    day.x1 as i32,
                    day.y1 as i32,
                    RED,
                    0.16 * x,
                );
                f.frame_rect(
                    day.x0 as i32,
                    day.y0 as i32,
                    day.x1 as i32,
                    day.y1 as i32,
                    RED,
                    x,
                );
                f.frame_rect(
                    day.x0 as i32 + 3,
                    day.y0 as i32 + 3,
                    day.x1 as i32 - 3,
                    day.y1 as i32 - 3,
                    RED,
                    0.5 * x,
                );
            }
        }
        return;
    }
    // The fire: it slams from over the board into its slot.
    let fa = t - perf.fire;
    let p = smooth(fa / 0.18);
    let mut r = ca.hover.lerp(to, p);
    if fa < 0.18 {
        // Stretched by the speed.
        r.y0 -= 30.0 * (1.0 - p);
    }
    let cool = if feasible_age >= 0.0 {
        smooth(feasible_age / 0.6)
    } else {
        0.0
    };
    let landed = fa - 0.18;
    let hot = mix(RED, 0xffffff, 0.6 * (-landed.max(0.0) * 4.0).exp());
    let (cx, cy) = r.centre();
    f.glow(cx, cy, 90.0, RED, 0.4 * (1.0 - cool) * keep);
    Board::draw_block(f, &r, b, mix(hot, AMBER, cool), keep);
    if landed >= 0.0 {
        fx::ring(
            f,
            cx,
            cy,
            30.0 + 600.0 * ease_out(landed / 0.7),
            4.0,
            0xffffff,
            0.8 * (1.0 - smooth(landed / 0.7)),
        );
        fx::sparks(f, cx, cy, landed, 9090, 120, 2.0);
        fx::sparks(f, cx, cy, landed - 0.05, 9091, 80, 1.2);
    }
}

/// The change leaving the board for the running app: a comet from the
/// urgent job to the score panel; the panel lights as it arrives.
fn signal(f: &mut Frame, ca: &Cast, bd: &Breakdown, t: f32) {
    let len = bd.arrive - bd.signal;
    let u = (t - bd.signal) / len;
    let (sx, sy) = ca.hover.centre();
    let (ex, ey) = INBOX;
    let at = |u: f32| {
        let p = smooth(u);
        let lift = (p * std::f32::consts::PI).sin() * 30.0;
        (sx + (ex - sx) * p, sy + (ey - sy) * p - lift)
    };
    if (0.0..1.0).contains(&u) {
        // The path ahead, dotted; the comet and its trail.
        for i in 0..=30 {
            let v = i as f32 / 30.0;
            if v > u {
                let (x, y) = at(v);
                f.glow(x, y, 4.0, RED, 0.35);
            }
        }
        for k in 0..14 {
            let v = (u - k as f32 * 0.025).max(0.0);
            let (x, y) = at(v);
            let fade = 1.0 - k as f32 / 14.0;
            f.glow(x, y, 26.0 * fade + 6.0, RED, 0.5 * fade);
        }
        let (x, y) = at(u);
        f.glow(x, y, 60.0, RED, 0.8);
        f.glow(x, y, 18.0, 0xffffff, 1.0);
        fx::sparks(
            f,
            x,
            y,
            (u * 8.0).fract() * 0.4,
            3030 + (u * 8.0) as i32,
            14,
            1.0,
        );
    }
    // Arrival: the panel rings white, a shockwave from the inbox.
    let a = t - bd.arrive;
    if (0.0..0.8).contains(&a) {
        let k = 1.0 - smooth(a / 0.8);
        let (x0, y0, x1, y1) = PANEL;
        f.frame_rect(x0 - 4, y0 - 4, x1 + 4, y1 + 4, 0xffffff, k);
        f.frame_rect(x0 - 8, y0 - 8, x1 + 8, y1 + 8, RED, 0.7 * k);
        fx::ring(
            f,
            ex,
            ey,
            20.0 + 380.0 * ease_out(a / 0.6),
            3.0,
            RED,
            0.8 * k,
        );
        fx::flash(f, RED, 0.12 * k);
    }
}

/// While Cordelia charges, heat streams off the urgent job into her eyes,
/// faster and denser towards the fire.
fn charge_stream(f: &mut Frame, cd: &Cordelia, ca: &Cast, bd: &Breakdown, s: &Script, t: f32) {
    let perf = s.cordelia;
    let k = 0.3 + 0.7 * smooth((t - bd.arrive) / (perf.fire - bd.arrive));
    let (sx, sy) = hot_rect(ca, bd, t).0.centre();
    let eyes = cd.eyes(t);
    let n = (20.0 + 60.0 * k) as i32;
    for i in 0..n {
        let (ex, ey) = eyes[(i % 2) as usize];
        let speed = 0.6 + 1.8 * k;
        let u = (fx::h1(i * 13 + 5) + (t - bd.arrive) * speed * (0.7 + 0.6 * fx::h1(i))).fract();
        let wob = (fx::h1(i * 7) - 0.5) * 120.0 * (1.0 - u) * u * 4.0;
        let (x, y) = (sx + (ex - sx) * u, sy + (ey - sy) * u + wob);
        let col = mix(RED, MINT, u);
        let a = (0.5 + 0.5 * k) * (1.0 - (u - 0.9).max(0.0) * 10.0);
        f.glow(x, y, 10.0 + 10.0 * u, col, a);
        f.glow(x, y, 3.0, 0xffffff, a);
    }
}

/// While a caption about the plan is read, the board shows what it says.
/// "N jobs move..., the other M stay put": the moved jobs ping in turn,
/// then the untouched ones are swept once. "Nobody ... rebuild the week":
/// the week runs, a day at a time, every job ticking off as its day passes.
fn caption_motion(f: &mut Frame, s: &Script, board: &Board, t: f32) {
    for c in &s.captions {
        if !(c.from..c.to).contains(&t) {
            continue;
        }
        let a = t - c.from;
        let out = 1.0 - smooth((t - c.to + 0.4) / 0.4);
        match c.cap {
            Cap::Moved => {
                let n = board
                    .blocks
                    .iter()
                    .filter(|b| b.change.is_some())
                    .count()
                    .max(1);
                let cycle = (n as f32 + 2.0) * BEAT;
                let ca = a % cycle;
                for b in &board.blocks {
                    let (Some(to), Some(k)) = (b.repaired, b.change) else {
                        continue;
                    };
                    let ping = ca - 0.3 - k as f32 * BEAT;
                    if (0.0..0.8).contains(&ping) {
                        let e = 1.0 - smooth(ping / 0.8);
                        let g = 6.0 + 14.0 * ease_out(ping / 0.5);
                        f.frame_rect(
                            (to.x0 - g) as i32,
                            (to.y0 - g) as i32,
                            (to.x1 + g) as i32,
                            (to.y1 + g) as i32,
                            AMBER,
                            e * out,
                        );
                        let (cx, cy) = to.centre();
                        f.glow(cx, cy, 70.0, AMBER, 0.45 * e * out);
                    }
                }
                // The untouched jobs, swept once a cycle in lime.
                let sw = (ca - 0.3 - n as f32 * BEAT) / (2.0 * BEAT);
                if (0.0..1.2).contains(&sw) {
                    for b in board.blocks.iter().filter(|b| b.change.is_none()) {
                        let Some(r) = b.solved else { continue };
                        let d = (r.x0 - AREA.0 as f32) / (AREA.2 - AREA.0) as f32;
                        let k = (-((sw - d) * 8.0).powi(2)).exp() * out;
                        if k > 0.02 {
                            f.frame_rect(
                                r.x0 as i32 - 2,
                                r.y0 as i32 - 2,
                                r.x1 as i32 + 2,
                                r.y1 as i32 + 2,
                                LIME,
                                k,
                            );
                        }
                    }
                }
            }
            Cap::Done => {
                let days = board.days.len().max(1);
                let run = (c.to - c.from - 1.0).max(1.0);
                let u = ((a - 0.4) / run).clamp(0.0, 1.0) * days as f32;
                let (x0, x1) = (board.col_rect(0).x0, board.col_rect(days - 1).x1);
                let x = x0 + (x1 - x0) * u / days as f32;
                let (top, bottom) = (AREA.1 as f32, board.bottom() as f32);
                if a > 0.4 {
                    f.rect(
                        x0 as i32,
                        top as i32 + 34,
                        x as i32,
                        bottom as i32,
                        LIME,
                        0.06 * out,
                    );
                    f.rect(
                        x as i32 - 2,
                        top as i32,
                        x as i32 + 2,
                        bottom as i32,
                        LIME,
                        0.9 * out,
                    );
                    f.glow(x, top + 17.0, 40.0, LIME, 0.6 * out);
                }
                for b in &board.blocks {
                    let Some(r) = b.repaired.or(b.solved) else {
                        continue;
                    };
                    let pass = (x - r.x1) / 120.0;
                    if (0.0..1.0).contains(&pass) {
                        let e = 1.0 - pass;
                        f.rect(
                            r.x0 as i32,
                            r.y0 as i32,
                            r.x1 as i32,
                            r.y1 as i32,
                            LIME,
                            0.35 * e * out,
                        );
                        f.frame_rect(
                            r.x0 as i32 - 2,
                            r.y0 as i32 - 2,
                            r.x1 as i32 + 2,
                            r.y1 as i32 + 2,
                            0xffffff,
                            0.7 * e * out,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

/// The camera: a punch-in and a shake on the strike, a knock on each slot
/// the urgent job hits, a harder punch on the fire towards its slot, and a
/// kick on every beat while the beams hold.
fn camera(f: &mut Frame, board: &Board, ca: &Cast, s: &Script, t: f32) {
    let bd = &s.breakdown;
    let perf = s.cordelia;
    let mut zoom = 1.0;
    let (mut dx, mut dy) = (0.0, 0.0);
    let mut centre = (1310.0, 580.0);
    let since = t - s.event;
    if (0.0..1.2).contains(&since) {
        let e = (-since * 5.0).exp();
        zoom *= 1.0 + 0.06 * e;
        dx += 16.0 * (-since * 7.0).exp() * (since * 75.0).sin();
        dy += 10.0 * (-since * 7.0).exp() * (since * 61.0).cos();
        if let Some(r) = ca.grounded.first().and_then(|&i| board.blocks[i].solved) {
            centre = r.centre();
        }
    }
    for &(_, hit) in &bd.tries {
        let a = t - hit;
        if (0.0..0.4).contains(&a) {
            dy += 7.0 * (-a * 10.0).exp() * (a * 70.0).cos();
            zoom *= 1.0 + 0.015 * (-a * 10.0).exp();
        }
    }
    let fa = t - perf.fire - 0.18;
    if (0.0..1.2).contains(&fa) {
        zoom *= 1.0 + 0.07 * (-fa * 4.0).exp();
        dx += 18.0 * (-fa * 7.0).exp() * (fa * 70.0).sin();
        dy += 12.0 * (-fa * 7.0).exp() * (fa * 53.0).cos();
        if let Some(to) = ca.hot.and_then(|i| board.blocks[i].repaired) {
            centre = to.centre();
        }
    }
    if (perf.fire + 1.0..perf.cut).contains(&t) {
        let k = beat_pulse(t, 10.0);
        dx += 5.0 * k * (t * 90.0).sin();
        zoom *= 1.0 + 0.008 * k;
    }
    fx::camera(f, centre.0, centre.1, zoom, dx, dy);
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
    // A light runs along the word while it holds.
    let run = ((age - 0.3) / (STAMP_HOLD - 0.8)).clamp(0.0, 1.0);
    if run > 0.0 && run < 1.0 {
        let x = (cx - w / 2) as f32 + w as f32 * run;
        f.glow(x, (cy + size.h / 2) as f32, 110.0, rgb, 0.35 * k);
    }
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
