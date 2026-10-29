// What is on the calendar. The week, the inbox and the calendars are the ones
// in Planner123's README screenshots (Sep 14-20, 2026; Personal and Work).
// The proposal obeys the planner's real rules: inside the declared hours
// (Mon-Fri 09:00-17:00), never on top of an existing event, high cognitive
// load in the 09:00-12:00 focus window, at most one high-load block a day.

// Planner123's palette: the SolverForge theme's colors.toml, which is also
// the app's built-in fallback.
pub const BG: u32 = 0x0B0C16;
pub const FG: u32 = 0xDDF7FF;
pub const ACCENT: u32 = 0x82FB9C;
pub const GREEN: u32 = 0x50F872; // color1
pub const TEAL: u32 = 0x50F7D4; // color3
pub const BLUE: u32 = 0x829DD4; // color4
pub const CYAN: u32 = 0x7CF8F7; // color6
pub const DIM: u32 = 0x6A6E95; // color8
pub const ADJACENT: u32 = 0x3C3F5A;

pub struct Calendar {
    pub name: &'static str,
    pub color: u32,
}
pub const CALENDARS: [Calendar; 2] = [
    Calendar {
        name: "Personal",
        color: GREEN,
    },
    Calendar {
        name: "Work",
        color: TEAL,
    },
];

pub const DAYS: [&str; 7] = [
    "Mon 14", "Tue 15", "Wed 16", "Thu 17", "Fri 18", "Sat 19", "Sun 20",
];
/// Tuesday the 15th is today.
pub const TODAY: usize = 1;

#[derive(Clone, Copy)]
pub struct Event {
    pub day: usize,
    /// Start in hours (10.5 = 10:30).
    pub start: f32,
    pub hours: f32,
    pub title: &'static str,
    pub calendar: usize,
}

const fn ev(day: usize, start: f32, hours: f32, title: &'static str, calendar: usize) -> Event {
    Event {
        day,
        start,
        hours,
        title,
        calendar,
    }
}

/// The week as it stands before the planner runs.
pub const BUSY: [Event; 10] = [
    ev(0, 10.0, 1.5, "Quarterly planning", 1),
    ev(0, 18.5, 1.0, "Gym", 0),
    ev(1, 9.0, 0.5, "Standup", 1),
    ev(1, 11.0, 1.0, "Design review", 1),
    ev(1, 12.5, 1.0, "Lunch with Sam", 0),
    ev(1, 15.0, 0.5, "1:1 with Ana", 1),
    ev(2, 14.0, 1.5, "Conference dry run", 1),
    ev(3, 9.5, 0.5, "Dentist", 0),
    ev(4, 16.0, 1.0, "Release cut v0.4", 1),
    ev(5, 9.0, 1.5, "Farmers market", 0),
];

#[derive(Clone, Copy)]
pub struct Task {
    pub title: &'static str,
    pub minutes: u32,
    pub priority: &'static str,
    pub load: &'static str,
    /// Where the proposal puts it.
    pub slot: Event,
}

const fn task(
    title: &'static str,
    minutes: u32,
    priority: &'static str,
    load: &'static str,
    slot: Event,
) -> Task {
    Task {
        title,
        minutes,
        priority,
        load,
        slot,
    }
}

/// The Planner Inbox, in flight order.
pub const TASKS: [Task; 5] = [
    task(
        "Draft Q4 roadmap",
        120,
        "High",
        "High",
        ev(2, 9.0, 2.0, "Draft Q4 roadmap", 1),
    ),
    task(
        "Prepare conference demo",
        90,
        "High",
        "High",
        ev(3, 10.0, 1.5, "Prepare conference demo", 1),
    ),
    task(
        "Review solver benchmarks",
        90,
        "Normal",
        "Medium",
        ev(0, 13.0, 1.5, "Review solver benchmarks", 1),
    ),
    task(
        "Write release notes",
        45,
        "Low",
        "Low",
        ev(4, 14.0, 0.75, "Write release notes", 1),
    ),
    task(
        "Book conference travel",
        30,
        "Normal",
        "Low",
        ev(1, 14.0, 0.5, "Book conference travel", 0),
    ),
];

/// The proposal's explanation per task (the planner reports one per item).
pub const WHY: [&str; 5] = [
    "high load, inside the 09:00-12:00 focus window",
    "high load, after the dentist, one per day",
    "medium load, first free afternoon",
    "low load, before the release cut",
    "low load, between lunch and the 1:1",
];

pub fn hhmm(hours: f32) -> String {
    let h = hours as u32;
    let m = ((hours - h as f32) * 60.0).round() as u32;
    format!("{h:02}:{m:02}")
}

/// The proposal's timestamp for a slot, as the inbox panel prints it.
pub fn stamp(e: &Event) -> String {
    format!("2026-09-{:02} {}:00", 14 + e.day, hhmm(e.start))
}

pub const GREETINGS: &str = "GREETINGS FROM THE RUST DEMOSCENE  *  PLANNER123 IS A CALENDAR FOR \
YOUR TERMINAL: LOCAL SQLITE, GOOGLE SYNC WHEN YOU ASK, AND A PLANNER THAT SCHEDULES YOUR TO-DO \
LIST AROUND YOUR REAL WEEK  *  NOTHING LANDS ON YOUR CALENDAR UNTIL YOU PRESS A  *  HELLO TO \
EVERYONE IN THE SOLVERFORGE DISCORD, TO THE RATATUI CREW, AND TO EVERY DEMO THAT EVER FIT IN 64K  *  ";

/// The credits, laid out like the help overlay's key / action rows.
pub const CREDITS: [(&str, &str); 7] = [
    ("code", "Vittorio Distefano (blackopsrepl)"),
    ("music", "the Planner123 theme, remixed"),
    ("engine", "ratatui widgets, rasterised by hand"),
    ("planner", "SolverForge"),
    ("render", "software only, no GPU, no assets"),
    ("app", "github.com/blackopsrepl/Planner123"),
    ("demos", "github.com/SolverForge/solverforge-demoscene"),
];
