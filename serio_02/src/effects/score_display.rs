// ═══════════════════════════════════════════════════════════════
// SCORE DISPLAY -- Score-over-time graph
//
// Rolling line chart of soft score convergence.
// The main score HUD and constraint bars are now in commentary.rs.
// ═══════════════════════════════════════════════════════════════

use solverforge_core::score::HardSoftScore;

use crate::font;
use crate::palette;
use crate::palette::Surface;

const GRAPH_HEIGHT: usize = 80;
const GRAPH_WIDTH: usize = 400;
// One point per frame across the scene duration (35s * 60fps = 2100 frames → shown in 400px)
const GRAPH_MAX_POINTS: usize = 2100;

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
        s: &mut Surface,
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
                if bx < s.w && by < s.h {
                    let existing = s.buf[by * s.w + bx];
                    s.buf[by * s.w + bx] =
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

            palette::bresenham(s, px0, py0, px1, py1, col);
        }

        // Border
        let border_col = palette::dim(palette::EMERALD_800, fade);
        for px2 in 0..GRAPH_WIDTH {
            let bx = x as usize + px2;
            let top_y = y as usize;
            let bot_y = y as usize + gh - 1;
            if bx < s.w {
                if top_y < s.h {
                    s.buf[top_y * s.w + bx] = border_col;
                }
                if bot_y < s.h {
                    s.buf[bot_y * s.w + bx] = border_col;
                }
            }
        }

        // Label
        font::draw_text(
            s,
            "SCORE",
            x,
            y - 12,
            1,
            palette::dim(palette::EMERALD_500, fade),
        );
    }
}

