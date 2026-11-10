// The demoscene layer under and over the data: copper bars, the textmode
// river (the Serio, from hammer_05), forge sparks and steam, a warp
// starfield, a scanline glitch. Every "random" value is a hash of its index
// and seed, so `--render` is bit-identical run to run.
use std::f32::consts::{PI, TAU};

use crate::gfx::{add_rgb, ramp, scale, Frame, H, S1, W};

pub const HOT: [u32; 5] = [0x2a0600, 0xa32000, 0xff7a00, 0xffd88a, 0xffffff];

pub fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// 0..1 from two integers.
pub fn h2(a: i32, b: i32) -> f32 {
    let v = hash((a as u32).wrapping_mul(0x9E37_79B1) ^ hash((b as u32).wrapping_add(0x85EB_CA6B)));
    (v >> 8) as f32 / (1u32 << 24) as f32
}

pub fn h1(a: i32) -> f32 {
    h2(a, 0x2C1B)
}

/// Add a colour to a run of one pixel row (integer, saturating).
fn add_row(f: &mut Frame, y: i32, x0: i32, x1: i32, rgb: u32, k: f32) {
    if y < 0 || y >= H as i32 || k <= 0.0 {
        return;
    }
    let add = scale(rgb, k.min(1.0));
    let (ar, ag, ab) = ((add >> 16) & 255, (add >> 8) & 255, add & 255);
    let (x0, x1) = (x0.max(0) as usize, x1.min(W as i32).max(0) as usize);
    for p in &mut f.px[y as usize * W + x0..y as usize * W + x1] {
        let r = (((*p >> 16) & 255) + ar).min(255);
        let g = (((*p >> 8) & 255) + ag).min(255);
        let b = ((*p & 255) + ab).min(255);
        *p = (r << 16) | (g << 8) | b;
    }
}

/// Amiga copper bars: `n` bars swinging on sines through rows y0..y1, each
/// a ramp from dark through its colour to a white core.
pub fn copper(f: &mut Frame, t: f32, y0: f32, y1: f32, colours: &[u32], n: usize, k: f32) {
    if k <= 0.0 {
        return;
    }
    let mid = (y0 + y1) * 0.5;
    let amp = (y1 - y0) * 0.5 - 30.0;
    for i in 0..n {
        let phase = i as f32 * 0.42;
        let cy = mid + amp * (t * 1.7 + phase * 2.0).sin() * (0.6 + 0.4 * (t * 0.37 + phase).cos());
        let c = colours[i % colours.len()];
        let half = 22;
        for dy in -half..=half {
            let v = 1.0 - (dy as f32 / half as f32).abs();
            let rgb = ramp(&[0x000000, c, c, 0xffffff], v);
            add_row(f, cy as i32 + dy, 0, W as i32, rgb, k * v.powf(0.6) * 0.55);
        }
    }
}

/// The textmode river: rows of flowing '~' cells across a band, lit from
/// the scene's palette. `flow` is the speed.
pub fn river(f: &mut Frame, t: f32, y: i32, rows: i32, palette: &[u32], k: f32) {
    if k <= 0.0 {
        return;
    }
    let cols = W as i32 / S1.w;
    for r in 0..rows {
        let depth = r as f32 / rows.max(1) as f32;
        let speed = 9.0 + 7.0 * depth;
        let shift = (t * speed) as i32;
        for c in 0..cols {
            let u = c + shift;
            let wave = ((u as f32 * 0.21 + t * 2.3 + r as f32).sin() * 0.5 + 0.5) * 0.6
                + h2(u / 3, r) * 0.4;
            if wave < 0.25 {
                continue;
            }
            let ch = ['.', '-', '~', '~', '='][((wave * 5.0) as usize).min(4)];
            let rgb = ramp(palette, wave * (0.55 + 0.45 * depth));
            f.char_at(
                c * S1.w,
                y + r * S1.h,
                ch,
                S1,
                rgb,
                k * (0.35 + 0.65 * wave),
            );
        }
    }
}

