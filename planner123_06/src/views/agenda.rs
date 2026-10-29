// Agenda view as Planner123 draws it ("  Today — Tue, Sep 15  " headers, then
// "  ▌ HH:MM-HH:MM  Title (duration)" lines with a [Conferen] project badge),
// scrolling on its own, over a starfield, with the greetings on a sine.
use ratatui::buffer::Buffer;
use ratatui::style::{Modifier, Style};

use crate::canvas::{c, lerp_rgb, ramp, CELL_H, CELL_W};
use crate::data::{
    hhmm, Event, ACCENT, BG, BLUE, BUSY, CALENDARS, CYAN, DIM, FG, GREETINGS, TASKS, TEAL, TODAY,
};
use crate::noise::h2;
use crate::post::Sprite;
use crate::tui::{fg, put, text, CONTENT};

const FRIENDLY: [&str; 7] = [
    "Monday", "Today", "Tomorrow", "Thursday", "Friday", "Saturday", "Sunday",
];
const SHORT: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// The agenda lines: (text spans as (string, style)) from today on, busy
/// events and the applied tasks together, in time order.
fn lines() -> Vec<Vec<(String, Style)>> {
    let mut all: Vec<Event> = BUSY.to_vec();
    all.extend(TASKS.iter().map(|t| t.slot));
    all.retain(|e| e.day >= TODAY);
    all.sort_by(|a, b| (a.day, a.start).partial_cmp(&(b.day, b.start)).unwrap());
    let mut out = Vec::new();
    let mut prev = usize::MAX;
    for e in all {
        if e.day != prev {
            if prev != usize::MAX {
                out.push(vec![]);
            }
            let st = if e.day == TODAY {
                fg(ACCENT).add_modifier(Modifier::BOLD)
            } else {
                fg(BLUE).add_modifier(Modifier::BOLD)
            };
            out.push(vec![(
                format!(
                    "  {} — {}, Sep {}  ",
                    FRIENDLY[e.day],
                    SHORT[e.day],
                    14 + e.day
                ),
                st,
            )]);
            prev = e.day;
        }
        let mins = (e.hours * 60.0).round() as u32;
        let dur = if mins >= 60 && mins.is_multiple_of(60) {
            format!(" ({}h)", mins / 60)
        } else if mins > 60 {
            format!(" ({}h {}m)", mins / 60, mins % 60)
        } else {
            format!(" ({mins}m)")
        };
        let mut row = vec![
            (
                "  ▌".to_string(),
                fg(CALENDARS[e.calendar].color).add_modifier(Modifier::BOLD),
            ),
            (
                format!(
                    " {:12}",
                    format!("{}-{}", hhmm(e.start), hhmm(e.start + e.hours))
                ),
                fg(DIM),
            ),
            (e.title.to_string(), fg(FG)),
            (dur, fg(DIM)),
        ];
        if e.title.contains("onference") {
            row.push((
                " [Conferen]".to_string(),
                Style::default()
                    .bg(c(TEAL))
                    .fg(c(BG))
                    .add_modifier(Modifier::BOLD),
            ));
        }
        out.push(row);
    }
    out
}

pub fn stars(buf: &mut Buffer, t: f32) {
    for i in 0..220 {
        let z = 0.25 + 0.75 * h2(i, 7);
        let span = CONTENT.width as f32;
        let x = (h2(i, 3) * span - t * 14.0 * z).rem_euclid(span);
        let y = h2(i, 5) * CONTENT.height as f32;
        let ch = if z > 0.85 {
            '*'
        } else if z > 0.55 {
            '+'
        } else {
            '.'
        };
        put(
            buf,
            CONTENT.x + x as u16,
            CONTENT.y + y as u16,
            ch,
            lerp_rgb(BG, if i % 5 == 0 { CYAN } else { FG }, z * 0.8),
        );
    }
}

/// The list, scrolled by `scroll` rows, fading out at the edges.
pub fn list(buf: &mut Buffer, scroll: f32) {
    let ls = lines();
    let h = CONTENT.height as i32 - 12;
    let n = ls.len() as f32 + h as f32;
    for (i, row) in ls.iter().enumerate() {
        let y = (i as f32 - scroll.rem_euclid(n) + h as f32) as i32;
        if !(0..h).contains(&y) {
            continue;
        }
        let edge = (y.min(h - 1 - y) as f32 / 3.0).clamp(0.0, 1.0);
        let mut x = CONTENT.x + 4;
        for (s, st) in row {
            let mut st = *st;
            if let Some(col) = st.fg {
                st = st.fg(c(lerp_rgb(BG, crate::canvas::to_rgb(col, FG), edge)));
            }
            text(buf, x, CONTENT.y + y as u16, s, st);
            x += s.chars().count() as u16;
        }
    }
}

/// The greetings, double size, on a sine in pixel space.
pub fn scroller(sprites: &mut Vec<Sprite>, t: f32, amount: f32) {
    let x_min = (CONTENT.x as usize * CELL_W) as f32;
    let x_max = ((CONTENT.x + CONTENT.width) as usize * CELL_W) as f32;
    let base_y = ((CONTENT.y + CONTENT.height - 8) as usize * CELL_H) as f32;
    let step = (CELL_W * 2) as f32;
    let len = GREETINGS.chars().count() as f32 * step;
    let off = t * 190.0;
    for (i, ch) in GREETINGS.chars().enumerate() {
        if ch == ' ' {
            continue;
        }
        let x = x_max + i as f32 * step - off.rem_euclid(len + (x_max - x_min));
        if x < x_min || x > x_max - step {
            continue;
        }
        let y = base_y + (x * 0.012 + t * 3.0).sin() * 46.0;
        let col = ramp(&[ACCENT, CYAN, TEAL, ACCENT], (x / 700.0 + t * 0.2).fract());
        sprites.push(Sprite {
            x,
            y,
            ch,
            rgb: lerp_rgb(BG, col, amount),
            scale: 2,
        });
    }
}
