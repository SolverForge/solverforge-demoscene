// The week as a board: the shop's 24 lanes (16 machines, then 8 people)
// down the left, Monday 06:00 to Friday afternoon across, one card per
// operation where the plan puts it. The week, the powder and the repair all
// draw on it, so a card keeps its place from one scene to the next.
use super::{BLUE, JADE, PARCH, RED, SAGE};
use crate::data::{Data, Slot, Stage, DAY, FIRST_MIDNIGHT};
use crate::gfx::{mix, Frame};

pub const LANE_Y0: i32 = 196;
pub const LANE_H: i32 = 28;
/// Rows between the machines and the people.
pub const PEOPLE_GAP: i32 = 12;
pub const X0: i32 = 330;
pub const X1: i32 = 1880;
/// Minutes the board spans from Monday 06:00.
pub const SPAN: i32 = 6600;
pub const CARD_H: i32 = 18;

pub fn x_of(min: f32) -> f32 {
    X0 as f32 + min * (X1 - X0) as f32 / SPAN as f32
}

pub fn lane_y(lane: usize) -> i32 {
    LANE_Y0 + lane as i32 * LANE_H + if lane >= 16 { PEOPLE_GAP } else { 0 }
}

/// The card's top-left corner and width for a slot.
pub fn rect_of(slot: Slot) -> (f32, f32, f32) {
    let (lane, a, b) = slot;
    let x = x_of(a as f32);
    let w = (x_of(b as f32) - x).max(3.0);
    (x, (lane_y(lane as usize) + (LANE_H - CARD_H) / 2) as f32, w)
}

pub fn stage_rgb(stage: Stage) -> u32 {
    match stage {
        Stage::Cut => 0x3f7f6c,
        Stage::Bend => 0x6f9a86,
        Stage::Mill => 0x2f6a60,
        Stage::Finish => 0x4f8577,
        Stage::Paint => 0x000000,
        Stage::Weld => 0xa88f58,
        Stage::Assemble => 0x7d8f73,
        Stage::Inspect => 0xbdb6a3,
    }
}

pub fn card_rgb(data: &Data, task: usize) -> u32 {
    let t = &data.tasks[task];
    match t.paint {
        Some(p) => p.rgb(),
        None => stage_rgb(t.stage),
    }
}

/// The grid: lane rows, the day lines and the day names. `a` fades it,
/// `lit(lane)` dims lanes that are not the subject.
pub fn grid(f: &mut Frame, a: f32, lit: impl Fn(usize) -> f32, lanes: usize) {
    if a <= 0.0 {
        return;
    }
    for lane in 0..lanes {
        let y = lane_y(lane);
        let k = a * lit(lane);
        f.rect(X0, y + 1, X1, y + LANE_H - 1, 0x0b1a15, 0.9 * k);
        f.rect(X0, y + LANE_H - 1, X1, y + LANE_H, 0x16302a, k);
    }
    let bottom = lane_y(lanes - 1) + LANE_H;
    let mut m = FIRST_MIDNIGHT;
    let mut day = 0;
    let names = ["MON", "TUE", "WED", "THU", "FRI"];
    let mut start = 0;
    while start < SPAN {
        let end = m.min(SPAN);
        let cx = (x_of(start as f32) + x_of(end as f32)) / 2.0;
        if day < names.len() {
            f.text_centered(cx as i32, 160, names[day], 2, PARCH, a * 0.9);
        }
        if m < SPAN {
            f.dashed(
                x_of(m as f32) as i32,
                150,
                x_of(m as f32) as i32 + 1,
                bottom,
                3,
                5,
                SAGE,
                a * 0.5,
            );
        }
        // Noon, fainter.
        let noon = m - 12 * 60;
        if noon > 0 && noon < SPAN {
            f.dashed(
                x_of(noon as f32) as i32,
                186,
                x_of(noon as f32) as i32 + 1,
                bottom,
                1,
                9,
                SAGE,
                a * 0.25,
            );
        }
        start = m;
        m += DAY;
        day += 1;
    }
}

