// INBOX -> OPTIMIZE -> APPLY: tasks arrive in the Planner Inbox, `o` runs
// the solver and prints the proposal, the panels wipe away to the week, and
// each task flies from its inbox row to its proposed slot, landing on a
// downbeat with a ripple. `A` writes them as events in one sweep.
use ratatui::buffer::Buffer;
use ratatui::style::{Modifier, Style};

use crate::canvas::{c, lerp_rgb, CELL_H, CELL_W, COLS};
use crate::data::{ACCENT, BG, BUSY, TASKS};
use crate::post::Look;
use crate::timeline::{self as tl, bars, smooth, BAR, BEAT, TASK_COUNT};
use crate::tui::{self, bold, put_bg, text, Chrome};
use crate::views::{inbox, week};

use super::{pressed, scratch, spinner, week_chrome};

const READY: &str = "Planner proposal is ready for review; press A to apply.";
const APPLIED: &str = "Applied proposal 75fb4ef3: 5 events written.";

fn planner_chrome<'a>(t: f32, status: Option<&'a str>, clock: &'a str) -> Chrome<'a> {
    Chrome {
        header: "Planner123",
        view: "",
        chips: &tui::PLANNER_CHIPS,
        pressed: j_or_key(t),
        status,
        clock,
        bare: true,
    }
}

/// j presses walk the selection down the inbox, one per beat in bars 22-24.
fn j_or_key(t: f32) -> Option<(&'static str, f32)> {
    let b = t / BAR;
    if (22.0..24.0).contains(&b) {
        return Some(("j/k", t % BEAT));
    }
    pressed(t)
}

pub fn draw(buf: &mut Buffer, look: &mut Look, t: f32, clock: &str) {
    let b = t / BAR;
    let shown = (((b - 20.0) * 4.0).floor() as i32 + 1).clamp(0, 5) as usize;
    let selected = (((b - 22.0) * 4.0).floor().max(0.0) as usize).min(4);
    let chars = if b >= tl::PROPOSAL_BAR {
        let p = (b - tl::PROPOSAL_BAR) / 0.8;
        Some((p.min(1.0) * inbox::proposal_len() as f32) as usize)
    } else {
        None
    };
    let status = if (24.0..tl::PROPOSAL_BAR).contains(&b) {
        Some(spinner(t))
    } else if b >= 34.0 {
        Some(APPLIED)
    } else if b >= tl::PROPOSAL_BAR {
        Some(READY)
    } else {
        None
    };
    let state = inbox::State {
        shown,
        selected,
        proposal_chars: chars,
        gone: 0,
    };
    if b < tl::WIPE_BAR {
        tui::draw(buf, &planner_chrome(t, status, clock));
        inbox::draw(buf, &state);
        return;
    }

    // The week behind, the planner panels in front until the wipe passes.
    let chips: &[(&str, &str)] = if b >= 37.5 {
        &tui::WEEK_CHIPS
    } else {
        &tui::PLANNER_CHIPS
    };
    let mut ch = week_chrome(chips, t, status, clock);
    ch.pressed = pressed(t);
    tui::draw(buf, &ch);
    let copper = smooth((b - 34.0) * 2.0) * (1.0 - smooth((b - 37.0) * 2.0));
    week::copper(buf, t, copper);
    week::grid(buf, super::now_hour(t));
    for e in BUSY.iter() {
        week::event(buf, e, week::Kind::Solid);
    }
    let wipe = smooth((b - tl::WIPE_BAR) / 0.5);
    if wipe < 1.0 {
        let mut front = scratch();
        tui::draw(&mut front, &planner_chrome(t, status, clock));
        inbox::draw(&mut front, &inbox::State { gone: 5, ..state });
        let edge = tui::SIDEBAR_W as f32 + wipe * (COLS - tui::SIDEBAR_W) as f32;
        super::wipe(buf, &front, edge as u16);
    }

    if (tl::WIPE_BAR..tl::FLIGHT_FROM + TASK_COUNT as f32).contains(&b) {
        text(buf, 1, 4, "Planner Inbox", bold(ACCENT));
    }
    for (i, task) in TASKS.iter().enumerate() {
        let p = tl::flight(t, i);
        let landed = t - tl::landing(i);
        if p >= 1.0 {
            ripple(buf, task.slot.day, task.slot.start, landed);
            let kind = applied_kind(t, i).unwrap_or_else(|| {
                let lit = (1.0 - landed / 0.6).clamp(0.0, 1.0);
                if lit > 0.0 {
                    week::Kind::Lit(lit)
                } else {
                    week::Kind::Ghost(0.6 + 0.4 * (t * 5.0 + i as f32).sin())
                }
            });
            week::event(buf, &task.slot, kind);
        } else {
            fly(buf, t, i, p);
        }
    }
    // `A`: a white flash, then the copper bars celebrate.
    if b >= 34.0 {
        look.flash = (-(t - bars(34.0)) * 5.0).exp();
    }
}

