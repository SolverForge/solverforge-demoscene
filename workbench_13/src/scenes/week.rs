// The week: the solve redrawn from its improving plans. Two minutes of
// search play in the section's twenty seconds, six seconds a second; the
// clock in the header says so. The first pass pours the operations in, ten
// of them still waiting in the tray; after it every card glides from one
// improving plan to the next, blue while it moves, and the ones that find a
// slot fly out of the tray. When the last plan lands, a jade sweep runs
// across the board: emerald resolves.
use super::board::{self, card, card_rgb, rect_of, X0, X1};
use super::{dot_grid, scan, BLUE, JADE, NIGHT, PARCH, PICTURE_H, SAGE};
use crate::data::Data;
use crate::gfx::{Frame, W};
use crate::timeline::{ease_out, smooth, span, Now};

/// Seconds of search per second of picture.
const PACE: f32 = 6.0;
/// The tray of operations without a slot, in the header.
const TRAY_X: f32 = 790.0;
const TRAY_Y: f32 = 108.0;

pub fn draw(f: &mut Frame, data: &Data, n: Now, t: f32) {
    f.rect(0, 0, W as i32, PICTURE_H, NIGHT, 1.0);
    dot_grid(f, JADE, 0.08, t);
    let s = n.local * PACE;
    let snaps = &data.solve;
    let first = snaps[0].t;
    let open = span(n.local, 0.0, 0.6);
    board::grid(f, open, |_| 1.0, data.lanes.len());

    // Which plan is up, and how far towards the next.
    let i = snaps.iter().rposition(|p| p.t <= s);
    let tray_slot = |rank: usize| (TRAY_X + 160.0 + rank as f32 * 16.0, TRAY_Y);
    let mut waiting = Vec::new();
    match i {
        None => {
            // The first pass: every operation the first plan places pours
            // in from the left, the earliest first.
            let k = s / first;
            let p = &snaps[0];
            for (task, slot) in p.pos.iter().enumerate() {
                match slot {
                    Some(slot) => {
                        let (x, y, w) = rect_of(*slot);
                        let delay = slot.1 as f32 / board::SPAN as f32 * 0.55;
                        let e = ease_out((k - delay) / 0.45);
                        if e <= 0.0 {
                            continue;
                        }
                        let from = X0 as f32 - 60.0;
                        let cx = from + (x - from) * e;
                        card(
                            f,
                            cx,
                            y,
                            w,
                            card_rgb(data, task),
                            e,
                            Some((BLUE, 1.0 - e)),
                            0.0,
                        );
                    }
                    None => waiting.push(task),
                }
            }
        }
        Some(i) => {
            let a = &snaps[i];
            let (b, k) = match snaps.get(i + 1) {
                Some(b) => (b, smooth((s - a.t) / (b.t - a.t))),
                None => (a, 1.0),
            };
            let sweep = span(n.local, 18.6, 19.8);
            let sweep_x = X0 as f32 + (X1 - X0 + 200) as f32 * sweep;
            let mut rank_a = 0;
            for task in 0..data.tasks.len() {
                let (pa, pb) = (a.pos[task], b.pos[task]);
                let rgb = card_rgb(data, task);
                match (pa, pb) {
                    (Some(pa), Some(pb)) => {
                        let moving = pa != pb;
                        let (x, y, w, lift) = if moving {
                            board::travel(pa, pb, k)
                        } else {
                            let (x, y, w) = rect_of(pa);
                            (x, y, w, 0.0)
                        };
                        let mut edge = if moving {
                            Some((BLUE, 0.9 * (1.0 - k) + 0.2))
                        } else {
                            None
                        };
                        if sweep > 0.0 {
                            let d = (sweep_x - x).abs();
                            if d < 160.0 {
                                edge = Some((JADE, 1.0 - d / 160.0));
                            }
                        }
                        card(f, x, y, w, rgb, open, edge, lift);
                    }
                    (None, Some(pb)) => {
                        // Out of the tray into its slot.
                        let (tx, ty) = tray_slot(rank_a);
                        let (x1, y1, w1) = rect_of(pb);
                        let e = smooth(k);
                        let arc = -80.0 * (k * std::f32::consts::PI).sin();
                        let x = tx + (x1 - tx) * e;
                        let y = ty + (y1 - ty) * e + arc;
                        let w = 12.0 + (w1 - 12.0) * e;
                        card(f, x, y, w, rgb, 1.0, Some((JADE, 1.0)), 6.0 * (1.0 - e));
                        rank_a += 1;
                    }
                    (Some(pa), None) => {
                        let (x, y, w) = rect_of(pa);
                        card(f, x, y, w, rgb, open, None, 0.0);
                    }
                    (None, None) => waiting.push(task),
                }
            }
        }
    }
    // The tray.
    let a = span(n.local, 0.3, 0.8);
    if !waiting.is_empty() || i.is_none() {
        f.text(TRAY_X as i32, TRAY_Y as i32, "WAITING", 2, SAGE, a);
    }
    for (rank, task) in waiting.iter().enumerate() {
        let (x, y) = tray_slot(rank);
        let rgb = card_rgb(data, *task);
        card(
            f,
            x,
            y,
            12.0,
            rgb,
            a,
            Some((BLUE, 0.6 + 0.4 * (t * 6.0 + rank as f32).sin().abs())),
            0.0,
        );
    }

    // The score and the clock.
    let p = match i {
        Some(i) => &snaps[i],
        None => &snaps[0],
    };
    let shown = if i.is_some() {
        p.placed
    } else {
        (p.placed as f32 * (s / first).min(1.0)) as usize
    };
    let pa = span(n.local, 0.2, 0.8);
    board::score(f, p.hard, p.medium, p.soft, shown, data.tasks.len(), pa, t);
    let pts: Vec<(f32, f32)> = snaps.iter().map(|q| (q.t, q.soft as f32)).collect();
    board::curve(f, &pts, 1250.0, 132.0, 620.0, 18.0, s, JADE, pa);
    if n.local < 14.8 {
        let secs = s.min(120.0) as i32;
        let clock = format!("SEARCH {}:{:02} / 2:00", secs / 60, secs % 60);
        f.text(
            660,
            56,
            &clock,
            2,
            PARCH,
            pa * (1.0 - span(n.local, 14.3, 14.8)),
        );
        f.text(
            660,
            80,
            "SHOWN 6X",
            1,
            SAGE,
            pa * (1.0 - span(n.local, 14.3, 14.8)),
        );
    }
    scan(f, t, 1.0);
}
