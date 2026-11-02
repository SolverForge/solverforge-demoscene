// One clock for picture and sound: 120 BPM, so one bar is two seconds.
// Every section starts on a bar line; the song follows the same table.

pub const BPM: f32 = 120.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const TOTAL_BARS: f32 = 68.0;
pub const DURATION: f32 = TOTAL_BARS * BAR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// Who: the name, the portrait, a shell.
    Boot,
    /// Where: Bergamo's upper town at night, and SolverForge.
    City,
    /// Offer one: planning systems, a route plan being improved.
    Plan,
    /// Offer two: advisory, the path from discovery to a next step.
    Advice,
    /// Offer three: agents and the tools under them, a tiling desktop.
    Agents,
    /// Offer four: developer relations, a project's launch kit.
    DevRel,
    /// Proof: the published work, on a scroller.
    Work,
    /// How to reach him.
    Contact,
}

/// (section, first bar, end bar)
pub const SECTIONS: [(Section, f32, f32); 8] = [
    (Section::Boot, 0.0, 6.0),
    (Section::City, 6.0, 14.0),
    (Section::Plan, 14.0, 27.0),
    (Section::Advice, 27.0, 35.0),
    (Section::Agents, 35.0, 44.0),
    (Section::DevRel, 44.0, 52.0),
    (Section::Work, 52.0, 60.0),
    (Section::Contact, 60.0, 68.0),
];

pub const fn bars(b: f32) -> f32 {
    b * BAR
}

pub fn start_of(section: Section) -> f32 {
    SECTIONS
        .iter()
        .find(|(s, _, _)| *s == section)
        .map(|(_, b0, _)| bars(*b0))
        .unwrap_or(0.0)
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
