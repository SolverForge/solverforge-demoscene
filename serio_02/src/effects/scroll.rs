// ═══════════════════════════════════════════════════════════════
// SCROLLTEXT -- the Amiga-style greetings scroller for the outro.
// The story itself is told on cards (story.rs).
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::palette;
use crate::palette::Surface;

pub const SCROLL_TEXT: &str = "*** MUSICA UNIVERSALIS *** \
     SEVEN PLANETS. TWENTY-ONE PAIRS. ONE SOLVER. *** \
     WHEN ONE PLANET CHANGES ITS NOTE, ONLY THE 6 PAIRS IT BELONGS TO ARE RE-CHECKED. THE OTHER 15 KEEP THEIR SCORES. *** \
     THAT IS SERIO, THE SCORING ENGINE INSIDE SOLVERFORGE. *** \
     SOLVERFORGE: OPEN SOURCE, CODED IN RUST, BERGAMO MMXXVI. *** \
     GREETINGS TO THE RUST DEMOSCENE AND EVERY OPTIMIZATION HACKER. *** \
     ";

// ── Scrolltext (horizontal Amiga-style, for outro) ────────────

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

    pub fn render(&self, s: &mut Surface, time: f64, y_center: i32, fade: f32) {
        if !self.active {
            return;
        }

        let t = time as f32;
        let char_w = 8 * self.text_scale + self.text_scale;
        let chars: Vec<char> = SCROLL_TEXT.chars().collect();
        let num_chars = chars.len();

        let char_w_f = char_w as f32;
        let first_char = ((-self.scroll_x) / char_w_f).floor().max(0.0) as usize;
        let last_char_f = ((s.w as f32 - self.scroll_x) / char_w_f).ceil();
        let last_char = (last_char_f.max(0.0) as usize + 1).min(num_chars);

        let char_y = y_center - (8 * self.text_scale) / 2;

        for i in first_char..last_char {
            let char_x = self.scroll_x + (i as f32 * char_w_f);
            if char_x + char_w_f < 0.0 || char_x > s.w as f32 {
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
                            s,
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
            font::draw_char(s, ch, char_x as i32, char_y, self.text_scale, c);
        }
    }
}
