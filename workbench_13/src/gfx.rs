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

    /// One-pixel outline.
    pub fn frame_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
        self.rect(x0, y0, x1, y0 + 1, rgb, a);
        self.rect(x0, y1 - 1, x1, y1, rgb, a);
        self.rect(x0, y0, x0 + 1, y1, rgb, a);
        self.rect(x1 - 1, y0, x1, y1, rgb, a);
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

impl Frame {
    /// A line `w` pixels wide, its ends rounded.
    #[allow(clippy::too_many_arguments)]
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, w: f32, rgb: u32, a: f32) {
        if a <= 0.0 {
            return;
        }
        let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
        let steps = (len * 1.5).ceil().max(1.0) as i32;
        let r = (w * 0.5).max(0.5);
        let ri = r.ceil() as i32;
        let mut last = (i32::MIN, i32::MIN);
        for i in 0..=steps {
            let k = i as f32 / steps as f32;
            let (cx, cy) = (x0 + (x1 - x0) * k, y0 + (y1 - y0) * k);
            let c = (cx.round() as i32, cy.round() as i32);
            if c == last {
                continue;
            }
            last = c;
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d <= r {
                        self.blend(
                            c.0 + dx,
                            c.1 + dy,
                            rgb,
                            a * (r + 0.5 - d).clamp(0.0, 1.0) * 0.6,
                        );
                    }
                }
            }
        }
    }

    /// Dashes along a horizontal or vertical run.
    #[allow(clippy::too_many_arguments)]
    pub fn dashed(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        on: i32,
        off: i32,
        rgb: u32,
        a: f32,
    ) {
        if y0 == y1 {
            let mut x = x0;
            while x < x1 {
                self.rect(x, y0, (x + on).min(x1), y0 + 1, rgb, a);
                x += on + off;
            }
        } else {
            let mut y = y0;
            while y < y1 {
                self.rect(x0, y, x0 + 1, (y + on).min(y1), rgb, a);
                y += on + off;
            }
        }
    }
}

/// A binary PPM (P6) image, drawn scaled with bilinear sampling.
pub struct Image {
    pub w: usize,
    pub h: usize,
    px: Vec<u32>,
}

impl Image {
    pub fn ppm(bytes: &[u8]) -> Self {
        let mut fields = Vec::new();
        let mut i = 0;
        while fields.len() < 4 {
            while bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            let start = i;
            while !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            fields.push(
                std::str::from_utf8(&bytes[start..i])
                    .expect("PPM header")
                    .to_string(),
            );
        }
        assert_eq!(fields[0], "P6");
        let (w, h): (usize, usize) = (fields[1].parse().unwrap(), fields[2].parse().unwrap());
        let data = &bytes[i + 1..i + 1 + w * h * 3];
        let px = data
            .chunks(3)
            .map(|c| (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32)
            .collect();
        Self { w, h, px }
    }

    fn sample(&self, u: f32, v: f32) -> (f32, f32, f32) {
        let x = (u - 0.5).clamp(0.0, (self.w - 1) as f32);
        let y = (v - 0.5).clamp(0.0, (self.h - 1) as f32);
        let (x0, y0) = (x as usize, y as usize);
        let (x1, y1) = ((x0 + 1).min(self.w - 1), (y0 + 1).min(self.h - 1));
        let (tx, ty) = (x - x0 as f32, y - y0 as f32);
        let p = |x: usize, y: usize| parts(self.px[y * self.w + x]);
        let (a, b, c, d) = (p(x0, y0), p(x1, y0), p(x0, y1), p(x1, y1));
        let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
        (
            l(l(a.0, b.0, tx), l(c.0, d.0, tx), ty),
            l(l(a.1, b.1, tx), l(c.1, d.1, tx), ty),
            l(l(a.2, b.2, tx), l(c.2, d.2, tx), ty),
        )
    }

    /// Draw at (x, y), `scale` times its size. `row_alpha(sy)` gives each
    /// screen row's opacity (the reveals and wipes are row-wise), and
    /// `shift(sy)` a horizontal offset for that row (raster tears).
    pub fn draw(
        &self,
        f: &mut Frame,
        x: i32,
        y: i32,
        scale: f32,
        row_alpha: impl Fn(i32) -> f32 + Sync,
        shift: impl Fn(i32) -> f32 + Sync,
    ) {
        let (dw, dh) = (
            (self.w as f32 * scale) as i32,
            (self.h as f32 * scale) as i32,
        );
        par_rows(&mut f.px, |sy, row| {
            let sy = sy as i32;
            if sy < y || sy >= y + dh {
                return;
            }
            let a = row_alpha(sy);
            if a <= 0.0 {
                return;
            }
            let dx = shift(sy);
            let v = (sy - y) as f32 / scale;
            for sx in x.max(0)..(x + dw).min(W as i32) {
                let u = (sx as f32 - x as f32 - dx) / scale;
                if u < 0.0 || u >= self.w as f32 {
                    continue;
                }
                let (r, g, b) = self.sample(u, v);
                let p = &mut row[sx as usize];
                *p = lerp_to(*p, (r, g, b), a);
            }
        });
    }
}
