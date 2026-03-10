// ═══════════════════════════════════════════════════════════════
// PLASMA EFFECT -- Classic Amiga sine-wave plasma
// The original demoscene effect. Mathematically pure.
// ═══════════════════════════════════════════════════════════════

use crate::palette;

pub struct Plasma {
    // Precomputed sine LUT for speed
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

    pub fn render(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        time: f64,
        fade: f32,
        alt_palette: bool,
    ) {
        let t = time as f32;
        let w = width as f32;
        let h = height as f32;

        // Plasma parameters -- chosen for visual richness
        let freq1 = 3.2;
        let freq2 = 2.7;
        let freq3 = 4.1;
        let freq4 = 1.9;

        for y in 0..height {
            let yf = y as f32 / h;
            let vy = self.sin_fast(yf * freq2 * std::f32::consts::TAU + t * 1.3)
                + self.sin_fast(yf * freq4 * std::f32::consts::TAU + t * 0.7);

            for x in 0..width {
                let xf = x as f32 / w;

                // Classic multi-sine plasma formula
                let v1 = self.sin_fast(xf * freq1 * std::f32::consts::TAU + t * 1.1);
                let v2 = self.sin_fast(yf * freq2 * std::f32::consts::TAU + t * 0.9);
                let v3 = self.sin_fast((xf + yf) * freq3 * std::f32::consts::TAU * 0.5 + t * 1.4);
                // Circular ripple from center
                let dx = xf - 0.5;
                let dy = yf - 0.5;
                let dist = (dx * dx + dy * dy).sqrt();
                let v4 = self.sin_fast(dist * freq4 * std::f32::consts::TAU * 2.0 - t * 2.0);

                // Second ripple from off-center point
                let dx2 = xf - (0.3 + (t * 0.2).sin() * 0.15);
                let dy2 = yf - (0.7 + (t * 0.15).cos() * 0.12);
                let dist2 = (dx2 * dx2 + dy2 * dy2).sqrt();
                let v5 = self.sin_fast(dist2 * 18.0 + t * 1.8 + vy);

                // Combine and normalize to [0, 1]
                let combined = (v1 + v2 + v3 + v4 + v5) / 5.0;
                let normalized = (combined + 1.0) * 0.5;

                let color = if alt_palette {
                    palette::plasma_color_alt(normalized)
                } else {
                    palette::plasma_color(normalized)
                };

                buffer[y * width + x] = palette::fade(color, fade);
            }
        }
    }

    // Render plasma with alpha-blend overlay on existing buffer
    pub fn render_overlay(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        time: f64,
        alpha: f32,
    ) {
        let t = time as f32;
        let w = width as f32;
        let h = height as f32;

        for y in (0..height).step_by(2) {
            for x in (0..width).step_by(2) {
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
                        if px < width && py < height {
                            let existing = buffer[py * width + px];
                            buffer[py * width + px] =
                                palette::lerp_color(existing, plasma_col, alpha);
                        }
                    }
                }
            }
        }
    }
}
