// ═══════════════════════════════════════════════════════════════
// DUAL PANEL -- The educational heart of SERIO_02
//
// Splits the screen vertically at x=640.
// LEFT panel:  "CLASSICAL" -- all pairwise arcs flash on each move.
// RIGHT panel: "SERIO"     -- only the affected planet's arcs update.
//
// Same solver moves. Same score convergence. Different visual load.
// The audience sees the locality of incremental evaluation.
// ═══════════════════════════════════════════════════════════════

use crate::effects::orrery_render::{self, OrreryRenderOpts, TrailBuffer};
use crate::font;
use crate::orrery::model::{Orrery, PLANET_COUNT};
use crate::palette;
use crate::palette::Surface;


pub struct DualPanel {
    pub trail_left: TrailBuffer,
    pub trail_right: TrailBuffer,
    flash_timer: f32,
    flash_intensity: f32,
    highlighted_pairs: Vec<(usize, usize)>,
    highlight_timer: f32,
    /// Which planet was last moved (for ring highlight)
    last_moved_planet: Option<usize>,
    move_ring_timer: f32,
    /// Cumulative eval counters (these tick up visibly)
    pub total_classical_evals: u64,
    pub total_serio_evals: u64,
    /// Per-move counts (for the static label)
    pub classical_evals_per_move: usize,
    pub serio_evals_per_move: usize,
    /// Sweep timer: counts up from 0 to SWEEP_DURATION after each move
    sweep_timer: f32,
    /// Number of moves received (arcs hidden until first move)
    move_count: u32,
}

const DIVIDER_X: usize = 640;

const LEFT_CX: f64 = 320.0;
const RIGHT_CX: f64 = 960.0;
const PANEL_CY: f64 = 360.0;

const FLASH_DURATION: f32 = 1.5;
const HIGHLIGHT_DURATION: f32 = 1.5;
const RING_DURATION: f32 = 2.0;
const SWEEP_DURATION: f32 = 1.2;

impl DualPanel {
    pub fn new() -> Self {
        DualPanel {
            trail_left: TrailBuffer::new(1280, 720),
            trail_right: TrailBuffer::new(1280, 720),
            flash_timer: 0.0,
            flash_intensity: 0.0,
            highlighted_pairs: Vec::new(),
            highlight_timer: 0.0,
            last_moved_planet: None,
            move_ring_timer: 0.0,
            total_classical_evals: 0,
            total_serio_evals: 0,
            classical_evals_per_move: 0,
            serio_evals_per_move: 0,
            sweep_timer: -1.0,
            move_count: 0,
        }
    }

    pub fn on_move(&mut self, planet_idx: usize) {
        let total_pairs = PLANET_COUNT * (PLANET_COUNT - 1) / 2; // = 21
        let affected = PLANET_COUNT - 1; // = 6

        // Classical panel: ALL arcs flash
        self.flash_timer = FLASH_DURATION;
        self.flash_intensity = 1.0;
        self.classical_evals_per_move = total_pairs;

        // SERIO panel: only this planet's arcs light up
        self.highlighted_pairs = (0..PLANET_COUNT)
            .filter(|&j| j != planet_idx)
            .map(|j| (planet_idx.min(j), planet_idx.max(j)))
            .collect();
        self.highlight_timer = HIGHLIGHT_DURATION;
        self.serio_evals_per_move = affected;

        // Planet ring highlight
        self.last_moved_planet = Some(planet_idx);
        self.move_ring_timer = RING_DURATION;

        // Sweep: start arc-by-arc evaluation animation
        self.sweep_timer = 0.0;
        self.move_count += 1;

        // Cumulative counters
        self.total_classical_evals += total_pairs as u64;
        self.total_serio_evals += affected as u64;
    }

    pub fn update(&mut self, dt: f64) {
        let dt32 = dt as f32;
        if self.flash_timer > 0.0 {
            self.flash_timer -= dt32;
            self.flash_intensity = (self.flash_timer / FLASH_DURATION).clamp(0.0, 1.0);
        } else {
            self.flash_intensity = 0.0;
        }

        if self.highlight_timer > 0.0 {
            self.highlight_timer -= dt32;
        }

        if self.move_ring_timer > 0.0 {
            self.move_ring_timer -= dt32;
        }

        if self.sweep_timer >= 0.0 && self.sweep_timer < SWEEP_DURATION {
            self.sweep_timer += dt32;
        }
    }

