// One clock for picture and sound: 120 BPM, so one bar is two seconds and
// every section of the cut (cut.rs) starts on a bar line. The song follows
// the same table.
use crate::cut::SECTIONS;

pub const BPM: f32 = 120.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The warp, the title and the plugin's models.
    Intro,
    /// A recorded Desktop take fills the picture; the demo draws the band.
    Footage,
    /// The solve, redrawn from its improving plans.
    Week,
    /// The stock of powder along the week, and the delivery that slips.
    Powder,
    /// Cordelia: concern, defiance, resolve.
    Call,
    /// The repair, redrawn from the broken week and the repaired one.
    Repair,
    /// The mark, Cordelia at ease, the install line.
    Outro,
}

#[derive(Clone, Copy, Debug)]
pub struct Now {
    pub index: usize,
    pub kind: Kind,
    /// Seconds since the section started.
    pub local: f32,
    /// Section length in seconds.
    pub len: f32,
}

pub fn now(t: f32) -> Now {
    let index = SECTIONS
        .iter()
        .position(|s| t < s.3)
        .unwrap_or(SECTIONS.len() - 1);
    let (kind, _, from, to) = SECTIONS[index];
    Now {
        index,
        kind,
        local: t - from,
        len: to - from,
    }
}

/// Next section start after t (for SPACE), wrapping to 0.
pub fn next_boundary(t: f32) -> f32 {
    SECTIONS
        .iter()
        .map(|s| s.2)
        .find(|start| *start > t + 0.25)
        .unwrap_or(0.0)
}

pub fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Fast out, slow in: for things that land.
pub fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

/// 0..1 over [a, b] of the clock, eased.
pub fn span(t: f32, a: f32, b: f32) -> f32 {
    smooth((t - a) / (b - a))
}

/// A decaying flash on each beat: 1 on the beat, falling away.
pub fn pulse(t: f32) -> f32 {
    (-(t.rem_euclid(BEAT)) * 7.0).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cut::DURATION;

    #[test]
    fn every_section_starts_on_a_bar_line() {
        for (_, title, from, to) in SECTIONS {
            for edge in [from, to] {
                let bars = edge / BAR;
                assert!((bars - bars.round()).abs() < 1e-3, "{title} at {edge}");
            }
        }
        assert!((SECTIONS[SECTIONS.len() - 1].3 - DURATION).abs() < 1e-3);
    }

    #[test]
    fn the_sections_follow_each_other() {
        for w in SECTIONS.windows(2) {
            assert!((w[0].3 - w[1].2).abs() < 1e-3);
        }
    }
}
