// Month view: September 2026 as Planner123 draws it (weekday header, a
// five-week grid, today in a mint pill, adjacent-month days very dim, dots
// for days with events) -- with every day cell turned into a window onto one
// shared plasma, and the PLANNER123 banner wobbling across on top.
use ratatui::buffer::Buffer;
use ratatui::style::{Modifier, Style};

use crate::canvas::{c, lerp_rgb, ramp};
use crate::data::{ACCENT, ADJACENT, BG, BLUE, BUSY, CYAN, DIM, FG, TEAL, TODAY};
use crate::tui::{bold, fg, put, text, CONTENT};

const CELL_W: u16 = 14;
const CELL_H: u16 = 6;
const X0: u16 = CONTENT.x + 3;
const Y0: u16 = CONTENT.y + 1;
const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

pub const BANNER: &str = "\
██████╗ ██╗      █████╗ ███╗   ██╗███╗   ██╗███████╗██████╗  ██╗██████╗ ██████╗
██╔══██╗██║     ██╔══██╗████╗  ██║████╗  ██║██╔════╝██╔══██╗███║╚════██╗╚════██╗
██████╔╝██║     ███████║██╔██╗ ██║██╔██╗ ██║█████╗  ██████╔╝╚██║ █████╔╝ █████╔╝
██╔═══╝ ██║     ██╔══██║██║╚██╗██║██║╚██╗██║██╔══╝  ██╔══██╗ ██║██╔═══╝  ╚═══██╗
██║     ███████╗██║  ██║██║ ╚████║██║ ╚████║███████╗██║  ██║ ██║███████╗██████╔╝
╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═╝╚══════╝╚═════╝";

/// Day of month in grid cell (week, weekday); Monday Aug 31 starts the grid.
fn date(week: u16, wd: u16) -> (u16, bool) {
    let n = week * 7 + wd; // 0 = Aug 31
    if n == 0 {
        (31, false)
    } else if n <= 30 {
        (n, true)
    } else {
        (n - 30, false)
    }
}

fn plasma(x: f32, y: f32, t: f32) -> f32 {
    let v = (x * 0.11 + t).sin()
        + (y * 0.21 - t * 1.3).sin()
        + ((x + y * 1.6) * 0.07 + t * 0.7).sin()
        + ((x * x + y * y * 3.2).sqrt() * 0.09 - t * 1.1).sin();
    v * 0.125 + 0.5
}

/// `plasma_amt` fades the plasma in; `pulse` (0..1) brightens it on kicks.
pub fn draw(buf: &mut Buffer, t: f32, plasma_amt: f32, pulse: f32) {
    for (i, name) in WEEKDAYS.iter().enumerate() {
        let x = X0 + i as u16 * CELL_W;
        text(
            buf,
            x + 1,
            CONTENT.y,
            name,
            bold(if i >= 5 { DIM } else { BLUE }),
        );
    }
    let stops = [BG, 0x162A55, BLUE, TEAL, ACCENT, 0xF0FFF4];
    let shades = [' ', '░', '▒', '▓', '█'];
    for week in 0..5u16 {
        for wd in 0..7u16 {
            let (x0, y0) = (X0 + wd * CELL_W, Y0 + week * CELL_H);
            if plasma_amt > 0.0 {
                for y in 1..CELL_H {
                    for x in 0..CELL_W - 1 {
                        let (px, py) = ((x0 + x) as f32, (y0 + y) as f32);
                        let v = plasma(px, py, t);
                        let lit = (v * (0.75 + 0.35 * pulse)).min(1.0);
                        let ch = shades[(lit * 4.99) as usize];
                        let col = lerp_rgb(BG, ramp(&stops, lit), plasma_amt * 0.7);
                        put(buf, x0 + x, y0 + y, ch, col);
                    }
                }
            }
            // Cell frame: a faint rule under each week, between days.
            for x in 0..CELL_W {
                put(
                    buf,
                    x0 + x,
                    y0 + CELL_H - 1 + 1,
                    '─',
                    lerp_rgb(BG, DIM, 0.3),
                );
            }
            let (day, this_month) = date(week, wd);
            let label = format!("{day:>2}");
            let st = if this_month && day as usize == 14 + TODAY {
                Style::default()
                    .bg(c(ACCENT))
                    .fg(c(BG))
                    .add_modifier(Modifier::BOLD)
            } else if !this_month {
                fg(ADJACENT).bg(c(BG))
            } else {
                fg(if wd >= 5 { DIM } else { FG }).bg(c(BG))
            };
            text(buf, x0, y0, &label, st);
            // Dots for days with events (week of the 14th only).
            if this_month && (14..=20).contains(&day) {
                let n = BUSY.iter().filter(|e| e.day as u16 == day - 14).count();
                for k in 0..n.min(4) {
                    put(buf, x0 + 3 + k as u16 * 2, y0, '•', TEAL);
                }
            }
        }
    }
}

/// The banner over the grid, each column on its own sine; `amount` slides it
/// in from the right.
pub fn banner(buf: &mut Buffer, t: f32, amount: f32) {
    let lines: Vec<Vec<char>> = BANNER.lines().map(|l| l.chars().collect()).collect();
    let w = lines[0].len() as f32;
    let centre = CONTENT.x as f32 + (CONTENT.width as f32 - w) / 2.0;
    let x0 = centre + (1.0 - amount) * (CONTENT.width as f32 + 10.0);
    for (r, line) in lines.iter().enumerate() {
        for (i, &ch) in line.iter().enumerate() {
            if ch == ' ' {
                continue;
            }
            let x = x0 + i as f32;
            if x < CONTENT.x as f32 || x >= (CONTENT.x + CONTENT.width) as f32 {
                continue;
            }
            let wob = (i as f32 * 0.16 + t * 3.1).sin() * 2.2;
            let y = (CONTENT.y as f32 + 13.0 + r as f32 + wob).round() as u16;
            let col = ramp(
                &[ACCENT, CYAN, TEAL, ACCENT],
                (i as f32 / w + t * 0.25).fract(),
            );
            let cell = &mut buf[(x as u16, y)];
            cell.set_char(ch).set_fg(c(col)).set_bg(c(BG));
        }
    }
}
