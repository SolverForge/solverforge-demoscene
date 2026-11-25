// The finish over the whole frame, in one threaded pass after a quarter-
// resolution bloom: Amiga raster wobble (each pixel row shifted on a sine),
// a little chromatic fringe, scanlines and a vignette, film grain, and the
// scene's fade. `Look` is what the scene asks of it this frame.
use crate::fx::hash;
use crate::gfx::{Frame, H, W};

const DS: usize = 4;
const SW: usize = W / DS;
const SH: usize = H / DS;
/// Channel level below which nothing blooms.
const THRESHOLD: u32 = 80;

#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub bloom: f32,
    /// Film grain amplitude (0..1, about 0.05 is visible).
    pub grain: f32,
    /// Raster wobble: amplitude in pixels.
    pub wobble: f32,
    /// Chromatic fringe in pixels at the frame edge.
    pub fringe: f32,
    /// 0 black .. 1 the picture.
    pub fade: f32,
    /// Additive white flash.
    pub flash: f32,
    /// Hue of the flash and the shadows' tint (0xRRGGBB).
    pub flash_rgb: u32,
}

impl Default for Look {
    fn default() -> Self {
        Self {
            bloom: 0.8,
            grain: 0.045,
            wobble: 0.0,
            fringe: 1.0,
            fade: 1.0,
            flash: 0.0,
            flash_rgb: 0xffffff,
        }
    }
}

pub struct Post {
    shade: Vec<u16>,
    small: Vec<[u32; 3]>,
    tmp: Vec<[u32; 3]>,
    src: Vec<u32>,
}

impl Post {
    pub fn new() -> Self {
        let mut shade = vec![0u16; W * H];
        for y in 0..H {
            let scan = if y % 3 == 2 { 0.86 } else { 1.0 };
            for x in 0..W {
                let dx = (x as f32 / W as f32 - 0.5) * 2.0;
                let dy = (y as f32 / H as f32 - 0.5) * 2.0;
                let vig = (1.0 - (dx * dx * 0.16 + dy * dy * 0.26)).clamp(0.0, 1.0);
                shade[y * W + x] = (scan * vig * 256.0) as u16;
            }
        }
        Self {
            shade,
            small: vec![[0; 3]; SW * SH],
            tmp: vec![[0; 3]; SW * SH],
            src: vec![0; W * H],
        }
    }

    pub fn apply(&mut self, f: &mut Frame, look: Look, t: f32) {
        let k = (look.bloom.max(0.0) * 256.0) as u32;
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
            for _ in 0..3 {
                box_blur(&self.small, &mut self.tmp, SH, SW, SW, 1);
                box_blur(&self.tmp, &mut self.small, SW, SH, 1, SW);
            }
        }
        self.src.copy_from_slice(&f.px);
        let src = &self.src;
        let small = &self.small;
        let shade = &self.shade;
        let frame_seed = (t * 60.0) as u32;
        let fade = (look.fade.clamp(0.0, 1.0) * 256.0) as u32;
        let (fr, fg, fb) = (
            (look.flash_rgb >> 16) & 255,
            (look.flash_rgb >> 8) & 255,
            look.flash_rgb & 255,
        );
        let flash = (look.flash.clamp(0.0, 1.0) * 256.0) as u32;
        let grain = (look.grain * 255.0) as i32;
        let cores = std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .clamp(1, 16);
        let rows_per = H.div_ceil(cores);
        std::thread::scope(|s| {
            for (band, chunk) in f.px.chunks_mut(rows_per * W).enumerate() {
                s.spawn(move || {
                    for (j, row) in chunk.chunks_mut(W).enumerate() {
                        let y = band * rows_per + j;
                        let shift = if look.wobble != 0.0 {
                            (look.wobble
                                * ((y as f32 * 0.021 + t * 3.1).sin()
                                    + 0.4 * (y as f32 * 0.057 - t * 4.7).sin()))
                                as i32
                        } else {
                            0
                        };
                        let (sy0, sy1, wy) = upsample_tap(y, SH);
                        let srow = &src[y * W..(y + 1) * W];
                        let shrow = &shade[y * W..(y + 1) * W];
                        for (x, px) in row.iter_mut().enumerate() {
                            let edge = (x as f32 / W as f32 - 0.5) * 2.0;
                            let fr_off = (look.fringe * edge * edge.abs()) as i32;
                            let xi = x as i32 - shift;
                            let at = |xx: i32| srow[xx.clamp(0, W as i32 - 1) as usize];
                            let base = at(xi);
                            let mut ch = [
                                (at(xi + fr_off) >> 16) & 255,
                                (base >> 8) & 255,
                                at(xi - fr_off) & 255,
                            ];
                            if k > 0 {
                                let (sx0, sx1, wx) = upsample_tap(x, SW);
                                let (a, b) = (small[sy0 * SW + sx0], small[sy0 * SW + sx1]);
                                let (c, d) = (small[sy1 * SW + sx0], small[sy1 * SW + sx1]);
                                for (i, v) in ch.iter_mut().enumerate() {
                                    let top = a[i] * (256 - wx) + b[i] * wx;
                                    let bot = c[i] * (256 - wx) + d[i] * wx;
                                    let glow = (top * (256 - wy) + bot * wy) >> 16;
                                    *v = (*v + ((glow * k) >> 8)).min(255);
                                }
                            }
                            let s = shrow[x] as u32;
                            let n = if grain > 0 {
                                let h = hash(
                                    (x as u32)
                                        ^ (y as u32) << 11
                                        ^ frame_seed.wrapping_mul(0x9E37_79B9),
                                );
                                ((h & 255) as i32 - 128) * grain / 128
                            } else {
                                0
                            };
                            let mut out = 0u32;
                            for (i, v) in ch.iter().enumerate() {
                                let mut c = ((*v * s) >> 8) as i32 + n;
                                c = c.clamp(0, 255);
                                let mut c = (c as u32 * fade) >> 8;
                                if flash > 0 {
                                    let to = [fr, fg, fb][i];
                                    c = (c + ((to * flash) >> 8)).min(255);
                                }
                                out |= c << (16 - 8 * i);
                            }
                            *px = out;
                        }
                    }
                });
            }
        });
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
