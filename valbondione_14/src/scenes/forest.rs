// FOREST: the firs close in and the fog comes down. A knight stands at the
// forest's edge in a shaft of moonlight, sword planted. She stops; he
// joins her; they walk on together. The nearest trunks pass in front.
use super::party::{self, MOON, PACE};
use super::stage::{cell_px, Px, Stage};
use crate::fx::window;
use crate::sprites::Id;
use crate::world::{self, Stand};

const FEET: i32 = 54;
/// Where the knight stands, in world columns.
const KNIGHT_AT: f32 = 174.25;
/// Her screen column (the camera follows her).
const HER_COL: f32 = 70.0;
/// Seconds: walking until STOP, meeting, then walking from GO.
pub const STOP: f32 = 7.5;
pub const JOIN: f32 = 11.0;
pub const GO: f32 = 14.0;

/// Camera position: the walking pace, slowing to a stop over the last
/// 1.5 s before STOP, still until GO, back to pace over one second.
fn cam(local: f32) -> f32 {
    let a = STOP - 1.5;
    let stop_cam = PACE * (a + 0.75);
    if local < a {
        PACE * local
    } else if local < STOP {
        let u = local - a;
        PACE * (a + u - u * u / 3.0)
    } else if local < GO {
        stop_cam
    } else {
        let u = local - GO;
        stop_cam + PACE * if u < 1.0 { u * u / 2.0 } else { u - 0.5 }
    }
}

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    let (cam, stand) = (cam(local), KNIGHT_AT - cam(GO));
    st.sky(&[0x010304, 0x04100e, 0x0c2220], 34);
    st.moon(160.0 - cam * 0.02, 8.0, 40.0, 1.0);
    world::stars(st.g, t, cam * 0.03, 16, 0.03, 0.6);
    world::firs(
        st.g,
        cam * 0.15,
        40,
        30,
        7.0,
        41,
        Stand {
            dark: 0x0c1c1a,
            edge: 0x4a6a66,
            lit_k: 0.3,
        },
    );
    world::fog(st.g, t, cam * 0.15, 20, 44, 0x3e5a5c, 0.65);
    world::firs(
        st.g,
        cam * 0.45,
        50,
        36,
        10.0,
        42,
        Stand {
            dark: 0x060f0d,
            edge: 0x3f6a5a,
            lit_k: 0.45,
        },
    );
    world::ground(st.g, cam, 55, 0x0a130f, 0x2f4a32);
    world::fog(st.g, t, cam * 0.6, 42, 60, 0x4a6466, 0.45);

    // The knight's shaft of moonlight, and the knight.
    let kc = KNIGHT_AT - cam;
    let (kx, _) = cell_px(kc, 0.0);
    st.px.push(Px::Beam {
        x: kx + 170.0,
        y: -40.0,
        dir: 1.75,
        width: 140.0,
        len: 1050.0,
        rgb: 0x9fb8c8,
        k: 0.5 * window(local, 2.0, 6.0) * (1.0 - window(local, GO, GO + 2.0)),
    });
    if local < GO {
        st.sprite(
            Id::VictorGuard,
            kx,
            party::feet(FEET),
            0xc0d0dc,
            0.3,
            Some((0xeef4fa, 1, 0.6)),
            1.0,
            0,
        );
    } else {
        party::victor(st, t, stand, FEET, true, MOON, 1.0);
    }
    let walking_her = !(STOP - 0.2..GO).contains(&local);
    party::cordelia(st, t, HER_COL, FEET, walking_her, MOON, 1.0);
    // The moment he joins: a ring of light where he stands.
    let j = local - JOIN;
    if (0.0..1.6).contains(&j) {
        let (x, y) = cell_px(kc, FEET as f32 - 12.0);
        st.glow(x, y, 120.0 + 420.0 * j, 0x9fffd0, 0.45 * (1.0 - j / 1.6));
    }
    // The nearest trunks pass in front of them.
    world::firs(
        st.front,
        cam * 1.5,
        66,
        46,
        41.0,
        43,
        Stand {
            dark: 0x010302,
            edge: 0x10201a,
            lit_k: 0.15,
        },
    );
}
