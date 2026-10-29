// One clock for picture and sound: 125 BPM, 56 bars. Every section boundary
// sits on a bar line, and the water flow / hammer hits that drive both the
// renderer and the mix are defined here once.

pub const BPM: f32 = 125.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const STEP: f32 = BEAT / 4.0;
pub const TOTAL_BARS: f32 = 56.0;
pub const DURATION: f32 = TOTAL_BARS * BAR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Night,
    Dam,
    Falls,
    River,
    Iron,
    Hammer,
    Quench,
    Mark,
    Close,
}

/// (section, first bar, end bar)
pub const SECTIONS: [(Section, f32, f32); 9] = [
    (Section::Night, 0.0, 4.0),
    (Section::Dam, 4.0, 8.0),
    (Section::Falls, 8.0, 16.0),
    (Section::River, 16.0, 20.0),
    (Section::Iron, 20.0, 24.0),
    (Section::Hammer, 24.0, 36.0),
    (Section::Quench, 36.0, 40.0),
    (Section::Mark, 40.0, 52.0),
    (Section::Close, 52.0, 56.0),
];

pub const fn bars(b: f32) -> f32 {
    b * BAR
}

pub struct Now {
    pub section: Section,
    /// Seconds since the section started.
    pub local: f32,
    /// 0..1 through the section.
    pub progress: f32,
}

pub fn now(t: f32) -> Now {
    let bar = t / BAR;
    let &(section, b0, b1) = SECTIONS
        .iter()
        .find(|(_, _, end)| bar < *end)
        .unwrap_or(&SECTIONS[SECTIONS.len() - 1]);
    let (start, end) = (bars(b0), bars(b1));
    Now {
        section,
        local: t - start,
        progress: ((t - start) / (end - start)).clamp(0.0, 1.0),
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

/// Water released by the Barbellino dam, 0..1. Opens during DAM, runs through
/// the falls and the river, closes in CLOSE.
pub fn flow(t: f32) -> f32 {
    let b = t / BAR;
    if b < 4.0 {
        0.06
    } else if b < 8.0 {
        0.06 + 0.94 * smooth((b - 5.0) / 3.0)
    } else if b < 52.0 {
        1.0
    } else {
        1.0 - smooth((b - 52.5) / 3.0)
    }
}

/// Hammer strikes: every beat of HAMMER, except while the ingot is
/// thrown apart and reassembled in the last bars.
pub const HAMMER_FROM: f32 = 24.0;
pub const HAMMER_STOP: f32 = 33.0;

/// Time of the most recent hammer strike at or before t, with its index.
pub fn last_strike(t: f32) -> Option<(f32, usize)> {
    let (t0, t1) = (bars(HAMMER_FROM), bars(HAMMER_STOP));
    if t < t0 {
        return None;
    }
    let tt = t.min(t1 - 0.001);
    let k = ((tt - t0) / BEAT) as usize;
    Some((t0 + k as f32 * BEAT, k))
}
