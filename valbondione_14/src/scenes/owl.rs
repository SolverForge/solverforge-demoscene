// OWL: the opening. Night over the firs of the upper valley, the moon
// rising out of drifting fog. On a bough in the foreground an owl resolves
// out of noise (glyphfx decrypt of the owl's own picture as text), then
// stands there as itself; its eyes light amber, and it blinks.
use super::stage::Stage;
use crate::fx::{h2, window};
use crate::glyphs;
use crate::sprites::{Id, SCALE};
use crate::world::{self, Range, Stand};

/// The owl's feet: pixel x, cell row (on the bough).
const OWL_X: i32 = 560;
const BOUGH: i32 = 40;
const OWL_SPEED: f32 = 1.6;
const OWL_AT: f32 = 1.5;

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    st.sky(&[0x010307, 0x06121c, 0x0f2630, 0x1b3a3c], 44);
    let rise = window(local, 0.0, 6.0);
    st.moon(150.0, 15.0 - 3.0 * rise, 62.0, window(local, 0.5, 4.0));
    world::stars(st.g, t, local * 0.2, 40, 0.05, window(local, 0.3, 3.0));
    world::ridge(
        st.g,
        local * 0.3,
        44,
        16.0,
        0.018,
        3.0,
        Range {
            rock: 0x0a1416,
            lit: 0x5a7480,
            snow: Some(0xc8d8e0),
            haze: 0x1b3a3c,
        },
    );
    world::fog(st.g, t, local * 2.0, 30, 48, 0x47606a, 0.75);
    world::firs(
        st.g,
        local * 0.8,
        56,
        26,
        9.0,
        4,
        Stand {
            dark: 0x050d0c,
            edge: 0x5f8078,
            lit_k: 0.45,
        },
    );
    world::fog(st.g, t, local * 3.0, 46, 60, 0x2c4448, 0.6);

    // The bough: bark along a slight sag, sprays of needles.
    let owl_col = OWL_X / 10;
    for c in 0..owl_col + 22 {
        let sag = (c as f32 / 40.0).powi(2) * 0.6;
        let r = BOUGH + 1 + sag as i32;
        let bark = if h2(c, 3) < 0.3 { '~' } else { '-' };
        st.g.set(c, r, bark, 0x8a7458, 0x2a1e14);
        st.g.set(
            c,
            r + 1,
            if h2(c, 4) < 0.4 { '_' } else { ' ' },
            0x4a3a28,
            0x140e09,
        );
        // Sprays of needles hanging from the bough.
        if h2(c / 4, 5) < 0.6 {
            let len = 1 + (h2(c, 6) * 2.5) as i32;
            for k in 0..len {
                let ch = if (c + k) % 2 == 0 { '/' } else { '\\' };
                st.g.put(c, r + 2 + k, ch, if k == 0 { 0x4a7a5c } else { 0x2e5a40 });
            }
            st.g.put(c, r - 1, if c % 2 == 0 { '\\' } else { '/' }, 0x5a8a6c);
        }
    }

    // The owl: decrypt, then the sprite over it.
    let owl = st.cast.get(Id::Owl);
    let feet_y = (BOUGH + 1) * 18 + 4;
    let (ox, oy) = owl.origin(OWL_X, feet_y);
    let since = local - OWL_AT;
    let done = st.glyphs.owl.seconds() / OWL_SPEED;
    let solid = window(since, done - 0.6, done + 0.8);
    glyphs::play(
        st.g,
        &st.glyphs.owl,
        since,
        OWL_SPEED,
        ox / 10,
        oy / 18,
        1.0 - solid,
    );
    let moonlit = Some((0xdfe8f0, 1, 0.5));
    st.sprite(
        Id::Owl,
        OWL_X as f32,
        feet_y as f32,
        0x95a8b8,
        0.45,
        moonlit,
        solid,
        0,
    );
    // Its eyes: amber, with a slow blink.
    if solid > 0.0 {
        let blink = ((local - 2.0) * 0.45).fract() < 0.04;
        for (ex, ey) in owl.warm_points() {
            let (x, y) = (
                OWL_X as f32 + ex * SCALE as f32,
                feet_y as f32 + ey * SCALE as f32,
            );
            let k = solid * if blink { 0.1 } else { 1.0 };
            st.glow(x, y, 20.0, 0xffa21c, 0.5 * k);
            st.glow(x, y, 6.0, 0xfff0c0, 0.4 * k);
        }
    }
}
