// Week view, laid out like Planner123's week_view.rs: a day header row (today
// in a mint pill), hour labels down the left, faint hour rules, events as a
// calendar-coloured ▌ rail with the title and start time, and the bold mint
// "now" beam. Two text rows per hour from 08:00.
use ratatui::buffer::Buffer;
use ratatui::style::Style;

use crate::canvas::{c, lerp_rgb, ramp};
use crate::data::{hhmm, Event, ACCENT, BG, BLUE, CALENDARS, DAYS, DIM, FG, TEAL, TODAY};
use crate::tui::{bold, fg, put, put_bg, text, CONTENT};

pub const LABEL_W: u16 = 6;
pub const DAY_W: u16 = 14;
pub const GRID_X: u16 = CONTENT.x + LABEL_W;
pub const HOUR0: f32 = 8.0;
pub const HOUR1: f32 = 24.0;
const GRID_Y: u16 = CONTENT.y + 1;

pub fn row_of(hour: f32) -> u16 {
    GRID_Y + ((hour - HOUR0) * 2.0).round() as u16
}

pub fn col_of(day: usize) -> u16 {
    GRID_X + day as u16 * DAY_W
}

/// Copper bars behind the grid: three raster bars bouncing through the
/// hours, tinting only the cell backgrounds so the calendar stays legible.
pub fn copper(buf: &mut Buffer, t: f32, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    let stops = [BG, 0x10324A, TEAL, ACCENT, 0xEFFFF4];
    let span = (row_of(HOUR1) - GRID_Y) as f32;
    for y in GRID_Y..row_of(HOUR1) {
        let mut v = 0.0f32;
        for k in 0..3 {
            let centre = span * (0.5 + 0.45 * (t * (0.9 + k as f32 * 0.23) + k as f32 * 2.1).sin());
            let d = ((y - GRID_Y) as f32 - centre).abs();
            v = v.max((1.0 - d / 3.2).max(0.0));
        }
        if v <= 0.0 {
            continue;
        }
        let col = lerp_rgb(BG, ramp(&stops, v), 0.42 * amount);
        for x in CONTENT.x..CONTENT.x + CONTENT.width {
            put_bg(buf, x, y, col);
        }
    }
}

pub fn grid(buf: &mut Buffer, now_hour: f32) {
    for (d, name) in DAYS.iter().enumerate() {
        let st = if d == TODAY {
            Style::default()
                .bg(c(ACCENT))
                .fg(c(BG))
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            bold(if d >= 5 { DIM } else { BLUE })
        };
        let label = format!("{:^w$}", name, w = DAY_W as usize - 2);
        text(buf, col_of(d) + 1, CONTENT.y, &label, st);
    }
    let mut h = HOUR0;
    while h < HOUR1 {
        let y = row_of(h);
        text(buf, CONTENT.x, y, &hhmm(h), fg(DIM));
        for x in GRID_X..CONTENT.x + CONTENT.width {
            put(buf, x, y, '─', lerp_rgb(BG, DIM, 0.28));
        }
        h += 1.0;
    }
    beam(buf, now_hour, 1.0);
}

/// The now line. `gain` lets the boot fade it in.
pub fn beam(buf: &mut Buffer, now_hour: f32, gain: f32) {
    let y = GRID_Y + ((now_hour - HOUR0) * 2.0) as u16;
    let col = lerp_rgb(BG, ACCENT, gain);
    for x in CONTENT.x..CONTENT.x + CONTENT.width {
        put(buf, x, y, '━', col);
    }
    text(buf, CONTENT.x, y, &hhmm(now_hour), bold(col));
}

pub enum Kind {
    /// A plain event.
    Solid,
    /// A proposal slot not yet applied: dashed mint rail, pulsing.
    Ghost(f32),
    /// Just landed or just applied: a lit block.
    Lit(f32),
}

pub fn event(buf: &mut Buffer, e: &Event, kind: Kind) {
    let x = col_of(e.day);
    let y0 = row_of(e.start);
    let rows = ((e.hours * 2.0).round() as u16).max(2);
    let base = CALENDARS[e.calendar].color;
    let (rail, glyph, glow) = match kind {
        Kind::Solid => (base, '▌', 0.0),
        Kind::Ghost(p) => (lerp_rgb(DIM, ACCENT, p), '┊', 0.12 * p),
        Kind::Lit(p) => (lerp_rgb(base, 0xFFFFFF, p * 0.5), '▌', p),
    };
    for r in 0..rows {
        put(buf, x, y0 + r, glyph, rail);
        if glow > 0.0 {
            for cx in 1..DAY_W - 1 {
                put_bg(buf, x + cx, y0 + r, lerp_rgb(BG, ACCENT, 0.32 * glow));
            }
        }
    }
    let title: String = e.title.chars().take(DAY_W as usize - 2).collect();
    let title = if e.title.chars().count() > DAY_W as usize - 2 {
        format!("{}…", &title[..title.len() - 1])
    } else {
        title
    };
    text(buf, x + 1, y0, &title, fg(FG));
    text(buf, x + 1, y0 + 1, &hhmm(e.start), fg(DIM));
}
