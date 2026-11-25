// The pixel frame and the few primitives every scene draws with: soft
// discs and lines, rectangles, and text in the series' 8x8 font at any
// integer scale. Colours are 0x00RRGGBB, as minifb and demofx expect.
use demofx::cell::lerp_rgb;
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
        self.px[i] = lerp_px(self.px[i], rgb, a.min(1.0));
    }

    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
        if a <= 0.0 {
            return;
        }
        let (x0, x1) = (x0.max(0) as usize, x1.min(W as i32).max(0) as usize);
        if x0 >= x1 {
            return;
        }
        for y in y0.max(0)..y1.min(H as i32) {
            let row = &mut self.px[y as usize * W + x0..y as usize * W + x1];
            if a >= 1.0 {
                row.fill(rgb);
            } else {
                let to = parts(rgb);
                for p in row {
                    *p = lerp_to(*p, to, a);
                }
            }
        }
    }

    /// Additive soft glow, falling off to zero at `r`.
    pub fn glow(&mut self, cx: f32, cy: f32, r: f32, rgb: u32, k: f32) {
        if k <= 0.0 {
            return;
        }
        let (x0, x1) = (
            ((cx - r) as i32).max(0),
            ((cx + r) as i32).min(W as i32 - 1),
        );
        let (y0, y1) = ((cy - r) as i32, (cy + r) as i32);
        let (br, bg, bb) = parts(rgb);
        par_rows(&mut self.px, |y, row| {
            let y = y as i32;
            if y < y0 || y > y1 {
                return;
            }
            for x in x0..=x1 {
                let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() / r;
                if d < 1.0 {
                    let fall = (1.0 - d) * (1.0 - d);
                    let a = k * fall;
                    if a > 0.0 {
                        let p = &mut row[x as usize];
                        let (ar, ag, ab) = parts(*p);
                        *p = pack_px(ar + br * a, ag + bg * a, ab + bb * a);
                    }
                }
            }
        });
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

/// Run `f(y, row)` over every row of a frame-sized buffer, the rows split
/// into one band per core. Each row is touched by one thread only, so a pass
/// gives exactly the pixels its single-threaded loop would.
pub fn par_rows(px: &mut [u32], f: impl Fn(usize, &mut [u32]) + Sync) {
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    let per = H.div_ceil(cores.clamp(1, 32));
    std::thread::scope(|s| {
        for (band, chunk) in px.chunks_mut(per * W).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (j, row) in chunk.chunks_mut(W).enumerate() {
                    f(band * per + j, row);
                }
            });
        }
    });
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

/// demofx's `cell::pack` for the per-pixel passes, inlined: rounding a
/// channel as `(v + 0.5) as u32` equals `v.round()` for every v >= 0, and
/// the cast saturates negatives to 0 where `pack` clamps them, so the result
/// is the same packed pixel without a call to `roundf` per channel.
#[inline(always)]
pub fn pack_px(r: f32, g: f32, b: f32) -> u32 {
    let c = |v: f32| ((v + 0.5) as u32).min(255);
    (c(r) << 16) | (c(g) << 8) | c(b)
}

/// demofx's `cell::lerp_rgb`, inlined (same arithmetic, `pack_px` rounding).
#[inline(always)]
pub fn lerp_px(a: u32, b: u32, t: f32) -> u32 {
    lerp_to(a, parts(b), t)
}

/// `lerp_px` toward a colour already split into channels.
#[inline(always)]
pub fn lerp_to(a: u32, (br, bg, bb): (f32, f32, f32), t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let (ar, ag, ab) = parts(a);
    pack_px(ar + (br - ar) * t, ag + (bg - ag) * t, ab + (bb - ab) * t)
}

#[inline(always)]
pub fn parts(rgb: u32) -> (f32, f32, f32) {
    (
        ((rgb >> 16) & 255) as f32,
        ((rgb >> 8) & 255) as f32,
        (rgb & 255) as f32,
    )
}

pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    lerp_rgb(a, b, t)
}
