// The pixel frame and the few primitives every scene draws with: soft
// discs and lines, rectangles, and text in the series' 8x8 font at any
// integer scale. Colours are 0x00RRGGBB, as minifb and demofx expect.
use demofx::cell::{lerp_rgb, pack, rgb_parts};
use demofx::font::{mask, Mask};
use demofx::{Cell, CellSink, Clip, Pixels, CELL_H, CELL_W};

pub const W: usize = 1280;
pub const H: usize = 720;

pub struct Frame {
    pub px: Vec<u32>,
    masks: Vec<(char, Mask)>,
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

impl Frame {
    pub fn new() -> Self {
        Self {
            px: vec![0; W * H],
            masks: Vec::new(),
        }
    }

    pub fn clear(&mut self, rgb: u32) {
        self.px.fill(rgb);
    }

    /// Vertical gradient through evenly spaced stops.
    pub fn gradient(&mut self, stops: &[u32]) {
        for y in 0..H {
            let c = ramp(stops, y as f32 / (H - 1) as f32);
            self.px[y * W..(y + 1) * W].fill(c);
        }
    }

    pub fn blend(&mut self, x: i32, y: i32, rgb: u32, a: f32) {
        if a <= 0.0 || x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let i = y as usize * W + x as usize;
        self.px[i] = lerp_rgb(self.px[i], rgb, a.min(1.0));
    }

