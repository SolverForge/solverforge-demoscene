// The Amiga effects, in textmode: each is a per-cell shader whose value
// picks a character by brightness and colours it from a palette, plus the
// pixel-space copper bars. Fire is the classic heat buffer (seeded at the
// bottom, each cell the cooled average of the ones under it).
use std::f32::consts::PI;

use crate::ascii::{ramp_char, Grid, COLS, ROWS};
use crate::fx::{h2, hash};
use crate::gfx::{add_rgb, mix, ramp, scale, Frame, H, W};

/// Plasma: the sum of four moving sines, into cell colours and characters
/// over rows r0..r1, blended in by `k`.
pub fn plasma(g: &mut Grid, t: f32, r0: i32, r1: i32, palette: &[u32], k: f32) {
    if k <= 0.0 {
        return;
    }
    for r in r0.max(0)..r1.min(ROWS) {
        let y = r as f32 * 0.16;
        let fade = ((r - r0) as f32 / (r1 - r0).max(1) as f32).min(1.0);
        for c in 0..COLS {
            let x = c as f32 * 0.06;
            let v = (x * 1.3 + t * 0.7).sin()
                + (y * 1.7 - t * 0.9).sin()
                + ((x + y) * 0.8 + t * 0.4).sin()
                + ((x * x + y * y).sqrt() * 1.1 - t * 1.2).sin();
            let u = (v * 0.125 + 0.5).clamp(0.0, 1.0);
            let col = ramp(palette, u);
            // Curtains of light: brightest in vertical ribbons.
            let ribbon = (0.5 + 0.5 * (c as f32 * 0.09 + v * 0.6 + t * 0.3).sin()).powf(3.0);
            let a = k * ribbon * (1.0 - fade) * 0.9;
            if let Some(cell) = g.cell(c, r) {
                cell.bg = mix(cell.bg, scale(col, 0.55), a);
                if a > 0.25 && (cell.ch == ' ' || cell.ch == '.') {
                    cell.ch = ['|', '\'', ':'][(h2(c, r) * 3.0) as usize % 3];
                    cell.fg = mix(cell.fg, col, a);
                }
            }
        }
    }
}

/// The heat buffer behind the fire, one value per cell.
pub struct Fire {
    heat: Vec<f32>,
    cols: i32,
    rows: i32,
    last: i64,
}

impl Fire {
    pub fn new(cols: i32, rows: i32) -> Self {
        Self {
            heat: vec![0.0; (cols * rows) as usize],
            cols,
            rows,
            last: -1,
        }
    }

    /// Advance to tick `tick` (30 ticks a second), seeding the bottom with
    /// intensity `fuel`. Deterministic: the state at a tick depends only on
    /// the ticks before it, replayed from zero if time runs backwards.
    pub fn advance(&mut self, tick: i64, fuel: f32) {
        if tick < self.last {
            self.heat.fill(0.0);
            self.last = -1;
        }
        while self.last < tick {
            self.last += 1;
            self.step(self.last, fuel);
        }
    }

    fn step(&mut self, n: i64, fuel: f32) {
        let (w, h) = (self.cols, self.rows);
        for c in 0..w {
            let centre = 1.0 - ((c as f32 / (w - 1) as f32) - 0.5).abs() * 2.0;
            let flick = (hash((c as u32).wrapping_mul(7919) ^ (n as u32).wrapping_mul(104_729))
                & 255) as f32
                / 255.0;
            self.heat[((h - 1) * w + c) as usize] =
                (fuel * centre.powf(0.7) * (0.55 + 0.45 * flick)).min(1.0);
        }
        for r in 0..h - 1 {
            for c in 0..w {
                let below = |dc: i32, dr: i32| {
                    let cc = (c + dc).clamp(0, w - 1);
                    let rr = (r + dr).min(h - 1);
                    self.heat[(rr * w + cc) as usize]
                };
                let drift =
                    ((hash((c as u32) << 8 ^ (r as u32) ^ (n as u32) << 20) & 3) as i32) - 1;
                let avg =
                    (below(drift, 1) + below(drift - 1, 1) + below(drift + 1, 1) + below(drift, 2))
                        * 0.25;
                let cool = 0.045 + 0.03 * h2(c, r + n as i32 % 97);
                self.heat[(r * w + c) as usize] = (avg - cool * 0.6).max(0.0);
            }
        }
    }

    /// Draw with the buffer's bottom-centre at cell (cx, base).
    pub fn draw(&self, g: &mut Grid, cx: i32, base: i32, k: f32) {
        const HOT: [u32; 6] = [0x000000, 0x5a0a02, 0xb32a05, 0xff7a10, 0xffc84a, 0xfff6d8];
        for r in 0..self.rows {
            for c in 0..self.cols {
                let v = self.heat[(r * self.cols + c) as usize] * k;
                if v < 0.08 {
                    continue;
                }
                let gc = cx - self.cols / 2 + c;
                let gr = base - self.rows + 1 + r;
                let col = ramp(&HOT, v);
                let ch = ramp_char(0.2 + v * 0.8);
                if let Some(cell) = g.cell(gc, gr) {
                    cell.ch = ch;
                    cell.fg = col;
                    cell.bg = mix(cell.bg, scale(col, 0.5), v.min(1.0));
                }
            }
        }
    }
}

