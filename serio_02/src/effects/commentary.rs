// ═══════════════════════════════════════════════════════════════
// COMMENTARY -- Right-side unified technical panel
//
// The spiffiest demoscene telemetry panel ever built.
// No mystification. What is happening. What calculation.
// Why. In real time.
//
// Layout (top to bottom, right column):
//   - Scene header (what are we looking at)
//   - Score (hard + soft, big numbers)
//   - Constraint bars (hard: collision, soft: harmonic)
//   - Solver throughput (moves, moves/sec, accepted%)
//   - Last accepted move (planet, freq change, delta)
//   - Frequency assignment table (7 planets)
//   - Cumulative eval comparison (classical vs SERIO)
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::orrery::model::PLANET_COUNT;
use crate::orrery::solver::SolverState;
use crate::palette::{self, Surface};

/// Per-scene one-liner explaining what the viewer is looking at.
const SCENE_HEADER: [&str; 6] = [
    "THE ORRERY AWAKENS",
    "CLASSICAL EVALUATION",
    "CLASSICAL vs INCREMENTAL",
    "SERIO -- FULL SPEED",
    "MUSICA UNIVERSALIS",
    "SOLVERFORGE",
];

const PANEL_X: i32 = 935;
const PANEL_W: i32 = 340;

/// Render the unified right-side panel.
pub fn render_commentary(
    s: &mut Surface,
    solver: &SolverState,
    scene: usize,
    fade: f32,
) {
    let x = PANEL_X;
    let x_inner = x + 8;

    // ── Dark panel background with subtle border ─────────────
    render_panel_bg(
        s,
        x - 4,
        8,
        PANEL_W + 8,
        s.h as i32 - 16,
        fade,
    );

    let mut y = 16i32;

    // ── Scene header ─────────────────────────────────────────
    let header = if scene < SCENE_HEADER.len() {
        SCENE_HEADER[scene]
    } else {
        ""
    };
    font::draw_text(
        s,
        header,
        x_inner,
        y,
        1,
        palette::dim(palette::GOLD, fade * 0.9),
    );
    y += 16;

    // Separator
    render_hline(s, x_inner, y, PANEL_W - 20, fade * 0.3);
    y += 8;

    // ── Score (big numbers) ──────────────────────────────────
    let score = solver.current_score();
    let hard = score.hard();
    let soft = score.soft();

    font::draw_text(
        s,
        "SCORE",
        x_inner,
        y,
        1,
        palette::dim(palette::EMERALD_600, fade * 0.6),
    );
    y += 14;

    // Hard score — big
    let hard_str = format!("{} hard", hard);
    let hard_col = if hard >= 0 {
        palette::EMERALD_400
    } else {
        palette::RUST
    };
    font::draw_text(
        s,
        &hard_str,
        x_inner,
        y,
        2,
        palette::dim(hard_col, fade),
    );
    y += 20;

    // Soft score — big
    let soft_str = format!("{} soft", soft);
    let soft_col = if soft >= 0 {
        palette::EMERALD_300
    } else {
        palette::AMBER_400
    };
    font::draw_text(
        s,
        &soft_str,
        x_inner,
        y,
        2,
        palette::dim(soft_col, fade),
    );
    y += 20;

    // Best
    let best_str = format!("Best: {} soft", solver.best_score.soft());
    font::draw_text(
        s,
        &best_str,
        x_inner,
        y,
        1,
        palette::dim(palette::GOLD, fade * 0.7),
    );
    y += 16;

    // ── Constraint bars ──────────────────────────────────────
    render_hline(s, x_inner, y, PANEL_W - 20, fade * 0.3);
    y += 6;

    // Hard constraint bar
    font::draw_text(
        s,
        "HARD: Unique Voices",
        x_inner,
        y,
        1,
        palette::dim(palette::CHROME, fade * 0.6),
    );
    y += 12;

    let hard_violations = (-hard).max(0) as f32;
    let hard_frac = (hard_violations / 7.0).min(1.0); // max 7 collisions possible
    render_bar(
        s,
        [x_inner, y, PANEL_W - 24, 8],
        hard_frac,
        palette::RUST,
        fade,
    );
    y += 14;

    // Soft constraint bar
    font::draw_text(
        s,
        "SOFT: Pythagorean Ratio",
        x_inner,
        y,
        1,
        palette::dim(palette::CHROME, fade * 0.6),
    );
    y += 12;

    // Soft bar: show improvement. When soft=0, bar is empty (perfect).
    // When soft is very negative, bar is full. Scale to worst seen.
    let soft_pen = (-soft).max(0) as f64;
    let worst = (-solver.best_score.soft()).max(1) as f64; // avoid div by 0
                                                           // Use initial worst estimate: 21 pairs * 1000 penalty = 21000
    let scale_max = if worst < 100.0 { 21000.0 } else { worst * 1.5 };
    let soft_frac = (soft_pen / scale_max).min(1.0) as f32;
    render_bar(
        s,
        [x_inner, y, PANEL_W - 24, 8],
        soft_frac,
        palette::AMBER_600,
        fade,
    );
    y += 16;

    // ── Solver throughput ────────────────────────────────────
    render_hline(s, x_inner, y, PANEL_W - 20, fade * 0.3);
    y += 6;

    if solver.total_moves == 0 {
        font::draw_text(
            s,
            "SOLVER: WAITING",
            x_inner,
            y,
            1,
            palette::dim(palette::CHROME, fade * 0.5),
        );
        return;
    }

    let mps = solver.moves_per_sec();
    let accept_pct = (solver.accepted_moves as f64 / solver.total_moves as f64 * 100.0) as u32;

    // Status
    let feasible = hard >= 0;
    let status_str = if feasible { "FEASIBLE" } else { "INFEASIBLE" };
    let status_col = if feasible {
        palette::EMERALD_400
    } else {
        palette::RUST
    };
    font::draw_text(
        s,
        "STATUS:",
        x_inner,
        y,
        1,
        palette::dim(palette::EMERALD_600, fade * 0.6),
    );
    font::draw_text(
        s,
        status_str,
        x_inner + 72,
        y,
        1,
        palette::dim(status_col, fade * 0.9),
    );
    y += 14;

    // Moves
    let moves_str = palette::format_count(solver.total_moves);
    font::draw_text(
        s,
        "MOVES:",
        x_inner,
        y,
        1,
        palette::dim(palette::EMERALD_600, fade * 0.6),
    );
    font::draw_text(
        s,
        &moves_str,
        x_inner + 72,
        y,
        1,
        palette::dim(palette::CHROME, fade * 0.9),
    );
    y += 12;

    // Moves/sec — the money number
    let mps_str = palette::format_count(mps as u64);
    font::draw_text(
        s,
        "MOVES/SEC:",
        x_inner,
        y,
        1,
        palette::dim(palette::EMERALD_600, fade * 0.6),
    );
    font::draw_text(
        s,
        &mps_str,
        x_inner + 100,
        y,
        1,
        palette::dim(palette::EMERALD_400, fade),
    );
    y += 12;

    // Accepted%
    let accept_str = format!("{}%", accept_pct);
    font::draw_text(
        s,
        "ACCEPTED:",
        x_inner,
        y,
        1,
        palette::dim(palette::EMERALD_600, fade * 0.6),
    );
    font::draw_text(
        s,
        &accept_str,
        x_inner + 90,
        y,
        1,
        palette::dim(palette::CHROME, fade * 0.8),
    );
    y += 16;

    // ── Last accepted move ───────────────────────────────────
    render_hline(s, x_inner, y, PANEL_W - 20, fade * 0.3);
    y += 6;

    if let Some(mv) = solver.last_accepted_move() {
        font::draw_text(
            s,
            "LAST MOVE",
            x_inner,
            y,
            1,
            palette::dim(palette::GOLD, fade * 0.8),
        );
        y += 14;

        let planet_name = solver.solution().planets[mv.planet_idx].name;
        let line1 = format!(
            "{}: {} -> {}",
            planet_name,
            mv.old_freq_name(),
            mv.new_freq_name()
        );
        font::draw_text(
            s,
            &line1,
            x_inner + 4,
            y,
            1,
            palette::dim(palette::CHROME, fade * 0.9),
        );
        y += 12;

        let delta = mv.soft_delta();
        let delta_str = if delta > 0 {
            format!("Score +{}", delta)
        } else {
            format!("Score {}", delta)
        };
        let delta_col = if delta > 0 {
            palette::EMERALD_400
        } else if delta == 0 {
            palette::CHROME
        } else {
            palette::RUST
        };
        font::draw_text(
            s,
            &delta_str,
            x_inner + 4,
            y,
            1,
            palette::dim(delta_col, fade * 0.8),
        );
        y += 12;

        font::draw_text(
            s,
            "Untouched arcs preserved",
            x_inner + 4,
            y,
            1,
            palette::dim(palette::SERIO_BLUE, fade * 0.7),
        );
        y += 16;
    } else {
        y += 44;
    }

    // ── Frequency assignment table ───────────────────────────
    render_hline(s, x_inner, y, PANEL_W - 20, fade * 0.3);
    y += 6;

    font::draw_text(
        s,
        "VOICES",
        x_inner,
        y,
        1,
        palette::dim(palette::EMERALD_600, fade * 0.6),
    );
    y += 14;
    let assignments = solver.freq_assignment_summary();
    for (name, note, hz) in &assignments {
        let line = format!("{:<8} {:>3}  {:.0}Hz", name, note, hz);
        let planet_col = solver
            .solution()
            .planets
            .iter()
            .find(|p| p.name == *name)
            .map(|p| p.color)
            .unwrap_or(palette::CHROME);
        font::draw_text(
            s,
            &line,
            x_inner + 4,
            y,
            1,
            palette::dim(planet_col, fade * 0.8),
        );
        y += 11;
    }
    y += 6;

    // ── Cumulative eval comparison ───────────────────────────
    render_hline(s, x_inner, y, PANEL_W - 20, fade * 0.3);
    y += 6;

    let total_serio_evals = solver.total_moves * (PLANET_COUNT as u64 - 1);
    let total_classical_evals = solver.total_moves * (PLANET_COUNT * (PLANET_COUNT - 1) / 2) as u64;

    // The ratio is a fixed property of the problem (21/6 = 3.5x),
    // so it's valid to show even when classical isn't running.
    let ratio = if total_serio_evals > 0 {
        total_classical_evals as f64 / total_serio_evals as f64
    } else {
        0.0
    };

    if scene <= 2 {
        // Scenes 0-2: show both Classical and SERIO comparison
        font::draw_text(
            s,
            "TOTAL ARC EVALUATIONS",
            x_inner,
            y,
            1,
            palette::dim(palette::EMERALD_600, fade * 0.6),
        );
        y += 14;

        let classical_str = format!("Classical: {}", palette::format_count(total_classical_evals));
        font::draw_text(
            s,
            &classical_str,
            x_inner + 4,
            y,
            1,
            palette::dim(palette::CLASSICAL_RED, fade * 0.8),
        );
        y += 12;

        let serio_str = format!("SERIO:     {}", palette::format_count(total_serio_evals));
        font::draw_text(
            s,
            &serio_str,
            x_inner + 4,
            y,
            1,
            palette::dim(palette::SERIO_BLUE, fade * 0.8),
        );
        y += 12;

        if ratio > 0.0 {
            let ratio_str = format!("{:.1}x fewer evals", ratio);
            font::draw_text(
                s,
                &ratio_str,
                x_inner + 4,
                y,
                1,
                palette::dim(palette::GOLD, fade * 0.9),
            );
        }
    } else {
        // Scenes 3+: classical is done — show only SERIO stats
        font::draw_text(
            s,
            "SERIO ARC EVALUATIONS",
            x_inner,
            y,
            1,
            palette::dim(palette::SERIO_BLUE, fade * 0.6),
        );
        y += 14;

        let serio_str = palette::format_count(total_serio_evals);
        font::draw_text(
            s,
            &serio_str,
            x_inner + 4,
            y,
            1,
            palette::dim(palette::SERIO_BLUE, fade * 0.9),
        );
        y += 12;

        font::draw_text(
            s,
            "Only changed arcs re-weighed",
            x_inner + 4,
            y,
            1,
            palette::dim(palette::EMERALD_400, fade * 0.7),
        );
        y += 12;

        if ratio > 0.0 {
            let ratio_str = format!("{:.1}x advantage", ratio);
            font::draw_text(
                s,
                &ratio_str,
                x_inner + 4,
                y,
                1,
                palette::dim(palette::GOLD, fade * 0.9),
            );
        }
    }
}