/// Sparks thrown from (x, y) `age` seconds ago: ballistic, cooling from
/// white to red, as characters.
pub fn sparks(f: &mut Frame, x: f32, y: f32, age: f32, seed: i32, count: usize, spread: f32) {
    if !(0.0..1.4).contains(&age) {
        return;
    }
    for i in 0..count as i32 {
        let s = seed * 977 + i;
        let a = PI * (0.05 + 0.9 * h1(s)) + (spread - 1.0) * (h1(s + 7) - 0.5);
        let speed = 280.0 + 820.0 * h1(s + 1);
        let life = 0.35 + 0.9 * h1(s + 2);
        if age > life {
            continue;
        }
        let px = x + a.cos() * speed * age;
        let py = y - a.sin() * speed * age * 0.8 + 900.0 * age * age;
        let heat = 1.0 - age / life;
        let ch = if heat > 0.66 {
            '*'
        } else if heat > 0.33 {
            '+'
        } else {
            '.'
        };
        f.char_at(
            px as i32,
            py as i32,
            ch,
            S1,
            ramp(&HOT, 0.3 + 0.7 * heat),
            1.0,
        );
    }
}

/// Steam rising from a band (the quench).
pub fn steam(f: &mut Frame, x0: f32, x1: f32, y: f32, t: f32, k: f32, seed: i32) {
    if k <= 0.01 {
        return;
    }
    for i in 0..90 {
        let s = seed * 131 + i;
        let period = 1.4 + h1(s) * 2.0;
        let age = (t + h1(s + 1) * period).rem_euclid(period);
        let px = x0 + h1(s + 2) * (x1 - x0) + (age * 1.7 + i as f32).sin() * 14.0;
        let py = y - age * 120.0;
        let a = (PI * age / period).sin() * k;
        if a > 0.05 {
            let ch = ['~', '.', '\'', ':', '~'][(h1(s + 3) * 5.0) as usize % 5];
            f.char_at(px as i32, py as i32, ch, S1, 0xd8e2ea, a * 0.7);
        }
    }
}

/// A warp starfield towards the centre: stars are characters that grow
/// brighter and longer as they pass the camera.
pub fn starfield(f: &mut Frame, t: f32, speed: f32, k: f32, tint: u32) {
    if k <= 0.0 {
        return;
    }
    let (cx, cy) = (W as f32 * 0.5, H as f32 * 0.5);
    for i in 0..420 {
        let ang = h1(i) * TAU;
        let z = (h1(i + 999) - t * speed * (0.6 + 0.4 * h1(i + 3))).rem_euclid(1.0);
        let z = z.max(0.02);
        let r = 40.0 / z;
        if r < 160.0 {
            continue;
        }
        let (x, y) = (cx + ang.cos() * r, cy + ang.sin() * r * 0.75);
        if x < 0.0 || y < 0.0 || x >= W as f32 || y >= H as f32 {
            continue;
        }
        let b = ((1.0 - z) * 1.2).clamp(0.0, 1.0);
        let ch = if b > 0.8 {
            '*'
        } else if b > 0.5 {
            '+'
        } else {
            '.'
        };
        f.char_at(
            x as i32,
            y as i32,
            ch,
            S1,
            add_rgb(tint, 0xffffff, b * 0.5),
            k * b,
        );
    }
}

/// Scanline tear: rows of a band slide sideways, red split off.
pub fn glitch(f: &mut Frame, t: f32, amount: f32, seed: i32) {
    if amount <= 0.01 {
        return;
    }
    let tq = (t * 30.0) as i32;
    let mut row = vec![0u32; W];
    let mut y = 0usize;
    while y < H {
        let band = 4 + (h2(y as i32, tq + seed) * 40.0) as usize;
        let on = h2(y as i32 + 7, tq * 3 + seed) < amount * 0.6;
        let shift = if on {
            ((h2(y as i32 + 3, tq + seed) - 0.5) * 160.0 * amount) as i32
        } else {
            0
        };
        for yy in y..(y + band).min(H) {
            if shift == 0 {
                continue;
            }
            row.copy_from_slice(&f.px[yy * W..(yy + 1) * W]);
            for x in 0..W {
                let sx = (x as i32 - shift).clamp(0, W as i32 - 1) as usize;
                let rx =
                    (x as i32 - shift - (12.0 * amount) as i32).clamp(0, W as i32 - 1) as usize;
                f.px[yy * W + x] = (row[rx] & 0xff0000) | (row[sx] & 0x00ffff);
            }
        }
        y += band;
    }
}

/// A full-frame flash towards a colour.
pub fn flash(f: &mut Frame, rgb: u32, k: f32) {
    if k <= 0.003 {
        return;
    }
    for y in 0..H as i32 {
        add_row(f, y, 0, W as i32, rgb, k);
    }
}

