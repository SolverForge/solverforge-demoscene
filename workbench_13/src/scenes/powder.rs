// The powder: the four powders' stock along the week, drawn on the board's
// time axis, over the two powder lines. The deliveries come on Tuesday at
// six; the supplier moves them to Thursday at six in the evening, and the
// markers slide there. Red, blue and grey then run below zero on Thursday
// morning: the lines turn red under zero, and the five coating jobs that
// would start with nothing to spray light up on the lines below, listed with
// their times. The rule that breaks is the one the check named.
use super::board::{self, card, card_rgb, lane_y, rect_of, x_of, LANE_H, X0, X1};
use super::{dot_grid, scan, JADE, NIGHT, PARCH, PICTURE_H, RED, SAGE};
use crate::data::{Data, Part, DAY};
use crate::gfx::{mix, Frame, W};
use crate::timeline::{ease_out, pulse, span, Now};

const CHART_Y0: i32 = 196;
const CHART_H: i32 = 84;
const CHART_GAP: i32 = 14;
/// Where the powder lines sit while the stock is read.
const LINES_Y: i32 = 612;
const LIST_Y: i32 = 704;
const PAINT_LANES: [usize; 2] = [12, 13];

pub fn when(min: i32) -> String {
    let abs = min + 6 * 60;
    let day = ["MON", "TUE", "WED", "THU", "FRI", "SAT"][(abs / DAY) as usize];
    let m = abs % DAY;
    format!("{day} {:02}:{:02}", m / 60, m % 60)
}

/// The delivery's minute at this moment of the scene.
pub fn delivery(data: &Data, local: f32) -> f32 {
    let s = &data.stock;
    s.delivery_from as f32
        + (s.delivery_to - s.delivery_from) as f32 * ease_out(span(local, 4.0, 6.4))
}

pub fn draw(
    f: &mut Frame,
    data: &Data,
    n: Now,
    t: f32,
    masks: &mut Vec<(char, demofx::font::Mask)>,
) {
    let _ = masks;
    f.rect(0, 0, W as i32, PICTURE_H, NIGHT, 1.0);
    dot_grid(f, JADE, 0.08, t);
    let local = n.local;
    let issued = data.issued();
    let d = delivery(data, local);
    let broken = local >= 6.4;

    // The board steps back, all but the powder lines, which move down under
    // the charts.
    let fade = 1.0 - span(local, 0.0, 0.8);
    let slide = ease_out(span(local, 0.2, 1.2));
    board::grid(
        f,
        1.0,
        |lane| {
            if PAINT_LANES.contains(&lane) {
                0.0
            } else {
                fade
            }
        },
        data.lanes.len(),
    );
    for (task, slot) in issued.pos.iter().enumerate() {
        let Some(slot) = *slot else { continue };
        let (x, y, w) = rect_of(slot);
        let lane = slot.0 as usize;
        if let Some(k) = PAINT_LANES.iter().position(|l| *l == lane) {
            let to = (LINES_Y + k as i32 * LANE_H + (LANE_H - board::CARD_H) / 2) as f32;
            let y = y + (to - y) * slide;
            let short = broken && is_short(data, task, d);
            let edge = if short {
                Some((RED, 0.6 + 0.4 * pulse(t)))
            } else {
                None
            };
            card(f, x, y, w, card_rgb(data, task), 1.0, edge, 0.0);
        } else {
            card(f, x, y, w, card_rgb(data, task), fade, None, 0.0);
        }
    }
    // The powder lines' own rows and names.
    let la = span(local, 0.8, 1.4);
    for (k, lane) in PAINT_LANES.iter().enumerate() {
        let y = LINES_Y + k as i32 * LANE_H;
        f.rect(X0, y + LANE_H - 1, X1, y + LANE_H, 0x16302a, la);
        f.text(28, y + 5, &data.lanes[*lane].label, 2, PARCH, la);
    }
    let _ = lane_y;

    // The charts.
    let reveal = span(local, 1.2, 3.2);
    for (k, part) in data.stock.parts.iter().enumerate() {
        let y0 = CHART_Y0 + k as i32 * (CHART_H + CHART_GAP);
        chart(f, data, part, y0, d, reveal, broken, t);
    }

    // The jobs without powder, in the order they would start.
    if broken {
        let mut short: Vec<(i32, usize, &Part)> = Vec::new();
        for part in &data.stock.parts {
            for u in part.short(d, false) {
                short.push((u.issued, u.task, part));
            }
        }
        short.sort_by_key(|s| (s.0, s.1));
        for (row, (min, task, part)) in short.iter().enumerate() {
            let a = span(local, 7.0 + row as f32 * 0.35, 7.4 + row as f32 * 0.35);
            let y = LIST_Y + row as i32 * 36;
            f.rect(X0, y + 4, X0 + 14, y + 18, part.powder.rgb(), a);
            f.frame_rect(X0 - 1, y + 3, X0 + 15, y + 19, RED, a);
            let line = format!(
                "{}  {:<5} {}",
                when(*min),
                part.powder.name(),
                data.tasks[*task].product
            );
            f.text(X0 + 30, y + 2, &line, 2, mix(RED, PARCH, 0.55), a);
            // A thread from the row to the card it names.
            let slot = issued.pos[*task].unwrap();
            let (x, _, w) = rect_of(slot);
            let k = PAINT_LANES
                .iter()
                .position(|l| *l == slot.0 as usize)
                .unwrap_or(0);
            let cy = (LINES_Y + k as i32 * LANE_H + LANE_H) as f32;
            let tx = X0 as f32 + 30.0 + line.len() as f32 * 20.0 + 12.0;
            let run = ease_out(span(
                local,
                7.3 + row as f32 * 0.35,
                7.9 + row as f32 * 0.35,
            ));
            let (ex, ey) = (x + w / 2.0, cy);
            let (sx, sy) = (tx, y as f32 + 11.0);
            f.line(
                sx,
                sy,
                sx + (ex - sx) * run,
                sy + (ey - sy) * run,
                1.0,
                RED,
                0.7 * a,
            );
        }
    }

    // The score: the check's, once the deliveries have moved.
    let pa = span(local, 0.3, 0.9);
    let hard = if broken { data.stock.broken_hard } else { 0 };
    board::score(
        f,
        hard,
        0,
        issued.soft,
        issued.placed,
        data.tasks.len(),
        pa,
        t,
    );
    scan(f, t, 1.0);
}

