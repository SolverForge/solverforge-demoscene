// FALLS: the Cascate del Serio. Five nights a year the Barbellino dam is
// opened and the river falls 315 metres in three drops, under floodlights.
// They stop on the rocks and watch. The gates open, the water pours down
// each drop in turn (glyphfx pour), floodlight beams climb the cliff,
// copper bars sweep it, spray rises from the pool and a moonbow stands
// in it.
use std::f32::consts::PI;

use super::party::{self, Light};
use super::stage::{cell_px, Px, Stage};
use crate::ascii::{COLS, ROWS};
use crate::fx::{fbm, h2, noise, window};
use crate::gfx::{mix, scale};
use crate::glyphs;

const FEET: i32 = 51;
/// The pool's surface row, and where the outcrop ends in it.
const POOL: i32 = 50;
const BANK_END: i32 = 92;

/// The outcrop's top row at column c, or None past its end.
fn outcrop_top(c: i32) -> Option<i32> {
    if !(0..BANK_END).contains(&c) {
        return None;
    }
    // Flat where they stand (cols 30..80), a rise of boulders behind them
    // on the left, falling away into the water at the end.
    let rough = (fbm(c as f32 * 0.18, 5.0, 3) - 0.5) * 2.0;
    let rise = ((30 - c).max(0) as f32 * 0.18).min(4.0);
    let fall = (c - (BANK_END - 12)).max(0) as f32 * 0.5;
    let flat = (30..80).contains(&c);
    Some(
        FEET + 1
            + if flat {
                0
            } else {
                (rough - rise + fall).round() as i32
            },
    )
}
/// The gates open.
pub const OPEN: f32 = 3.0;
/// The three drops: (first col, top row, bottom row).
const DROPS: [(i32, i32, i32); 3] = [(134, 9, 21), (126, 23, 35), (118, 37, 49)];
const WATER_W: i32 = 14;

