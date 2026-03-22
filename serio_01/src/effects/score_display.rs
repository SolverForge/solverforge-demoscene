// ═══════════════════════════════════════════════════════════════
// SCORE DISPLAY -- Real-time SERIO metrics overlay
//
// Shows:
//   - Current HardSoftScore (top right)
//   - Hard / Soft constraint bars
//   - Score-over-time graph (bottom strip)
//   - Moves/sec counter
//   - Evaluation counter comparison
// ═══════════════════════════════════════════════════════════════

use solverforge_core::score::HardSoftScore;

use crate::font;
use crate::palette;

const GRAPH_HEIGHT: usize = 80;
const GRAPH_WIDTH: usize = 400;
const GRAPH_MAX_POINTS: usize = GRAPH_WIDTH;

pub struct ScoreGraph {
    /// Rolling score history
    scores: Vec<i64>, // soft score values
    /// Minimum soft score seen (for Y scale)
    min_score: i64,
    /// Maximum soft score seen
    max_score: i64,
}

impl ScoreGraph {
    pub fn new() -> Self {
        ScoreGraph {
            scores: Vec::with_capacity(GRAPH_MAX_POINTS + 16),
            min_score: 0,
            max_score: 1,
        }
    }

    pub fn push(&mut self, score: HardSoftScore) {
        let s = score.soft();
        self.scores.push(s);
        if self.scores.len() > GRAPH_MAX_POINTS {
            self.scores.remove(0);
        }
        self.min_score = self.min_score.min(s);
        self.max_score = self.max_score.max(s);
    }

    pub fn render(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        x: i32,
        y: i32,
        fade: f32,
    ) {
        if self.scores.len() < 2 {
            return;
        }

        let range = (self.max_score - self.min_score).max(1);
        let gw = GRAPH_WIDTH.min(self.scores.len());
        let gh = GRAPH_HEIGHT;

        // Background panel
        for py in 0..gh {
            for px2 in 0..GRAPH_WIDTH {
                let bx = x as usize + px2;
                let by = y as usize + py;
                if bx < width && by < height {
                    let existing = buffer[by * width + bx];
                    buffer[by * width + bx] =
                        palette::lerp_color(existing, palette::MIDNIGHT, 0.7 * fade);
                }
            }
        }

        // Score curve
        let start = self.scores.len().saturating_sub(GRAPH_MAX_POINTS);
        for i in 1..gw {
            let s0 = self.scores[start + i - 1];
            let s1 = self.scores[start + i];

            let px0 = x + (i - 1) as i32 * GRAPH_WIDTH as i32 / gw as i32;
            let py0 = y + gh as i32 - 1 - ((s0 - self.min_score) * gh as i64 / range) as i32;
            let px1 = x + i as i32 * GRAPH_WIDTH as i32 / gw as i32;
            let py1 = y + gh as i32 - 1 - ((s1 - self.min_score) * gh as i64 / range) as i32;

            let t = i as f32 / gw as f32;
            let col = palette::lerp_color(palette::RUST, palette::EMERALD_400, t);
            let col = palette::dim(col, fade);

            // Draw line segment
            bresenham_buf(buffer, width, height, px0, py0, px1, py1, col);
        }

        // Border
        for px2 in 0..GRAPH_WIDTH {
            let bx = x as usize + px2;
            let top_y = y as usize;
            let bot_y = y as usize + gh - 1;
            if bx < width {
                if top_y < height {
                    buffer[top_y * width + bx] = palette::dim(palette::EMERALD_800, fade);
                }
                if bot_y < height {
                    buffer[bot_y * width + bx] = palette::dim(palette::EMERALD_800, fade);
                }
            }
        }

        // Label
        font::draw_text(
            buffer,
            width,
            height,
            "SCORE",
            x,
            y - 12,
            1,
            palette::dim(palette::EMERALD_500, fade),
        );
    }
}

/// Render the score counter (top-right corner)
pub fn render_score_hud(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    score: HardSoftScore,
    total_moves: u64,
    moves_per_sec: f64,
    fade: f32,
) {
    let x = width as i32 - 340;
    let y = 20i32;

    // Dark panel background
    for py in (y - 5)..(y + 80) {
        for px in (x - 10)..(width as i32) {
            if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                let idx = py as usize * width + px as usize;
                buffer[idx] = palette::lerp_color(buffer[idx], palette::NEAR_BLACK, 0.75 * fade);
            }
        }
    }

    // Score label
    font::draw_text(
        buffer,
        width,
        height,
        "SCORE",
        x,
        y,
        1,
        palette::dim(palette::EMERALD_500, fade * 0.7),
    );

    // Hard score
    let hard = score.hard();
    let hard_str = format!("{}hard", hard);
    let hard_col = if hard >= 0 {
        palette::EMERALD_400
    } else {
        palette::RUST
    };
    font::draw_text(
        buffer,
        width,
        height,
        &hard_str,
        x,
        y + 14,
        2,
        palette::dim(hard_col, fade),
    );

    // Soft score
    let soft = score.soft();
    let soft_str = format!("{}soft", soft);
    let soft_col = if soft >= 0 {
        palette::EMERALD_300
    } else {
        palette::AMBER_400
    };
    font::draw_text(
        buffer,
        width,
        height,
        &soft_str,
        x,
        y + 32,
        2,
        palette::dim(soft_col, fade),
    );

    // Moves counter
    let moves_str = format!("{} moves", total_moves);
    font::draw_text(
        buffer,
        width,
        height,
        &moves_str,
        x,
        y + 54,
        1,
        palette::dim(palette::CHROME, fade * 0.6),
    );

    let mps_str = format!("{:.0}/s", moves_per_sec);
    font::draw_text(
        buffer,
        width,
        height,
        &mps_str,
        x + 80,
        y + 54,
        1,
        palette::dim(palette::EMERALD_500, fade * 0.8),
    );
}

