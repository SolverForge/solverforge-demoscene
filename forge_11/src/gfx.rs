// The pixel frame and its primitives: rectangles, lines, soft glows, and
// text at any cell size. Text is the series' 8x8 font (demofx), rasterised
// per cell size into anti-aliased coverage masks and cached, so a code
// listing at 12x22 is as cheap as one at 10x18. glyphfx clips draw through
// `Ink`, a CellSink that places cells at a pixel origin.
use std::collections::HashMap;
use std::rc::Rc;

use demofx::cell::{lerp_rgb, pack, rgb_parts};
use demofx::font::mask;
use demofx::{Cell, CellSink, Pixels, CELL_H, CELL_W};

pub const W: usize = 1920;
pub const H: usize = 1080;

pub struct Frame {
    pub px: Vec<u32>,
    glyphs: HashMap<(char, u16, u16), Rc<[u8]>>,
}

impl Pixels for Frame {
    fn size(&self) -> (usize, usize) {
        (W, H)
    }
    fn get(&self, x: usize, y: usize) -> u32 {
        self.px[y * W + x]
    }
    fn put(&mut self, x: usize, y: usize, rgb: u32) {
        self.px[y * W + x] = rgb;
    }
}

/// A text cell size in pixels. The font's cell is 10x18 at 1x.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

pub const fn size(w: i32, h: i32) -> Size {
    Size { w, h }
}

/// 1x: the textmode grid (192x60 cells over the frame).
pub const S1: Size = size(10, 18);
/// 2x.
pub const S2: Size = size(20, 36);
/// 3x.
pub const S3: Size = size(30, 54);

impl Frame {
    pub fn new() -> Self {
        Self {
            px: vec![0; W * H],
            glyphs: HashMap::new(),
        }
    }

