// The repair: the broken week, the line the replan keeps, and what moved.
// The five coating jobs without powder glow red; the work that starts before
// Thursday 06:00 dims behind the line and stays where it is. The search's
// three minutes draw as its hard score's curve; then the replay of what
// changed: the five coating jobs one at a time, each lifted out, carried and
// set down where there is powder for it, named as it moves; then the other
// ninety-one together. When they have landed the score turns to the
// repaired plan's and a jade sweep runs across: emerald resolves.
use super::board::{self, card, card_rgb, rect_of, x_of, LANE_Y0, X0, X1};
use super::powder::when;
use super::{dot_grid, scan, BLUE, JADE, NIGHT, PARCH, PICTURE_H, RED, SAGE};
use crate::data::Data;
use crate::gfx::{Frame, W};
use crate::timeline::{ease_out, pulse, span, Now};

const ONE_BY_ONE: f32 = 4.6;
const EACH: f32 = 1.6;
const ALL_AT: f32 = 12.8;
const LANDED: f32 = 15.2;

pub fn draw(
    f: &mut Frame,
    data: &Data,
    n: Now,
    t: f32,
    masks: &mut Vec<(char, demofx::font::Mask)>,
) {
    let _ = masks;
    let local = n.local;
    f.rect(0, 0, W as i32, PICTURE_H, NIGHT, 1.0);
    dot_grid(f, JADE, 0.08, t);
    let (a, b) = (data.issued(), data.repaired());
    let stock = &data.stock;
    let open = span(local, 0.0, 0.6);
    board::grid(f, open, |_| 1.0, data.lanes.len());

    // The jobs without powder, in time order, then everything else that moved.
    let mut short: Vec<(i32, usize)> = stock
        .parts
        .iter()
        .flat_map(|p| p.short(stock.delivery_to as f32, false))
        .map(|u| (u.issued, u.task))
        .collect();
    short.sort();
    let order: Vec<usize> = short.iter().map(|s| s.1).collect();
    let bottom = board::lane_y(data.lanes.len() - 1) + board::LANE_H;

    // The kept side of the line, dimmed and hatched.
    let keep_x = x_of(stock.keep as f32) as i32;
    let line = ease_out(span(local, 0.6, 1.8));
    let veil = 0.5 * span(local, 1.2, 2.2);
    let sweep = span(local, LANDED, LANDED + 1.2);
    let sweep_x = X0 as f32 + (X1 - X0 + 300) as f32 * sweep;
    for (task, slot) in a.pos.iter().enumerate() {
        let Some(from) = *slot else { continue };
        let to = b.pos[task].unwrap();
        let rgb = card_rgb(data, task);
        let is_short = order.contains(&task);
        // When this card moves: the short ones one at a time, the rest
        // together, staggered down the lanes.
        let start = match order.iter().position(|x| *x == task) {
            Some(rank) => ONE_BY_ONE + rank as f32 * EACH,
            None => ALL_AT + from.0 as f32 * 0.045,
        };
        let dur = if is_short { EACH - 0.25 } else { 1.0 };
        let k = span(local, start, start + dur);
        let moving = from != to;
        let (x, y, w, lift) = if moving {
            board::travel(from, to, k)
        } else {
            let (x, y, w) = rect_of(from);
            (x, y, w, 0.0)
        };
        let mut edge = None;
        if is_short {
            edge = Some(if k >= 1.0 {
                (
                    JADE,
                    0.5 + 0.5 * (1.0 - span(local, start + dur, start + dur + 1.5)),
                )
            } else if k > 0.0 {
                (BLUE, 1.0)
            } else {
                (RED, 0.6 + 0.4 * pulse(t))
            });
        } else if moving && k > 0.0 && k < 1.0 {
            edge = Some((BLUE, 0.8));
        } else if moving && k >= 1.0 && local < LANDED {
            edge = Some((JADE, 0.4));
        }
        if sweep > 0.0 && sweep < 1.0 {
            let d = (sweep_x - x).abs();
            if d < 180.0 {
                edge = Some((JADE, 1.0 - d / 180.0));
            }
        }
        if is_short && k > 0.0 && k < 1.0 {
            // Where it was, in red, and where it goes, in jade.
            let (fx, fy, fw) = rect_of(from);
            f.frame_rect(
                fx as i32 - 2,
                fy as i32 - 2,
                (fx + fw) as i32 + 2,
                fy as i32 + 20,
                RED,
                0.8 * (1.0 - k),
            );
            let (tx, ty, tw) = rect_of(to);
            let ghost = 0.5 + 0.5 * pulse(t);
            f.frame_rect(
                tx as i32 - 3,
                ty as i32 - 3,
                (tx + tw) as i32 + 3,
                ty as i32 + 21,
                JADE,
                ghost,
            );
            f.glow(x + w / 2.0, y + 9.0 - lift * 2.0, 60.0, BLUE, 0.35);
        }
        let dim = if (from.1) < stock.keep && !moving {
            1.0 - veil
        } else {
            1.0
        };
        card(f, x, y, w, rgb, open * dim, edge, lift * 2.0);
        if moving && k > 0.0 && k < 1.0 {
            let (fx, fy, fw) = rect_of(from);
            board::trail(
                f,
                (fx + fw / 2.0, fy + 9.0),
                (x + w / 2.0, y + 9.0 - lift * 2.0),
                BLUE,
                1.0 - k,
            );
        }
    }
    // Hatching over the kept side.
    if veil > 0.0 {
        let mut x = X0 - (bottom - LANE_Y0);
        while x < keep_x {
            let x0 = x.max(X0);
            let x1 = (x + (bottom - LANE_Y0)).min(keep_x);
            if x1 > x0 {
                let y0 = LANE_Y0 + (x0 - x);
                let y1 = LANE_Y0 + (x1 - x);
                f.line(
                    x0 as f32,
                    y0 as f32,
                    x1 as f32,
                    y1 as f32,
                    1.0,
                    SAGE,
                    veil * 0.25,
                );
            }
            x += 22;
        }
    }
    // The line itself, and the delivery's.
    if line > 0.0 {
        let y1 = LANE_Y0 as f32 + (bottom - LANE_Y0) as f32 * line;
        f.rect(keep_x - 1, LANE_Y0 - 8, keep_x + 2, y1 as i32, PARCH, 1.0);
        f.glow(keep_x as f32, y1, 40.0, PARCH, 0.5 * (1.0 - line) + 0.1);
        f.text(keep_x - 70, 186, "KEPT <", 1, PARCH, line);
        let dx = x_of(stock.delivery_to as f32) as i32;
        f.dashed(
            dx,
            LANE_Y0 - 8,
            dx + 2,
            (y1 as i32).max(LANE_Y0),
            6,
            4,
            JADE,
            0.9,
        );
        f.text(dx + 6, 186, "POWDER 18:00", 1, JADE, line);
    }

    // The search: its hard score over the three minutes.
    let pa = span(local, 0.3, 0.9);
    let pts: Vec<(f32, f32)> = data.replan.iter().map(|q| (q.t, q.hard as f32)).collect();
    let upto = 180.0 * span(local, 1.0, 4.0);
    let tmax = pts.last().unwrap().0;
    let mut pts = pts;
    pts.push((180.0, pts.last().unwrap().1));
    let _ = tmax;
    board::curve(
        f,
        &pts,
        1250.0,
        132.0,
        620.0,
        18.0,
        upto,
        if upto < 42.0 { RED } else { JADE },
        pa,
    );
    f.text(
        790,
        108,
        "SEARCH 3:00",
        2,
        PARCH,
        pa * (1.0 - span(local, 4.4, 4.8)),
    );
    let (hard, medium, soft) = if local >= LANDED {
        (b.hard, b.medium, b.soft)
    } else {
        (stock.broken_hard, 0, a.soft)
    };
    board::score(f, hard, medium, soft, 409, data.tasks.len(), pa, t);

    // The name of the job on the move, under the board.
    for (rank, task) in order.iter().enumerate() {
        let start = ONE_BY_ONE + rank as f32 * EACH;
        let k = span(local, start - 0.1, start + 0.2)
            * (1.0 - span(local, start + EACH - 0.2, start + EACH));
        if k <= 0.0 {
            continue;
        }
        let (from, to) = (a.pos[*task].unwrap(), b.pos[*task].unwrap());
        let paint = data.tasks[*task].paint.map_or("", |p| p.name());
        let text = format!(
            "{} -> {}   {:<5} {}",
            when(from.1),
            when(to.1),
            paint,
            data.tasks[*task].product
        );
        // Typed in as it starts to move.
        let shown = ((local - start + 0.1) * 60.0).max(0.0) as usize;
        let typed: String = text.chars().take(shown).collect();
        f.text(X0, 890, &typed, 2, PARCH, k);
        f.rect(
            X0 - 18,
            894,
            X0 - 6,
            906,
            RED,
            k * (1.0 - span(local, start + 1.0, start + 1.3)),
        );
        f.rect(
            X0 - 18,
            894,
            X0 - 6,
            906,
            JADE,
            k * span(local, start + 1.0, start + 1.3),
        );
    }
    if (ALL_AT - 0.2..LANDED + 0.4).contains(&local) {
        let k = span(local, ALL_AT - 0.2, ALL_AT + 0.2) * (1.0 - span(local, LANDED, LANDED + 0.4));
        let rest = stock.moved - order.len();
        f.text(
            X0,
            890,
            &format!("AND {rest} MORE, AROUND THEM"),
            2,
            BLUE,
            k,
        );
    }
    scan(f, t, 1.0);
}
