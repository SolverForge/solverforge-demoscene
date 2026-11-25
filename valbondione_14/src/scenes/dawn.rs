// DAWN: the fire is embers, the sky over the Orobie goes from night to
// rose to gold and the peaks catch the first light. She sleeps on his
// shoulder; he keeps the watch. The fireflies swarm one last time and
// settle into a ring above the valley (glyphfx swarm), and inside it the
// SolverForge mark draws itself. Fade.
use super::stage::{cell_px, Px, Stage};
use crate::fx::{smooth, window};
use crate::gfx::{mix, ramp};
use crate::glyphs;
use crate::sprites::Id;
use crate::world::{self, Range};

pub const RING_AT: f32 = 4.5;
pub const MARK_AT: f32 = 11.0;
pub const FADE: (f32, f32) = (16.5, 20.0);
pub const RING_COL: i32 = 61;
pub const RING_ROW: i32 = 1;

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    let day = smooth(local / 14.0);
    let night = [0x010205, 0x030a10, 0x0a1418];
    let dawn = [0x1a2a4a, 0x7a5a7a, 0xf0b080];
    let stops: Vec<u32> = (0..3).map(|i| mix(night[i], dawn[i], day)).collect();
    st.sky(&stops, 38);
    world::stars(st.g, t, 0.0, 30, 0.05, 1.0 - day);
    // Alpenglow: the light faces of the far range turn rose.
    world::ridge(
        st.g,
        0.0,
        38,
        22.0,
        0.015,
        11.0,
        Range {
            rock: mix(0x0e1517, 0x3a2a3a, day),
            lit: mix(0x3a4a58, 0xffa898, day),
            snow: Some(mix(0xc6d2d8, 0xffe0d0, day)),
            haze: mix(0x0a1418, 0xb08080, day),
        },
    );
    world::ridge(
        st.g,
        0.0,
        46,
        12.0,
        0.03,
        5.0,
        Range {
            rock: mix(0x070d0c, 0x1a1a24, day),
            lit: mix(0x14231f, 0x5a4a50, day),
            snow: None,
            haze: mix(0x0a1418, 0x3a2a34, day),
        },
    );
    world::fog(
        st.g,
        t,
        local * 1.5,
        36,
        52,
        mix(0x3a4a52, 0xe0b8b0, day),
        0.85,
    );
    world::ground(st.g, 0.0, 51, mix(0x0a0e0a, 0x1a1a14, day), 0x3a4a2a);
    // Embers and a thread of smoke.
    for k in 0..8 {
        let ph = (t * 0.3 + k as f32 * 0.125) % 1.0;
        let c = 140.0 + 2.0 * (ph * 9.0 + k as f32).sin();
        st.g.light(
            c as i32,
            (51.0 - ph * 16.0) as i32,
            '~',
            ramp(&[0x9a9a9a, 0x2a2a2a], ph),
        );
    }
    st.g.set(140, 52, '*', 0xff6a20, 0x3a1004);
    let (ex, ey) = cell_px(140.0, 52.0);
    st.glow(ex, ey, 40.0, 0xff5a10, 0.5 * (1.0 - day * 0.5));

    // The pair, from behind, on the rise.
    let tint = mix(0x8aa0b8, 0xffd0b0, day);
    let rim = Some((mix(0xa0b8c8, 0xffe0a0, day), 1, 0.6));
    st.sprite(Id::Pair, 960.0, 59.0 * 18.0, tint, 0.35, rim, 1.0, 0);

    // The fireflies' ring, then the mark inside it.
    let gone = 1.0 - window(local, FADE.0 - 1.5, FADE.0);
    glyphs::play(
        st.g,
        &st.glyphs.ring,
        local - RING_AT,
        8.6,
        RING_COL,
        RING_ROW,
        gone,
    );
    let mark = local - MARK_AT;
    if mark > 0.0 {
        let (cx, cy) = cell_px(RING_COL as f32 + 35.0, RING_ROW as f32 + 13.0);
        st.px.push(Px::Mark {
            cx,
            cy,
            r: 120.0,
            progress: smooth(mark / 3.0),
            gain: window(mark, 0.0, 0.8),
        });
        st.glow(
            cx,
            cy,
            300.0,
            0x3dbb8a,
            0.25 * window(mark, 2.0, 3.5) * gone,
        );
        st.corner = 0.0;
    }
    st.look.fade = 1.0 - window(local, FADE.0, FADE.1);
}
