// One clock for picture and sound: 125 BPM (the Planner123 theme's tempo),
// 56 bars. Every section starts on a bar line with a real Planner123 key
// press, and the planner's task landings that drive both the renderer and the
// mix are defined here once.

pub const BPM: f32 = 125.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const STEP: f32 = BEAT / 4.0;
pub const TOTAL_BARS: f32 = 56.0;
pub const DURATION: f32 = TOTAL_BARS * BAR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// A shell; `planner123` is typed and the app draws itself.
    Boot,
    /// Week view; the now line sweeps the day like a copper bar.
    Week,
    /// `1`: month view, every day cell a window onto one plasma.
    Month,
    /// `p`: the Planner Inbox with five unscheduled tasks.
    Inbox,
    /// `o`: the solver runs; the tasks fly into their slots.
    Optimize,
    /// `A`: the proposal is written as events.
    Apply,
    /// `4`: the agenda becomes the greetings scroller.
    Agenda,
    /// `?`: the help overlay becomes the credits, then the logo.
    Credits,
    /// `q`: back to the shell.
    Quit,
}

/// (section, first bar, end bar)
pub const SECTIONS: [(Section, f32, f32); 9] = [
    (Section::Boot, 0.0, 4.0),
    (Section::Week, 4.0, 12.0),
    (Section::Month, 12.0, 20.0),
    (Section::Inbox, 20.0, 24.0),
    (Section::Optimize, 24.0, 34.0),
    (Section::Apply, 34.0, 38.0),
    (Section::Agenda, 38.0, 48.0),
    (Section::Credits, 48.0, 54.0),
    (Section::Quit, 54.0, 56.0),
];

/// The key presses, in bars: each lights its status-bar chip and clicks in
/// the mix. These are Planner123's real bindings for those views.
pub const KEYS: [(f32, &str); 9] = [
    (12.0, "1"),
    (20.0, "p"),
    (24.0, "o"),
    (34.0, "A"),
    (37.5, "Esc"),
    (38.0, "4"),
    (48.0, "?"),
    (53.75, "Esc"),
    (54.0, "q"),
];

/// The solver needs a moment (the status bar's braille spinner); the
/// proposal is printed from PROPOSAL_BAR, then the planner panels wipe away
/// to the week at WIPE_BAR.
pub const PROPOSAL_BAR: f32 = 25.5;
pub const WIPE_BAR: f32 = 26.5;
/// Task i leaves the inbox at FLIGHT_FROM + i bars and lands one bar later,
/// on the downbeat.
pub const FLIGHT_FROM: f32 = 27.0;
pub const TASK_COUNT: usize = 5;

pub const fn bars(b: f32) -> f32 {
    b * BAR
}

/// The section playing at t.
pub fn section(t: f32) -> Section {
    let bar = t / BAR;
    SECTIONS
        .iter()
        .find(|(_, _, end)| bar < *end)
        .unwrap_or(&SECTIONS[SECTIONS.len() - 1])
        .0
}

/// Next section start after t (for SPACE), wrapping to 0.
pub fn next_boundary(t: f32) -> f32 {
    SECTIONS
        .iter()
        .map(|(_, b0, _)| bars(*b0))
        .find(|start| *start > t + 0.25)
        .unwrap_or(0.0)
}

pub fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// The most recent key press at or before t: (seconds since, label).
pub fn last_key(t: f32) -> Option<(f32, &'static str)> {
    KEYS.iter()
        .rev()
        .find(|(b, _)| bars(*b) <= t)
        .map(|(b, k)| (t - bars(*b), *k))
}

/// Flight progress of task i, 0 (in the inbox) .. 1 (landed).
pub fn flight(t: f32, i: usize) -> f32 {
    (t / BAR - FLIGHT_FROM - i as f32).clamp(0.0, 1.0)
}

/// Landing time of task i in seconds.
pub fn landing(i: usize) -> f32 {
    bars(FLIGHT_FROM + i as f32 + 1.0)
}

/// Keystrokes typed at the shell: (bar, text so far is the prefix up to it).
/// `planner123` is typed in the first bars of BOOT, Enter at bar 2.
pub const BOOT_TYPED: &str = "planner123";
pub const BOOT_TYPE_FROM: f32 = 0.5;
pub const BOOT_TYPE_STEP: f32 = 1.0 / 8.0; // one key per eighth note, in bars
pub const BOOT_ENTER: f32 = 2.0;
