// The call: Cordelia, in three stills from the art direction, on three
// beats. She hears it first: the still resolves out of glyphs from her face
// outwards while the ring pulses in amber from the left, where she looks,
// and the news types in beside her. She decides: an interlaced wipe to the
// second still and the question the film is named for. She raises her hand:
// the third still lands on the downbeat with a flash and a shudder, and
// "REPLAN." The last half second dissolves into the replan take.
use demofx::clip::Clip;
use demofx::font::Mask;
use demofx::Place;

use super::{clip, draw_clip, mosaic, AMBER, LIME};
use crate::cordelia::Cordelia;
use crate::gfx::{lerp_px, Frame, H, W};
use crate::timeline::{hash, smooth, Now, BEAT};

/// When each still takes over, in seconds into the call: on the beat.
const SECOND: f32 = 5.0 * BEAT;
const THIRD: f32 = 8.0 * BEAT;
const FACE: (f32, f32) = (1330.0, 380.0);
const RAMP: [char; 9] = ['.', ':', '-', '=', '+', '*', '#', '%', '@'];

pub struct Words {
    name: Clip,
    news: Clip,
    question: Clip,
    replan: Clip,
}

const AMBER_STOPS: &[&str] = &["--final-gradient-stops", "f59e0b", "fde68a"];
const LIME_STOPS: &[&str] = &["--final-gradient-stops", "50fc00", "e8ffd6"];
const PAPER_STOPS: &[&str] = &["--final-gradient-stops", "a7f3d0", "ecfdf5"];

impl Words {
    pub fn new() -> Self {
        Self {
            name: clip("decrypt", "CAS SOUTH", 11, 31, AMBER_STOPS),
            news: clip("decrypt", "SICK ON THURSDAY", 18, 32, PAPER_STOPS),
            question: clip("decrypt", "WHO COVERS THURSDAY?", 22, 33, PAPER_STOPS),
            replan: clip("slice", "REPLAN.", 9, 34, LIME_STOPS),
        }
    }
}

pub fn draw(
    f: &mut Frame,
    cordelia: &Cordelia,
    words: &Words,
    masks: &mut Vec<(char, Mask)>,
    n: Now,
    t: f32,
) -> (f32, f32) {
    let l = n.local;
    let [hears, resolve, hand] = [
        &cordelia.stills[0],
        &cordelia.stills[1],
        &cordelia.stills[2],
    ];
    if l < SECOND {
        glyph_resolve(f, hears, smooth(l / 1.1), t);
        // The phone: three rings in amber from the left, on the beats.
        for k in 0..3 {
            let age = l - k as f32 * BEAT;
            if age > 0.0 {
                arc(
                    f,
                    (-180.0, 420.0),
                    260.0 + 1500.0 * age,
                    AMBER,
                    0.55 * (1.0 - age / 1.2).max(0.0),
                );
            }
        }
        let flash = (-l * 9.0).exp();
        f.rect(0, 0, W as i32, H as i32, AMBER, 0.22 * flash);
        words_at(
            f,
            masks,
            &words.name,
            Place::new(110, 400, 640, 56, 100),
            l - 0.45,
            1.0,
        );
        words_at(
            f,
            masks,
            &words.news,
            Place::new(110, 530, 640, 32, 58),
            l - 1.0,
            1.0,
        );
    } else if l < THIRD {
        let p = smooth((l - SECOND) / 0.25);
        f.px.copy_from_slice(hears);
        interlace(f, resolve, p, l - SECOND);
        words_at(
            f,
            masks,
            &words.question,
            Place::new(80, 470, 720, 32, 58),
            l - SECOND - 0.1,
            1.0,
        );
    } else {
        let age = l - THIRD;
        let shake = (9.0 * (-age * 7.0).exp() * (hash((t * 60.0) as u32) - 0.5) * 2.0) as i32;
        blit_shifted(f, hand, shake);
        let flash = (-age * 10.0).exp();
        f.rect(0, 0, W as i32, H as i32, LIME, 0.35 * flash);
        words_at(
            f,
            masks,
            &words.replan,
            Place::new(110, 460, 560, 56, 100),
            age - 0.05,
            1.0,
        );
    }
    // Out into the replan take.
    let out = n.len - l;
    if out < 0.5 {
        mosaic(f, out / 0.5, FACE, t);
    }
    (1.0, 0.0)
}

/// glyphfx words, played from `age` seconds.
/// The words settle within 0.8 s, however long the effect runs natively.
fn words_at(
    f: &mut Frame,
    masks: &mut Vec<(char, Mask)>,
    c: &Clip,
    place: Place,
    age: f32,
    gain: f32,
) {
    if age < 0.0 {
        return;
    }
    let speed = (c.seconds() / 0.8).max(1.0);
    let idx = (age * 60.0 * speed) as usize;
    draw_clip(f, masks, c, place, idx, gain * smooth(age / 0.15));
}

