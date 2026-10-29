// The help overlay ("Keybindings", a centred 70% x 85% popup with a mint
// border, as in Planner123's ui/help.rs): first the Global section, then
// `/credits` is typed into its filter and the rows become the credits.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Clear, Widget};

use crate::canvas::{c, COLS, ROWS};
use crate::data::{ACCENT, BG, BLUE, CREDITS, DIM, FG};
use crate::tui::{bold, fg, text};

const GLOBAL: [(&str, &str); 9] = [
    ("?", "Toggle this help"),
    (":", "Open the command palette"),
    ("1", "Month view"),
    ("2", "Week view"),
    ("3", "Day view"),
    ("4", "Agenda view"),
    ("Tab", "Focus calendar sidebar"),
    ("p", "Open Planner Inbox"),
    ("q / Ctrl+C", "Quit"),
];

pub fn area() -> Rect {
    let (w, h) = (COLS * 70 / 100, ROWS * 85 / 100);
    Rect::new((COLS - w) / 2, (ROWS - h) / 2, w, h)
}

/// `typed`: how much of "credits" has been typed into the filter (None: no
/// filter yet). `rows`: credit rows revealed.
pub fn draw(buf: &mut Buffer, typed: Option<usize>, rows: usize) {
    let a = area();
    Clear.render(a, buf);
    let title = match typed {
        None => " Keybindings ".to_string(),
        Some(n) => format!(" Keybindings — filter: {}▏ ", &"credits"[..n.min(7)]),
    };
    Block::default()
        .title(Span::styled(title, bold(ACCENT)))
        .borders(Borders::ALL)
        .border_style(fg(ACCENT))
        .style(Style::default().bg(c(BG)).fg(c(FG)))
        .render(a, buf);
    let (x, mut y) = (a.x + 2, a.y + 2);
    if typed.is_some_and(|n| n >= 7) {
        text(buf, x, y, "▸ Credits", bold(ACCENT));
        y += 1;
        for (k, v) in CREDITS.iter().take(rows) {
            text(
                buf,
                x,
                y,
                &format!("  {k:<12}"),
                fg(BLUE).add_modifier(Modifier::BOLD),
            );
            text(buf, x + 14, y, v, fg(FG));
            y += 1;
        }
    } else {
        text(buf, x, y, "▸ Global", bold(ACCENT));
        y += 1;
        for (k, v) in GLOBAL {
            text(
                buf,
                x,
                y,
                &format!("  {k:<12}"),
                fg(BLUE).add_modifier(Modifier::BOLD),
            );
            text(buf, x + 14, y, v, fg(FG));
            y += 1;
        }
        text(buf, x, y + 1, "  / filters this list by typing", fg(DIM));
    }
}
