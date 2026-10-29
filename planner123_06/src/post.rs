// Cells -> pixels. The ratatui Buffer is rasterised cell by cell (background
// fill, then the glyph mask in the foreground colour, bold a little
// brighter); then pixel-positioned sprites (the sine scroller), the logo, and
// one fixed-point pass of phosphor bloom, scanlines and vignette.
use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::style::Modifier;

use crate::canvas::{lerp_rgb, scale_rgb, to_rgb, CELL_H, CELL_W, COLS, ROWS};
use crate::data::{BG, FG};
use crate::font::{mask, Mask};
use crate::logo::LogoState;

pub const WIDTH: usize = COLS as usize * CELL_W;
pub const HEIGHT: usize = ROWS as usize * CELL_H;
const DS: usize = 4;
const SW: usize = WIDTH / DS;
const SH: usize = HEIGHT / DS;

/// A glyph at an arbitrary pixel position, `scale` times the cell size.
#[derive(Clone, Copy)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub ch: char,
    pub rgb: u32,
    pub scale: usize,
}

/// Per-frame look: everything that is not a cell.
pub struct Look {
    pub bloom: f32,
    /// 0..1 white-mint flash added over the frame.
    pub flash: f32,
    /// Overall gain (fades).
    pub gain: f32,
    pub sprites: Vec<Sprite>,
    pub logo: Option<LogoState>,
}

impl Default for Look {
    fn default() -> Self {
        Self {
            bloom: 0.55,
            flash: 0.0,
            gain: 1.0,
            sprites: Vec::new(),
            logo: None,
        }
    }
}

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
            let scan = if y % 3 == 2 { 0.78 } else { 1.0 };
            for x in 0..WIDTH {
                let dx = (x as f32 / WIDTH as f32 - 0.5) * 2.0;
                let dy = (y as f32 / HEIGHT as f32 - 0.5) * 2.0;
                let vig = (1.0 - (dx * dx * 0.14 + dy * dy * 0.22)).clamp(0.0, 1.0);
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

    pub fn render(&mut self, buf: &Buffer, look: &Look) {
        self.raster(buf);
        for s in &look.sprites {
            self.sprite(s);
        }
        if let Some(state) = &look.logo {
            crate::logo::draw(&mut self.pixels, WIDTH, HEIGHT, state);
        }
        if look.flash > 0.0 || look.gain < 1.0 {
            let (flash, gain) = (look.flash.min(1.0), look.gain.clamp(0.0, 1.0));
            for px in self.pixels.iter_mut() {
                *px = scale_rgb(lerp_rgb(*px, 0xE8FFF0, flash * 0.6), gain);
            }
        }
        self.bloom_and_shade(look.bloom);
    }

    fn glyph(&mut self, ch: char) -> Mask {
        *self.masks.entry(ch).or_insert_with(|| mask(ch))
    }

    fn raster(&mut self, buf: &Buffer) {
        for row in 0..ROWS {
            for col in 0..COLS {
                let cell = &buf[(col, row)];
                let bg = to_rgb(cell.bg, BG);
                let mut fg = to_rgb(cell.fg, FG);
                if cell.modifier.contains(Modifier::BOLD) {
                    fg = lerp_rgb(fg, 0xFFFFFF, 0.18);
                }
                let ch = cell.symbol().chars().next().unwrap_or(' ');
                let m = if ch == ' ' {
                    [0; CELL_H]
                } else {
                    self.glyph(ch)
                };
                let (x0, y0) = (col as usize * CELL_W, row as usize * CELL_H);
                for (y, bits) in m.iter().enumerate() {
                    let line = &mut self.pixels[(y0 + y) * WIDTH + x0..][..CELL_W];
                    for (x, px) in line.iter_mut().enumerate() {
                        *px = if (bits >> x) & 1 == 1 { fg } else { bg };
                    }
                }
            }
        }
    }

    fn sprite(&mut self, s: &Sprite) {
        let m = self.glyph(s.ch);
        let (x0, y0) = (s.x.round() as i32, s.y.round() as i32);
        for y in 0..CELL_H * s.scale {
            let py = y0 + y as i32;
            if py < 0 || py >= HEIGHT as i32 {
                continue;
            }
            let bits = m[y / s.scale];
            if bits == 0 {
                continue;
            }
            for x in 0..CELL_W * s.scale {
                let px = x0 + x as i32;
                if px >= 0 && px < WIDTH as i32 && (bits >> (x / s.scale)) & 1 == 1 {
                    self.pixels[py as usize * WIDTH + px as usize] = s.rgb;
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
                            // Only light above the background glows.
                            acc[0] += ((px >> 16) & 0xFF).saturating_sub(40);
                            acc[1] += ((px >> 8) & 0xFF).saturating_sub(40);
                            acc[2] += (px & 0xFF).saturating_sub(48);
                        }
                    }
                    let n = (DS * DS) as u32;
                    self.small[sy * SW + sx] = [acc[0] / n, acc[1] / n, acc[2] / n];
                }
            }
            for _ in 0..2 {
                box_blur(&self.small, &mut self.tmp, SH, SW, SW, 1);
                box_blur(&self.tmp, &mut self.small, SW, SH, 1, SW);
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