fn is_short(data: &Data, task: usize, d: f32) -> bool {
    data.stock
        .parts
        .iter()
        .any(|p| p.short(d, false).iter().any(|u| u.task == task))
}

/// One powder's stock: a step line that falls at each use and rises when
/// the delivery comes, red where it is below zero.
#[allow(clippy::too_many_arguments)]
fn chart(
    f: &mut Frame,
    data: &Data,
    part: &Part,
    y0: i32,
    d: f32,
    reveal: f32,
    broken: bool,
    t: f32,
) {
    let a = reveal.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    let top = part.init + part.late;
    let lo = -3;
    let y_of =
        |level: i32| y0 as f32 + CHART_H as f32 * (1.0 - (level - lo) as f32 / (top - lo) as f32);
    let zero = y_of(0);
    // Label and frame.
    f.rect(28, y0 + 10, 42, y0 + 24, part.powder.rgb(), a);
    f.frame_rect(27, y0 + 9, 43, y0 + 25, SAGE, a);
    f.text(52, y0 + 8, &part.label, 2, PARCH, a);
    f.text(28, y0 + 34, &format!("{} IN STOCK", part.init), 2, SAGE, a);
    f.text(
        28,
        y0 + 56,
        &format!("+{} DELIVERED", part.late),
        2,
        SAGE,
        a,
    );
    f.dashed(X0, zero as i32, X1, zero as i32, 4, 6, SAGE, 0.5 * a);
    f.text(X0 - 26, zero as i32 - 8, "0", 2, SAGE, 0.6 * a);
    let limit = X0 as f32 + (X1 - X0) as f32 * reveal;

    // Events in time: each use, and the delivery.
    let mut events: Vec<(f32, i32, Option<usize>)> = part
        .uses
        .iter()
        .map(|u| (u.issued as f32, -u.qty, Some(u.task)))
        .collect();
    events.push((d, part.late, None));
    events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut level = part.init;
    let mut x = X0 as f32;
    let mut y = y_of(level);
    for (when, delta, task) in &events {
        let nx = x_of(*when).min(limit);
        let rgb = if level < 0 { RED } else { PARCH };
        if nx > x {
            if level < 0 {
                f.rect(x as i32, zero as i32, nx as i32, y as i32, RED, 0.18 * a);
            }
            f.line(x, y, nx, y, if level < 0 { 3.0 } else { 2.0 }, rgb, a);
        }
        if x_of(*when) > limit {
            x = limit;
            break;
        }
        level += delta;
        let ny = y_of(level);
        f.line(nx, y, nx, ny, 2.0, if level < 0 { RED } else { PARCH }, a);
        match task {
            Some(_) => {
                let bad = broken && level < 0;
                let c = if bad {
                    RED
                } else {
                    part.powder.rgb().max(0x50504c)
                };
                f.rect(
                    nx as i32 - 2,
                    ny as i32 - 2,
                    nx as i32 + 3,
                    ny as i32 + 3,
                    c,
                    a,
                );
                if bad {
                    f.glow(nx, ny, 16.0, RED, 0.5 * a * (0.6 + 0.4 * pulse(t)));
                }
            }
            None => {
                // The delivery: a jade marker and its amount.
                f.rect(
                    nx as i32 - 1,
                    y0,
                    nx as i32 + 1,
                    y0 + CHART_H,
                    JADE,
                    0.8 * a,
                );
                f.text(
                    nx as i32 + 6,
                    y0 + 2,
                    &format!("+{}", part.late),
                    2,
                    JADE,
                    a,
                );
                f.glow(nx, ny, 14.0, JADE, 0.5 * a);
            }
        }
        x = nx;
        y = ny;
    }
    if x < limit {
        let rgb = if level < 0 { RED } else { PARCH };
        f.line(x, y, limit, y, 2.0, rgb, a);
    }
    let _ = data;
}
