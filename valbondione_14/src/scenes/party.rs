// How the two of them move through a scene: walking in place while the
// world scrolls (the camera follows), or across a still frame; standing
// in the passing pose. Positions are in cells (feet on `row`).
use super::stage::{cell_px, Stage};
use crate::sprites::{walk_frame, Id};
use crate::timeline::BEAT;

/// Two beats a stride cycle: they walk to the music.
pub const STRIDE: f32 = 2.0 * BEAT;
/// Ground covered per second at that stride, in cells.
pub const PACE: f32 = 11.0;

#[derive(Clone, Copy)]
pub struct Light {
    /// Multiplied into the figure.
    pub tint: u32,
    pub k: f32,
    /// A rim light: colour, side (-1 left, 1 right), strength.
    pub rim: Option<(u32, i32, f32)>,
}

pub const MOON: Light = Light {
    tint: 0x8ea4b8,
    k: 0.5,
    rim: Some((0xe8f0f8, -1, 0.5)),
};

/// The pixel y of feet standing on cell row `row` (its bottom edge).
pub fn feet(row: i32) -> f32 {
    ((row + 1) * 18) as f32
}

/// Cordelia at (col, feet row): walking when `walking`, else standing.
pub fn cordelia(st: &mut Stage, t: f32, col: f32, row: i32, walking: bool, l: Light, alpha: f32) {
    let f = if walking {
        walk_frame(t, STRIDE, 0.0)
    } else {
        1
    };
    st.sprite(
        Id::CordeliaWalk(f),
        cell_px(col, 0.0).0,
        feet(row),
        l.tint,
        l.k,
        l.rim,
        alpha,
        0,
    );
}

/// Victor at (col, feet row), half a stride out of step with her.
pub fn victor(st: &mut Stage, t: f32, col: f32, row: i32, walking: bool, l: Light, alpha: f32) {
    let f = if walking {
        walk_frame(t, STRIDE, 0.5)
    } else {
        1
    };
    st.sprite(
        Id::VictorWalk(f),
        cell_px(col, 0.0).0,
        feet(row),
        l.tint,
        l.k,
        l.rim,
        alpha,
        0,
    );
}
