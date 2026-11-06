// One clock for picture and sound: 120 BPM, so one bar is two seconds.
// Every section starts on a bar line; the song follows the same table.
//
// The middle section is a hole the size of the live Desktop take: the demo
// draws the frame around it and the compose step lays the take inside.
// LIVE_BARS is set from the cut take's length, rounded up to a whole bar.

pub const BPM: f32 = 120.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const INTRO_BARS: f32 = 8.0;
pub const LIVE_BARS: f32 = 14.0;
pub const OUTRO_BARS: f32 = 6.0;
pub const TOTAL_BARS: f32 = INTRO_BARS + LIVE_BARS + OUTRO_BARS;
pub const DURATION: f32 = TOTAL_BARS * BAR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// The install line, then a trip sketch solved into one loop.
    Intro,
    /// The live Desktop take, framed.
    Live,
    /// What the plugin guarantees, the logo, where to get it.
    Outro,
}

/// (section, first bar, end bar)
pub const SECTIONS: [(Section, f32, f32); 3] = [
    (Section::Intro, 0.0, INTRO_BARS),
    (Section::Live, INTRO_BARS, INTRO_BARS + LIVE_BARS),
    (Section::Outro, INTRO_BARS + LIVE_BARS, TOTAL_BARS),
];

pub const fn bars(b: f32) -> f32 {
    b * BAR
}

/// Where the live take starts and ends on the demo clock.
pub const LIVE_FROM: f32 = bars(INTRO_BARS);
pub const LIVE_TO: f32 = bars(INTRO_BARS + LIVE_BARS);

#[derive(Clone, Copy, Debug)]
pub struct Now {
    pub section: Section,
    /// Seconds since the section started.
    pub local: f32,
    /// Section length in seconds.
    pub len: f32,
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