/// Copper bars in pixel space: `n` horizontal bars of light swinging on
/// sines through rows y0..y1 (the floodlights over the falls).
pub fn copper(f: &mut Frame, t: f32, y0: f32, y1: f32, colours: &[u32], n: usize, k: f32) {
    if k <= 0.0 {
        return;
    }
    let mid = (y0 + y1) * 0.5;
    let amp = (y1 - y0) * 0.5 - 20.0;
    for i in 0..n {
        let phase = i as f32 * 0.55;
        let cy = mid + amp * (t * 0.9 + phase * 2.0).sin() * (0.6 + 0.4 * (t * 0.31 + phase).cos());
        let c = colours[i % colours.len()];
        let half = 18;
        for dy in -half..=half {
            let v = 1.0 - (dy as f32 / half as f32).abs();
            let rgb = ramp(&[0x000000, c, c, 0xffffff], v);
            let y = cy as i32 + dy;
            if y < 0 || y >= H as i32 {
                continue;
            }
            let kk = k * v.powf(0.7) * 0.35;
            for p in &mut f.px[y as usize * W..(y as usize + 1) * W] {
                *p = add_rgb(*p, rgb, kk);
            }
        }
    }
}

/// Searchlight beams in pixel space: additive cones from (x, y) swinging
/// along `dir`; the cone is `width` wide at its far end, `len` long.
pub fn beam(
    f: &mut Frame,
    (x, y): (f32, f32),
    dir: f32,
    (width, len): (f32, f32),
    rgb: u32,
    k: f32,
) {
    if k <= 0.0 {
        return;
    }
    let (s, c) = dir.sin_cos();
    let y0 = (y - len).max(0.0) as i32;
    let y1 = (y + len).min(H as f32) as i32;
    for py in y0..y1 {
        for px in ((x - len).max(0.0) as i32..(x + len).min(W as f32) as i32).step_by(1) {
            let (dx, dy) = (px as f32 - x, py as f32 - y);
            let along = dx * c + dy * s;
            if along <= 0.0 || along > len {
                continue;
            }
            let across = (-dx * s + dy * c).abs();
            let wdt = width * along / len + 2.0;
            if across > wdt {
                continue;
            }
            let a = (1.0 - across / wdt) * (1.0 - along / len) * k;
            f.add(px, py, rgb, a * 0.5);
        }
    }
}

/// A meteor: a short bright streak with a tail, travelling across `life`.
pub fn meteor(
    g: &mut Grid,
    t: f32,
    start: f32,
    life: f32,
    (c0, r0): (f32, f32),
    (dc, dr): (f32, f32),
) {
    let a = (t - start) / life;
    if !(0.0..1.0).contains(&a) {
        return;
    }
    for k in 0..12 {
        let back = k as f32 * 0.8;
        let (c, r) = (
            c0 + dc * a * life - dc.signum() * back,
            r0 + dr * a * life - dr.signum() * back * 0.4,
        );
        let fade = (1.0 - k as f32 / 12.0) * (PI * a).sin();
        g.light(
            c as i32,
            r as i32,
            if k == 0 { '*' } else { '-' },
            scale(0xfff4d6, fade),
        );
    }
}

/// The sky's gradient in pixels down to `to_y`, then its last colour.
pub fn sky_gradient(f: &mut Frame, stops: &[u32], to_y: f32) {
    for y in 0..H {
        let c = ramp(stops, (y as f32 / to_y.max(1.0)).min(1.0));
        f.px[y * W..(y + 1) * W].fill(c);
    }
}

/// The moon: an anti-aliased disc with limb darkening and its seas, a soft
/// halo around it.
pub fn moon(f: &mut Frame, cx: f32, cy: f32, r: f32, k: f32) {
    if k <= 0.0 {
        return;
    }
    f.glow(cx, cy, r * 5.0, 0x5a7488, 0.35 * k);
    f.glow(cx, cy, r * 2.0, 0x9fb4c8, 0.3 * k);
    let (x0, x1) = ((cx - r - 2.0) as i32, (cx + r + 2.0) as i32);
    let (y0, y1) = ((cy - r - 2.0) as i32, (cy + r + 2.0) as i32);
    for y in y0.max(0)..y1.min(H as i32) {
        for x in x0.max(0)..x1.min(W as i32) {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let d = (dx * dx + dy * dy).sqrt();
            let cover = (r - d + 0.5).clamp(0.0, 1.0);
            if cover <= 0.0 {
                continue;
            }
            let limb = (1.0 - (d / r).powi(2)).max(0.0).sqrt();
            let sea = crate::fx::fbm(dx / r * 2.2 + 3.0, dy / r * 2.2 + 1.0, 4);
            let l = (0.72 + 0.28 * limb) * (1.0 - 0.32 * ((sea - 0.48) * 4.0).clamp(0.0, 1.0));
            let c = mix(0x8a8f86, 0xfbf6e4, l);
            f.blend(x, y, c, cover * k);
        }
    }
}
