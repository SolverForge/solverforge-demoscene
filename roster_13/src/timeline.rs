// One clock for picture and sound: 120 BPM, so one bar is two seconds. The
// cut (cut.rs) puts every segment on a bar line; the song reads the same
// table, so a drop lands where a take starts and the break is the call.
use crate::cut::{Kind, SEGMENTS, TOTAL};

pub const BPM: f32 = 120.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;
pub const STEP: f32 = BEAT / 4.0;
pub const DURATION: f32 = TOTAL;
pub const TOTAL_BARS: usize = (TOTAL / BAR) as usize;

pub const fn bars(b: f32) -> f32 {
    b * BAR
}

#[derive(Clone, Copy, Debug)]
pub struct Now {
    /// Index into cut::SEGMENTS.
    pub seg: usize,
    pub kind: Kind,
    /// Seconds since the segment started, and its length.
    pub local: f32,
    pub len: f32,
}

impl Now {
    pub fn prev(&self) -> Option<Kind> {
        self.seg.checked_sub(1).map(|i| SEGMENTS[i].2)
    }
    pub fn next(&self) -> Option<Kind> {
        SEGMENTS.get(self.seg + 1).map(|s| s.2)
    }
}

pub fn now(t: f32) -> Now {
    let seg = SEGMENTS
        .iter()
        .position(|(_, end, _)| t < *end)
        .unwrap_or(SEGMENTS.len() - 1);
    let (a, b, kind) = SEGMENTS[seg];
    Now {
        seg,
        kind,
        local: t - a,
        len: b - a,
    }
}

/// Next segment start after t (for SPACE), wrapping to 0.
pub fn next_boundary(t: f32) -> f32 {
    SEGMENTS
        .iter()
        .map(|(a, _, _)| *a)
        .find(|start| *start > t + 0.25)
        .unwrap_or(0.0)
}

pub fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Stable 0..1 hash.
pub fn hash(i: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_segment_starts_on_a_bar_line_and_the_film_is_whole_bars() {
        for (a, b, _) in SEGMENTS {
            assert!(
                (a / BAR).fract().abs() < 1e-4,
                "segment at {a} is off the bar"
            );
            assert!(b > a);
        }
        assert!((TOTAL / BAR).fract().abs() < 1e-4);
        for w in SEGMENTS.windows(2) {
            assert_eq!(w[0].1, w[1].0, "segments leave a gap");
        }
    }
}
