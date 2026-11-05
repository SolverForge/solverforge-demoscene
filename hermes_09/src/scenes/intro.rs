// Intro: the install line typed into a shell, then a week. Meetings appear,
// five task blocks drop onto the first time that looks free -- on top of the
// meetings, so they glow red and the score goes negative -- and then slide,
// one by one, to where the solver puts them. The score is recomputed from the
// blocks' positions every frame.
use super::{dot_grid, typed, EMERALD, LIME, MINT, PAPER, RED, SLATE};
use crate::gfx::{advance, ease_out, Frame, W};
use crate::statue::Statue;
use crate::timeline::{smooth, Now, BEAT};
use crate::week::{hard, Slot, DAYS, DAY_FROM, DAY_TO, MEETINGS, TASKS};
use demofx::CELL_H;

const INSTALL: &str = "$ hermes plugins install blackopsrepl/hermes-solverforge-planner";

/// Grid geometry.
const GX0: f32 = 420.0;
const GX1: f32 = 1840.0;
const GY0: f32 = 250.0;
const GY1: f32 = 1000.0;

const GRID_IN: f32 = 2.6;
const MEETINGS_IN: f32 = 3.2;
const DROP_AT: f32 = 4.5;
const SOLVE_AT: f32 = 7.6;

fn col_w() -> f32 {
    (GX1 - GX0) / DAYS.len() as f32
}

fn hour_h() -> f32 {
    (GY1 - GY0) / (DAY_TO - DAY_FROM)
}

fn rect_of(s: &Slot) -> (f32, f32, f32, f32) {
    let x0 = GX0 + s.day * col_w() + 6.0;
    let y0 = GY0 + (s.start - DAY_FROM) * hour_h() + 2.0;
    (x0, y0, x0 + col_w() - 12.0, y0 + s.hours * hour_h() - 4.0)
}

/// When task i lands on its first guess (one per beat), then on its solved slot.
fn drop_time(i: usize) -> f32 {
    DROP_AT + i as f32 * BEAT
}

fn solve_time(i: usize) -> f32 {
    SOLVE_AT + i as f32 * BEAT * 1.5
}

pub fn landings() -> Vec<f32> {
    let mut out: Vec<f32> = (0..TASKS.len()).map(|i| drop_time(i) + 0.35).collect();
    out.extend((0..TASKS.len()).map(|i| solve_time(i) + 0.6));
    out
}

/// Where task i is drawn at time t (None before it appears) and whether it
/// has started moving to its solved slot.
fn position(i: usize, t: f32) -> Option<Slot> {
    let task = &TASKS[i];
    let d = t - drop_time(i);
    if d < 0.0 {
        return None;
    }
    let s = (t - solve_time(i)) / 0.6;
    if s <= 0.0 {
        // Dropping the last half hour into place, inside the grid.
        let fall = ease_out((d / 0.35).min(1.0));
        let mut slot = task.first;
        slot.start = task.first.start - 0.5 * (1.0 - fall);
        return Some(slot);
    }
    let k = smooth(s);
    Some(Slot {
        day: task.first.day + (task.solved.day - task.first.day) * k,
        start: task.first.start + (task.solved.start - task.first.start) * k,
        hours: task.first.hours,
    })
}

