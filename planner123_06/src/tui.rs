// Planner123's chrome, drawn with ratatui widgets the way the app draws it:
// the mint header row, the 22-column sidebar (Calendars, Projects), the
// focused main block, and the status bar of key chips with the
// google/clock block on the right. Styles follow the app's theme/styles.rs.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Widget};

use crate::canvas::{c, lerp_rgb, COLS, ROWS};
use crate::data::{ACCENT, BG, BLUE, CALENDARS, DIM, FG};

pub const SIDEBAR_W: u16 = 22;

/// The main content area inside the focused block.
pub const CONTENT: Rect = Rect {
    x: SIDEBAR_W + 1,
    y: 2,
    width: COLS - SIDEBAR_W - 2,
    height: ROWS - 4,
};

pub fn put(buf: &mut Buffer, x: u16, y: u16, ch: char, fg: u32) {
    if x < COLS && y < ROWS {
        buf[(x, y)].set_char(ch).set_fg(c(fg));
    }
}

pub fn put_bg(buf: &mut Buffer, x: u16, y: u16, bg: u32) {
    if x < COLS && y < ROWS {
        buf[(x, y)].set_bg(c(bg));
    }
}

pub fn text(buf: &mut Buffer, x: u16, y: u16, s: &str, st: Style) {
    if y < ROWS && x < COLS {
        buf.set_stringn(x, y, s, (COLS - x) as usize, st);
    }
}

pub fn fg(rgb: u32) -> Style {
    Style::default().fg(c(rgb))
}

pub fn bold(rgb: u32) -> Style {
    fg(rgb).add_modifier(Modifier::BOLD)
}

pub fn clear(buf: &mut Buffer) {
    Block::default()
        .style(Style::default().bg(c(BG)).fg(c(FG)))
        .render(Rect::new(0, 0, COLS, ROWS), buf);
}

pub fn block(title: &str, focused: bool) -> Block<'_> {
    let col = if focused { ACCENT } else { DIM };
    Block::default()
        .borders(Borders::ALL)
        .border_style(fg(col))
        .title(Span::styled(
            format!(" {title} "),
            if focused { bold(ACCENT) } else { fg(DIM) },
        ))
}

pub struct Chrome<'a> {
    pub header: &'a str,
    pub view: &'a str,
    pub chips: &'a [(&'a str, &'a str)],
    /// The chip that was just pressed, and how long ago.
    pub pressed: Option<(&'a str, f32)>,
    /// Right-hand status block: a message (accent) or the google/clock line.
    pub status: Option<&'a str>,
    pub clock: &'a str,
    /// Skip the focused main block (the planner draws its own panels).
    pub bare: bool,
}

pub fn draw(buf: &mut Buffer, ch: &Chrome) {
    clear(buf);
    let head = Style::default().bg(c(ACCENT)).fg(c(BG));
    buf.set_style(Rect::new(0, 0, COLS, 1), head);
    text(
        buf,
        0,
        0,
        &format!("  ▦  Planner123     {}  ", ch.header),
        head,
    );

    block("Calendars", false).render(Rect::new(0, 1, SIDEBAR_W, ROWS - 10), buf);
    for (i, cal) in CALENDARS.iter().enumerate() {
        text(buf, 1, 2 + i as u16, "● ", fg(cal.color));
        text(buf, 3, 2 + i as u16, cal.name, fg(FG));
    }
    block("Projects", false).render(Rect::new(0, ROWS - 9, SIDEBAR_W, 8), buf);
    text(buf, 1, ROWS - 7, "▸ Conference", fg(FG));
    text(buf, 3, ROWS - 6, "░░░░░░░░░░", fg(DIM));
    text(buf, 14, ROWS - 6, "0/2", fg(DIM));

    if !ch.bare {
        block(ch.view, true).render(Rect::new(SIDEBAR_W, 1, COLS - SIDEBAR_W, ROWS - 2), buf);
    }
    status_bar(buf, ch);
}

fn status_bar(buf: &mut Buffer, ch: &Chrome) {
    let y = ROWS - 1;
    let (msg, st) = match ch.status {
        Some(m) => (format!(" {m} "), fg(ACCENT)),
        None => (format!(" google off  {} ", ch.clock), fg(DIM)),
    };
    let w = msg.chars().count() as u16;
    text(buf, COLS.saturating_sub(w), y, &msg, st);
    // Like the app, drop chips from the tail rather than clip one, but keep
    // the help chip pinned.
    let width = |(k, l): &(&str, &str)| k.chars().count() as u16 + l.chars().count() as u16 + 5;
    let budget = COLS.saturating_sub(w + 1);
    let pinned = ch.chips.iter().find(|(k, _)| *k == "?");
    let mut used = pinned.map(width).unwrap_or(0);
    let mut shown: Vec<&(&str, &str)> = Vec::new();
    for chip in ch.chips.iter().filter(|(k, _)| *k != "?") {
        if used + width(chip) > budget {
            break;
        }
        used += width(chip);
        shown.push(chip);
    }
    shown.extend(pinned);
    let mut x = 0u16;
    for (key, label) in shown {
        // A pressed chip flashes mint and fades back to the theme's blue.
        let glow = match ch.pressed {
            Some((k, ago)) if k == *key => (1.0 - ago / 0.6).clamp(0.0, 1.0),
            _ => 0.0,
        };
        let chip = Style::default()
            .bg(c(lerp_rgb(BLUE, ACCENT, glow)))
            .fg(c(BG))
            .add_modifier(Modifier::BOLD);
        text(buf, x, y, &format!(" {key} "), chip);
        x += key.chars().count() as u16 + 2;
        text(buf, x, y, &format!(" {label}  "), fg(DIM));
        x += label.chars().count() as u16 + 3;
    }
}

// Status-bar chips per surface, in the app's keymap order.
pub const WEEK_CHIPS: [(&str, &str); 11] = [
    ("c", "create"),
    ("e", "edit"),
    ("d", "del"),
    ("/", "quick"),
    ("j/k", "event"),
    ("1-4", "view"),
    ("Tab", "sidebar"),
    ("p", "planner"),
    ("n", "now"),
    (":", "cmd"),
    ("?", "help"),
];
pub const MONTH_CHIPS: [(&str, &str); 11] = [
    ("c", "create"),
    ("e", "edit"),
    ("d", "del"),
    ("/", "quick"),
    ("j/k", "row"),
    ("1-4", "view"),
    ("Tab", "sidebar"),
    ("p", "planner"),
    ("n", "today"),
    (":", "cmd"),
    ("?", "help"),
];
pub const AGENDA_CHIPS: [(&str, &str); 11] = [
    ("c", "create"),
    ("e", "edit"),
    ("d", "del"),
    ("/", "quick"),
    ("j/k", "scroll"),
    ("1-4", "view"),
    ("Tab", "sidebar"),
    ("p", "planner"),
    ("n", "today"),
    (":", "cmd"),
    ("?", "help"),
];
pub const PLANNER_CHIPS: [(&str, &str); 6] = [
    ("j/k", "inspect task"),
    ("n", "new task"),
    ("o", "optimize"),
    ("A", "apply"),
    ("s", "settings"),
    ("Esc", "close"),
];
pub const HELP_CHIPS: [(&str, &str); 1] = [("Esc", "close")];

/// The view keys (1-4) share one chip, as in the app.
pub fn chip_for(key: &str) -> &str {
    match key {
        "1" | "2" | "3" | "4" => "1-4",
        k => k,
    }
}
