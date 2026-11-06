// The pixel frame and the few primitives every scene draws with: soft
// discs and lines, rectangles, and text in the series' 8x8 font at any
// integer scale. Colours are 0x00RRGGBB, as minifb and demofx expect.
use demofx::cell::{lerp_rgb, pack, rgb_parts};
use demofx::font::{mask, Mask};
use demofx::{Pixels, CELL_W};

pub const W: usize = 1920;
pub const H: usize = 1080;

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

    /// One-pixel additive segment: the trip sketch draws its roads and its
    /// polylines with it.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.add(x, y, rgb, a);
            if x == x1 && y == y1 {
                break;
            }
            let twice = err * 2;
            if twice >= dy {
                err += dy;
                x += sx;
            }
            if twice <= dx {
                err += dx;
                y += sy;
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
}

/// Horizontal distance between characters at `scale`.
pub const fn advance(scale: i32) -> i32 {
    CELL_W as i32 * scale
}

pub fn ramp(stops: &[u32], t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (t as usize).min(stops.len() - 2);
    lerp_rgb(stops[i], stops[i + 1], t - i as f32)
}

pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    lerp_rgb(a, b, t)
}

pub fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}