/// The cliff's left edge at row r (it leans back as it rises).
fn cliff_edge(r: i32) -> i32 {
    (78.0 + (50 - r).max(0) as f32 * 0.55 + 6.0 * fbm(r as f32 * 0.15, 2.0, 3)) as i32
}

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    st.sky(&[0x010206, 0x030a12, 0x0a1a20], 50);
    st.moon(36.0, 8.0, 34.0, 1.0);
    crate::world::stars(st.g, t, 0.0, 40, 0.04, 0.8);
    // The far side of the valley, and the firs along the pool.
    crate::world::ridge(
        st.g,
        0.0,
        46,
        12.0,
        0.03,
        41.0,
        crate::world::Range {
            rock: 0x070e0e,
            lit: 0x2a3e40,
            snow: None,
            haze: 0x0a1a20,
        },
    );
    crate::world::firs(
        st.g,
        0.0,
        50,
        12,
        6.0,
        81,
        crate::world::Stand {
            dark: 0x040a09,
            edge: 0x2a4a40,
            lit_k: 0.3,
        },
    );
    // The cliff: a high wall, its face hatched along the strata, its edge
    // lit by the moon.
    for r in 3..ROWS {
        let edge = cliff_edge(r);
        for c in edge..COLS {
            let n = fbm(c as f32 * 0.08, r as f32 * 0.18, 4);
            let strata = ((r as f32 * 0.9 + c as f32 * 0.25 + n * 6.0).sin() * 0.5 + 0.5) > 0.82;
            let face = mix(0x0a1010, 0x24302e, n);
            let (ch, fg) = if c == edge {
                ('/', 0x8aa0a8)
            } else if strata {
                ('\\', mix(face, 0x5a6a68, 0.6))
            } else if noise(c as f32 * 0.7, r as f32) > 0.75 {
                ('.', mix(face, 0x5a6a68, 0.4))
            } else {
                (' ', face)
            };
            st.g.set(c, r, ch, fg, face);
        }
    }
    for &(c0, _, bot) in &DROPS {
        for c in c0 - 9..c0 + WATER_W + 6 {
            st.g.set(c, bot + 1, '=', 0x5a6664, 0x182020);
            st.g.set(c, bot + 2, '_', 0x343e3c, 0x0e1312);
        }
    }
    // The dam at the top: a concrete wall with its gates.
    let open = window(local, OPEN, OPEN + 1.5);
    for r in 3..9 {
        for c in 118..164 {
            let gate = (132..154).contains(&c) && (c - 132) % 6 < 4 && r > 4;
            let ch = if r == 3 {
                '_'
            } else if gate && open < 0.5 {
                '#'
            } else if c % 8 == 0 {
                '|'
            } else {
                ' '
            };
            let bg = if gate && open >= 0.5 {
                0x9fd8f0
            } else {
                0x2a302f
            };
            st.g.set(c, r, ch, 0x7a807e, bg);
        }
    }

    // The water: glyphfx pours it down each drop in turn, then it keeps
    // falling as animated strokes.
    let speed = 0.9;
    let pour = st.glyphs.falls.seconds() / speed;
    for (i, &(c0, top, bot)) in DROPS.iter().enumerate() {
        let start = OPEN + 0.6 + i as f32 * pour * 0.55;
        let since = local - start;
        if since < 0.0 {
            continue;
        }
        if since < pour {
            glyphs::play(st.g, &st.glyphs.falls, since, speed, c0, top + 1, 1.0);
            glyphs::play(
                st.g,
                &st.glyphs.falls,
                since - 0.2,
                speed,
                c0 + 7,
                top + 1,
                1.0,
            );
        } else {
            let k = window(since, pour, pour + 0.5);
            for r in top..=bot {
                // The sheet sways and frays at its edges.
                let sway = 1.2 * (r as f32 * 0.4 + t * 2.3).sin();
                let (l, rr) = (
                    c0 as f32 + 1.0 + sway * 0.5,
                    (c0 + WATER_W) as f32 - 1.0 + sway,
                );
                for c in c0 - 2..c0 + WATER_W + 2 {
                    let inside = (c as f32 - l) / (rr - l);
                    if !(0.0..=1.0).contains(&inside) && h2(c, r + (t * 10.0) as i32) > 0.25 {
                        continue;
                    }
                    let core = 1.0 - (inside - 0.5).abs() * 2.0;
                    let fall = t * 22.0 + h2(c, 1) * 30.0;
                    let v = ((r as f32 - fall) * 0.35 + h2(c, 3) * 6.0).sin() * 0.5 + 0.5;
                    let ch = ['\'', ':', '!', '|', '|'][(v * 4.99) as usize];
                    let fg = mix(0x5a90b0, 0xf2faff, (v * 0.6 + core.max(0.0) * 0.5).min(1.0));
                    let cell = st.g.at(c, r);
                    st.g.set(
                        c,
                        r,
                        ch,
                        mix(cell.fg, fg, k),
                        mix(cell.bg, 0x1c3a4a, k * 0.45 * core.max(0.0)),
                    );
                }
            }
        }
    }
    // The pool and its spray.
    let roar = window(local, OPEN + 4.0, OPEN + 7.0);
    crate::world::river(
        st.g,
        t,
        0.0,
        POOL,
        ROWS - POOL,
        &[0x05141c, 0x1a4a5a, 0x7fc0d8, 0xffffff],
        14.0 * roar + 1.0,
    );
    for k in 0..(160.0 * roar) as i32 {
        let life = 1.6;
        let ph = ((t + h2(k, 1) * life) / life).fract();
        let c = 112.0 + 40.0 * h2(k, 2) + (h2(k, 4) - 0.5) * 30.0 * ph;
        let r = 50.0 - 14.0 * ph * h2(k, 3) - 4.0 * (PI * ph).sin();
        let fade = (1.0 - ph) * roar;
        let ch = ['.', '\'', '`', '*'][(h2(k, 5) * 4.0) as usize];
        st.g.light(c as i32, r as i32, ch, scale(0xdff4ff, fade));
    }
    crate::world::fog(st.g, t, local * 3.0, 38, 56, 0x5a7a88, 0.9 * roar);

    // Floodlights: beams up the cliff from its foot, then copper bars.
    let lights = window(local, OPEN - 1.0, OPEN);
    for (i, (x, dir, rgb)) in [
        (1080.0, -1.3, 0xdfefff),
        (1700.0, -1.9, 0xdfefff),
        (1380.0, -1.6, 0x9ff0d0),
    ]
    .iter()
    .enumerate()
    {
        st.px.push(Px::Beam {
            x: *x,
            y: 1020.0,
            dir: dir + 0.12 * (t * 0.6 + i as f32 * 2.0).sin(),
            width: 110.0,
            len: 980.0,
            rgb: *rgb,
            k: 0.42 * lights,
        });
    }
    st.px.push(Px::Copper {
        y0: 120.0,
        y1: 900.0,
        k: 0.6 * window(local, OPEN + 6.0, OPEN + 9.0) * (1.0 - window(local, 20.0, 22.5)),
    });
    // The moonbow over the pool: a faint arc of the spectrum.
    let bow = window(local, 12.0, 15.0);
    if bow > 0.0 {
        let (cx, cy) = cell_px(126.0, 55.0);
        for (j, rgb) in [0xff6060, 0xffc060, 0xfff080, 0x80ff90, 0x80c0ff, 0xb080ff]
            .iter()
            .enumerate()
        {
            let rad = 330.0 - j as f32 * 9.0;
            for s in 0..64 {
                let a = PI + PI * s as f32 / 63.0;
                st.glow(
                    cx + rad * a.cos(),
                    cy + rad * 0.85 * a.sin(),
                    12.0,
                    *rgb,
                    0.09 * bow,
                );
            }
        }
    }
    // The near bank: a granite outcrop on the left, boulders piled on
    // boulders, mossy on top and lit by the moon on its left faces; it
    // slopes down into the pool, and a few stones break the water.
    for c in 0..COLS {
        let Some(top) = outcrop_top(c) else { continue };
        // The slope of the top contour: faces turned up-left catch the moon.
        let slope = (outcrop_top(c + 2).unwrap_or(ROWS) - outcrop_top(c - 2).unwrap_or(top)) as f32;
        for r in top..ROWS {
            let depth = (r - top) as f32;
            let n = fbm(c as f32 * 0.09, r as f32 * 0.2, 4);
            let crack = fbm(c as f32 * 0.35 + 3.0, r as f32 * 0.12, 2);
            let lit = ((1.0 - depth * 0.12).max(0.0) * (0.6 + 0.15 * slope) + (n - 0.45) * 0.8)
                .clamp(0.0, 1.0);
            let rock = mix(0x0b100f, 0x46524f, lit);
            let (ch, fg) = if depth == 0.0 {
                (
                    if h2(c, 11) < 0.5 { '"' } else { ',' },
                    mix(0x4a8a50, 0x9ac880, h2(c, 12)),
                )
            } else if depth == 1.0 && h2(c, 13) < 0.45 {
                (if h2(c, 14) < 0.5 { '\'' } else { '`' }, 0x4a7a50)
            } else if (crack - 0.5).abs() < 0.025 {
                (if slope >= 0.0 { '\\' } else { '/' }, 0x050807)
            } else if n > 0.66 {
                ('.', mix(rock, 0xb8c4c0, 0.35 + lit * 0.3))
            } else {
                (' ', rock)
            };
            let bg = if depth == 0.0 {
                mix(rock, 0x26402a, 0.6)
            } else {
                rock
            };
            st.g.set(c, r, ch, fg, bg);
        }
        // Where the rock meets the pool: a lip of foam.
        if (BANK_END - 8..BANK_END).contains(&c) {
            st.g.light(
                c + 1,
                top,
                '~',
                scale(0xdff4ff, 0.4 + 0.5 * (t * 3.0 + c as f32).sin().abs()),
            );
        }
    }
    for &(c0, r0, w) in &[(102, 55, 7), (150, 57, 9), (176, 53, 5)] {
        for dc in 0..w {
            let k = (dc as f32 + 0.5) / w as f32;
            let hgt = ((PI * k).sin() * 2.2) as i32;
            for r in r0 - hgt..=r0 {
                let lit = 1.0 - k;
                st.g.set(
                    c0 + dc,
                    r,
                    if r == r0 - hgt { '_' } else { ' ' },
                    0x8a9a98,
                    mix(0x0c1212, 0x2e3a3a, lit),
                );
            }
            let foam = (t * 2.5 + dc as f32 * 0.7).sin() * 0.5 + 0.5;
            st.g.light(
                c0 + dc,
                r0 + 1,
                if foam > 0.5 { '~' } else { '-' },
                scale(0xdff4ff, 0.35 + 0.5 * foam),
            );
        }
    }
    let light = Light {
        tint: 0xc8dcea,
        k: 0.35,
        rim: Some((0xeaf6ff, 1, 0.65 * lights)),
    };
    party::cordelia(st, t, 44.0, FEET, false, light, 1.0);
    party::victor(st, t, 68.0, FEET, false, light, 1.0);
}