    /// Additive light.
    pub fn add(&mut self, x: i32, y: i32, rgb: u32, k: f32) {
        if k <= 0.0 || x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
            return;
        }
        let i = y as usize * W + x as usize;
        let (ar, ag, ab) = rgb_parts(self.px[i]);
        let (br, bg, bb) = rgb_parts(rgb);
        self.px[i] = pack(ar + br * k, ag + bg * k, ab + bb * k);
    }

    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
        for y in y0.max(0)..y1.min(H as i32) {
            for x in x0.max(0)..x1.min(W as i32) {
                self.blend(x, y, rgb, a);
            }
        }
    }

    /// One-pixel outline.
    pub fn frame_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
        self.rect(x0, y0, x1, y0 + 1, rgb, a);
        self.rect(x0, y1 - 1, x1, y1, rgb, a);
        self.rect(x0, y0, x0 + 1, y1, rgb, a);
        self.rect(x1 - 1, y0, x1, y1, rgb, a);
    }

    /// Anti-aliased filled disc.
    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, rgb: u32, a: f32) {
        let (x0, x1) = ((cx - r - 1.0) as i32, (cx + r + 1.0) as i32);
        let (y0, y1) = ((cy - r - 1.0) as i32, (cy + r + 1.0) as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let cov = (r + 0.5 - d).clamp(0.0, 1.0);
                self.blend(x, y, rgb, a * cov);
            }
        }
    }

    /// Additive soft glow, falling off to zero at `r`.
    pub fn glow(&mut self, cx: f32, cy: f32, r: f32, rgb: u32, k: f32) {
        let (x0, x1) = ((cx - r) as i32, (cx + r) as i32);
        let (y0, y1) = ((cy - r) as i32, (cy + r) as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() / r;
                if d < 1.0 {
                    let f = (1.0 - d) * (1.0 - d);
                    self.add(x, y, rgb, k * f);
                }
            }
        }
    }

    /// Anti-aliased ring.
    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, width: f32, rgb: u32, a: f32) {
        let o = r + width;
        let (x0, x1) = ((cx - o - 1.0) as i32, (cx + o + 1.0) as i32);
        let (y0, y1) = ((cy - o - 1.0) as i32, (cy + o + 1.0) as i32);
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
                let cov = (width * 0.5 + 0.5 - (d - r).abs()).clamp(0.0, 1.0);
                self.blend(x, y, rgb, a * cov);
            }
        }
    }

    /// Anti-aliased line of the given width; `upto` 0..1 draws only the
    /// first part (for lines that trace themselves in).
    #[allow(clippy::too_many_arguments)]
    pub fn line(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        width: f32,
        rgb: u32,
        a: f32,
        upto: f32,
    ) {
        let upto = upto.clamp(0.0, 1.0);
        if upto <= 0.0 || a <= 0.0 {
            return;
        }
        let (x1, y1) = (x0 + (x1 - x0) * upto, y0 + (y1 - y0) * upto);
        let (dx, dy) = (x1 - x0, y1 - y0);
        let len2 = (dx * dx + dy * dy).max(1e-6);
        let hw = width * 0.5;
        let (bx0, bx1) = (x0.min(x1) - hw - 1.0, x0.max(x1) + hw + 1.0);
        let (by0, by1) = (y0.min(y1) - hw - 1.0, y0.max(y1) + hw + 1.0);
        for y in by0 as i32..=by1 as i32 {
            for x in bx0 as i32..=bx1 as i32 {
                let (px, py) = (x as f32 + 0.5 - x0, y as f32 + 0.5 - y0);
                let h = ((px * dx + py * dy) / len2).clamp(0.0, 1.0);
                let d = ((px - dx * h).powi(2) + (py - dy * h).powi(2)).sqrt();
                let cov = (hw + 0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend(x, y, rgb, a * cov);
                }
            }
        }
    }

    fn glyph(&mut self, ch: char) -> Mask {
        if let Some((_, m)) = self.masks.iter().find(|(c, _)| *c == ch) {
            return *m;
        }
        let m = mask(ch);
        self.masks.push((ch, m));
        m
    }

    /// One character at a pixel position, `scale` pixels per font pixel.
    pub fn char_at(&mut self, x: i32, y: i32, ch: char, scale: i32, rgb: u32, a: f32) {
        if ch == ' ' || a <= 0.0 {
            return;
        }
        let m = self.glyph(ch);
        for (my, bits) in m.iter().enumerate() {
            if *bits == 0 {
                continue;
            }
            for mx in 0..CELL_W {
                if (bits >> mx) & 1 == 1 {
                    let (px, py) = (x + mx as i32 * scale, y + my as i32 * scale / 2);
                    let hh = (scale / 2).max(1);
                    self.rect(px, py, px + scale, py + hh, rgb, a);
                }
            }
        }
    }

    /// Text at a pixel position. A cell is 10x9 font pixels: the font's
    /// 8x8 glyph is stored double height, drawn here at square pixels.
    pub fn text(&mut self, x: i32, y: i32, s: &str, scale: i32, rgb: u32, a: f32) {
        for (i, ch) in s.chars().enumerate() {
            self.char_at(x + i as i32 * advance(scale), y, ch, scale, rgb, a);
        }
    }

    pub fn text_centered(&mut self, cx: i32, y: i32, s: &str, scale: i32, rgb: u32, a: f32) {
        let w = s.chars().count() as i32 * advance(scale);
        self.text(cx - w / 2, y, s, scale, rgb, a);
    }

    /// A glyphfx clip frame, each cell drawn as a character at `scale`.
    pub fn clip(&mut self, clip: &Clip, idx: usize, x: i32, y: i32, scale: i32, gain: f32) {
        let mut sink = ClipSink {
            frame: self,
            x,
            y,
            scale,
            gain,
        };
        clip.for_each_cell(idx, |col, row, cell| sink.set(col as i32, row as i32, cell));
    }
}

/// Horizontal distance between characters at `scale`.
pub const fn advance(scale: i32) -> i32 {
    CELL_W as i32 * scale
}

/// Line height at `scale`.
pub const fn line_height(scale: i32) -> i32 {
    CELL_H as i32 * scale / 2
}

struct ClipSink<'a> {
    frame: &'a mut Frame,
    x: i32,
    y: i32,
    scale: i32,
    gain: f32,
}

impl CellSink for ClipSink<'_> {
    fn set(&mut self, col: i32, row: i32, cell: Cell) {
        let (px, py) = (
            self.x + col * advance(self.scale),
            self.y + row * line_height(self.scale),
        );
        let cell = cell.scaled(self.gain);
        self.frame
            .char_at(px, py, cell.ch, self.scale, cell.rgb, 1.0);
    }
}

pub fn ramp(stops: &[u32], t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (t as usize).min(stops.len() - 2);
    lerp_rgb(stops[i], stops[i + 1], t - i as f32)
}

pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    lerp_rgb(a, b, t)
}

/// Stable 0..1 hash of two integers.
pub fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    (h & 0xFFFF) as f32 / 65535.0
}

pub fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}