/// After `A`, a sweep crosses the week left to right; each slot lights as
/// it passes, then stays a solid event.
fn applied_kind(t: f32, i: usize) -> Option<week::Kind> {
    let b = t / BAR;
    if b < 34.0 {
        return None;
    }
    let at = 34.0 + TASKS[i].slot.day as f32 * 0.12;
    let lit = (1.0 - (b - at) * 2.0).clamp(0.0, 1.0);
    Some(if b < at {
        week::Kind::Ghost(1.0)
    } else if lit > 0.0 {
        week::Kind::Lit(lit)
    } else {
        week::Kind::Solid
    })
}

/// Where task i waits once the panels have wiped away: queued down the
/// sidebar, which becomes the inbox for the flights.
fn queue(i: usize) -> (f32, f32) {
    (1.0, 6.0 + i as f32 * 2.0)
}

/// Before its flight: sliding from its inbox row to the sidebar queue.
fn waiting(t: f32, i: usize) -> (f32, f32) {
    let (lx, ly) = inbox::row_of(i);
    let (qx, qy) = queue(i);
    let e = smooth((t / BAR - tl::WIPE_BAR) / 0.5);
    let (lx, ly) = (lx as f32 + 2.0, ly as f32);
    (lx + (qx - lx) * e, ly + (qy - ly) * e)
}

/// Position of task i's chip at flight progress p: an arc from the sidebar
/// queue up and over to its slot, easing in and out.
fn path(i: usize, p: f32) -> (f32, f32) {
    let (sx, sy) = queue(i);
    let slot = &TASKS[i].slot;
    let (tx, ty) = (
        week::col_of(slot.day) as f32 + 1.0,
        week::row_of(slot.start) as f32,
    );
    let e = smooth(p);
    let lift = (e * std::f32::consts::PI).sin() * 7.0;
    (sx + (tx - sx) * e, sy + (ty - sy) * e - lift)
}

fn fly(buf: &mut Buffer, t: f32, i: usize, p: f32) {
    let shades = ['█', '▓', '▒', '░', '·'];
    if p > 0.0 {
        for k in 1..16 {
            let q = (p - k as f32 * 0.025).max(0.0);
            let (x, y) = path(i, q);
            let fade = 1.0 - k as f32 / 16.0;
            tui::put(
                buf,
                x as u16,
                y as u16,
                shades[(k / 4).min(4)],
                lerp_rgb(BG, ACCENT, fade),
            );
        }
    }
    let (x, y) = if p > 0.0 { path(i, p) } else { waiting(t, i) };
    // Waiting chips pulse on the beat.
    let pulse = if p == 0.0 {
        0.75 + 0.25 * (-(t % BEAT) * 6.0).exp()
    } else {
        1.0
    };
    let label: String = TASKS[i].title.chars().take(12).collect();
    let st = Style::default()
        .bg(c(lerp_rgb(BG, ACCENT, pulse)))
        .fg(c(BG))
        .add_modifier(Modifier::BOLD);
    text(buf, x as u16, y as u16, &format!(" {label} "), st);
}

/// A ring of light expanding from a landing, in cell backgrounds.
fn ripple(buf: &mut Buffer, day: usize, hour: f32, age: f32) {
    if !(0.0..0.9).contains(&age) {
        return;
    }
    let (cx, cy) = (
        (week::col_of(day) as f32 + 7.0) * CELL_W as f32,
        week::row_of(hour) as f32 * CELL_H as f32,
    );
    let r = age * 520.0;
    let a = (1.0 - age / 0.9) * 0.55;
    let c0 = tui::CONTENT;
    for y in c0.y..c0.y + c0.height {
        for x in c0.x..c0.x + c0.width {
            let (px, py) = (x as f32 * CELL_W as f32, y as f32 * CELL_H as f32);
            let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
            let ring = (1.0 - (d - r).abs() / 34.0).max(0.0);
            if ring > 0.0 {
                let bg = crate::canvas::to_rgb(buf[(x, y)].bg, BG);
                put_bg(buf, x, y, lerp_rgb(bg, ACCENT, ring * a));
            }
        }
    }
}
