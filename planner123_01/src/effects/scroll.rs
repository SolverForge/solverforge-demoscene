// ═══════════════════════════════════════════════════════════════
// GRADIENT SCROLLTEXT -- The quintessential demoscene greeting
// "GREETS TO ALL THE CODERS IN THE SCENE"
// Full horizontal scroll + per-character gradient color cycling.
// Colors: brand emerald → amber → rust → amber → emerald (loop).
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::palette;

/// The holy scripture of the greetings scroll.
pub const SCROLL_TEXT: &str = "*** PLANNER123 *** YOUR WEEK, OPTIMIZED *** \
     PERSONAL TASK SCHEDULING POWERED BY CONSTRAINT OPTIMIZATION *** \
     NATIVE RUST. ONE BINARY. ZERO GC PAUSES *** \
     8 CONSTRAINTS. 6 HARD. 2 SOFT. NO AI HALLUCINATIONS. NO GUESSING. MATH. *** \
     THIS IS NOT A SCHEDULER. THIS IS A WEAPON. *** \
     DEADLINES, DEPENDENCIES, PRIORITIES. \
     THE ENGINE FINDS THE BEST ARRANGEMENT - OR TELLS YOU IT CAN'T. *** \
     MULTI-PROJECT MANAGEMENT WITH DEPENDENCY ORDERING *** \
     PLAN, GANTT, AND CALENDAR VIEWS *** FULLY OFFLINE - LINUX, MACOS, WINDOWS *** \
     SOLVERFORGE - OPEN SOURCE. TRULY OPEN SOURCE. *** \
     NO SUBSCRIPTIONS. NO SEAT LICENSES. NO ENTERPRISE NEGOTIATIONS. *** \
     SHAREWARE. THE OLDEST DEAL IN SOFTWARE: A DEVELOPER AND A USER. *** \
     GREETS TO ALL OPTIMIZATION HACKERS *** \
     HELLO FROM THE RUST DEMOSCENE *** \
     BUILT IN ITALY *** \
     ";

pub struct ScrollText {
    /// Horizontal scroll position in pixels (decreasing = scrolling left)
    pub scroll_x: f32,
    scroll_speed: f32,
    text_scale: i32,
    pub active: bool,
}

impl ScrollText {
    pub fn new(scale: i32, speed: f32, start_width: usize) -> Self {
        Self {
            // Start off-screen to the RIGHT so text enters from the right
            scroll_x: start_width as f32,
            scroll_speed: speed,
            text_scale: scale,
            active: true,
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.scroll_x -= self.scroll_speed * dt as f32;

        // Loop: when text has scrolled fully off screen, reset
        let total_width = font::text_width(SCROLL_TEXT, self.text_scale);
        if self.scroll_x < -(total_width as f32) {
            self.scroll_x += total_width as f32;
        }
    }

    /// Render the scrolltext with per-character gradient color cycling.
    /// Colors slide through the brand gradient (emerald → amber → rust → amber → emerald)
    /// as text scrolls, giving a smooth chromatic wave without any vertical bounce.
    /// y_center: vertical center position for the text baseline
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

        // Compute which chars are visible on screen.
        // char_x for char i = scroll_x + i * char_w
        // Visible when: 0 <= char_x <= width
        let char_w_f = char_w as f32;
        let first_char = ((-self.scroll_x) / char_w_f).floor().max(0.0) as usize;
        let last_char_f = ((width as f32 - self.scroll_x) / char_w_f).ceil();
        let last_char = (last_char_f.max(0.0) as usize + 1).min(num_chars);

        // Text sits flat at y_center
        let char_y = y_center - (8 * self.text_scale) / 2;

        for i in first_char..last_char {
            let char_x = self.scroll_x + (i as f32 * char_w_f);

            let width_f = width as f32;
            let char_right = char_x + char_w_f;
            if char_right < 0.0 || char_x > width_f {
                continue;
            }

            let ch = chars[i % num_chars];

            // Per-character gradient position: spread full cycle across ~24 chars,
            // slide over time so the gradient drifts through the text.
            let gradient_t = (i as f32 / 24.0 + t * 0.18) % 1.0;
            let char_color = palette::scroll_gradient(gradient_t);

            // Glow pass — same gradient color, dimmed
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

            // Sharp character — full gradient color with subtle brightness pulse
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

    /// Render a simpler static-position line of text for titles
    pub fn render_title(
        buffer: &mut [u32],
        width: usize,
        height: usize,
        text: &str,
        y: i32,
        scale: i32,
        color: u32,
        glow_color: u32,
        fade: f32,
        time: f64,
    ) {
        let t = time as f32;
        // Subtle vertical pulse
        let pulse_y = ((t * 1.5).sin() * 3.0) as i32;
        let glow_bright = (t * 2.0).sin() * 0.2 + 0.8;
        let char_w = 8 * scale + scale;
        let text_w = text.len() as i32 * char_w;
        let x = (width as i32 - text_w) / 2;

        let gc = palette::dim(glow_color, fade * 0.7 * glow_bright);
        font::draw_text_glow(
            buffer,
            width,
            height,
            text,
            x,
            y + pulse_y,
            scale,
            palette::dim(color, fade),
            gc,
        );
    }
}
