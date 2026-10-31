// Cells -> pixels: glyph raster, phosphor bloom, scanlines, vignette.
use std::collections::HashMap;

use crate::canvas::{TextCanvas, CELL_H, CELL_W, COLS, ROWS};
use crate::scenes::Look;
use demofx::font::{mask, Mask};

pub const WIDTH: usize = COLS * CELL_W;
pub const HEIGHT: usize = ROWS * CELL_H;
const DS: usize = 4;
const SW: usize = WIDTH / DS;
const SH: usize = HEIGHT / DS;

pub struct Post {
    masks: HashMap<char, Mask>,
    shade: Vec<u16>,
    small: Vec<[u32; 3]>,
    tmp: Vec<[u32; 3]>,
    line_a: Vec<[u32; 3]>,
    line_b: Vec<[u32; 3]>,
    xtab: Vec<(usize, usize, u32)>,
    pub pixels: Vec<u32>,
}

impl Post {
    pub fn new() -> Self {
        let mut shade = vec![0u16; WIDTH * HEIGHT];
        for y in 0..HEIGHT {
            let scan = if y % 3 == 2 { 0.74 } else { 1.0 };
            for x in 0..WIDTH {
                let dx = (x as f32 / WIDTH as f32 - 0.5) * 2.0;
                let dy = (y as f32 / HEIGHT as f32 - 0.5) * 2.0;
                let vig = (1.0 - (dx * dx * 0.18 + dy * dy * 0.28)).clamp(0.0, 1.0);
                shade[y * WIDTH + x] = (scan * vig * 256.0) as u16;
            }
        }
        Self {
            masks: HashMap::new(),
            shade,
            small: vec![[0; 3]; SW * SH],
            tmp: vec![[0; 3]; SW * SH],
            line_a: vec![[0; 3]; WIDTH],
            line_b: vec![[0; 3]; WIDTH],
            xtab: (0..WIDTH).map(|x| upsample_tap(x, SW)).collect(),
            pixels: vec![0; WIDTH * HEIGHT],
        }
    }

    /// Render both text layers and the logo over the look's
    /// background gradient.
    pub fn render(&mut self, canvas: &TextCanvas, big: &TextCanvas, look: &Look) {
        for y in 0..HEIGHT {
            let c = crate::canvas::lerp_rgb(look.bg_top, look.bg_bottom, y as f32 / HEIGHT as f32);
            self.pixels[y * WIDTH..(y + 1) * WIDTH].fill(c);
        }
        self.raster(canvas, 1);
        self.raster(big, 2);
        if let Some(state) = &look.logo {
            demofx::logo::draw(
                &mut demofx::RgbBuffer {
                    buf: &mut self.pixels,
                    w: WIDTH,
                    h: HEIGHT,
                },
                state,
            );
        }
        demofx::logo::corner(
            &mut demofx::RgbBuffer {
                buf: &mut self.pixels,
                w: WIDTH,
                h: HEIGHT,
            },
            look.time,
            look.corner,
        );
        self.bloom_and_shade(look.bloom);
    }

    fn raster(&mut self, canvas: &TextCanvas, scale: usize) {
        let (cw, ch) = (CELL_W * scale, CELL_H * scale);
        for row in 0..canvas.rows {
            for col in 0..canvas.cols {
                let cell = canvas.cells[row * canvas.cols + col];
                if cell.ch == ' ' || cell.rgb == 0 {
                    continue;
                }
                let m = *self.masks.entry(cell.ch).or_insert_with(|| mask(cell.ch));
                let (x0, y0) = (col * cw, row * ch);
                for y in 0..ch {
                    let bits = m[y / scale];
                    if bits == 0 {
                        continue;
                    }
                    let line = &mut self.pixels[(y0 + y) * WIDTH + x0..(y0 + y) * WIDTH + x0 + cw];
                    for (x, px) in line.iter_mut().enumerate() {
                        if (bits >> (x / scale)) & 1 == 1 {
                            *px = cell.rgb;
                        }
                    }
                }
            }
        }
    }