// ── Panel background ─────────────────────────────────────────

fn render_panel_bg(
    s: &mut Surface,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    fade: f32,
) {
    // Fill with semi-transparent dark
    for py in y..(y + h) {
        for px in x..(x + w) {
            if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                let idx = py as usize * s.w + px as usize;
                s.buf[idx] = palette::lerp_color(s.buf[idx], palette::NEAR_BLACK, 0.82 * fade);
            }
        }
    }

    // Border — subtle emerald
    let border = palette::dim(palette::EMERALD_800, fade * 0.5);

    // Top + bottom
    for px in x..(x + w) {
        if px >= 0 && px < s.w as i32 {
            if y >= 0 && (y as usize) < s.h {
                s.buf[y as usize * s.w + px as usize] = border;
            }
            let by = y + h - 1;
            if by >= 0 && (by as usize) < s.h {
                s.buf[by as usize * s.w + px as usize] = border;
            }
        }
    }
    // Left + right
    for py in y..(y + h) {
        if py >= 0 && (py as usize) < s.h {
            if x >= 0 && (x as usize) < s.w {
                s.buf[py as usize * s.w + x as usize] = border;
            }
            let rx = x + w - 1;
            if rx >= 0 && (rx as usize) < s.w {
                s.buf[py as usize * s.w + rx as usize] = border;
            }
        }
    }
}

