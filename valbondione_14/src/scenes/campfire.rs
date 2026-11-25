// CAMPFIRE: a clearing, the stars, the firs around. Victor lays the
// kindling, it catches (glyphfx burn) and the fire takes (the Amiga heat
// buffer). They sit on two logs with marshmallows on sticks. Hers catches
// fire; she blows it out; they laugh. The fire settles; a shooting star.
use super::party::{self, Light};
use super::stage::{cell_px, Stage};
use crate::amiga;
use crate::ascii::COLS;
use crate::fx::{ease_in_out, h2, window};
use crate::gfx::{mix, scale};
use crate::glyphs;
use crate::sprites::{Id, Sprite, SCALE};
use crate::timeline::{span, Scene};
use crate::world::{self, Stand};

pub const FIRE_COL: i32 = 96;
pub const FIRE_BASE: i32 = 52;
const FEET: i32 = 54;
pub const KINDLE: f32 = 4.0;
pub const SPARK: f32 = 6.2;
/// Her marshmallow catches, she blows it out, the laugh.
pub const CATCH: f32 = 19.0;
pub const BLOW: f32 = 21.6;
pub const LAUGH: f32 = 22.4;
pub const LAUGH2: f32 = 31.0;
pub const STAR: f32 = 33.0;

/// The far tip of the stick (the marshmallow), in art pixels from the
/// anchor: the outermost opaque pixel in the upper half on that side.
fn tip(s: &Sprite, right: bool) -> (i32, i32) {
    let mut best: Option<(i32, i32)> = None;
    for y in 0..s.h / 2 {
        for x in 0..s.w {
            if s.px[(y * s.w + x) as usize].is_some() {
                let better = match best {
                    None => true,
                    Some((bx, _)) => (right && x > bx) || (!right && x < bx),
                };
                if better {
                    best = Some((x, y));
                }
            }
        }
    }
    let (x, y) = best.unwrap_or((s.ax, 0));
    (x - s.ax, y - (s.ay + 1))
}