    pub fn render(
        &mut self,
        s: &mut Surface,
        orrery: &Orrery,
        time: f64,
        fade: f32,
        scene_t: f64,
    ) {
        // Before first move: no arcs on either side
        let arcs_visible = if self.move_count == 0 { 0.0 } else { 1.0 };

        // Left panel: CLASSICAL
        let left_opts = OrreryRenderOpts {
            cx: LEFT_CX,
            cy: PANEL_CY,
            fade,
            show_labels: false,
            show_arcs: arcs_visible,
            highlighted_pairs: None,
            flash_all: true,
            flash_alpha: self.flash_intensity,
            show_rings: true,
            moved_planet: self.active_moved_planet(),
            move_ring_alpha: self.ring_alpha(),
            sweep_frontier: self.left_sweep_frontier(),
        };
        orrery_render::render(
            s,
            orrery,
            &mut self.trail_left,
            &left_opts,
            time,
        );

        // Right panel: SERIO (incremental)
        // After first move, always show highlighted pairs (6 bright, 15 dim).
        // This persists between moves so the viewer sees the structural difference.
        let right_pairs = if self.move_count > 0 && !self.highlighted_pairs.is_empty() {
            Some(self.highlighted_pairs.clone())
        } else {
            None
        };
        // Right panel: highlighted arcs pulse bright when a move fires, then settle to dim-others.
        // highlight_timer decays from HIGHLIGHT_DURATION to 0 after each move.
        let right_flash = if self.highlight_timer > 0.0 {
            (self.highlight_timer / HIGHLIGHT_DURATION).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let right_opts = OrreryRenderOpts {
            cx: RIGHT_CX,
            cy: PANEL_CY,
            fade,
            show_labels: false,
            show_arcs: arcs_visible,
            highlighted_pairs: right_pairs,
            flash_all: false,
            flash_alpha: right_flash,
            show_rings: true,
            moved_planet: self.active_moved_planet(),
            move_ring_alpha: self.ring_alpha(),
            sweep_frontier: -1.0,
        };
        orrery_render::render(
            s,
            orrery,
            &mut self.trail_right,
            &right_opts,
            time,
        );

        // Central divider (animated: grows from center, flashes bright)
        render_divider(s, fade, scene_t);

        // "VS" flash during transition
        if scene_t < 3.0 {
            let vs_alpha = if scene_t < 0.5 {
                palette::smoothstep(0.2, 0.5, scene_t) as f32
            } else {
                ((1.0 - (scene_t - 0.5) / 2.5).max(0.0)) as f32
            } * fade;
            if vs_alpha > 0.01 {
                font::draw_text_centered_glow(
                    s,
                    "VS",
                    DIVIDER_X as i32,
                    s.h as i32 / 2,
                    3,
                    palette::dim(palette::WHITE, vs_alpha),
                    palette::dim(palette::EMERALD_700, vs_alpha * 0.5),
                );
            }
        }

        // Panel labels (delayed fade-in)
        render_panel_labels(s, fade, time, scene_t);

        // Cumulative eval comparison (ticking counters)
        render_cumulative_evals(
            s,
            self.total_classical_evals,
            self.total_serio_evals,
            self.classical_evals_per_move,
            self.serio_evals_per_move,
            fade,
        );
    }

    fn active_moved_planet(&self) -> Option<usize> {
        if self.move_ring_timer > 0.0 {
            self.last_moved_planet
        } else {
            None
        }
    }

    fn ring_alpha(&self) -> f32 {
        if self.move_ring_timer > 0.0 {
            (self.move_ring_timer / RING_DURATION).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    fn left_sweep_frontier(&self) -> f32 {
        if self.sweep_timer < 0.0 {
            return -1.0; // no sweep active
        }
        let progress = (self.sweep_timer / SWEEP_DURATION).clamp(0.0, 1.0);
        if progress >= 1.0 {
            -1.0 // sweep done, show all arcs normally
        } else {
            progress * 21.0 // sweep from arc 0 to arc 21
        }
    }
}

fn render_divider(s: &mut Surface, fade: f32, scene_t: f64) {
    let x = DIVIDER_X;

    // Vertical reveal: line grows from center outward (0-0.5s)
    let reveal = palette::smoothstep(0.0, 0.5, scene_t) as f32;
    let half_h = s.h as f32 / 2.0;
    let visible_half = (half_h * reveal) as usize;
    let center_y = s.h / 2;
    let y_min = center_y.saturating_sub(visible_half);
    let y_max = (center_y + visible_half).min(s.h);

    // Flash: bright white at start, fading to normal emerald
    let flash = if scene_t < 0.5 {
        1.0_f32
    } else {
        ((1.0 - (scene_t - 0.5) / 1.5).max(0.0)) as f32
    };

    let base_col = palette::dim(palette::EMERALD_700, fade * 0.8);
    let flash_col = palette::dim(palette::WHITE, fade * flash * 0.9);
    let col = palette::add_color(base_col, flash_col);

    for y in y_min..y_max {
        if x < s.w {
            s.buf[y * s.w + x] = col;
        }
        if x + 1 < s.w {
            s.buf[y * s.w + x + 1] = palette::dim(col, 0.4);
        }
        if x > 0 {
            s.buf[y * s.w + x - 1] = palette::dim(col, 0.4);
        }
    }
}

fn render_panel_labels(s: &mut Surface, fade: f32, time: f64, scene_t: f64) {
    // Labels fade in after 1.5s
    let label_alpha = palette::smoothstep(1.5, 2.5, scene_t) as f32;
    if label_alpha < 0.01 {
        return;
    }
    let fade = fade * label_alpha;
    let pulse = ((time * 1.5).sin() as f32 * 0.1 + 0.9).abs();

    // LEFT: "CLASSICAL"
    font::draw_text_centered_glow(
        s,
        "CLASSICAL",
        LEFT_CX as i32,
        30,
        2,
        palette::dim(palette::CLASSICAL_RED, fade * pulse),
        palette::dim(palette::CLASSICAL_RED, fade * 0.4),
    );
    font::draw_text_centered(
        s,
        "Every arc. Every time.",
        LEFT_CX as i32,
        52,
        1,
        palette::dim(palette::CHROME, fade * 0.5),
    );
    // (removed third left label)

    // RIGHT: "SERIO"
    font::draw_text_centered_glow(
        s,
        "SERIO",
        RIGHT_CX as i32,
        30,
        2,
        palette::dim(palette::SERIO_BLUE, fade * pulse),
        palette::dim(palette::SERIO_BLUE, fade * 0.4),
    );
    font::draw_text_centered(
        s,
        "Only what changed.",
        RIGHT_CX as i32,
        52,
        1,
        palette::dim(palette::CHROME, fade * 0.5),
    );
    // (removed third right label)

    // "SAME RESULT" across the center
    font::draw_text_centered(
        s,
        "=  SAME ANSWER  =",
        DIVIDER_X as i32,
        s.h as i32 - 30,
        1,
        palette::dim(palette::GOLD, fade * 0.6),
    );
}

fn render_cumulative_evals(
    s: &mut Surface,
    total_classical: u64,
    total_serio: u64,
    _per_move_classical: usize,
    _per_move_serio: usize,
    fade: f32,
) {
    let y_base = s.h as i32 - 100;

    // Header: make it unmistakably about WORK, not score
    font::draw_text_centered(
        s,
        "WORK PERFORMED",
        DIVIDER_X as i32,
        y_base,
        1,
        palette::dim(palette::CHROME, fade * 0.5),
    );

    // Left: classical work
    let left_total = palette::format_count(total_classical);
    font::draw_text_centered(
        s,
        &left_total,
        LEFT_CX as i32,
        y_base + 14,
        1,
        palette::dim(palette::CLASSICAL_RED, fade * 0.8),
    );

    // Right: SERIO work
    let right_total = palette::format_count(total_serio);
    font::draw_text_centered(
        s,
        &right_total,
        RIGHT_CX as i32,
        y_base + 14,
        1,
        palette::dim(palette::SERIO_BLUE, fade * 0.8),
    );

    // Speedup ratio — the punchline
    if total_serio > 0 {
        let ratio = total_classical as f64 / total_serio as f64;
        let ratio_str = format!("{:.1}x less work. Same truth.", ratio);
        font::draw_text_centered(
            s,
            &ratio_str,
            DIVIDER_X as i32,
            y_base + 32,
            1,
            palette::dim(palette::GOLD, fade * 0.9),
        );
    }
}

