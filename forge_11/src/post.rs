// Phosphor finish over the whole frame: a bloom from the bright parts
// (quarter-resolution, two box blurs each way, added back bilinearly),
// scanlines and a vignette, all in fixed point in one pass.
use crate::gfx::{Frame, H, W};

const DS: usize = 4;
const SW: usize = W / DS;
const SH: usize = H / DS;
/// Channel level below which nothing blooms.
const THRESHOLD: u32 = 96;

pub struct Post {
    shade: Vec<u16>,
    small: Vec<[u32; 3]>,
    tmp: Vec<[u32; 3]>,
    line_a: Vec<[u32; 3]>,
    line_b: Vec<[u32; 3]>,
    xtab: Vec<(usize, usize, u32)>,
}

impl Post {
    pub fn new() -> Self {
        let mut shade = vec![0u16; W * H];
        for y in 0..H {
            let scan = if y % 3 == 2 { 0.82 } else { 1.0 };
            for x in 0..W {
                let dx = (x as f32 / W as f32 - 0.5) * 2.0;
                let dy = (y as f32 / H as f32 - 0.5) * 2.0;
                let vig = (1.0 - (dx * dx * 0.12 + dy * dy * 0.2)).clamp(0.0, 1.0);
                shade[y * W + x] = (scan * vig * 256.0) as u16;
            }
        }
        Self {
            shade,
            small: vec![[0; 3]; SW * SH],
            tmp: vec![[0; 3]; SW * SH],
            line_a: vec![[0; 3]; W],
            line_b: vec![[0; 3]; W],
            xtab: (0..W).map(|x| upsample_tap(x, SW)).collect(),
        }
    }

    pub fn apply(&mut self, f: &mut Frame, bloom: f32) {
        let k = (bloom.max(0.0) * 256.0) as u32;
        if k > 0 {
            for sy in 0..SH {
                for sx in 0..SW {
                    let mut acc = [0u32; 3];
                    for y in 0..DS {
                        let row = &f.px[(sy * DS + y) * W + sx * DS..][..DS];
                        for px in row {
                            acc[0] += ((px >> 16) & 0xFF).saturating_sub(THRESHOLD);
                            acc[1] += ((px >> 8) & 0xFF).saturating_sub(THRESHOLD);
                            acc[2] += (px & 0xFF).saturating_sub(THRESHOLD);
                        }
                    }
                    let n = (DS * DS) as u32;
                    self.small[sy * SW + sx] = [acc[0] * 2 / n, acc[1] * 2 / n, acc[2] * 2 / n];
                }
            }
            for _ in 0..2 {
                box_blur(&self.small, &mut self.tmp, SW, SH, 1, SW);
                box_blur(&self.tmp, &mut self.small, SH, SW, SW, 1);
            }
        }
        let mut cached_row = usize::MAX;
        for y in 0..H {
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
            let row = &mut f.px[y * W..(y + 1) * W];
            let shade = &self.shade[y * W..(y + 1) * W];
            for x in 0..W {
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
}

/// The frame as packed BGR24 for ffmpeg.
pub fn to_bgr(f: &Frame, out: &mut Vec<u8>) {
    out.clear();
    for p in &f.px {
        out.extend_from_slice(&[*p as u8, (*p >> 8) as u8, (*p >> 16) as u8]);
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
