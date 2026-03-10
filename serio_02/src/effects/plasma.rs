// ═══════════════════════════════════════════════════════════════
// PLASMA EFFECT -- Classic Amiga sine-wave plasma
// The original demoscene effect. Mathematically pure.
// ═══════════════════════════════════════════════════════════════

use crate::palette::{self, Surface};

pub struct Plasma {
    /// Precomputed sine LUT for speed
    sin_lut: Vec<f32>,
}

impl Plasma {
    pub fn new() -> Self {
        let lut_size = 4096;
        let sin_lut: Vec<f32> = (0..lut_size)
            .map(|i| (i as f32 * std::f32::consts::TAU / lut_size as f32).sin())
            .collect();
        Self { sin_lut }
    }

    #[inline(always)]
    fn sin_fast(&self, v: f32) -> f32 {
        let lut_size = self.sin_lut.len();
        let idx = ((v * lut_size as f32 / std::f32::consts::TAU) as i64).rem_euclid(lut_size as i64)
            as usize;
        self.sin_lut[idx]
    }

    /// Render plasma with alpha-blend overlay on existing buffer
    pub fn render_overlay(
        &self,
        s: &mut Surface,
        time: f64,
        alpha: f32,
    ) {
        let t = time as f32;
        let w = s.w as f32;
        let h = s.h as f32;

        for y in (0..s.h).step_by(2) {
            for x in (0..s.w).step_by(2) {
                let xf = x as f32 / w;
                let yf = y as f32 / h;

                let v1 = self.sin_fast(xf * 4.0 * std::f32::consts::TAU + t * 0.8);
                let v2 = self.sin_fast(yf * 3.5 * std::f32::consts::TAU + t * 1.2);
                let dx = xf - 0.5;
                let dy = yf - 0.5;
                let dist = (dx * dx + dy * dy).sqrt();
                let v3 = self.sin_fast(dist * 8.0 - t * 1.5);

                let combined = (v1 + v2 + v3) / 3.0;
                let normalized = (combined + 1.0) * 0.5;
                let plasma_col = palette::plasma_color(normalized);

                // 2x2 block
                for dy2 in 0..2 {
                    for dx2 in 0..2 {
                        let px = x + dx2;
                        let py = y + dy2;
                        if px < s.w && py < s.h {
                            let existing = s.buf[py * s.w + px];
                            s.buf[py * s.w + px] =
                                palette::lerp_color(existing, plasma_col, alpha);
                        }
                    }
                }
            }
        }
    }
}
