// The pixel frame and the few primitives every scene draws with: soft
// discs and lines, rectangles, and text in the series' 8x8 font at any
// integer scale. Colours are 0x00RRGGBB, as minifb and demofx expect.
use demofx::cell::lerp_rgb;
use demofx::font::{mask, Mask};
use demofx::{Cell, CellSink, Clip, Pixels, CELL_H, CELL_W};

pub const W: usize = 1080;
pub const H: usize = 1920;

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
        self.px[i] = over(self.px[i], rgb, a);
    }

    /// Run `paint(y, row)` over rows y0..y1 on every core, in bands.
    pub fn par_rows(&mut self, y0: usize, y1: usize, paint: impl Fn(usize, &mut [u32]) + Sync) {
        let (y0, y1) = (y0.min(H), y1.min(H));
        if y0 >= y1 {
            return;
        }
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
        let rows = (y1 - y0).div_ceil(threads).max(1);
        let paint = &paint;
        std::thread::scope(|sc| {
            for (band, chunk) in self.px[y0 * W..y1 * W].chunks_mut(rows * W).enumerate() {
                sc.spawn(move || {
                    for (r, row) in chunk.chunks_mut(W).enumerate() {
                        paint(y0 + band * rows + r, row);
                    }
                });
            }
        });
    }

    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
        let (x0, x1) = (x0.max(0) as usize, x1.min(W as i32).max(0) as usize);
        if a <= 0.0 || x0 >= x1 {
            return;
        }
        for y in y0.max(0)..y1.min(H as i32) {
            let row = &mut self.px[y as usize * W + x0..y as usize * W + x1];
            if a >= 1.0 {
                row.fill(rgb);
            } else {
                for p in row {
                    *p = over(*p, rgb, a);
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

    /// Filled polygon (even-odd), with a soft half-pixel edge.
    pub fn poly_fill(&mut self, pts: &[(f32, f32)], rgb: u32, a: f32) {
        if pts.len() < 3 || a <= 0.0 {
            return;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for p in pts {
            x0 = x0.min(p.0);
            y0 = y0.min(p.1);
            x1 = x1.max(p.0);
            y1 = y1.max(p.1);
        }
        for y in y0.max(0.0) as i32..=(y1.min(H as f32 - 1.0)) as i32 {
            let yf = y as f32 + 0.5;
            let mut xs: Vec<f32> = Vec::new();
            for i in 0..pts.len() {
                let (p, q) = (pts[i], pts[(i + 1) % pts.len()]);
                if (p.1 > yf) != (q.1 > yf) {
                    xs.push(p.0 + (yf - p.1) / (q.1 - p.1) * (q.0 - p.0));
                }
            }
            xs.sort_by(f32::total_cmp);
            for pair in xs.chunks(2) {
                if let [l, r] = pair {
                    for x in l.max(x0).floor() as i32..=r.min(x1).ceil() as i32 {
                        let xf = x as f32 + 0.5;
                        let cov = ((xf - l + 0.5).min(r - xf + 0.5)).clamp(0.0, 1.0);
                        self.blend(x, y, rgb, a * cov);
                    }
                }
            }
        }
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
        if k <= 0.0 || r <= 0.0 {
            return;
        }
        let (x0, x1) = (
            ((cx - r).max(0.0)) as usize,
            ((cx + r + 1.0).max(0.0) as usize).min(W),
        );
        let (y0, y1) = (
            ((cy - r).max(0.0)) as usize,
            ((cy + r + 1.0).max(0.0) as usize).min(H),
        );
        if x0 >= x1 {
            return;
        }
        let inv = 1.0 / r;
        let row = |y: usize, line: &mut [u32]| {
            let dy = (y as f32 - cy) * inv;
            if dy.abs() >= 1.0 {
                return;
            }
            for (x, p) in line.iter_mut().enumerate().take(x1).skip(x0) {
                let dx = (x as f32 - cx) * inv;
                let d2 = dx * dx + dy * dy;
                if d2 < 1.0 {
                    *p = light(*p, rgb, k * falloff(d2));
                }
            }
        };
        // Big glows are worth spreading over the cores.
        if r > 120.0 {
            self.par_rows(y0, y1, row);
        } else {
            for y in y0..y1 {
                row(y, &mut self.px[y * W..(y + 1) * W]);
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

/// The glow's (1 - d)^2 falloff, looked up by d^2 (0..1).
#[inline]
fn falloff(d2: f32) -> f32 {
    static LUT: std::sync::OnceLock<Vec<f32>> = std::sync::OnceLock::new();
    let lut = LUT.get_or_init(|| {
        (0..=4096)
            .map(|i| {
                let f = 1.0 - (i as f32 / 4096.0).sqrt();
                f * f
            })
            .collect()
    });
    lut[(d2 * 4096.0) as usize]
}

/// `rgb` over `dst` at opacity `a` (0..1), rounded to the nearest level.
#[inline]
pub fn over(dst: u32, rgb: u32, a: f32) -> u32 {
    if a >= 1.0 {
        return rgb;
    }
    let ch = |s: u32| {
        let (d, c) = (((dst >> s) & 255) as f32, ((rgb >> s) & 255) as f32);
        ((d + (c - d) * a + 0.5) as u32).min(255) << s
    };
    ch(16) | ch(8) | ch(0)
}

/// Additive light: `rgb` scaled by `k`, added to `dst`, saturating.
#[inline]
pub fn light(dst: u32, rgb: u32, k: f32) -> u32 {
    let ch = |s: u32| {
        let (d, c) = (((dst >> s) & 255) as f32, ((rgb >> s) & 255) as f32);
        ((d + c * k + 0.5) as u32).min(255) << s
    };
    ch(16) | ch(8) | ch(0)
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
