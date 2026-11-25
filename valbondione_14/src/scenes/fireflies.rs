// FIREFLIES: the gorge of the Serio below Valbondione. Fireflies rise over
// the water, a light each; a faint, tangled round joins them, then
// untangles one move at a time (2-opt, tour.rs) until no move helps, and
// the shortest loop glows on the beat. The day's work, as the valley does
// it. They walk along the bank underneath.
use super::party::{self, Light, PACE};
use super::stage::{cell_px, Stage};
use crate::fx::{h2, window};
use crate::gfx::mix;
use crate::timeline::BEAT;
use crate::tour::length;
use crate::world::{self, Range, Stand};

const FEET: i32 = 56;
/// The search plays from SEARCH.0 to SEARCH.1 seconds.
pub const SEARCH: (f32, f32) = (3.5, 11.5);

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    let cam = local * PACE;
    st.sky(&[0x010307, 0x03101a, 0x082020], 30);
    world::stars(st.g, t, cam * 0.04, 24, 0.035, 0.7);
    world::ridge(
        st.g,
        cam * 0.12,
        34,
        24.0,
        0.045,
        17.0,
        Range {
            rock: 0x060d0c,
            lit: 0x2c4440,
            snow: None,
            haze: 0x0a1c1c,
        },
    );
    world::firs(
        st.g,
        cam * 0.3,
        41,
        14,
        5.5,
        51,
        Stand {
            dark: 0x040b09,
            edge: 0x2e4a40,
            lit_k: 0.3,
        },
    );
    world::river(
        st.g,
        t,
        cam * 0.6,
        42,
        10,
        &[0x020a0c, 0x0a2a2c, 0x2f6a62, 0xa8e0c8],
        8.0,
    );
    world::fog(st.g, t, cam * 0.5, 36, 50, 0x264640, 0.5);
    world::ground(st.g, cam, 52, 0x08100c, 0x2a4a30);

    // The fireflies, bobbing about their points; their reflections.
    let rise = window(local, 0.5, 3.0);
    let tour = st.tour;
    let bob = |i: usize| {
        let (x, y) = tour.pts[i];
        let k = i as f32;
        (
            x + 9.0 * (t * 0.9 + k).sin(),
            y + 7.0 * (t * 1.3 + k * 1.7).cos() + 160.0 * (1.0 - rise),
        )
    };
    let n = tour.pts.len();
    let (s0, s1) = SEARCH;
    let steps = tour.steps.len();
    let progress = ((local - s0) / (s1 - s0)).clamp(0.0, 1.0);
    let pos = progress * (steps - 1) as f32;
    let idx = (pos.floor() as usize).min(steps - 1);
    let since_move = pos.fract();
    let order = &tour.steps[idx];
    let prev = &tour.steps[idx.saturating_sub(1)];
    let done = local > s1;
    let beat = (-((t / BEAT).fract()) * 5.0).exp();
    let line_k = window(local, s0 - 1.0, s0) * (1.0 - window(local, 15.5, 16.6));
    if line_k > 0.0 {
        let best = length(&tour.pts, tour.last());
        let cur = length(&tour.pts, order);
        let tangle = ((cur - best) / best).clamp(0.0, 1.0);
        for i in 0..n {
            let (a, b) = (order[i], order[(i + 1) % n]);
            // Edges the last move made flash white.
            let fresh = !(0..n).any(|j| {
                let (p, q) = (prev[j], prev[(j + 1) % n]);
                (p == a && q == b) || (p == b && q == a)
            });
            let base = mix(0x9cf26a, 0x3a6a2a, tangle);
            let (rgb, k) = if done {
                (0xc8ff8a, 0.45 + 0.55 * beat)
            } else if fresh {
                (mix(0xffffff, base, since_move), 0.9)
            } else {
                (base, 0.3 + 0.3 * (1.0 - tangle))
            };
            st.line(
                bob(a),
                bob(b),
                if done { 2.4 } else { 1.4 },
                rgb,
                k * line_k,
            );
        }
    }
    let (_, water) = cell_px(0.0, 42.0);
    for i in 0..n {
        let (x, y) = bob(i);
        let tw = 0.6 + 0.4 * (t * (2.0 + h2(i as i32, 5) * 3.0) + i as f32).sin();
        let k = rise * tw;
        st.glow(x, y, 32.0, 0xb8ff5a, 0.75 * k);
        st.glow(x, y, 7.0, 0xffffff, 0.9 * k);
        st.glow(x, 2.0 * water - y + 240.0, 22.0, 0x7fd06a, 0.22 * k);
    }
    let light = Light {
        tint: 0xc0e4b0,
        k: 0.3,
        rim: Some((0xd8ffa0, -1, 0.35 + 0.3 * rise)),
    };
    party::victor(st, t, 104.0, FEET, true, light, 1.0);
    party::cordelia(st, t, 62.0, FEET, true, light, 1.0);
}
