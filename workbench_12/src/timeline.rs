// One clock for picture and sound: 125 BPM, Planner123's tempo, so one bar
// is 1.92 seconds. Every section starts on a bar line; the song follows the
// same table.
//
// The middle section is a hole the size of the live Desktop take: the demo
// draws the frame around it and the compose step lays the take inside.
// LIVE_BARS comes from the cut's length, rounded up to a whole bar.
pub const BPM: f32 = 125.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const STEP: f32 = BEAT / 4.0;
pub const INTRO_BARS: f32 = 7.0;
pub const LIVE_BARS: f32 = 88.0;
pub const OUTRO_BARS: f32 = 6.0;
pub const TOTAL_BARS: f32 = INTRO_BARS + LIVE_BARS + OUTRO_BARS;
pub const DURATION: f32 = TOTAL_BARS * BAR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// The title, then the five models land one per beat, and the stars
    /// warp into the first downbeat.
    Intro,
    /// The live Desktop take, framed, with the Nous Girl narrating.
    Live,
    /// The logo and where to get the plugin.
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

/// When each of the intro's five model panels lands: one every two beats
/// from the middle of the second bar. The song strikes a taiko on each.
pub const PANEL_AT: [f32; 5] = [
    bars(1.5),
    bars(1.5) + 2.0 * BEAT,
    bars(1.5) + 4.0 * BEAT,
    bars(1.5) + 6.0 * BEAT,
    bars(1.5) + 8.0 * BEAT,
];

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cut::TAKE_SECONDS;

    #[test]
    fn the_live_window_holds_the_whole_cut_and_less_than_a_bar_more() {
        let live = LIVE_TO - LIVE_FROM;
        assert!(TAKE_SECONDS <= live, "the cut outlasts the live window");
        assert!(
            live - TAKE_SECONDS < BAR,
            "the live window holds more than a bar of frozen take"
        );
    }
}