/// Section wipe: a column of glyph rain that sweeps the frame left to right
/// at progress p (0..1), cells flickering in the colour.
pub fn glyph_wipe(f: &mut Frame, t: f32, p: f32, rgb: u32) {
    if !(0.0..1.0).contains(&p) {
        return;
    }
    let cols = W as i32 / S1.w;
    let rows = H as i32 / S1.h;
    let edge = p * (cols as f32 + 40.0) - 20.0;
    let tq = (t * 24.0) as i32;
    for c in 0..cols {
        let d = c as f32 - edge;
        if !(-22.0..6.0).contains(&d) {
            continue;
        }
        let k = if d > 0.0 {
            1.0 - d / 6.0
        } else {
            1.0 + d / 22.0
        };
        for r in 0..rows {
            let v = h2(c * 31 + r, tq + c);
            if v > 0.55 * k + 0.1 {
                continue;
            }
            let ch = (b'!' + (h2(c, r + tq) * 90.0) as u8) as char;
            let x = c * S1.w;
            let y = r * S1.h;
            f.rect(x, y, x + S1.w, y + S1.h, 0x000000, 0.85 * k);
            f.char_at(x, y, ch, S1, add_rgb(rgb, 0xffffff, (k - 0.7).max(0.0)), k);
        }
    }
}

/// A box-drawing frame in textmode cells around a pixel rectangle.
pub fn box_frame(f: &mut Frame, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
    let (cw, ch) = (S1.w, S1.h);
    let cols = (x1 - x0) / cw;
    let rows = (y1 - y0) / ch;
    for c in 1..cols - 1 {
        f.char_at(x0 + c * cw, y0, '─', S1, rgb, a);
        f.char_at(x0 + c * cw, y0 + (rows - 1) * ch, '─', S1, rgb, a);
    }
    for r in 1..rows - 1 {
        f.char_at(x0, y0 + r * ch, '│', S1, rgb, a);
        f.char_at(x0 + (cols - 1) * cw, y0 + r * ch, '│', S1, rgb, a);
    }
    f.char_at(x0, y0, '┌', S1, rgb, a);
    f.char_at(x0 + (cols - 1) * cw, y0, '┐', S1, rgb, a);
    f.char_at(x0, y0 + (rows - 1) * ch, '└', S1, rgb, a);
    f.char_at(x0 + (cols - 1) * cw, y0 + (rows - 1) * ch, '┘', S1, rgb, a);
}

/// A dark panel with a box-drawn border; the outer size snaps to cells.
pub fn panel(f: &mut Frame, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
    if a <= 0.0 {
        return;
    }
    let (w, h) = ((x1 - x0) / S1.w * S1.w, (y1 - y0) / S1.h * S1.h);
    f.dim(x0, y0, x0 + w, y0 + h, 1.0 - 0.82 * a);
    box_frame(f, x0, y0, x0 + w, y0 + h, rgb, a);
}

/// A soft hot glow (the forge behind the code).
pub fn heat_glow(f: &mut Frame, cx: f32, cy: f32, r: f32, heat: f32) {
    f.glow(cx, cy, r, ramp(&HOT, 0.35 + 0.5 * heat), 0.25 * heat);
}

/// Section reveal: a stack of copper bars rolls down the frame at progress
/// p (0..1); below it the new section is still held dark.
pub fn copper_wipe(f: &mut Frame, p: f32, colours: &[u32]) {
    if !(0.0..1.0).contains(&p) {
        return;
    }
    let band = 260.0;
    let lead = -band + p * (H as f32 + band * 2.0);
    let below = (lead + band * 0.5) as i32;
    if below < H as i32 {
        f.dim(0, below.max(0), W as i32, H as i32, 0.12);
    }
    for i in 0..5 {
        let cy = lead + i as f32 * band / 5.0;
        let c = colours[i % colours.len()];
        let half = 18;
        for dy in -half..=half {
            let v = 1.0 - (dy as f32 / half as f32).abs();
            let rgb = ramp(&[0x000000, c, c, 0xffffff], v);
            add_row(
                f,
                cy as i32 + dy,
                0,
                W as i32,
                rgb,
                0.9 * v.powf(0.6) * (1.0 - p * 0.5),
            );
        }
    }
}

/// A thin raster line sweeping down the frame with a soft trail.
pub fn raster_sweep(f: &mut Frame, p: f32, rgb: u32) {
    if !(0.0..1.0).contains(&p) {
        return;
    }
    let y = (p * H as f32) as i32;
    for dy in 0..60 {
        let k = (1.0 - dy as f32 / 60.0).powi(2) * (1.0 - p * 0.6);
        add_row(f, y - dy, 0, W as i32, rgb, 0.5 * k);
    }
    add_row(f, y, 0, W as i32, 0xffffff, 0.8 * (1.0 - p));
}
