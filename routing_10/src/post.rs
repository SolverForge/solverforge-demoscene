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
    vignette: Vec<f32>,
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
        Self {
            small: vec![[0.0; 3]; BW * BH],
            tmp: vec![[0.0; 3]; BW * BH],
            vignette,
        }
    }

    pub fn apply(&mut self, f: &mut Frame) {
        // Bright pass, downsampled.
        for by in 0..BH {
            for bx in 0..BW {
                let mut acc = [0.0f32; 3];
                for y in 0..DS {
                    for x in 0..DS {
                        let p = f.px[(by * DS + y) * W + bx * DS + x];
                        let c = [(p >> 16) & 255, (p >> 8) & 255, p & 255];
                        for k in 0..3 {
                            acc[k] += (c[k] as f32 - THRESHOLD).max(0.0);
                        }
                    }
                }
                let n = (DS * DS) as f32;
                self.small[by * BW + bx] = [acc[0] / n, acc[1] / n, acc[2] / n];
            }
        }
        // Two box blurs each way approximate a gaussian.
        for _ in 0..2 {
            blur(&self.small, &mut self.tmp, BW, BH, 1, 0, 4);
            blur(&self.tmp, &mut self.small, BW, BH, 0, 1, 4);
        }
        // Add back, bilinear, then scanlines and vignette.
        for y in 0..H {
            let fy = ((y as f32 + 0.5) / DS as f32 - 0.5).clamp(0.0, (BH - 1) as f32);
            let (y0, ty) = (fy as usize, fy.fract());
            let y1 = (y0 + 1).min(BH - 1);
            for x in 0..W {
                let fx = ((x as f32 + 0.5) / DS as f32 - 0.5).clamp(0.0, (BW - 1) as f32);
                let (x0, tx) = (fx as usize, fx.fract());
                let x1 = (x0 + 1).min(BW - 1);
                let s = |xx: usize, yy: usize| self.small[yy * BW + xx];
                let (a, b, c, d) = (s(x0, y0), s(x1, y0), s(x0, y1), s(x1, y1));
                let i = y * W + x;
                let p = f.px[i];
                let v = self.vignette[i];
                let mut out = 0u32;
                for k in 0..3 {
                    let top = a[k] + (b[k] - a[k]) * tx;
                    let bot = c[k] + (d[k] - c[k]) * tx;
                    let glow = (top + (bot - top) * ty) * BLOOM;
                    let base = ((p >> (16 - 8 * k)) & 255) as f32;
                    let val = ((base + glow) * v).round().clamp(0.0, 255.0) as u32;
                    out |= val << (16 - 8 * k);
                }
                f.px[i] = out;
            }
        }
    }
}

fn blur(src: &[[f32; 3]], dst: &mut [[f32; 3]], w: usize, h: usize, dx: usize, dy: usize, r: i32) {
    let n = (2 * r + 1) as f32;
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 3];
            for k in -r..=r {
                let sx = (x as i32 + k * dx as i32).clamp(0, w as i32 - 1) as usize;
                let sy = (y as i32 + k * dy as i32).clamp(0, h as i32 - 1) as usize;
                let p = src[sy * w + sx];
                acc[0] += p[0];
                acc[1] += p[1];
                acc[2] += p[2];
            }
            dst[y * w + x] = [acc[0] / n, acc[1] / n, acc[2] / n];
        }
    }
}

/// The frame as packed BGR24 for ffmpeg.
pub fn to_bgr(f: &Frame, out: &mut Vec<u8>) {
    out.clear();
    for p in &f.px {
        out.extend_from_slice(&[*p as u8, (*p >> 8) as u8, (*p >> 16) as u8]);
    }
}