    /// Quarter-res blur added back with bilinear upsampling, then scanlines
    /// and vignette, all in fixed point in one pass over the frame.
    fn bloom_and_shade(&mut self, strength: f32) {
        let k = (strength.max(0.0) * 256.0) as u32;
        if k > 0 {
            for sy in 0..SH {
                for sx in 0..SW {
                    let mut acc = [0u32; 3];
                    for y in 0..DS {
                        let row = &self.pixels[(sy * DS + y) * WIDTH + sx * DS..][..DS];
                        for px in row {
                            acc[0] += (px >> 16) & 0xFF;
                            acc[1] += (px >> 8) & 0xFF;
                            acc[2] += px & 0xFF;
                        }
                    }
                    let n = (DS * DS) as u32;
                    self.small[sy * SW + sx] = [acc[0] / n, acc[1] / n, acc[2] / n];
                }
            }
            for _ in 0..2 {
                box_blur(&self.small, &mut self.tmp, SW, SH, 1, SW);
                box_blur(&self.tmp, &mut self.small, SH, SW, SW, 1);
            }
        }
        let mut cached_row = usize::MAX;
        for y in 0..HEIGHT {
            let (y0, y1, wy) = upsample_tap(y, SH);
            if k > 0 && y0 != cached_row {
                cached_row = y0;
                for (x, &(x0, x1, wx)) in self.xtab.iter().enumerate() {
                    let (a, b) = (self.small[y0 * SW + x0], self.small[y0 * SW + x1]);
                    let (c, d) = (self.small[y1 * SW + x0], self.small[y1 * SW + x1]);
                    for i in 0..3 {
                        self.line_a[x][i] = a[i] * (256 - wx) + b[i] * wx;
                        self.line_b[x][i] = c[i] * (256 - wx) + d[i] * wx;
                    }
                }
            }
            let row = &mut self.pixels[y * WIDTH..(y + 1) * WIDTH];
            let shade = &self.shade[y * WIDTH..(y + 1) * WIDTH];
            for x in 0..WIDTH {
                let px = row[x];
                let mut ch = [(px >> 16) & 0xFF, (px >> 8) & 0xFF, px & 0xFF];
                if k > 0 {
                    for (i, c) in ch.iter_mut().enumerate() {
                        let glow = (self.line_a[x][i] * (256 - wy) + self.line_b[x][i] * wy) >> 16;
                        *c = (*c + ((glow * k) >> 8)).min(255);
                    }
                }
                let s = shade[x] as u32;
                row[x] =
                    (((ch[0] * s) >> 8) << 16) | (((ch[1] * s) >> 8) << 8) | ((ch[2] * s) >> 8);
            }
        }
    }

    pub fn to_bgr(&self, out: &mut Vec<u8>) {
        out.clear();
        for px in &self.pixels {
            out.push((*px & 0xFF) as u8);
            out.push(((*px >> 8) & 0xFF) as u8);
            out.push(((*px >> 16) & 0xFF) as u8);
        }
    }
}

/// Source taps and weight (0..256) for upsampling coordinate `i` from a
/// quarter-resolution axis of length `n`.
fn upsample_tap(i: usize, n: usize) -> (usize, usize, u32) {
    let f = (i as f32 + 0.5) / DS as f32 - 0.5;
    let i0 = (f.floor().max(0.0) as usize).min(n - 1);
    let i1 = (i0 + 1).min(n - 1);
    let w = ((f - i0 as f32).clamp(0.0, 1.0) * 256.0) as u32;
    (i0, i1, w)
}

/// Running-sum box blur (radius 3) along lines of length `len`; `step` is
/// the stride along a line, `pitch` the stride between lines.
fn box_blur(
    src: &[[u32; 3]],
    dst: &mut [[u32; 3]],
    lines: usize,
    len: usize,
    pitch: usize,
    step: usize,
) {
    const R: usize = 3;
    for l in 0..lines {
        let at = |i: usize| l * pitch + i.min(len - 1) * step;
        let mut sum = [0u32; 3];
        for i in 0..=R {
            let v = src[at(i)];
            let w = if i == 0 { R as u32 + 1 } else { 1 };
            for c in 0..3 {
                sum[c] += v[c] * w;
            }
        }
        for i in 0..len {
            dst[at(i)] = [sum[0] / 7, sum[1] / 7, sum[2] / 7];
            let add = src[at(i + R + 1)];
            let sub = src[at(i.saturating_sub(R))];
            for c in 0..3 {
                sum[c] = sum[c] + add[c] - sub[c];
            }
        }
    }
}