    pub fn gradient(&mut self, stops: &[u32]) {
        for y in 0..H {
            let c = ramp(stops, y as f32 / (H - 1) as f32);
            self.px[y * W..(y + 1) * W].fill(c);
        }
    }

    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, rgb: u32, a: f32) {
        if a <= 0.0 || x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let i = y as usize * W + x as usize;
        self.px[i] = if a >= 1.0 {
            rgb
        } else {
            lerp_rgb(self.px[i], rgb, a)
        };
    }

    /// Additive light.
    #[inline]
    pub fn add(&mut self, x: i32, y: i32, rgb: u32, k: f32) {
        if k <= 0.0 || x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let i = y as usize * W + x as usize;
        self.px[i] = add_rgb(self.px[i], rgb, k);
    }

    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
        if a <= 0.0 {
            return;
        }
        let (x0, x1) = (x0.max(0), x1.min(W as i32));
        let (y0, y1) = (y0.max(0), y1.min(H as i32));
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let a8 = (a.min(1.0) * 256.0) as u32;
        let (r, g, b) = split(rgb);
        for y in y0..y1 {
            let row = &mut self.px[y as usize * W + x0 as usize..y as usize * W + x1 as usize];
            for p in row {
                let (pr, pg, pb) = split(*p);
                *p = join(
                    pr + (((r as i32 - pr as i32) * a8 as i32) >> 8) as u32,
                    pg + (((g as i32 - pg as i32) * a8 as i32) >> 8) as u32,
                    pb + (((b as i32 - pb as i32) * a8 as i32) >> 8) as u32,
                );
            }
        }
    }

    /// Darken a rectangle towards black (panel backdrops).
    pub fn dim(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, keep: f32) {
        let k = (keep.clamp(0.0, 1.0) * 256.0) as u32;
        let (x0, x1) = (x0.max(0), x1.min(W as i32));
        let (y0, y1) = (y0.max(0), y1.min(H as i32));
        for y in y0..y1 {
            for p in
                &mut self.px[y as usize * W + x0 as usize..y as usize * W + x1.max(x0) as usize]
            {
                let (r, g, b) = split(*p);
                *p = join((r * k) >> 8, (g * k) >> 8, (b * k) >> 8);
            }
        }
    }

    pub fn frame_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
        self.rect(x0, y0, x1, y0 + 1, rgb, a);
        self.rect(x0, y1 - 1, x1, y1, rgb, a);
        self.rect(x0, y0, x0 + 1, y1, rgb, a);
        self.rect(x1 - 1, y0, x1, y1, rgb, a);
    }

    /// Additive anti-aliased line of width ~`w` pixels.
    pub fn line(&mut self, (x0, y0): (f32, f32), (x1, y1): (f32, f32), w: f32, rgb: u32, k: f32) {
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len2 = (dx * dx + dy * dy).max(1e-6);
        let r = w * 0.5 + 1.0;
        let (bx0, bx1) = ((x0.min(x1) - r) as i32, (x0.max(x1) + r) as i32 + 1);
        let (by0, by1) = ((y0.min(y1) - r) as i32, (y0.max(y1) + r) as i32 + 1);
        for y in by0.max(0)..by1.min(H as i32) {
            for x in bx0.max(0)..bx1.min(W as i32) {
                let (px, py) = (x as f32 + 0.5 - x0, y as f32 + 0.5 - y0);
                let t = ((px * dx + py * dy) / len2).clamp(0.0, 1.0);
                let (ex, ey) = (px - dx * t, py - dy * t);
                let d = (ex * ex + ey * ey).sqrt();
                let cover = (w * 0.5 + 0.5 - d).clamp(0.0, 1.0);
                if cover > 0.0 {
                    self.add(x, y, rgb, k * cover);
                }
            }
        }
    }

    /// Additive soft glow, falling off to zero at `r`.
    pub fn glow(&mut self, cx: f32, cy: f32, r: f32, rgb: u32, k: f32) {
        if k <= 0.0 || r <= 0.0 {
            return;
        }
        let (x0, x1) = ((cx - r) as i32, (cx + r) as i32);
        let (y0, y1) = ((cy - r) as i32, (cy + r) as i32);
        let inv = 1.0 / (r * r);
        for y in y0.max(0)..=y1.min(H as i32 - 1) {
            let dy = y as f32 - cy;
            for x in x0.max(0)..=x1.min(W as i32 - 1) {
                let dx = x as f32 - cx;
                let d2 = (dx * dx + dy * dy) * inv;
                if d2 < 1.0 {
                    let f = (1.0 - d2) * (1.0 - d2);
                    self.add(x, y, rgb, k * f);
                }
            }
        }
    }

    fn glyph(&mut self, ch: char, s: Size) -> Rc<[u8]> {
        self.glyphs
            .entry((ch, s.w as u16, s.h as u16))
            .or_insert_with(|| Rc::from(coverage(ch, s)))
            .clone()
    }

    /// One character with its top-left at (x, y).
    pub fn char_at(&mut self, x: i32, y: i32, ch: char, s: Size, rgb: u32, a: f32) {
        if ch == ' ' || a <= 0.0 || x >= W as i32 || y >= H as i32 || x + s.w <= 0 || y + s.h <= 0 {
            return;
        }
        let a = a.min(1.0);
        let cov = self.glyph(ch, s);
        for gy in 0..s.h {
            let py = y + gy;
            if py < 0 || py >= H as i32 {
                continue;
            }
            for gx in 0..s.w {
                let c = cov[(gy * s.w + gx) as usize];
                if c != 0 {
                    self.blend(x + gx, py, rgb, a * c as f32 / 255.0);
                }
            }
        }
    }

    pub fn text(&mut self, x: i32, y: i32, s: &str, size: Size, rgb: u32, a: f32) {
        for (i, ch) in s.chars().enumerate() {
            self.char_at(x + i as i32 * size.w, y, ch, size, rgb, a);
        }
    }

    pub fn text_centered(&mut self, cx: i32, y: i32, s: &str, size: Size, rgb: u32, a: f32) {
        let w = s.chars().count() as i32 * size.w;
        self.text(cx - w / 2, y, s, size, rgb, a);
    }

    /// Text with a drop shadow: legible over the copper and the board.
    pub fn text_shadow(&mut self, x: i32, y: i32, s: &str, size: Size, rgb: u32, a: f32) {
        let o = (size.w / 10).max(1);
        self.text(x + o, y + o, s, size, 0x000000, a * 0.8);
        self.text(x, y, s, size, rgb, a);
    }
}

