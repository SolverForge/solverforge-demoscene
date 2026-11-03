// One clock for picture and sound: 120 BPM, so one bar is two seconds.
// Every section starts on a bar line; the song follows the same table.

pub const BPM: f32 = 120.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const TOTAL_BARS: f32 = 70.0;
pub const DURATION: f32 = TOTAL_BARS * BAR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// The black Les Paul, close up, strings ringing with the intro.
    Guitar,
    /// Old online worlds: a frost mage on a night road, the bags, the trade.
    Worlds,
    /// Hard games: a small knight against a giant, then a survival-horror hall.
    Challenge,
    /// The machine: a Sway desktop, tmux, a terminal, a real keyboard.
    Machine,
    /// Reading: books on a desk, then a desert under two moons.
    Reading,
    /// Home in Bergamo: rooftops, a cat at the window, his reflection.
    Home,
    /// Work: strings become routes, and the routes untangle.
    Work,
    /// The name card.
    Name,
}

/// (section, first bar, end bar)
pub const SECTIONS: [(Section, f32, f32); 8] = [
    (Section::Guitar, 0.0, 8.0),
    (Section::Worlds, 8.0, 20.0),
    (Section::Challenge, 20.0, 32.0),
    (Section::Machine, 32.0, 42.0),
    (Section::Reading, 42.0, 49.0),
    (Section::Home, 49.0, 57.0),
    (Section::Work, 57.0, 64.0),
    (Section::Name, 64.0, 70.0),
];

pub const fn bars(b: f32) -> f32 {
    b * BAR
}

#[derive(Clone, Copy, Debug)]
pub struct Now {
    pub section: Section,
    /// Seconds since the section started.
    pub local: f32,
    /// Section length in seconds.
    pub len: f32,
}

impl Now {
    /// Fade in over the first `a` seconds and out over the last `b`.
    pub fn envelope(&self, a: f32, b: f32) -> f32 {
        smooth(self.local / a) * smooth((self.len - self.local) / b)
    }
}

pub fn now(t: f32) -> Now {
    let bar = t / BAR;
    let &(section, b0, b1) = SECTIONS
        .iter()
        .find(|(_, _, end)| bar < *end)
        .unwrap_or(&SECTIONS[SECTIONS.len() - 1]);
    Now {
        section,
        local: t - bars(b0),
        len: bars(b1 - b0),
    }
}

pub fn span(section: Section) -> (usize, usize) {
    SECTIONS
        .iter()
        .find(|(s, _, _)| *s == section)
        .map(|(_, b0, b1)| (*b0 as usize, *b1 as usize))
        .unwrap_or((0, 0))
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
