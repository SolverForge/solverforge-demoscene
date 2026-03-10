// ═══════════════════════════════════════════════════════════════
// SCROLLTEXT -- Centered narration that EXPLAINS the demo
//
// Big, readable, front-and-center.
// Tells the viewer what they are seeing and hearing.
// No hiding in corners. The narration IS the demo.
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::palette;
use crate::palette::Surface;

// ── Scene narration passages ──────────────────────────────────
// SHORT. CLEAR. EXPLANATORY. One line every ~3s.

/// Scene 0: Genesis — wonder, reverence
pub const NARRATION_GENESIS: &[&str] = &[
    "Pythagoras heard it first.",
    "Ratios in the ringing of hammers.",
    "Seven worlds. Each singing a tone.",
    "Twenty-one intervals between them.",
];

/// Scene 1: Full eval — weight, burden
pub const NARRATION_FULL_EVAL: &[&str] = &[
    "Each arc is one interval between two planets.",
    "Watch: one planet retuned.",
    "Now every arc must be rechecked.",
    "One... two... three...",
    "All twenty-one. Every single time.",
];

/// Scene 2: Dual panel — recognition, surprise
pub const NARRATION_DUAL: &[&str] = &[
    "Left: check every arc. Right: only the six that changed.",
    "Same planet moved. Same answer.",
    "Watch the left count to twenty-one.",
    "The right stops at six.",
    "Fifteen arcs untouched. Their truth did not change.",
    "Why recompute what you already know?",
    "This is the principle.",
];

/// Scene 3: Convergence — power, momentum
pub const NARRATION_CONVERGENCE: &[&str] = &[
    "Millions of tunings per second.",
    "Each move: retract six arcs.",
    "Recompute six. Leave fifteen alone.",
    "The frequencies drift toward ratio.",
    "Dissonance yields.",
    "",
    "Red becomes gold.",
    "Listen.",
];

/// Scene 4: Solution — transcendence, stillness
pub const NARRATION_SOLUTION: &[&str] = &[
    "Every arc is gold.",
    "Octaves. Fifths. Fourths.",
    "The cosmos computes its own harmony.",
];

/// Scene 5: Outro — quiet pride
pub const NARRATION_OUTRO: &[&str] = &[
    "When one thing changes,",
    "only its relations are re-weighed.",
    "The rest endures.",
    "",
    "This is incremental scoring.",
    "This is SERIO.",
    "SolverForge.",
];

/// Bottom scrolltext for the outro scene
pub const SCROLL_TEXT: &str = "*** MUSICA UNIVERSALIS *** \
     SEVEN WORLDS. TWENTY-ONE INTERVALS. ONE SOLUTION. *** \
     WHEN ONE ENTITY CHANGES, ONLY ITS CONSTRAINTS ARE RE-EVALUATED. \
     THE REST ENDURES. THAT IS SERIO. *** \
     ZERO-ERASURE ARCHITECTURE. FULLY MONOMORPHIZED. CODED IN RUST. *** \
     SOLVERFORGE -- TRULY OPEN SOURCE. BERGAMO MMXXVI. *** \
     ";

// ── Big centered narration renderer ──────────────────────────
// Scale 2, centered horizontally, with dark backing panel.
// This is the PRIMARY text the viewer reads.

/// Render narration as BIG centered text with dark backing.
/// `progress` controls how many lines are visible (0.0 = none, 1.0 = all).
pub fn render_narration_centered(
    s: &mut Surface,
    lines: &[&str],
    center_x: i32,
    y_start: i32,
    progress: f32,
    fade: f32,
) {
    let scale = 2;
    let line_height = 22i32; // 8*2 + 6px spacing
    let total_lines = lines.len();
    let visible = ((total_lines as f32 * progress) as usize).min(total_lines);

    if visible == 0 {
        return;
    }

    // Dark backing panel behind all visible text
    let max_text_w = lines
        .iter()
        .map(|l| font::text_width(l, scale))
        .max()
        .unwrap_or(0);
    let panel_w = max_text_w + 40;
    let panel_h = visible as i32 * line_height + 16;
    let panel_x = center_x - panel_w / 2;
    let panel_y = y_start - 8;

    for py in panel_y..(panel_y + panel_h) {
        for px in panel_x..(panel_x + panel_w) {
            if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                let idx = py as usize * s.w + px as usize;
                s.buf[idx] = palette::lerp_color(s.buf[idx], palette::NEAR_BLACK, 0.75 * fade);
            }
        }
    }

    for (i, &line) in lines.iter().enumerate().take(visible) {
        if line.is_empty() {
            continue;
        }

        let line_progress = if total_lines > 1 {
            let line_start = i as f32 / total_lines as f32;
            ((progress - line_start) / (1.0 / total_lines as f32)).clamp(0.0, 1.0)
        } else {
            progress
        };

        let alpha = line_progress * fade;
        if alpha < 0.01 {
            continue;
        }

        let ly = y_start + i as i32 * line_height;
        if ly < 0 || ly >= s.h as i32 {
            continue;
        }

        // Glow
        let glow_col = palette::dim(palette::EMERALD_800, alpha * 0.4);
        font::draw_text_centered(s, line, center_x, ly, scale, glow_col);

        // Main text
        let col = palette::dim(palette::CHROME, alpha * 0.95);
        font::draw_text_centered(s, line, center_x, ly, scale, col);
    }
}

/// Render small narration (scale 1) at a specific position — used for outro/credits only.
pub fn render_narration(
    s: &mut Surface,
    lines: &[&str],
    x: i32,
    y: i32,
    progress: f32,
    fade: f32,
) {
    let total_lines = lines.len();
    let visible = ((total_lines as f32 * progress) as usize).min(total_lines);

    for (i, &line) in lines.iter().enumerate().take(visible) {
        let line_progress = if total_lines > 1 {
            let line_start = i as f32 / total_lines as f32;
            ((progress - line_start) / (1.0 / total_lines as f32)).clamp(0.0, 1.0)
        } else {
            progress
        };

        let alpha = line_progress * fade;
        if alpha < 0.01 {
            continue;
        }

        let ly = y + i as i32 * 14;
        if ly < 0 || ly >= s.h as i32 {
            continue;
        }

        let glow_col = palette::dim(palette::EMERALD_800, alpha * 0.3);
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                if dx != 0 || dy != 0 {
                    font::draw_text(s, line, x + dx, ly + dy, 1, glow_col);
                }
            }
        }

        let col = palette::dim(palette::CHROME, alpha * 0.9);
        font::draw_text(s, line, x, ly, 1, col);
    }
}

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

    pub fn render(
        &self,
        s: &mut Surface,
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
            font::draw_char(
                s,
                ch,
                char_x as i32,
                char_y,
                self.text_scale,
                c,
            );
        }
    }
}