pub fn draw(f: &mut Frame, statue: &Statue, n: Now, t: f32) -> f32 {
    f.gradient(&[0x020503, 0x03100a, 0x020503]);
    dot_grid(f, EMERALD, 0.18 * smooth(t / 1.0));
    // The statue fades in under the install line, at the live take's strength,
    // and stays through the intro into the take.
    statue.draw(f, MINT, 0.03 * smooth(t / 3.0), t);
    let fade_out = smooth((n.len - n.local) / 0.5);

    // The install line, typed, then pushed up out of the way.
    let shell = 1.0 - smooth((t - GRID_IN) / 0.6);
    if shell > 0.0 {
        let line = typed(INSTALL, 0.3, 40.0, t);
        // Measured with the font's own advance, so the line always fits.
        let w = INSTALL.chars().count() as i32 * advance(2);
        let x = (W as i32 - w) / 2;
        f.text(x, 524, &line, 2, LIME, shell);
        if (t * 2.0).fract() < 0.5 {
            let cx = x + line.chars().count() as i32 * advance(2);
            f.rect(
                cx,
                522,
                cx + advance(2) - 4,
                524 + 2 * CELL_H as i32,
                LIME,
                shell,
            );
        }
        if t > 2.0 {
            f.text_centered(
                W as i32 / 2,
                580,
                "solverforge-planner 0.3.0 enabled",
                2,
                MINT,
                shell * smooth((t - 2.0) / 0.3),
            );
        }
    }

    let grid = smooth((t - GRID_IN) / 0.8) * fade_out;
    if grid <= 0.0 {
        return 1.0;
    }
    // Columns, hour lines, labels.
    for (d, name) in DAYS.iter().enumerate() {
        let x = GX0 + d as f32 * col_w();
        f.rect(
            x as i32,
            GY0 as i32,
            x as i32 + 1,
            GY1 as i32,
            EMERALD,
            0.35 * grid,
        );
        f.text_centered(
            (x + col_w() / 2.0) as i32,
            GY0 as i32 - 34,
            name,
            3,
            MINT,
            grid,
        );
    }
    f.rect(
        GX1 as i32,
        GY0 as i32,
        GX1 as i32 + 1,
        GY1 as i32,
        EMERALD,
        0.35 * grid,
    );
    for h in 0..=(DAY_TO - DAY_FROM) as i32 {
        let y = GY0 + h as f32 * hour_h();
        f.rect(
            GX0 as i32,
            y as i32,
            GX1 as i32,
            y as i32 + 1,
            EMERALD,
            0.18 * grid,
        );
        if h < (DAY_TO - DAY_FROM) as i32 {
            f.text(
                GX0 as i32 - 96,
                y as i32 + 6,
                &format!("{:02}:00", DAY_FROM as i32 + h),
                2,
                SLATE,
                grid,
            );
        }
    }

    // Meetings: hatched, fixed.
    for (i, m) in MEETINGS.iter().enumerate() {
        let a = smooth((t - MEETINGS_IN - i as f32 * 0.08) / 0.3) * grid;
        if a <= 0.0 {
            continue;
        }
        let (x0, y0, x1, y1) = rect_of(&m.slot);
        f.rect(x0 as i32, y0 as i32, x1 as i32, y1 as i32, SLATE, 0.28 * a);
        // Diagonal hatching, clipped to the block: busy time, not free.
        for y in y0 as i32..y1 as i32 {
            for x in x0 as i32..x1 as i32 {
                if (x + y).rem_euclid(14) < 2 {
                    f.blend(x, y, SLATE, 0.35 * a);
                }
            }
        }
        f.frame_rect(x0 as i32, y0 as i32, x1 as i32, y1 as i32, SLATE, 0.8 * a);
        f.text(x0 as i32 + 10, y0 as i32 + 6, m.title, 2, PAPER, 0.7 * a);
    }

    // Task blocks, red while they sit on anything.
    let placed: Vec<(usize, Slot)> = (0..TASKS.len())
        .filter_map(|i| position(i, t).map(|s| (i, s)))
        .collect();
    for &(i, s) in &placed {
        let grid = grid * smooth((t - drop_time(i)) / 0.2);
        let clash = MEETINGS.iter().any(|m| s.overlaps(&m.slot))
            || placed.iter().any(|&(j, o)| j != i && s.overlaps(&o));
        let rgb = if clash { RED } else { LIME };
        let (x0, y0, x1, y1) = rect_of(&s);
        f.rect(x0 as i32, y0 as i32, x1 as i32, y1 as i32, rgb, 0.32 * grid);
        f.frame_rect(x0 as i32, y0 as i32, x1 as i32, y1 as i32, rgb, grid);
        f.rect(x0 as i32, y0 as i32, x0 as i32 + 5, y1 as i32, rgb, grid);
        f.text(
            x0 as i32 + 14,
            y0 as i32 + 8,
            TASKS[i].title,
            2,
            PAPER,
            grid,
        );
        if clash {
            f.glow((x0 + x1) / 2.0, (y0 + y1) / 2.0, 90.0, RED, 0.12 * grid);
        }
    }

    // The score, from the same positions.
    if !placed.is_empty() {
        let slots: Vec<Slot> = placed.iter().map(|&(_, s)| s).collect();
        let h = hard(&slots);
        let all_in = placed.len() == TASKS.len() && t > solve_time(TASKS.len() - 1) + 0.6;
        let text = format!(
            "SCORE {}hard/0medium/0soft",
            if h == 0 { "0".into() } else { format!("-{h}") }
        );
        let rgb = if h == 0 { LIME } else { RED };
        f.text(GX0 as i32, 1030, &text, 3, rgb, grid);
        if all_in {
            let pulse = 0.5 + 0.5 * (t * 6.0).sin();
            let kept = "EVERY HARD RULE KEPT";
            f.text(
                GX1 as i32 - kept.chars().count() as i32 * advance(3),
                1030,
                kept,
                3,
                PAPER,
                grid * (0.6 + 0.4 * pulse),
            );
        }
    }
    1.0
}
