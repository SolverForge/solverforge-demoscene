// LAKE: Lake Barbellino, out of the mine and high above the falls. The
// aurora moves over the peaks (a plasma), the lake holds the sky upside
// down with a ripple, a meteor crosses it. A ring of standing stones stands
// on the meadow; their runes wake one after another in falling code
// (glyphfx matrix) and, once all are lit, light runs between them and
// closes the ring. They walk past in front.
use super::party::{self, Light};
use super::stage::{cell_px, Stage};
use crate::amiga;
use crate::ascii::COLS;
use crate::fx::{ease_in_out, window};
use crate::gfx::{mix, scale};
use crate::glyphs;
use crate::timeline::{span, Scene};
use crate::world::{self, Range};

const FEET: i32 = 58;
const SHORE: i32 = 34;
/// Stones: (col, base row, height), left to right around the ring.
const STONES: [(i32, i32, i32); 6] = [
    (18, 54, 16),
    (46, 50, 13),
    (78, 48, 11),
    (110, 48, 11),
    (142, 50, 13),
    (170, 54, 16),
];
/// Runes wake from RUNES_AT, one stone every RUNES_GAP seconds.
pub const RUNES_AT: f32 = 1.5;
pub const RUNES_GAP: f32 = 0.6;

pub fn draw(st: &mut Stage, t: f32, local: f32) {
    st.sky(&[0x01030a, 0x031018, 0x0a1d26], SHORE);
    world::stars(st.g, t, 0.0, SHORE, 0.05, 0.9);
    amiga::plasma(
        st.g,
        t,
        0,
        24,
        &[0x0a3a2a, 0x22c07a, 0x6affc8, 0x7a5aff],
        window(local, 0.0, 3.0) * 0.8,
    );
    world::ridge(
        st.g,
        0.0,
        SHORE,
        18.0,
        0.032,
        29.0,
        Range {
            rock: 0x0b1214,
            lit: 0x5a7888,
            snow: Some(0xd8e4ee),
            haze: 0x0a1d26,
        },
    );
    let start = span(Scene::Lake).0;
    amiga::meteor(st.g, t, start + 3.0, 1.2, (150.0, 3.0), (-60.0, 8.0));
    amiga::meteor(st.g, t, start + 8.5, 0.9, (40.0, 5.0), (70.0, 6.0));
    // The lake: rows below the shore mirror the rows above, rippled and
    // darkened.
    for r in SHORE..SHORE + 12 {
        let src = SHORE - 1 - (r - SHORE);
        let d = (r - SHORE) as f32;
        let wob = ((r as f32 * 0.9 + t * 2.2).sin() * (1.0 + d * 0.15)) as i32;
        let k = 0.55 - d * 0.02;
        for c in 0..COLS {
            let s = st.g.at(c + wob, src);
            let bg = st.g.behind(c + wob, src);
            let ch = if s.ch == ' ' && (c + r * 3) % 13 == 0 {
                '-'
            } else {
                s.ch
            };
            st.g.set(
                c,
                r,
                ch,
                scale(mix(s.fg, 0x2a5a6a, 0.3), k),
                scale(mix(bg, 0x061820, 0.4), k),
            );
        }
    }
    world::ground(st.g, 0.0, SHORE + 12, 0x0a1510, 0x2a4a2e);

    // The stones and their runes.
    for (i, &(c, base, h)) in STONES.iter().enumerate() {
        for r in base - h..base {
            for dc in 0..6 {
                let (ch, fg) = match dc {
                    0 => ('/', 0x8a9692),
                    5 => ('|', 0x3a4442),
                    _ => (' ', 0),
                };
                st.g.set(c + dc, r, ch, fg, mix(0x2c3432, 0x161c1a, dc as f32 / 5.0));
            }
        }
        for dc in 1..5 {
            st.g.put(c + dc, base - h - 1, '_', 0x8a9692);
        }
        let since = local - RUNES_AT - i as f32 * RUNES_GAP;
        glyphs::play(
            st.g,
            &st.glyphs.runes[i],
            since,
            1.4,
            c + 2,
            base - h + 3,
            1.0,
        );
        if since > 0.0 {
            let (x, y) = cell_px(c as f32 + 2.0, (base - h / 2) as f32);
            st.glow(x, y, 90.0, 0x3dbb8a, 0.35 * window(since, 0.0, 1.0));
        }
    }
    // Once all are lit, light runs from stone to stone and closes the ring.
    let all = RUNES_AT + STONES.len() as f32 * RUNES_GAP + 0.6;
    let run = (local - all) / 2.4;
    if run > 0.0 {
        let n = STONES.len();
        for i in 0..n {
            let (a, b) = (STONES[i], STONES[(i + 1) % n]);
            let k = ease_in_out(run * n as f32 - i as f32);
            if k <= 0.0 {
                continue;
            }
            let pa = cell_px(a.0 as f32 + 3.0, (a.1 - a.2 + 6) as f32);
            let pb = cell_px(b.0 as f32 + 3.0, (b.1 - b.2 + 6) as f32);
            let end = (pa.0 + (pb.0 - pa.0) * k, pa.1 + (pb.1 - pa.1) * k);
            st.line(pa, end, 2.0, 0x9fffd0, 0.75);
            st.glow(end.0, end.1, 40.0, 0xd9fff0, 0.6 * (1.0 - k) + 0.2);
        }
    }
    // They walk across in front of the ring, left to right.
    let x = -20.0 + (COLS as f32 + 40.0) * (local / 13.3);
    let light = Light {
        tint: 0xb0e4d8,
        k: 0.35,
        rim: Some((0x8affd8, -1, 0.5)),
    };
    party::cordelia(st, t, x, FEET, true, light, 1.0);
    party::victor(st, t, x - 34.0, FEET, true, light, 1.0);
}
