// CROSSING: the edge of the village, an empty road into the dark. One
// traffic light swings on its wire in the wind. Cordelia walks in and stops
// under it. Red. She waits. Green. She walks on.
use super::party::{self, Light};
use super::stage::{cell_px, Stage};
use crate::ascii::COLS;
use crate::fx::{ease_in_out, window};
use crate::gfx::{mix, scale};
use crate::world::{self, Stand};

const FEET: i32 = 53;
/// The light turns green.
pub const GREEN_AT: f32 = 4.4;
/// She stops under the light, and walks on.
const STOP: (f32, f32) = (2.6, 4.8);

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    st.sky(&[0x010204, 0x040a12, 0x0a1620], 40);
    world::stars(st.g, t, 0.0, 30, 0.03, 0.6);
    world::firs(
        st.g,
        0.0,
        42,
        20,
        8.5,
        31,
        Stand {
            dark: 0x030706,
            edge: 0x2a4038,
            lit_k: 0.25,
        },
    );
    // The road in perspective: verges converging, the centre line dashed.
    for r in 40..60 {
        let d = (r - 40) as f32 / 20.0;
        for c in 0..COLS {
            let x = (c - COLS / 2) as f32 / (COLS as f32 / 2.0);
            let w = 0.12 + 0.88 * d;
            let on = x.abs() < w;
            let verge = (x.abs() - w).abs() < 0.025;
            let centre = x.abs() < 0.008 + 0.012 * d && (d * 6.0 + local * 1.2).fract() < 0.5;
            let (ch, fg, bg) = if verge {
                (if x < 0.0 { '/' } else { '\\' }, 0x8a8f8f, 0x101414)
            } else if centre {
                ('=', 0xd8c880, 0x181a1a)
            } else if on {
                (
                    if (c + r) % 11 == 0 { '.' } else { ' ' },
                    0x3a3e3e,
                    scale(0x161a1c, 0.6 + 0.4 * d),
                )
            } else {
                (
                    if (c * 7 + r) % 5 == 0 { ',' } else { ' ' },
                    0x2a3a2a,
                    0x060a08,
                )
            };
            st.g.set(c, r, ch, fg, bg);
        }
    }
    world::fog(st.g, t, local * 4.0, 32, 50, 0x34444c, 0.6);

    // The wire and the swinging light.
    let swing = 2.4 * (t * 1.3).sin() + 0.7 * (t * 2.9).sin();
    let hx = COLS as f32 / 2.0 + swing;
    for c in 0..COLS {
        let u = (c as f32 - hx) / (COLS as f32 / 2.0);
        let r = (5.0 + 3.0 * (1.0 - u * u)) as i32;
        st.g.put(c, r, '-', 0x5a6262);
    }
    let (bx, by) = (hx.round() as i32 - 3, 10);
    for r in 8..by {
        st.g.put(bx + 3, r, '|', 0x6a7272);
    }
    let green = local > GREEN_AT;
    let red_k = if green {
        0.0
    } else {
        0.9 + 0.1 * (t * 9.0).sin().abs()
    };
    let amber_k = window(local, GREEN_AT - 0.6, GREEN_AT - 0.3)
        * (1.0 - window(local, GREEN_AT, GREEN_AT + 0.1));
    let green_k = window(local, GREEN_AT, GREEN_AT + 0.15);
    // The housing: a dark box with a lit frame and a visor over each lamp.
    for r in by..by + 13 {
        for c in bx..bx + 7 {
            let edge = c == bx || c == bx + 6;
            let lid = r == by || r == by + 12;
            let ch = if edge {
                '|'
            } else if lid {
                '='
            } else {
                ' '
            };
            st.g.set(c, r, ch, 0x8a9494, 0x161a1a);
        }
    }
    let lamps = [(0xff2a1a, red_k), (0xffb21a, amber_k), (0x2aff8a, green_k)];
    for (i, (rgb, k)) in lamps.iter().enumerate() {
        let r = by + 2 + i as i32 * 4;
        for c in bx + 2..bx + 5 {
            st.g.put(c, r - 1, '_', 0x5a6262);
            let fg = mix(0x2a2a2a, *rgb, *k);
            st.g.set(c, r, '@', fg, mix(0x101212, scale(*rgb, 0.55), *k));
            st.g.set(
                c,
                r + 1,
                '@',
                scale(fg, 0.8),
                mix(0x101212, scale(*rgb, 0.4), *k),
            );
        }
        let (x, y) = cell_px(bx as f32 + 3.0, r as f32 + 0.5);
        st.glow(x, y, 110.0, *rgb, 0.55 * k);
        st.glow(x, y, 26.0, 0xffffff, 0.25 * k);
        // Its colour on the wet road.
        st.glow(x, 930.0, 460.0, *rgb, 0.14 * k);
    }

    // Cordelia: in from the left, stops under the light, walks on.
    let enter = ease_in_out(local / STOP.0);
    let leave = ease_in_out((local - STOP.1) / 2.2);
    let col = -20.0 + (COLS as f32 / 2.0 - 14.0 + 20.0) * enter + 90.0 * leave.max(0.0);
    let walking = local < STOP.0 - 0.1 || local > STOP.1;
    let light = Light {
        tint: if green { 0xa8e8c0 } else { 0xf0a898 },
        k: 0.45,
        rim: Some((if green { 0x6affb0 } else { 0xff5a4a }, 1, 0.55)),
    };
    party::cordelia(st, t, col, FEET, walking, light, 1.0);
}