/// One card. `glow` is the light around it (explore blue, conflict red,
/// resolve jade), `lift` raises it out of its lane.
#[allow(clippy::too_many_arguments)]
pub fn card(
    f: &mut Frame,
    x: f32,
    y: f32,
    w: f32,
    rgb: u32,
    a: f32,
    edge: Option<(u32, f32)>,
    lift: f32,
) {
    if a <= 0.0 {
        return;
    }
    let y = y - lift;
    let (x0, y0, x1, y1) = (
        x as i32,
        y as i32,
        (x + w) as i32,
        (y + CARD_H as f32) as i32,
    );
    if lift > 0.5 {
        f.rect(x0 + 3, y0 + 5, x1 + 3, y1 + 5, 0x000000, 0.5 * a);
    }
    f.rect(x0, y0, x1.max(x0 + 2), y1, rgb, a);
    // A lighter top edge: the cards read as tiles, not as a bar chart.
    f.rect(x0, y0, x1.max(x0 + 2), y0 + 2, mix(rgb, PARCH, 0.35), a);
    if rgb == 0x1b2220 {
        f.frame_rect(x0, y0, x1.max(x0 + 2), y1, 0x56605b, a);
    }
    if let Some((c, k)) = edge {
        if k > 0.0 {
            f.frame_rect(x0 - 1, y0 - 1, x1.max(x0 + 2) + 1, y1 + 1, c, a * k);
            f.frame_rect(x0 - 2, y0 - 2, x1.max(x0 + 2) + 2, y1 + 2, c, a * k * 0.5);
            f.glow(
                (x0 + x1) as f32 / 2.0,
                (y0 + y1) as f32 / 2.0,
                22.0 + w * 0.3,
                c,
                0.18 * k * a,
            );
        }
    }
}

/// Move a card from one slot to another: it lifts, travels on an arc and
/// sets down. `k` is 0..1 through the move.
pub fn travel(from: Slot, to: Slot, k: f32) -> (f32, f32, f32, f32) {
    let (x0, y0, w0) = rect_of(from);
    let (x1, y1, w1) = rect_of(to);
    let e = crate::timeline::smooth(k);
    let lift = 10.0 * (k * std::f32::consts::PI).sin().max(0.0);
    let arc = -40.0 * (k * std::f32::consts::PI).sin() * ((x1 - x0).abs() / 600.0).min(1.0);
    (
        x0 + (x1 - x0) * e,
        y0 + (y1 - y0) * e + arc,
        w0 + (w1 - w0) * e,
        lift,
    )
}

/// A comet tail from where a card was to where it is.
pub fn trail(f: &mut Frame, from: (f32, f32), to: (f32, f32), rgb: u32, a: f32) {
    f.line(from.0, from.1, to.0, to.1, 1.5, rgb, a * 0.6);
}

/// The score panel, top right: hard, medium, soft and placed, coloured by
/// what they say.
#[allow(clippy::too_many_arguments)]
pub fn score(
    f: &mut Frame,
    hard: i32,
    medium: i32,
    soft: i32,
    placed: usize,
    total: usize,
    a: f32,
    t: f32,
) {
    let x = 1250;
    let hard_rgb = if hard < 0 { RED } else { JADE };
    let med_rgb = if medium < 0 {
        mix(RED, PARCH, 0.4)
    } else {
        JADE
    };
    let flicker = if hard < 0 {
        0.75 + 0.25 * (t * 9.0).sin().abs()
    } else {
        1.0
    };
    f.text(x, 44, "HARD", 2, SAGE, a);
    f.text(x, 66, &format!("{hard}"), 3, hard_rgb, a * flicker);
    f.text(x + 170, 44, "MEDIUM", 2, SAGE, a);
    f.text(x + 170, 66, &format!("{medium}"), 3, med_rgb, a);
    f.text(x + 380, 44, "SOFT", 2, SAGE, a);
    f.text(x + 380, 66, &format!("{soft}"), 3, PARCH, a);
    let full = placed == total;
    f.text(
        x,
        106,
        &format!("PLACED {placed} / {total}"),
        2,
        if full { JADE } else { BLUE },
        a,
    );
}

/// A small sparkline of a score over the search, drawn up to `upto`.
#[allow(clippy::too_many_arguments)]
pub fn curve(
    f: &mut Frame,
    pts: &[(f32, f32)],
    x0: f32,
    y0: f32,
    w: f32,
    h: f32,
    upto: f32,
    rgb: u32,
    a: f32,
) {
    if pts.len() < 2 || a <= 0.0 {
        return;
    }
    let tmax = pts.last().unwrap().0.max(1e-3);
    let (lo, hi) = pts.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
        (lo.min(p.1), hi.max(p.1))
    });
    let span = (hi - lo).max(1e-3);
    let px = |p: (f32, f32)| (x0 + p.0 / tmax * w, y0 + h - (p.1 - lo) / span * h);
    let mut last = px(pts[0]);
    for p in pts.iter().skip(1) {
        if p.0 > upto {
            break;
        }
        let q = px(*p);
        // Steps: the score holds until the next improvement.
        f.line(last.0, last.1, q.0, last.1, 1.5, rgb, a);
        f.line(q.0, last.1, q.0, q.1, 1.5, rgb, a);
        last = q;
    }
    let end = x0 + (upto / tmax).min(1.0) * w;
    f.line(last.0, last.1, end, last.1, 1.5, rgb, a);
    f.glow(end, last.1, 10.0, rgb, 0.6 * a);
}
