// VILLAGE: Valbondione at nightfall. The last of the light behind Pizzo
// Coca's snow, the slate roofs and lit windows, San Lorenzo's bell tower,
// lamps along the road, the Serio running in front. Cordelia walks out of
// the village, tired; the camera follows her along the road.
use super::party::{self, Light, PACE};
use super::stage::{cell_px, Stage};
use crate::fx::{h2, window};
use crate::world::{self, Range, Stand};

const FEET: i32 = 49;
const LAMPS: [i32; 6] = [20, 64, 118, 160, 204, 246];

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    let cam = local * PACE;
    st.sky(&[0x040814, 0x111c34, 0x3a2c4c, 0x7a4a52], 32);
    world::stars(st.g, t, cam * 0.04, 22, 0.035, 0.85);
    world::ridge(
        st.g,
        cam * 0.06,
        32,
        20.0,
        0.014,
        11.0,
        Range {
            rock: 0x141a22,
            lit: 0x6a7488,
            snow: Some(0xdfe2ee),
            haze: 0x2a2a3c,
        },
    );
    world::ridge(
        st.g,
        cam * 0.2,
        39,
        11.0,
        0.03,
        5.0,
        Range {
            rock: 0x080e0e,
            lit: 0x283a3a,
            snow: None,
            haze: 0x141c22,
        },
    );
    world::firs(
        st.g,
        cam * 0.35,
        44,
        12,
        6.5,
        21,
        Stand {
            dark: 0x050b0a,
            edge: 0x3a5a50,
            lit_k: 0.35,
        },
    );
    world::fog(st.g, t, cam * 0.3, 34, 44, 0x3a3a50, 0.55);
    world::village(st.g, t, cam, 44, 1.0);
    // The road: setts in the lamplight.
    for r in 44..51 {
        for c in 0..crate::ascii::COLS {
            let wc = c + cam as i32;
            let ch = if (wc + r * 2) % 4 == 0 {
                '_'
            } else if h2(wc, r) < 0.12 {
                '.'
            } else {
                ' '
            };
            st.g.set(c, r, ch, 0x4a4640, 0x181715);
        }
    }
    world::river(
        st.g,
        t,
        cam * 1.1,
        51,
        9,
        &[0x03101a, 0x0c2c3a, 0x3f7f8f, 0xbfe3ea],
        6.0,
    );
    // Street lamps: a post and a warm head; light pools on the road and
    // in the water.
    for &lx in &LAMPS {
        let c = lx - cam as i32;
        for r in 35..FEET + 1 {
            st.g.put(c, r, '|', 0x6a645a);
        }
        st.g.put(c - 1, 34, '/', 0x8a8070);
        st.g.set(c, 34, '#', 0xfff2c4, 0x8a5a14);
        st.g.put(c + 1, 34, '\\', 0x8a8070);
        let (x, y) = cell_px(c as f32, 34.0);
        st.glow(x, y, 120.0, 0xffb347, 0.5);
        st.glow(x, y + 240.0, 200.0, 0xff9a2a, 0.12);
        st.glow(
            x,
            y + 400.0,
            40.0,
            0xffc870,
            0.25 * (0.7 + 0.3 * (t * 2.0 + lx as f32).sin()),
        );
    }
    let light = Light {
        tint: 0xd0bca0,
        k: 0.3,
        rim: Some((0xffc27a, -1, 0.45)),
    };
    party::cordelia(st, t, 70.0, FEET, true, light, window(local, 0.0, 1.2));
}
