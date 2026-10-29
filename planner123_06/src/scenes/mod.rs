// The demo as one function of time: which Planner123 surface is on screen,
// what has been pressed, and which effect is living inside it.
mod close;
mod flights;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::canvas::{c, lerp_rgb, COLS, ROWS};
use crate::data::{hhmm, ACCENT, BG, BUSY};
use crate::post::Look;
use crate::timeline::{self as tl, bars, smooth, Section, BAR, BEAT};
use crate::tui::{self, chip_for, Chrome};
use crate::views::{month, shell, week};

/// The app's braille loading spinner (ui/status_bar.rs).
pub const SPINNER: [&str; 8] = [
    "\u{2801}", "\u{2809}", "\u{2819}", "\u{281b}", "\u{281e}", "\u{2836}", "\u{2834}", "\u{2824}",
];

pub struct Demo;

/// The most recent key press as (chip label, seconds since).
pub fn pressed(t: f32) -> Option<(&'static str, f32)> {
    tl::last_key(t).map(|(ago, k)| (chip_for(k), ago))
}

pub fn spinner(t: f32) -> &'static str {
    SPINNER[((t / (BEAT / 2.0)) as usize) % SPINNER.len()]
}

/// The now line: swings through the day during WEEK, otherwise 14:53.
pub fn now_hour(t: f32) -> f32 {
    let b = t / BAR;
    if (4.0..12.0).contains(&b) {
        14.9 + 6.4 * (std::f32::consts::TAU * (b - 4.0) / 8.0).sin()
    } else {
        14.9
    }
}

pub fn week_chrome<'a>(
    chips: &'a [(&'a str, &'a str)],
    t: f32,
    status: Option<&'a str>,
    clock: &'a str,
) -> Chrome<'a> {
    let _ = t;
    Chrome {
        header: "Sep 14 — Sep 20, 2026",
        view: "Week",
        chips,
        pressed: pressed(t),
        status,
        clock,
        bare: false,
    }
}

/// Rows below the scan line `amount` (0..1) go blank, and the scan row
/// itself is typed out left to right with a glowing head: the app redrawing
/// the terminal at launch, and erasing it at quit.
pub fn reveal(buf: &mut Buffer, amount: f32) {
    let scan = amount.clamp(0.0, 1.0) * ROWS as f32;
    let (row, frac) = (scan as u16, scan.fract());
    for y in 0..ROWS {
        for x in 0..COLS {
            let head = (frac * COLS as f32) as u16;
            let cell = &mut buf[(x, y)];
            if y > row || (y == row && x > head) {
                cell.reset();
                cell.set_bg(c(BG));
            } else if y == row && head - x < 6 {
                cell.set_fg(c(0xEFFFF4))
                    .set_bg(c(lerp_rgb(BG, ACCENT, 0.4)));
            }
        }
    }
}

/// Copy columns x >= edge from `from` into `buf` (the planner panels wiping
/// away to the week), with a lit seam at the edge.
pub fn wipe(buf: &mut Buffer, from: &Buffer, edge: u16) {
    for y in 1..ROWS - 1 {
        for x in edge.min(COLS)..COLS {
            buf[(x, y)] = from[(x, y)].clone();
        }
        if edge < COLS {
            buf[(edge, y)].set_char('│').set_fg(c(0xEFFFF4));
        }
    }
}

impl Demo {
    pub fn new() -> Self {
        Demo
    }

    pub fn frame(&self, buf: &mut Buffer, t: f32) -> Look {
        let bar = t / BAR;
        let mut look = Look::default();
        let clock = hhmm(now_hour(t));
        match tl::section(t) {
            Section::Boot => boot(buf, t, &clock),
            Section::Week => {
                let ch = week_chrome(&tui::WEEK_CHIPS, t, None, &clock);
                tui::draw(buf, &ch);
                let amt = smooth((bar - 4.0) * 1.5) * (1.0 - smooth((bar - 11.25) * 1.5));
                week::copper(buf, t, amt);
                week::grid(buf, now_hour(t));
                events_arrive(buf, t);
            }
            Section::Month => {
                let ch = Chrome {
                    header: "September 2026",
                    view: "Month",
                    chips: &tui::MONTH_CHIPS,
                    pressed: pressed(t),
                    status: None,
                    clock: &clock,
                    bare: false,
                };
                tui::draw(buf, &ch);
                let kick = (-(t % BEAT) * 9.0).exp();
                month::draw(buf, t, smooth(bar - 12.0), kick);
                let slide = smooth((bar - 13.0) / 1.5) * (1.0 - smooth((bar - 19.0) / 1.0));
                if slide > 0.0 {
                    month::banner(buf, t, slide);
                }
            }
            Section::Inbox | Section::Optimize | Section::Apply => {
                flights::draw(buf, &mut look, t, &clock)
            }
            Section::Agenda | Section::Credits | Section::Quit => {
                close::draw(buf, &mut look, t, &clock)
            }
        }
        look
    }
}

fn boot(buf: &mut Buffer, t: f32, clock: &str) {
    let bar = t / BAR;
    let beat_on = ((t / BEAT) as usize).is_multiple_of(2);
    if bar < 0.5 {
        // CRT power-on: a line draws across the middle, then opens to the
        // full screen.
        tui::clear(buf);
        let w = smooth(bar / 0.25) * COLS as f32;
        let open = smooth((bar - 0.25) / 0.25) * (ROWS / 2) as f32;
        let mid = ROWS / 2;
        for x in 0..w as u16 {
            for dy in 0..=open as u16 {
                let k = 1.0 - dy as f32 / (open + 1.0);
                for y in [mid.saturating_sub(dy), (mid + dy).min(ROWS - 1)] {
                    buf[(x, y)].set_bg(c(lerp_rgb(BG, 0xEFFFF4, k * (1.0 - open / 20.0))));
                }
            }
        }
        return;
    }
    if bar < tl::BOOT_ENTER + 0.25 {
        let n = (((bar - tl::BOOT_TYPE_FROM) / tl::BOOT_TYPE_STEP)
            .floor()
            .max(0.0) as usize)
            .min(tl::BOOT_TYPED.len());
        shell::draw(
            buf,
            &tl::BOOT_TYPED[..n],
            beat_on || n < tl::BOOT_TYPED.len(),
            false,
        );
        return;
    }
    // The app draws itself while its database loads (spinner on the right).
    let ch = week_chrome(&tui::WEEK_CHIPS, t, Some(spinner(t)), clock);
    tui::draw(buf, &ch);
    week::grid(buf, now_hour(t));
    let amount = smooth((bar - 2.25) / 1.5);
    reveal(buf, amount * 1.05);
}

/// At the start of WEEK the events load in, one per eighth note, each lit
/// as it lands.
fn events_arrive(buf: &mut Buffer, t: f32) {
    for (i, e) in BUSY.iter().enumerate() {
        let at = bars(4.0) + i as f32 * BEAT / 2.0;
        if t < at {
            continue;
        }
        let lit = (1.0 - (t - at) / 0.5).clamp(0.0, 1.0);
        let kind = if lit > 0.0 {
            week::Kind::Lit(lit)
        } else {
            week::Kind::Solid
        };
        week::event(buf, e, kind);
    }
}

/// A scratch buffer the size of the screen.
pub fn scratch() -> Buffer {
    Buffer::empty(Rect::new(0, 0, COLS, ROWS))
}
