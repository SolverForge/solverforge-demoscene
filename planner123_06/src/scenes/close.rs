// AGENDA -> CREDITS -> QUIT: the agenda scrolls over a starfield with the
// greetings on a sine; `?` opens the help overlay and `/credits` is typed
// into its filter; the ouroboros draws itself; `q` erases the app back to
// the shell prompt.
use ratatui::buffer::Buffer;

use crate::canvas::{scale_rgb, to_rgb, CELL_H, CELL_W, COLS, ROWS};
use crate::data::{BG, FG};
use crate::logo::LogoState;
use crate::post::Look;
use crate::timeline::{bars, smooth, BAR, BEAT};
use crate::tui::{self, Chrome};
use crate::views::{agenda, help, shell};

use super::{pressed, reveal};

pub fn draw(buf: &mut Buffer, look: &mut Look, t: f32, clock: &str) {
    let b = t / BAR;
    if b >= 54.75 {
        let blink = ((t / BEAT) as usize).is_multiple_of(2);
        shell::draw(buf, "planner123", blink, true);
        logo(look, t);
        return;
    }
    let chips: &[(&str, &str)] = if (48.0..53.75).contains(&b) {
        &tui::HELP_CHIPS
    } else {
        &tui::AGENDA_CHIPS
    };
    let ch = Chrome {
        header: "Agenda",
        view: "Agenda",
        chips,
        pressed: pressed(t),
        status: None,
        clock,
        bare: false,
    };
    tui::draw(buf, &ch);
    agenda::stars(buf, t);
    agenda::list(buf, (t - bars(38.0)) / BEAT * 0.5 + 14.0);
    let scroll_amt = smooth(b - 39.0) * (1.0 - smooth((b - 47.0) * 1.5));
    if scroll_amt > 0.0 {
        agenda::scroller(&mut look.sprites, t - bars(38.0), scroll_amt);
    }

    if (48.0..53.75).contains(&b) {
        // Dim the agenda behind the popup.
        dim(buf, 0.45);
        let typed = if b < 49.0 {
            None
        } else {
            Some((((b - 49.25) * 8.0).floor().max(0.0) as usize).min(7))
        };
        let rows = (((b - 50.25) * 4.0).floor().max(0.0) as usize).min(7);
        if b < 52.0 {
            help::draw(buf, typed, rows);
        } else {
            dim(buf, 0.6);
        }
    }
    if b >= 52.0 {
        if b < 53.75 {
            dim(buf, 0.6);
        }
        logo(look, t);
    }
    if b >= 54.0 {
        // `q`: the app erases itself back to the shell.
        reveal(buf, 1.05 - smooth((b - 54.0) / 0.75) * 1.1);
        if b >= 54.6 {
            let mut sh = super::scratch();
            shell::draw(&mut sh, "planner123", true, true);
            for y in 0..ROWS {
                for x in 0..COLS {
                    if buf[(x, y)].symbol() == " " {
                        buf[(x, y)] = sh[(x, y)].clone();
                    }
                }
            }
        }
    }
}

fn dim(buf: &mut Buffer, gain: f32) {
    for y in 1..ROWS - 1 {
        for x in tui::SIDEBAR_W..COLS {
            let cell = &mut buf[(x, y)];
            let fg = scale_rgb(to_rgb(cell.fg, FG), gain);
            let bg = scale_rgb(to_rgb(cell.bg, BG), gain.max(0.8));
            cell.set_fg(crate::canvas::c(fg))
                .set_bg(crate::canvas::c(bg));
        }
    }
}

/// The ouroboros draws itself from bar 52 over the content area, holds
/// through the quit and fades in the last bar.
fn logo(look: &mut Look, t: f32) {
    let b = t / BAR;
    let c0 = tui::CONTENT;
    let cx = (c0.x as f32 + c0.width as f32 / 2.0) * CELL_W as f32;
    let cy = (c0.y as f32 + c0.height as f32 / 2.0) * CELL_H as f32;
    look.logo = Some(LogoState {
        cx,
        cy,
        scale: 2.6,
        progress: smooth((b - 52.0) / 1.5),
        detail: smooth((b - 53.0) / 1.0),
        gain: 1.0 - smooth((b - 55.0) / 0.9),
    });
}
