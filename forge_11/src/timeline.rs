// One clock for picture and sound: 120 BPM, one bar is two seconds, and
// every section starts on a bar line. The song (audio/song.rs) follows the
// same table, so every cut lands on a downbeat.

pub const BPM: f32 = 120.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const STEP: f32 = BEAT / 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    /// The forge lights: the title, the corner mark draws itself.
    Boot,
    /// 01: the plain-language problem typed to the coding agent.
    Describe,
    /// 02: the agent and the solverforge-modeling skill write the model.
    Generate,
    /// 03: the model solves on the laptop.
    Solve,
    /// 04: a disruption enters the running solve; the plan heals.
    Repair,
    /// 05: your model, your machine, your solver version.
    Close,
}

/// (section, first bar, end bar)
pub const SECTIONS: [(Section, f32, f32); 6] = [
    (Section::Boot, 0.0, 10.0),
    (Section::Describe, 10.0, 20.0),
    (Section::Generate, 20.0, 36.0),
    (Section::Solve, 36.0, 51.0),
    (Section::Repair, 51.0, 68.0),
    (Section::Close, 68.0, 81.0),
];

pub const TOTAL_BARS: f32 = 81.0;
pub const DURATION: f32 = TOTAL_BARS * BAR;

pub const fn bars(b: f32) -> f32 {
    b * BAR
}

pub fn span(section: Section) -> (f32, f32) {
    SECTIONS
        .iter()
        .find(|(s, _, _)| *s == section)
        .map(|(_, b0, b1)| (bars(*b0), bars(*b1)))
        .unwrap_or((0.0, 0.0))
}

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

/// The nearest beat to `t`.
pub fn snap_beat(t: f32) -> f32 {
    (t / BEAT).round() * BEAT
}

/// Exponential pulse after the most recent beat (0..1), for flashes.
pub fn beat_pulse(t: f32, decay: f32) -> f32 {
    let since = t.rem_euclid(BEAT);
    (-since * decay).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_tile_the_song() {
        let mut end = 0.0;
        for (_, b0, b1) in SECTIONS {
            assert_eq!(b0, end);
            assert!(b1 > b0);
            end = b1;
        }
        assert_eq!(end, TOTAL_BARS);
        // Long enough to read every caption, brisk otherwise.
        assert!((140.0..=170.0).contains(&DURATION));
    }
}