/// Anti-aliased coverage (0..255) of glyph `ch` scaled to a `s` cell: each
/// output pixel is the area of the 10x18 mask under it.
fn coverage(ch: char, s: Size) -> Vec<u8> {
    let m = mask(ch);
    let (sw, sh) = (s.w.max(1) as usize, s.h.max(1) as usize);
    let (fx, fy) = (CELL_W as f32 / sw as f32, CELL_H as f32 / sh as f32);
    let mut out = vec![0u8; sw * sh];
    for oy in 0..sh {
        let (y0, y1) = (oy as f32 * fy, (oy + 1) as f32 * fy);
        for ox in 0..sw {
            let (x0, x1) = (ox as f32 * fx, (ox + 1) as f32 * fx);
            let mut area = 0.0;
            let mut my = y0.floor() as usize;
            while (my as f32) < y1 && my < CELL_H {
                let hy = (y1.min(my as f32 + 1.0) - y0.max(my as f32)).max(0.0);
                let bits = m[my];
                let mut mx = x0.floor() as usize;
                while (mx as f32) < x1 && mx < CELL_W {
                    if (bits >> mx) & 1 == 1 {
                        let hx = (x1.min(mx as f32 + 1.0) - x0.max(mx as f32)).max(0.0);
                        area += hx * hy;
                    }
                    mx += 1;
                }
                my += 1;
            }
            // Slightly gamma-lifted so thin strokes keep their weight.
            let v = (area / (fx * fy)).clamp(0.0, 1.0).powf(0.8);
            out[oy * sw + ox] = (v * 255.0).round() as u8;
        }
    }
    out
}

/// Places glyphfx cells (or any cells) on the frame at a pixel origin.
pub struct Ink<'a> {
    pub f: &'a mut Frame,
    pub x: i32,
    pub y: i32,
    pub size: Size,
    pub gain: f32,
    /// Optional recolour: cells take this colour, weighted by `tint.1`.
    pub tint: Option<(u32, f32)>,
}

impl CellSink for Ink<'_> {
    fn set(&mut self, col: i32, row: i32, cell: Cell) {
        let rgb = match self.tint {
            Some((c, k)) => lerp_rgb(cell.rgb, c, k),
            None => cell.rgb,
        };
        self.f.char_at(
            self.x + col * self.size.w,
            self.y + row * self.size.h,
            cell.ch,
            self.size,
            rgb,
            self.gain,
        );
    }
}

#[inline]
fn split(p: u32) -> (u32, u32, u32) {
    ((p >> 16) & 255, (p >> 8) & 255, p & 255)
}

#[inline]
fn join(r: u32, g: u32, b: u32) -> u32 {
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

pub fn add_rgb(a: u32, b: u32, k: f32) -> u32 {
    let (ar, ag, ab) = rgb_parts(a);
    let (br, bg, bb) = rgb_parts(b);
    pack(ar + br * k, ag + bg * k, ab + bb * k)
}

pub fn ramp(stops: &[u32], t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (t as usize).min(stops.len() - 2);
    lerp_rgb(stops[i], stops[i + 1], t - i as f32)
}

pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    lerp_rgb(a, b, t)
}

pub fn scale(rgb: u32, k: f32) -> u32 {
    demofx::cell::scale_rgb(rgb, k)
}

pub fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

pub fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

/// Greedy word wrap to `cols` columns; words longer than a line are split.
/// Existing newlines are kept.
pub fn wrap(s: &str, cols: usize) -> Vec<String> {
    let cols = cols.max(1);
    let mut out = Vec::new();
    for para in s.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let mut word = word.to_string();
            loop {
                let need = if line.is_empty() {
                    word.chars().count()
                } else {
                    line.chars().count() + 1 + word.chars().count()
                };
                if need <= cols {
                    if !line.is_empty() {
                        line.push(' ');
                    }
                    line.push_str(&word);
                    break;
                }
                if !line.is_empty() {
                    out.push(std::mem::take(&mut line));
                    continue;
                }
                // The word alone is wider than a line: split it.
                let head: String = word.chars().take(cols).collect();
                word = word.chars().skip(cols).collect();
                out.push(head);
                if word.is_empty() {
                    break;
                }
            }
        }
        out.push(line);
    }
    out
}

/// Clip a line to `cols` characters, marking the cut with '>'.
pub fn clip(s: &str, cols: usize) -> String {
    if s.chars().count() <= cols {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(cols.saturating_sub(1)).collect();
        t.push('>');
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_never_exceeds_the_width() {
        let text = "a quick brown fox jumps over averyveryverylongwordthatdoesnotfit the lazy dog";
        for cols in [5, 8, 13, 40] {
            for line in wrap(text, cols) {
                assert!(line.chars().count() <= cols, "{line:?} > {cols}");
            }
        }
        assert_eq!(wrap("ab cd", 5), vec!["ab cd"]);
    }

    #[test]
    fn integer_scales_are_crisp() {
        let c = coverage('A', S2);
        assert!(c.iter().all(|&v| v == 0 || v == 255));
    }
}
