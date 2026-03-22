// ═══════════════════════════════════════════════════════════════
// DUAL PANEL -- The educational heart of SERIO_01
//
// Splits the screen vertically at x=640.
// LEFT panel:  "CLASSICAL" -- all pairwise arcs flash on each move.
// RIGHT panel: "SERIO"     -- only the affected planet's arcs update.
//
// Same solver moves. Same score convergence. Different visual load.
// The audience sees the locality of incremental evaluation.
// ═══════════════════════════════════════════════════════════════

use crate::effects::orrery_render::{self, OrreryRenderOpts, TrailBuffer};
use crate::effects::score_display;
use crate::font;
use crate::orrery::model::{Orrery, PLANET_COUNT};
use crate::palette;

pub struct DualPanel {
    /// Separate trail buffers for each panel
    pub trail_left: TrailBuffer,
    pub trail_right: TrailBuffer,

    /// Flash state for the classical (left) panel
    flash_timer: f32,
    flash_intensity: f32,

    /// Current highlighted pairs for SERIO panel
    highlighted_pairs: Vec<(usize, usize)>,
    highlight_timer: f32,

    /// Evaluation counters for display
    pub classical_evals: usize,
    pub serio_evals: usize,
}

const DIVIDER_X: usize = 640;

const LEFT_CX: f64 = 320.0;
const RIGHT_CX: f64 = 960.0;
const PANEL_CY: f64 = 360.0;

impl DualPanel {
    pub fn new() -> Self {
        DualPanel {
            trail_left: TrailBuffer::new(1280, 720),
            trail_right: TrailBuffer::new(1280, 720),
            flash_timer: 0.0,
            flash_intensity: 0.0,
            highlighted_pairs: Vec::new(),
            highlight_timer: 0.0,
            classical_evals: 0,
            serio_evals: 0,
        }
    }

    /// Call when a new solver move happens. Updates flash/highlight state.
    pub fn on_move(&mut self, planet_idx: usize) {
        let total_pairs = PLANET_COUNT * (PLANET_COUNT - 1) / 2; // = 21

        // Classical panel: ALL arcs flash
        self.flash_timer = 0.3;
        self.flash_intensity = 1.0;
        self.classical_evals = total_pairs;

        // SERIO panel: only this planet's arcs light up
        self.highlighted_pairs = (0..PLANET_COUNT)
            .filter(|&j| j != planet_idx)
            .map(|j| (planet_idx.min(j), planet_idx.max(j)))
            .collect();
        self.highlight_timer = 0.3;
        self.serio_evals = self.highlighted_pairs.len(); // = 6
    }

    pub fn update(&mut self, dt: f64) {
        let dt32 = dt as f32;
        if self.flash_timer > 0.0 {
            self.flash_timer -= dt32;
            self.flash_intensity = (self.flash_timer / 0.3).clamp(0.0, 1.0);
        } else {
            self.flash_intensity = 0.0;
        }

        if self.highlight_timer > 0.0 {
            self.highlight_timer -= dt32;
        }
    }

    /// Render both panels into the full 1280x720 buffer.
    pub fn render(
        &mut self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        orrery: &Orrery,
        time: f64,
        fade: f32,
    ) {
        // ── Left panel: CLASSICAL ─────────────────────────────
        let left_opts = OrreryRenderOpts {
            cx: LEFT_CX,
            cy: PANEL_CY,
            fade,
            show_labels: false,
            show_arcs: true,
            highlighted_pairs: None,
            flash_all: true,
            flash_alpha: self.flash_intensity,
            show_rings: true,
        };
        orrery_render::render(
            buffer,
            width,
            height,
            orrery,
            &mut self.trail_left,
            &left_opts,
            time,
        );

        // ── Right panel: SERIO ───────────────────────────────
        let right_pairs = if self.highlight_timer > 0.0 {
            Some(self.highlighted_pairs.clone())
        } else {
            None
        };
        let right_opts = OrreryRenderOpts {
            cx: RIGHT_CX,
            cy: PANEL_CY,
            fade,
            show_labels: false,
            show_arcs: true,
            highlighted_pairs: right_pairs,
            flash_all: false,
            flash_alpha: 0.0,
            show_rings: true,
        };
        orrery_render::render(
            buffer,
            width,
            height,
            orrery,
            &mut self.trail_right,
            &right_opts,
            time,
        );

        // ── Central divider ───────────────────────────────────
        render_divider(buffer, width, height, fade);

        // ── Panel labels ──────────────────────────────────────
        render_panel_labels(buffer, width, height, fade, time);

        // ── Eval comparison ───────────────────────────────────
        score_display::render_eval_comparison(
            buffer,
            width,
            height,
            self.classical_evals,
            self.serio_evals,
            fade,
            DIVIDER_X as i32 - 130,
            height as i32 - 90,
        );
    }
}

fn render_divider(buffer: &mut [u32], width: usize, height: usize, fade: f32) {
    let x = DIVIDER_X;
    let col = palette::dim(palette::EMERALD_700, fade * 0.8);
    for y in 0..height {
        if x < width {
            buffer[y * width + x] = col;
        }
        if x + 1 < width {
            buffer[y * width + x + 1] = palette::dim(col, 0.4);
        }
        if x > 0 {
            buffer[y * width + x - 1] = palette::dim(col, 0.4);
        }
    }
}

fn render_panel_labels(buf: &mut [u32], width: usize, height: usize, fade: f32, time: f64) {
    // LEFT: "CLASSICAL"
    let pulse = ((time * 1.5).sin() as f32 * 0.1 + 0.9).abs();
    font::draw_text_centered_glow(
        buf,
        width,
        height,
        "CLASSICAL",
        LEFT_CX as i32,
        30,
        2,
        palette::dim(palette::CLASSICAL_RED, fade * pulse),
        palette::dim(palette::CLASSICAL_RED, fade * 0.4),
    );

    font::draw_text_centered(
        buf,
        width,
        height,
        "ALL PAIRS FLASH",
        LEFT_CX as i32,
        52,
        1,
        palette::dim(palette::CHROME, fade * 0.5),
    );

    font::draw_text_centered(
        buf,
        width,
        height,
        "O(n^2) PER MOVE",
        LEFT_CX as i32,
        64,
        1,
        palette::dim(palette::CLASSICAL_RED, fade * 0.7),
    );

    // RIGHT: "SERIO"
    font::draw_text_centered_glow(
        buf,
        width,
        height,
        "SERIO",
        RIGHT_CX as i32,
        30,
        2,
        palette::dim(palette::SERIO_BLUE, fade * pulse),
        palette::dim(palette::SERIO_BLUE, fade * 0.4),
    );

    font::draw_text_centered(
        buf,
        width,
        height,
        "ONLY AFFECTED ARCS",
        RIGHT_CX as i32,
        52,
        1,
        palette::dim(palette::CHROME, fade * 0.5),
    );

    font::draw_text_centered(
        buf,
        width,
        height,
        "O(k) PER MOVE",
        RIGHT_CX as i32,
        64,
        1,
        palette::dim(palette::SERIO_BLUE, fade * 0.7),
    );

    // "SAME RESULT" across the center
    font::draw_text_centered(
        buf,
        width,
        height,
        "=  SAME SCORE  =",
        DIVIDER_X as i32,
        height as i32 - 110,
        1,
        palette::dim(palette::GOLD, fade * 0.6),
    );
}
