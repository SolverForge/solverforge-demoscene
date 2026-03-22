// ═══════════════════════════════════════════════════════════════
// SCROLLTEXT -- Greetings from the cosmos
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::palette;

pub const SCROLL_TEXT: &str = "*** MUSICA UNIVERSALIS *** \
     THE PLANETS SING. THE SOLVER LISTENS. *** \
     SERIO: SCORING ENGINE FOR REAL-TIME INCREMENTAL OPTIMIZATION *** \
     WHEN A PLANET MOVES, ONLY ITS ARCS ARE RE-EVALUATED. *** \
     NOT ALL 21 PAIRS. JUST THE 6 THAT CHANGED. *** \
     THAT IS THE SERIO DIFFERENCE. *** \
     PYTHAGORAS HEARD THE MUSIC OF THE SPHERES IN 530 BCE. *** \
     KEPLER WROTE IT DOWN IN HARMONICES MUNDI IN 1619. *** \
     WE OPTIMIZED IT WITH RUST IN 2026. *** \
     ZERO-ERASURE ARCHITECTURE. NO BOX DYN TRAIT. NO HEAP ALLOC IN HOT PATH. *** \
     FULLY MONOMORPHIZED. THE COMPILER KNOWS EVERYTHING. *** \
     SOLVERFORGE -- OPEN SOURCE. TRULY OPEN SOURCE. *** \
     NO SUBSCRIPTIONS. NO SEAT LICENSES. NO ENTERPRISE NEGOTIATIONS. *** \
     GREETS TO ALL OPTIMIZATION HACKERS *** \
     GREETS TO THE RUST DEMOSCENE *** \
     GREETS TO TIMEFOLD AND THE OPTAPLANNER LINEAGE *** \
     BUILT IN ITALY *** PYTHAGORAS APPROVED *** \
     CODED IN RUST. NO JAVASCRIPT WAS HARMED. *** \
     ";

pub struct ScrollText {
    pub scroll_x: f32,
    scroll_speed: f32,
    text_scale: i32,
    pub active: bool,
}

impl ScrollText {
    pub fn new(scale: i32, speed: f32, start_width: usize) -> Self {
        Self {
            scroll_x: start_width as f32,
            scroll_speed: speed,
            text_scale: scale,
            active: true,
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.scroll_x -= self.scroll_speed * dt as f32;
        let total_width = font::text_width(SCROLL_TEXT, self.text_scale);
        if self.scroll_x < -(total_width as f32) {
            self.scroll_x += total_width as f32;
        }
    }

    pub fn render(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        time: f64,
        y_center: i32,
        fade: f32,
    ) {
        if !self.active {
            return;
        }

        let t = time as f32;
        let char_w = 8 * self.text_scale + self.text_scale;
        let chars: Vec<char> = SCROLL_TEXT.chars().collect();
        let num_chars = chars.len();

        let char_w_f = char_w as f32;
        let first_char = ((-self.scroll_x) / char_w_f).floor().max(0.0) as usize;
        let last_char_f = ((width as f32 - self.scroll_x) / char_w_f).ceil();
        let last_char = (last_char_f.max(0.0) as usize + 1).min(num_chars);

        let char_y = y_center - (8 * self.text_scale) / 2;

        for i in first_char..last_char {
            let char_x = self.scroll_x + (i as f32 * char_w_f);
            if char_x + char_w_f < 0.0 || char_x > width as f32 {
                continue;
            }

            let ch = chars[i % num_chars];
            let gradient_t = (i as f32 / 24.0 + t * 0.18) % 1.0;
            let char_color = palette::scroll_gradient(gradient_t);

            let glow_bright = ((i as f32 * 0.15 + t * 1.5).sin() * 0.2 + 0.8).abs();
            let gc = palette::dim(char_color, fade * 0.5 * glow_bright);
            for gdy in -3i32..=3 {
                for gdx in -3i32..=3 {
                    if gdx != 0 || gdy != 0 {
                        let gf = 0.22 / (1.0 + (gdx * gdx + gdy * gdy) as f32 * 0.5) * glow_bright;
                        let gc2 = palette::dim(gc, gf);
                        font::draw_char(
                            buffer,
                            width,
                            height,
                            ch,
                            char_x as i32 + gdx,
                            char_y + gdy,
                            self.text_scale,
                            gc2,
                        );
                    }
                }
            }

            let brightness = ((i as f32 * 0.25 + t * 0.9).sin() * 0.1 + 0.9).abs();
            let c = palette::dim(char_color, fade * brightness);
            font::draw_char(
                buffer,
                width,
                height,
                ch,
                char_x as i32,
                char_y,
                self.text_scale,
                c,
            );
        }
    }
}
