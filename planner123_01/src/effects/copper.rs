// ═══════════════════════════════════════════════════════════════
// COPPER BARS -- Classic Amiga hardware raster trick
// On real hardware: copper list re-programs palette per scanline.
// ═══════════════════════════════════════════════════════════════

use crate::palette::{self, COPPER_BARS};

pub struct CopperBars {
    /// Number of bar groups
    bar_count: usize,
    /// Height of each bar group in pixels
    bar_height: usize,
    #[allow(dead_code)]
    bar_gap: usize,
}

impl CopperBars {
    pub fn new() -> Self {
        Self {
            bar_count: 5,
            bar_height: 32,
            bar_gap: 20,
        }
    }

    /// Render copper bars onto the buffer.
    /// bars_y: array of Y positions (center) for each bar
    /// alpha: overall opacity
    pub fn render(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        time: f64,
        alpha: f32,
        mode: CopperMode,
    ) {
        match mode {
            CopperMode::Classic => self.render_classic(buffer, width, height, time, alpha),
            CopperMode::Bands => self.render_full_bands(buffer, width, height, time, alpha),
            CopperMode::Scanlines => self.render_scanlines(buffer, width, height, time, alpha),
        }
    }

    fn render_classic(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        time: f64,
        alpha: f32,
    ) {
        let t = time as f32;
        let phase_step = std::f32::consts::TAU / self.bar_count as f32;

        for bar in 0..self.bar_count {
            // Each bar oscillates vertically with unique phase
            let phase = bar as f32 * phase_step;
            let base_y = height as f32 / 2.0
                + (t * 1.2 + phase).sin() * height as f32 * 0.3
                + (t * 0.7 + phase * 0.5).cos() * height as f32 * 0.1;

            let bar_start = (base_y - self.bar_height as f32 / 2.0) as i32;

            // Color offset per bar
            let color_offset = (bar * 3 + (time * 2.0) as usize) % COPPER_BARS.len();

            for row in 0..self.bar_height {
                let y = bar_start + row as i32;
                if y < 0 || y >= height as i32 {
                    continue;
                }

                // Copper gradient: bell curve across bar height
                let gradient_t = row as f32 / self.bar_height as f32;
                let bell = (gradient_t * std::f32::consts::PI).sin(); // 0 at edges, 1 at center

                // Sample from copper palette
                let palette_idx =
                    ((gradient_t * COPPER_BARS.len() as f32) as usize).min(COPPER_BARS.len() - 1);
                let bar_color = COPPER_BARS[(palette_idx + color_offset) % COPPER_BARS.len()];
                let final_color = palette::dim(bar_color, bell * alpha);

                // Full-width scanline
                for x in 0..width {
                    buffer[y as usize * width + x] =
                        palette::add_color(buffer[y as usize * width + x], final_color);
                }
            }
        }
    }

    /// Full-screen bands filling the entire display
    fn render_full_bands(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        time: f64,
        alpha: f32,
    ) {
        let t = time as f32;
        let band_height = height / COPPER_BARS.len();

        for y in 0..height {
            // Scroll offset: bars slide upward
            let scroll = (t * 40.0) as usize;
            let palette_idx = ((y + scroll) / (band_height.max(1))) % COPPER_BARS.len();

            // Smooth gradient between bands
            let within_band = ((y + scroll) % band_height.max(1)) as f32 / band_height as f32;
            let next_idx = (palette_idx + 1) % COPPER_BARS.len();
            let band_color =
                palette::lerp_color(COPPER_BARS[palette_idx], COPPER_BARS[next_idx], within_band);

            // Add sinusoidal horizontal distortion per scanline (Amiga copper trick)
            let distort = (t * 3.0 + y as f32 * 0.05).sin() * 0.1;
            let final_color = palette::dim(band_color, alpha * (0.85 + distort));

            for x in 0..width {
                buffer[y * width + x] =
                    palette::lerp_color(buffer[y * width + x], final_color, alpha);
            }
        }
    }

    /// Scanline overlay -- dark lines every other row (CRT-style)
    fn render_scanlines(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        _time: f64,
        alpha: f32,
    ) {
        for y in (0..height).step_by(2) {
            // Darken every other scanline
            for x in 0..width {
                let idx = y * width + x;
                buffer[idx] = palette::dim(buffer[idx], 1.0 - alpha * 0.4);
            }
        }
    }
}

pub enum CopperMode {
    Classic,   // Flying bars
    Bands,     // Full-screen color bands
    Scanlines, // CRT scanline darkening
}
