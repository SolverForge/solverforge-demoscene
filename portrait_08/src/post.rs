// Phosphor finish: a soft bloom from the bright parts of the frame, faint
// scanlines and a vignette. Runs on the finished frame, cards and logo
// included, so everything on screen sits in the same light.
use crate::gfx::{Frame, H, W};

const DS: usize = 4;
const BW: usize = W / DS;
const BH: usize = H / DS;
const BLOOM: f32 = 0.55;
const THRESHOLD: f32 = 120.0;

pub struct Post {
    small: Vec<[f32; 3]>,
    tmp: Vec<[f32; 3]>,
    /// Vignette and scanlines, as a 0..256 multiplier per pixel.
    vignette: Vec<u16>,
    /// Per screen column: the bloom column to its left and the weight of
    /// the one to its right.
    xs: Vec<(usize, f32)>,
}

impl Post {
    pub fn new() -> Self {
        let mut vignette = vec![0u16; W * H];
        for y in 0..H {
            for x in 0..W {
                let (u, v) = (x as f32 / W as f32 - 0.5, y as f32 / H as f32 - 0.5);
                let d = (u * u * 1.2 + v * v * 1.6).sqrt();
                let scan = if y % 3 == 2 { 0.9 } else { 1.0 };
                vignette[y * W + x] =
                    ((1.0 - 0.55 * (d * 1.25).powi(3)).clamp(0.0, 1.0) * scan * 256.0).round()
                        as u16;
            }
        }
        Self {
            small: vec![[0.0; 3]; BW * BH],
            tmp: vec![[0.0; 3]; BW * BH],
            vignette,
            xs: (0..W)
                .map(|x| {
                    let fx = ((x as f32 + 0.5) / DS as f32 - 0.5).clamp(0.0, (BW - 1) as f32);
                    (fx as usize, fx.fract())
                })
                .collect(),
        }
    }

    pub fn apply(&mut self, f: &mut Frame) {
        // Bright pass, downsampled, a band of block rows per core.
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
        let band = BH.div_ceil(threads);
        let src = &f.px;
        std::thread::scope(|sc| {
            for (b, chunk) in self.small.chunks_mut(band * BW).enumerate() {
                sc.spawn(move || {
                    for (i, cell) in chunk.iter_mut().enumerate() {
                        let (bx, by) = (i % BW, b * band + i / BW);
                        let mut acc = [0.0f32; 3];
                        for y in 0..DS {
                            for x in 0..DS {
                                let p = src[(by * DS + y) * W + bx * DS + x];
                                let c = [(p >> 16) & 255, (p >> 8) & 255, p & 255];
                                for k in 0..3 {
                                    acc[k] += (c[k] as f32 - THRESHOLD).max(0.0);
                                }
                            }
                        }
                        let n = (DS * DS) as f32;
                        *cell = [acc[0] / n, acc[1] / n, acc[2] / n];
                    }
                });
            }
        });
        // Two box blurs each way approximate a gaussian.
        for _ in 0..2 {
            blur(&self.small, &mut self.tmp, BW, BH, 1, 0, 4);
            blur(&self.tmp, &mut self.small, BW, BH, 0, 1, 4);
        }
        // Add back, bilinear, then scanlines and vignette. Each row first
        // interpolates its two bloom rows into one line, then walks it.
        let (small, vignette, xs) = (&self.small, &self.vignette, &self.xs);
        f.par_rows(0, H, |y, row| {
            let fy = ((y as f32 + 0.5) / DS as f32 - 0.5).clamp(0.0, (BH - 1) as f32);
            let (y0, ty) = (fy as usize, fy.fract());
            let y1 = (y0 + 1).min(BH - 1);
            let mut line = [[0.0f32; 3]; BW];
            for (bx, l) in line.iter_mut().enumerate() {
                let (a, c) = (small[y0 * BW + bx], small[y1 * BW + bx]);
                for k in 0..3 {
                    l[k] = (a[k] + (c[k] - a[k]) * ty) * BLOOM;
                }
            }
            let vig = &vignette[y * W..(y + 1) * W];
            // Bloom columns that are dark (most of them): those pixels only
            // take the vignette, in integer arithmetic.
            let lit: Vec<bool> = (0..BW)
                .map(|bx| {
                    let (a, b) = (line[bx], line[(bx + 1).min(BW - 1)]);
                    a.iter().chain(b.iter()).any(|&v| v > 0.25)
                })
                .collect();
            for (x, px) in row.iter_mut().enumerate() {
                let (x0, tx) = xs[x];
                let p = *px;
                let q = vig[x] as u32;
                if !lit[x0] {
                    let ch = |s: u32| ((((p >> s) & 255) * q + 128) >> 8).min(255) << s;
                    *px = ch(16) | ch(8) | ch(0);
                    continue;
                }
                let (a, b) = (line[x0], line[(x0 + 1).min(BW - 1)]);
                let ch = |k: usize, s: u32| {
                    let base = ((p >> s) & 255) as f32;
                    let glow = a[k] + (b[k] - a[k]) * tx;
                    (((base + glow) * q as f32 / 256.0 + 0.5) as u32).min(255) << s
                };
                *px = ch(0, 16) | ch(1, 8) | ch(2, 0);
            }
        });
    }
}

/// Box blur of radius r along (dx, dy), with a running sum per line and
/// the edge pixel repeated past the ends.
fn blur(src: &[[f32; 3]], dst: &mut [[f32; 3]], w: usize, h: usize, dx: usize, dy: usize, r: i32) {
    let n = (2 * r + 1) as f32;
    let (len, lines) = if dx == 1 { (w, h) } else { (h, w) };
    let at = |line: usize, i: i32| {
        let i = i.clamp(0, len as i32 - 1) as usize;
        if dx == 1 {
            line * w + i
        } else {
            i * w + line
        }
    };
    for line in 0..lines {
        let mut acc = [0.0f32; 3];
        for k in -r..=r {
            let p = src[at(line, k)];
            for c in 0..3 {
                acc[c] += p[c];
            }
        }
        for i in 0..len as i32 {
            dst[at(line, i)] = [acc[0] / n, acc[1] / n, acc[2] / n];
            let (add, sub) = (src[at(line, i + r + 1)], src[at(line, i - r)]);
            for c in 0..3 {
                acc[c] += add[c] - sub[c];
            }
        }
    }
    let _ = dy;
}

/// The frame as packed BGR24 for ffmpeg.
pub fn to_bgr(f: &Frame, out: &mut Vec<u8>) {
    out.clear();
    for p in &f.px {
        out.extend_from_slice(&[*p as u8, (*p >> 8) as u8, (*p >> 16) as u8]);
    }
}