/// Render constraint bars (hard + soft penalty visualization)
pub fn render_constraint_bars(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    score: HardSoftScore,
    worst_soft: i64, // maximum soft penalty seen (for scaling)
    fade: f32,
    x: i32,
    y: i32,
) {
    let bar_w = 200i32;
    let bar_h = 10i32;

    // Hard constraint bar
    font::draw_text(
        buffer,
        width,
        height,
        "HARD: No Collision",
        x,
        y,
        1,
        palette::dim(palette::CHROME, fade * 0.7),
    );

    let hard_violations = (-score.hard()).max(0) as f32;
    let hard_frac = (hard_violations / 10.0).min(1.0); // assume max 10 hard violations
    render_bar(
        buffer,
        width,
        height,
        x,
        y + 12,
        bar_w,
        bar_h,
        hard_frac,
        palette::RUST,
        palette::dim(palette::MIDNIGHT, 0.8),
        fade,
    );

    // Soft constraint bar
    font::draw_text(
        buffer,
        width,
        height,
        "SOFT: Harmonic Ratio",
        x,
        y + 32,
        1,
        palette::dim(palette::CHROME, fade * 0.7),
    );

    let soft_pen = (-score.soft()).max(0) as f32;
    let soft_frac = if worst_soft > 0 {
        (soft_pen / worst_soft as f32).min(1.0)
    } else {
        0.0
    };
    render_bar(
        buffer,
        width,
        height,
        x,
        y + 44,
        bar_w,
        bar_h,
        soft_frac,
        palette::AMBER_600,
        palette::dim(palette::MIDNIGHT, 0.8),
        fade,
    );
}

fn render_bar(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    bar_w: i32,
    bar_h: i32,
    frac: f32,
    fill_color: u32,
    bg_color: u32,
    fade: f32,
) {
    let filled = (bar_w as f32 * frac.clamp(0.0, 1.0)) as i32;

    for py in y..(y + bar_h) {
        for px in x..(x + bar_w) {
            if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                let idx = py as usize * width + px as usize;
                let col = if px < x + filled {
                    palette::dim(fill_color, fade)
                } else {
                    palette::dim(bg_color, fade)
                };
                buffer[idx] = palette::lerp_color(buffer[idx], col, 0.8);
            }
        }
    }

    // Bar border
    for px in x..(x + bar_w) {
        let col = palette::dim(palette::EMERALD_800, fade * 0.6);
        if py_safe(y, height) {
            buffer[y as usize * width + px as usize] = col;
        }
        if py_safe(y + bar_h - 1, height) {
            buffer[(y + bar_h - 1) as usize * width + px as usize] = col;
        }
    }
}

fn py_safe(y: i32, h: usize) -> bool {
    y >= 0 && (y as usize) < h
}

/// Render the "evaluations per move" counter comparison
pub fn render_eval_comparison(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    classical_evals: usize,
    serio_evals: usize,
    fade: f32,
    x: i32,
    y: i32,
) {
    font::draw_text(
        buffer,
        width,
        height,
        "EVALS PER MOVE:",
        x,
        y,
        1,
        palette::dim(palette::CHROME, fade * 0.7),
    );

    let classical_str = format!("CLASSICAL: {}", classical_evals);
    font::draw_text(
        buffer,
        width,
        height,
        &classical_str,
        x,
        y + 14,
        2,
        palette::dim(palette::CLASSICAL_RED, fade),
    );

    let serio_str = format!("SERIO: {}", serio_evals);
    font::draw_text(
        buffer,
        width,
        height,
        &serio_str,
        x,
        y + 32,
        2,
        palette::dim(palette::SERIO_BLUE, fade),
    );

    // Speedup ratio
    if serio_evals > 0 {
        let ratio = classical_evals as f32 / serio_evals as f32;
        let speedup_str = format!("{:.1}x FASTER", ratio);
        font::draw_text(
            buffer,
            width,
            height,
            &speedup_str,
            x,
            y + 52,
            1,
            palette::dim(palette::GOLD, fade),
        );
    }
}

// Minimal Bresenham for the score graph
fn bresenham_buf(
    buf: &mut [u32],
    w: usize,
    h: usize,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    col: u32,
) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1i32 } else { -1 };
    let sy = if y0 < y1 { 1i32 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
            buf[y as usize * w + x as usize] = col;
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
