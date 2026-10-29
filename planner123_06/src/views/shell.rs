// The shell the app is launched from and returns to.
use ratatui::buffer::Buffer;

use crate::data::{ACCENT, BLUE, DIM, FG};
use crate::tui::{bold, clear, fg, text};

const PROMPT: &str = "vittorio@solverforge ~ $ ";

/// `typed`: characters of the command typed so far; `cursor`: blink phase.
/// `after`: lines printed after the command returned (the quit prompt).
pub fn draw(buf: &mut Buffer, typed: &str, cursor: bool, returned: bool) {
    clear(buf);
    let y = 1;
    text(buf, 1, y, "vittorio@solverforge", bold(ACCENT));
    text(buf, 21, y, " ~ ", bold(BLUE));
    text(buf, 24, y, "$ ", fg(FG));
    text(buf, PROMPT.len() as u16 + 1, y, typed, fg(FG));
    let (cx, cy) = if returned {
        text(buf, 1, y + 1, "vittorio@solverforge", bold(ACCENT));
        text(buf, 21, y + 1, " ~ ", bold(BLUE));
        text(buf, 24, y + 1, "$ ", fg(FG));
        (PROMPT.len() as u16 + 1, y + 1)
    } else {
        (PROMPT.len() as u16 + 1 + typed.len() as u16, y)
    };
    if cursor {
        text(buf, cx, cy, "█", fg(DIM));
    }
}