pub fn fuel(local: f32) -> f32 {
    window(local, SPARK, SPARK + 2.0)
        * (1.0 - 0.35 * window(local, 34.0, 40.0))
        * (0.92 + 0.08 * (local * 1.7).sin())
}

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    st.sky(&[0x010205, 0x030a10, 0x08140f], 42);
    st.moon(30.0, 8.0, 30.0, 1.0);
    world::stars(st.g, t, 0.0, 38, 0.06, 1.0);
    amiga::meteor(
        st.g,
        t,
        span(Scene::Campfire).0 + STAR,
        1.4,
        (140.0, 4.0),
        (-70.0, 10.0),
    );
    world::firs(
        st.g,
        0.0,
        45,
        34,
        9.5,
        71,
        Stand {
            dark: 0x040a08,
            edge: 0x2a4a3c,
            lit_k: 0.25,
        },
    );
    world::ground(st.g, 0.0, 45, 0x0a0e0a, 0x283a22);

    let f = fuel(local);
    let flick = 0.85 + 0.15 * (t * 11.0).sin() * (t * 7.3).cos();
    // Firelight on the ground and the near trees.
    for r in 20..60 {
        for c in 0..COLS {
            let (dx, dy) = ((c - FIRE_COL) as f32 / 46.0, (r - FIRE_BASE) as f32 / 14.0);
            let k = (1.0 - (dx * dx + dy * dy).sqrt()).max(0.0) * f * flick;
            if k > 0.0 {
                if let Some(cell) = st.g.solid(c, r) {
                    cell.bg = mix(cell.bg, 0x4a2208, k * 0.7);
                    cell.fg = mix(cell.fg, 0xffb060, k * 0.8);
                }
            }
        }
    }
    // Stones ringing the fire.
    for dc in [-9, -7, -5, 5, 7, 9] {
        st.g.set(FIRE_COL + dc, FIRE_BASE + 1, 'o', 0x9a9088, 0x2a2420);
    }
    // The kindling burns in (glyphfx), the fire takes over.
    glyphs::play(
        st.g,
        &st.glyphs.kindling,
        local - KINDLE,
        0.8,
        FIRE_COL - 5,
        FIRE_BASE - 2,
        1.0,
    );
    st.fire.advance((local * 30.0) as i64, f);
    st.fire.draw(st.g, FIRE_COL, FIRE_BASE, 1.0);
    // The strike: sparks fly from the flint.
    let s = local - SPARK;
    if (0.0..0.8).contains(&s) {
        for k in 0..16 {
            let a = -1.6 + (h2(k, 9) - 0.5) * 2.2;
            let d = 220.0 * s * (0.5 + h2(k, 8));
            let (x, y) = cell_px(FIRE_COL as f32, FIRE_BASE as f32 - 1.0);
            st.glow(
                x + a.cos() * d,
                y + a.sin() * d,
                10.0,
                0xfff0b0,
                1.0 - s / 0.8,
            );
        }
    }
    // On the solo pickup a gust carries a second spiral of hot embers
    // away from the party and out into the sky.
    let solo = window(
        local,
        24.0 * crate::timeline::BEAT,
        26.0 * crate::timeline::BEAT,
    );
    if solo > 0.0 {
        for i in 0..32 {
            let u = (local * 0.16 + h2(i, 41)).fract();
            let a = u * 7.0 + i as f32 * 2.4;
            let (x, y) = cell_px(
                FIRE_COL as f32 + u * (26.0 + 12.0 * a.sin()),
                FIRE_BASE as f32 - 5.0 - u * 38.0,
            );
            st.glow(
                x,
                y,
                4.0 + 4.0 * (1.0 - u),
                0xffad56,
                solo * (1.0 - u) * 0.7,
            );
        }
    }
    // Rising sparks.
    for k in 0..(28.0 * f) as i32 {
        let life = 2.4;
        let ph = ((t * 0.7 + h2(k, 1) * life) / life).fract();
        let c = FIRE_COL as f32 + (h2(k, 2) - 0.5) * 8.0 + 6.0 * (ph * 6.0 + k as f32).sin();
        let r = FIRE_BASE as f32 - 10.0 - ph * 30.0;
        st.g.light(
            c as i32,
            r as i32,
            if ph < 0.4 { '*' } else { '.' },
            scale(0xffa040, (1.0 - ph) * f),
        );
    }
    let (fx, fy) = cell_px(FIRE_COL as f32, FIRE_BASE as f32 - 5.0);
    st.glow(fx, fy, 560.0, 0xff7a20, 0.32 * f * flick);
    st.glow(fx, fy + 40.0, 150.0, 0xffd080, 0.4 * f * flick);

    // The two of them: walking in, then on their logs.
    let sit = window(local, 3.0, 3.6);
    let laughing = (LAUGH..LAUGH + 5.0).contains(&local) || (LAUGH2..LAUGH2 + 3.0).contains(&local);
    let bob = if laughing && ((t * 7.0) as i32) % 2 == 0 {
        1
    } else {
        0
    };
    let warm = |dir: i32| Light {
        tint: 0xffc890,
        k: 0.3 + 0.25 * f,
        rim: Some((0xffa050, dir, (0.7 * f * flick).min(0.9))),
    };
    let (cc, vc) = (FIRE_COL - 32, FIRE_COL + 32);
    if sit < 1.0 {
        let x = -30.0 + (cc as f32 + 30.0) * ease_in_out(local / 3.0);
        party::victor(st, t, x + 40.0, FEET, true, warm(1), 1.0 - sit);
        party::cordelia(st, t, x, FEET, true, warm(1), 1.0 - sit);
    }
    if sit > 0.0 {
        for (c0, c1) in [(cc - 16, cc + 12), (vc - 12, vc + 16)] {
            for c in c0..c1 {
                st.g.set(c, FEET + 1, '=', 0x8a6a42, 0x2a1a0c);
            }
        }
        let (cx, vx) = (cell_px(cc as f32, 0.0).0, cell_px(vc as f32, 0.0).0);
        let y = party::feet(FEET);
        let l = warm(1);
        st.sprite(Id::CordeliaRoast, cx, y, l.tint, l.k, l.rim, sit, bob);
        let l = warm(-1);
        st.sprite(Id::VictorRoast, vx, y, l.tint, l.k, l.rim, sit, bob);
        // Her marshmallow catches, burns, is blown out: smoke.
        let (tx, ty) = tip(st.cast.get(Id::CordeliaRoast), true);
        let (mx, my) = (cx + (tx * SCALE) as f32, y + ((ty - bob) * SCALE) as f32);
        let burn = window(local, CATCH, CATCH + 0.6) * (1.0 - window(local, BLOW, BLOW + 0.2));
        if burn > 0.0 {
            for k in 0..7 {
                let h = 6.0 + 14.0 * h2(k, (t * 20.0) as i32);
                st.glow(
                    mx + (k as f32 - 3.0) * 3.0,
                    my - h,
                    9.0,
                    0xffb040,
                    0.8 * burn,
                );
            }
            st.glow(mx, my - 6.0, 46.0, 0xff8a20, 0.7 * burn);
        }
        let puff = local - BLOW;
        if (0.0..3.0).contains(&puff) {
            let a = puff / 3.0;
            for k in 0..12 {
                let x = mx + 10.0 + a * 120.0 * h2(k, 3) + 8.0 * (k as f32 + t * 3.0).sin();
                let yy = my - 10.0 - a * 150.0 * h2(k, 4);
                st.glow(x, yy, 14.0 + 30.0 * a, 0x8a8a8a, 0.3 * (1.0 - a));
            }
        }
    }
}
