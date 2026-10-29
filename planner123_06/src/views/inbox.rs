// The Planner Inbox surface, laid out as Planner123's ui/planner.rs does it:
// the content area split 62/38, the top again 55/45, into "Planner Inbox",
// "Selected Task" and "Planner123 Proposal". Tasks arrive one per beat, the
// selection walks down them (j/k), and the proposal prints once the solver
// returns.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::canvas::{COLS, ROWS};
use crate::data::{stamp, ACCENT, DIM, FG, TASKS, WHY};
use crate::tui::{block, bold, fg, text};

pub const LIST_X: u16 = crate::tui::SIDEBAR_W;
const CONTENT_TOP: u16 = 1;
pub const PROPOSAL_ID: &str = "75fb4ef3";

pub struct Areas {
    pub list: Rect,
    pub detail: Rect,
    pub proposal: Rect,
}

pub fn areas() -> Areas {
    let (x, y) = (LIST_X, CONTENT_TOP);
    let w = COLS - x;
    let h = ROWS - 1 - y;
    let top = (h as f32 * 0.62).round() as u16;
    let list_h = (top as f32 * 0.55).round() as u16;
    Areas {
        list: Rect::new(x, y, w, list_h),
        detail: Rect::new(x, y + list_h, w, top - list_h),
        proposal: Rect::new(x, y + top, w, h - top),
    }
}

/// The list row of task i (for the flights to start from).
pub fn row_of(i: usize) -> (u16, u16) {
    let a = areas();
    (a.list.x + 1, a.list.y + 1 + i as u16)
}

pub struct State {
    /// How many tasks are in the list.
    pub shown: usize,
    pub selected: usize,
    /// Characters of the proposal printed so far; None = no proposal yet.
    pub proposal_chars: Option<usize>,
    /// Tasks that have left the list (flying or landed).
    pub gone: usize,
}

pub fn draw(buf: &mut Buffer, s: &State) {
    let a = areas();
    block("Planner Inbox", true).render(a.list, buf);
    if s.shown == 0 {
        text(
            buf,
            a.list.x + 1,
            a.list.y + 1,
            "No inbox tasks. Press n to add one.",
            fg(DIM),
        );
    }
    for (i, task) in TASKS.iter().enumerate().take(s.shown).skip(s.gone) {
        let (x, y) = row_of(i);
        let sel = i == s.selected;
        text(buf, x, y, if sel { "> " } else { "  " }, fg(FG));
        text(
            buf,
            x + 2,
            y,
            task.title,
            if sel { bold(FG) } else { fg(FG) },
        );
        let rest = format!("  {}m  {}  {}", task.minutes, task.priority, task.load);
        text(buf, x + 2 + task.title.len() as u16, y, &rest, fg(FG));
    }

    block("Selected Task", false).render(a.detail, buf);
    if let Some(task) = TASKS.get(s.selected).filter(|_| s.shown > 0) {
        let cal = crate::data::CALENDARS[task.slot.calendar].name;
        let lines = [
            format!("{}  ({} minutes)", task.title, task.minutes),
            format!("Calendar: {cal}"),
            format!(
                "Priority: {}    Cognitive load: {}",
                task.priority, task.load
            ),
            "Earliest: none    Deadline: None".to_string(),
            format!(
                "Task ID: 1c2bbcbb-81e4-44c7-b16b-c748b3af94{:02}",
                s.selected
            ),
        ];
        for (k, l) in lines.iter().enumerate() {
            text(buf, a.detail.x + 1, a.detail.y + 1 + k as u16, l, fg(FG));
        }
    }

    block("Planner123 Proposal", false).render(a.proposal, buf);
    let (px, py) = (a.proposal.x + 1, a.proposal.y + 1);
    match s.proposal_chars {
        None => text(
            buf,
            px,
            py,
            "No proposal yet. Press s to configure Planner Settings, then press o.",
            fg(DIM),
        ),
        Some(n) => {
            let mut lines = vec![
                (format!("Proposal {PROPOSAL_ID}: 5/5 scheduled"), ACCENT),
                ("Press A to apply this reviewed proposal.".to_string(), FG),
            ];
            for (task, why) in TASKS.iter().zip(WHY) {
                lines.push((format!("{}  {why}", stamp(&task.slot)), FG));
            }
            let mut left = n;
            for (k, (l, col)) in lines.iter().enumerate() {
                let take = left.min(l.chars().count());
                let shown: String = l.chars().take(take).collect();
                text(buf, px, py + k as u16, &shown, fg(*col));
                left -= take;
                if left == 0 {
                    break;
                }
            }
        }
    }
}

/// Characters in the full proposal text (for the typing speed).
pub fn proposal_len() -> usize {
    let head = format!("Proposal {PROPOSAL_ID}: 5/5 scheduled").len()
        + "Press A to apply this reviewed proposal.".len();
    head + TASKS
        .iter()
        .zip(WHY)
        .map(|(t, w)| stamp(&t.slot).len() + 2 + w.len())
        .sum::<usize>()
}