// ── Horizontal separator line ────────────────────────────────

fn render_hline(
    s: &mut Surface,
    x: i32,
    y: i32,
    w: i32,
    fade: f32,
) {
    let col = palette::dim(palette::EMERALD_800, fade);
    if y >= 0 && (y as usize) < s.h {
        for px in x..(x + w) {
            if px >= 0 && (px as usize) < s.w {
                s.buf[y as usize * s.w + px as usize] =
                    palette::add_color(s.buf[y as usize * s.w + px as usize], col);
            }
        }
    }
}

// ── Constraint bar ───────────────────────────────────────────

fn render_bar(
    s: &mut Surface,
    rect: [i32; 4],
    frac: f32,
    fill_color: u32,
    fade: f32,
) {
    let [x, y, bar_w, bar_h] = rect;
    let filled = (bar_w as f32 * frac.clamp(0.0, 1.0)) as i32;
    let bg = palette::dim(palette::MIDNIGHT, fade * 0.8);
    let fill = palette::dim(fill_color, fade * 0.9);
    let border = palette::dim(palette::EMERALD_800, fade * 0.4);

    for py in y..(y + bar_h) {
        for px in x..(x + bar_w) {
            if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                let idx = py as usize * s.w + px as usize;
                if px < x + filled {
                    s.buf[idx] = palette::lerp_color(s.buf[idx], fill, 0.9);
                } else {
                    s.buf[idx] = palette::lerp_color(s.buf[idx], bg, 0.6);
                }
            }
        }
    }

    // Border top + bottom
    for px in x..(x + bar_w) {
        if px >= 0 && px < s.w as i32 {
            if y >= 0 && (y as usize) < s.h {
                s.buf[y as usize * s.w + px as usize] = border;
            }
            let by = y + bar_h - 1;
            if by >= 0 && (by as usize) < s.h {
                s.buf[by as usize * s.w + px as usize] = border;
            }
        }
    }
}
