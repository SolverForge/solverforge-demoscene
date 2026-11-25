// Phosphor finish: a soft bloom from the bright parts of the frame, faint
// scanlines and a vignette. Runs on the demo's own scenes, cards and logo
// included, so everything there sits in the same light. The Desktop takes
// stay as recorded: the finish fades out as a take comes in, so no scanline
// or vignette ever sits on the plugin's text.
use crate::gfx::{par_rows, Frame, H, W};

const DS: usize = 4;
const BW: usize = W / DS;
const BH: usize = H / DS;
const BLOOM: f32 = 0.55;
const THRESHOLD: f32 = 120.0;

pub struct Post {
    small: Vec<[f32; 3]>,
    tmp: Vec<[f32; 3]>,
    vignette: Vec<f32>,
    /// Per frame column: the two bloom columns it reads and the weight.
    cols: Vec<(usize, usize, f32)>,
}

impl Post {
    pub fn new() -> Self {
        let mut vignette = vec![0.0; W * H];
        for y in 0..H {
            for x in 0..W {
                let (u, v) = (x as f32 / W as f32 - 0.5, y as f32 / H as f32 - 0.5);
                let d = (u * u * 1.2 + v * v * 1.6).sqrt();
                let scan = if y % 3 == 2 { 0.9 } else { 1.0 };
                vignette[y * W + x] = (1.0 - 0.55 * (d * 1.25).powi(3)).clamp(0.0, 1.0) * scan;
            }
        }
        let cols = (0..W)
            .map(|x| {
                let fx = ((x as f32 + 0.5) / DS as f32 - 0.5).clamp(0.0, (BW - 1) as f32);
                let x0 = fx as usize;
                (x0, (x0 + 1).min(BW - 1), fx.fract())
            })
            .collect();
        Self {
            small: vec![[0.0; 3]; BW * BH],
            tmp: vec![[0.0; 3]; BW * BH],
            vignette,
            cols,
        }
    }

    /// The finish at strength `k`: 0 leaves the frame as it is, 1 applies it
    /// whole, between the two it mixes.
    pub fn apply(&mut self, f: &mut Frame, k: f32) {
        if k <= 0.0 {
            return;
        }
        let keep = (k < 1.0).then(|| f.px.clone());
        self.full(f);
        if let Some(orig) = keep {
            par_rows(&mut f.px, |y, row| {
                for (p, o) in row.iter_mut().zip(&orig[y * W..(y + 1) * W]) {
                    *p = crate::gfx::lerp_px(*o, *p, k);
                }
            });
        }
    }

    fn full(&mut self, f: &mut Frame) {
        // Bright pass, downsampled: one band of small rows per core.
        let cores = std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .clamp(1, 32);
        let per = BH.div_ceil(cores);
        let src = &f.px;
        std::thread::scope(|s| {
            for (band, chunk) in self.small.chunks_mut(per * BW).enumerate() {
                s.spawn(move || {
                    for (j, out) in chunk.chunks_mut(BW).enumerate() {
                        let by = band * per + j;
                        for (bx, cell) in out.iter_mut().enumerate() {
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
                    }
                });
            }
        });
        // Two box blurs each way approximate a gaussian.
        for _ in 0..2 {
            blur(&self.small, &mut self.tmp, BW, BH, 1, 0, 4);
            blur(&self.tmp, &mut self.small, BW, BH, 0, 1, 4);
        }
        // Add back, bilinear, then scanlines and vignette.
        let small = &self.small;
        let vignette = &self.vignette;
        let cols = &self.cols;
        par_rows(&mut f.px, |y, row| {
            let fy = ((y as f32 + 0.5) / DS as f32 - 0.5).clamp(0.0, (BH - 1) as f32);
            let (y0, ty) = (fy as usize, fy.fract());
            let y1 = (y0 + 1).min(BH - 1);
            let (top_row, bot_row) = (
                &small[y0 * BW..(y0 + 1) * BW],
                &small[y1 * BW..(y1 + 1) * BW],
            );
            let vig = &vignette[y * W..(y + 1) * W];
            for ((px, &(x0, x1, tx)), &v) in row.iter_mut().zip(cols).zip(vig) {
                let (a, b, c, d) = (top_row[x0], top_row[x1], bot_row[x0], bot_row[x1]);
                let p = *px;
                let mut out = 0u32;
                for k in 0..3 {
                    let top = a[k] + (b[k] - a[k]) * tx;
                    let bot = c[k] + (d[k] - c[k]) * tx;
                    let glow = (top + (bot - top) * ty) * BLOOM;
                    let base = ((p >> (16 - 8 * k)) & 255) as f32;
                    // (x + 0.5) as u32 is x.round() for x >= 0, saturated at 0.
                    let val = (((base + glow) * v + 0.5) as u32).min(255);
                    out |= val << (16 - 8 * k);
                }
                *px = out;
            }
        });
    }
}

fn blur(src: &[[f32; 3]], dst: &mut [[f32; 3]], w: usize, h: usize, dx: usize, dy: usize, r: i32) {
    let n = (2 * r + 1) as f32;
    // Each output row reads only `src`, so the rows split into one band per core.
    let cores = std::thread::available_parallelism()
        .map_or(1, |c| c.get())
        .clamp(1, 32);
    let per = h.div_ceil(cores);
    std::thread::scope(|s| {
        for (band, chunk) in dst.chunks_mut(per * w).enumerate() {
            s.spawn(move || {
                for (j, row) in chunk.chunks_mut(w).enumerate() {
                    let y = band * per + j;
                    for (x, out) in row.iter_mut().enumerate() {
                        let mut acc = [0.0f32; 3];
                        for k in -r..=r {
                            let sx = (x as i32 + k * dx as i32).clamp(0, w as i32 - 1) as usize;
                            let sy = (y as i32 + k * dy as i32).clamp(0, h as i32 - 1) as usize;
                            let p = src[sy * w + sx];
                            acc[0] += p[0];
                            acc[1] += p[1];
                            acc[2] += p[2];
                        }
                        *out = [acc[0] / n, acc[1] / n, acc[2] / n];
                    }
                }
            });
        }
    });
}

/// The frame as packed BGR24 for ffmpeg.
pub fn to_bgr(f: &Frame, out: &mut Vec<u8>) {
    out.clear();
    for p in &f.px {
        out.extend_from_slice(&[*p as u8, (*p >> 8) as u8, (*p >> 16) as u8]);
    }
}