/// The still, each 16x30 cell first a glyph of its own brightness and
/// colour, turning into the picture from her face outwards as `p` runs 0..1.
fn glyph_resolve(f: &mut Frame, img: &[u32], p: f32, t: f32) {
    f.clear(0x020806);
    const CW: i32 = 16;
    const CH: i32 = 30;
    let far = 1500.0;
    let tick = (t * 15.0) as u32;
    for cy in 0..(H as i32 / CH) {
        for cx in 0..(W as i32 / CW) {
            let (x0, y0) = (cx * CW, cy * CH);
            let (mx, my) = (x0 + CW / 2, y0 + CH / 2);
            let d = ((mx as f32 - FACE.0).powi(2) + (my as f32 - FACE.1).powi(2)).sqrt() / far;
            let id = (cy * 200 + cx) as u32;
            let turn = 0.08 + 0.7 * d + 0.15 * hash(id);
            if p >= turn + 0.12 {
                for y in y0..y0 + CH {
                    let i = y as usize * W + x0 as usize;
                    f.px[i..i + CW as usize].copy_from_slice(&img[i..i + CW as usize]);
                }
                continue;
            }
            let c = img[my as usize * W + mx as usize];
            let lum = ((c >> 16 & 255) * 3 + (c >> 8 & 255) * 6 + (c & 255)) as f32 / 2550.0;
            if lum < 0.06 {
                continue;
            }
            let jitter = if p < turn {
                (hash(id ^ tick) * 2.0) as usize
            } else {
                0
            };
            let g = RAMP[((lum * 8.0) as usize + jitter).min(8)];
            f.char_at(
                x0 + 3,
                y0 + 6,
                g,
                1,
                crate::gfx::mix(c, 0xffffff, 0.25),
                0.4 + 0.6 * lum,
            );
            if p >= turn {
                // Turning: the picture fades up under its glyph.
                let k = (p - turn) / 0.12;
                for y in y0..y0 + CH {
                    let i = y as usize * W + x0 as usize;
                    for x in 0..CW as usize {
                        f.px[i + x] = lerp_px(f.px[i + x], img[i + x], k);
                    }
                }
            }
        }
    }
}

/// An interlaced wipe to `img`: even rows switch first, odd rows half a beat
/// behind, with a red and cyan seam on the rows still turning.
fn interlace(f: &mut Frame, img: &[u32], p: f32, age: f32) {
    let drift = (-8.0 * age) as i32;
    for y in 0..H {
        let edge = if y % 2 == 0 { p * 2.0 } else { p * 2.0 - 1.0 };
        let reach = (edge.clamp(0.0, 1.0) * H as f32) as usize;
        if y > reach {
            continue;
        }
        let seam = reach.saturating_sub(y) < 14;
        for x in 0..W {
            let sx = (x as i32 - drift).clamp(0, W as i32 - 1) as usize;
            let c = img[y * W + sx];
            f.px[y * W + x] = if seam {
                let r = img[y * W + (sx + 9).min(W - 1)] & 0xff0000;
                r | (c & 0x00ffff)
            } else {
                c
            };
        }
    }
    if p >= 1.0 {
        // Settled: the second still drifts a few pixels left over its beats.
        for y in 0..H {
            for x in 0..W {
                let sx = (x as i32 - drift).clamp(0, W as i32 - 1) as usize;
                f.px[y * W + x] = img[y * W + sx];
            }
        }
    }
}

/// The still shifted `dx` pixels sideways (the landing's shudder).
fn blit_shifted(f: &mut Frame, img: &[u32], dx: i32) {
    for y in 0..H {
        for x in 0..W {
            let sx = (x as i32 - dx).clamp(0, W as i32 - 1) as usize;
            f.px[y * W + x] = img[y * W + sx];
        }
    }
}

/// A thin arc of a circle centred off the left edge: the phone's ring.
fn arc(f: &mut Frame, c: (f32, f32), r: f32, rgb: u32, a: f32) {
    if a <= 0.0 {
        return;
    }
    let steps = (r * 2.0) as i32;
    for i in 0..steps {
        let th = -1.2 + 2.4 * i as f32 / steps as f32;
        let (x, y) = (c.0 + r * th.cos(), c.1 + r * th.sin());
        if x < 0.0 || x > W as f32 * 0.62 {
            continue;
        }
        let fade = 1.0 - (x / (W as f32 * 0.62)).powi(2);
        for w in -2..=2 {
            f.blend(
                x as i32 + w,
                y as i32,
                rgb,
                a * fade * (1.0 - w.abs() as f32 * 0.3),
            );
        }
    }
}
