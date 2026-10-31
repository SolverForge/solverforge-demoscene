// ═══════════════════════════════════════════════════════════════
// COMMENTARY -- the right-side panel: the numbers behind the picture.
//
// Everything in it is read from the orrery that is on screen, with the
// same functions the SERIO constraints use, so the panel and the arcs
// can never disagree:
//   - NOTES: each planet's pitch in Hz
//   - PAIRS: all 21 pairs, their ratio (octaves folded), the nearest pure
//     interval, the orbit weight, and whether the pair is in tune
//   - SCORE: hard (shared notes) and soft (weighted distance from pure)
//   - WORK: what the current scene is doing, with real counters
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::orrery::constraints::{collision_count, pair_penalty, pair_weight};
use crate::orrery::harmony::harmonic_deviation;
use crate::orrery::model::{Orrery, PLANET_COUNT};
use crate::orrery::solver::SolverState;
use crate::palette::{self, Surface};

const PANEL_X: i32 = 935;
const PANEL_W: i32 = 340;
const PAIRS: u64 = (PLANET_COUNT * (PLANET_COUNT - 1) / 2) as u64;
const TOUCHED: u64 = (PLANET_COUNT - 1) as u64;

/// What the panel should describe this frame.
pub struct Panel<'a> {
    /// The orrery on screen.
    pub orrery: &'a Orrery,
    pub solver: &'a SolverState,
    pub scene: usize,
    /// The solver scene's slow-motion opening.
    pub slow: bool,
    /// The planet whose note just changed, if the picture marks one.
    pub moved: Option<usize>,
    /// The re-check sweep: index of the pair being checked right now.
    pub sweep: Option<f32>,
    /// Note changes shown so far in the re-check scene.
    pub changes: u64,
    pub fade: f32,
}

fn header(p: &Panel) -> &'static str {
    match p.scene {
        0 => "THE PLANETS AND THEIR NOTES",
        1 => "RE-CHECK EVERYTHING",
        3 if p.slow => "SOLVERFORGE, SLOWED DOWN",
        3 => "SOLVERFORGE, FULL SPEED",
        4 => "THE BEST TUNING FOUND",
        _ => "",
    }
}

fn short(name: &str) -> String {
    name.chars().take(3).collect::<String>().to_uppercase()
}

pub fn render_commentary(s: &mut Surface, p: &Panel) {
    let fade = p.fade;
    let x = PANEL_X + 8;
    let line = 11;
    render_panel_bg(s, PANEL_X - 4, 8, PANEL_W + 8, s.h as i32 - 16, fade);

    let mut y = 16;
    font::draw_text(s, header(p), x, y, 1, palette::dim(palette::GOLD, fade));
    y += 14;
    render_hline(s, x, y, PANEL_W - 20, fade * 0.3);
    y += 7;

    // ── Notes ────────────────────────────────────────────────
    let planets = &p.orrery.planets;
    section(s, "NOTES (PITCH IN HZ)", x, y, fade);
    y += 13;
    for (i, planet) in planets.iter().enumerate() {
        let col = (i % 2) as i32 * 165;
        let row = (i / 2) as i32;
        let marked = p.moved == Some(i);
        let text = format!(
            "{}{:<8}{:>4.0}",
            if marked { ">" } else { " " },
            planet.name.to_uppercase(),
            planet.frequency()
        );
        let k = if marked { 1.0 } else { 0.8 };
        font::draw_text(
            s,
            &text,
            x + col,
            y + row * line,
            1,
            palette::dim(planet.color, fade * k),
        );
    }
    y += 4 * line + 4;

    // ── Pairs ────────────────────────────────────────────────
    render_hline(s, x, y, PANEL_W - 20, fade * 0.3);
    y += 6;
    section(s, "PAIRS: HIGH / LOW PITCH", x, y, fade);
    y += 11;
    font::draw_text(
        s,
        "HALVED UNTIL BELOW 2. TUNE: 1.5%",
        x,
        y,
        1,
        palette::dim(palette::CHROME, fade * 0.45),
    );
    y += 13;
    let mut in_tune = 0u64;
    let mut idx = 0usize;
    for i in 0..PLANET_COUNT {
        for j in (i + 1)..PLANET_COUNT {
            let (fa, fb) = (planets[i].frequency(), planets[j].frequency());
            let (hi, lo, hi_name, lo_name) = if fa >= fb {
                (fa, fb, planets[i].name, planets[j].name)
            } else {
                (fb, fa, planets[j].name, planets[i].name)
            };
            let mut ratio = hi / lo;
            while ratio >= 2.0 {
                ratio /= 2.0;
            }
            let (penalty, interval, _) = harmonic_deviation(fa, fb);
            let ok = penalty == 0 && planets[i].freq_idx != planets[j].freq_idx;
            if ok {
                in_tune += 1;
            }
            let touched = p.moved.is_some_and(|m| m == i || m == j);
            // During the sweep, pairs not yet re-checked are dimmed.
            let pending = p.sweep.is_some_and(|f| idx as f32 > f);
            let checking = p.sweep.is_some_and(|f| (f - idx as f32).abs() < 0.5);
            let pure = format!("{}:{}", interval.num, interval.den);
            let text = format!(
                "{}{}/{} {:.3} {:>5} x{} {}",
                if touched || checking { ">" } else { " " },
                short(hi_name),
                short(lo_name),
                ratio,
                pure,
                pair_weight(i, j),
                if ok { "OK" } else { "OFF" }
            );
            let base = if ok {
                palette::GOLD
            } else {
                palette::rgb(255, 96, 64)
            };
            let k = if pending {
                0.25
            } else if checking || touched {
                1.0
            } else if p.moved.is_some() {
                0.55
            } else {
                0.8
            };
            let color = if checking {
                palette::lerp_color(base, palette::WHITE, 0.6)
            } else {
                base
            };
            font::draw_text(s, &text, x, y, 1, palette::dim(color, fade * k));
            y += line + 2;
            idx += 1;
        }
    }
    y += 4;

    // ── Score ────────────────────────────────────────────────
    render_hline(s, x, y, PANEL_W - 20, fade * 0.3);
    y += 6;
    let hard = -collision_count(p.orrery);
    let soft = -(0..PLANET_COUNT)
        .flat_map(|i| ((i + 1)..PLANET_COUNT).map(move |j| (i, j)))
        .map(|(i, j)| pair_penalty(p.orrery, i, j))
        .sum::<i64>();
    section(s, "SCORE (ZERO IS PERFECT)", x, y, fade);
    y += 13;
    let hard_col = if hard >= 0 {
        palette::EMERALD_400
    } else {
        palette::RUST
    };
    let soft_col = if soft >= 0 {
        palette::EMERALD_300
    } else {
        palette::AMBER_400
    };
    font::draw_text(
        s,
        &format!("{hard} HARD"),
        x,
        y,
        2,
        palette::dim(hard_col, fade),
    );
    y += 20;
    font::draw_text(
        s,
        &format!("{soft} SOFT"),
        x,
        y,
        2,
        palette::dim(soft_col, fade),
    );
    y += 20;
    let tune_col = if in_tune == PAIRS {
        palette::GOLD
    } else {
        palette::CHROME
    };
    font::draw_text(
        s,
        &format!("{in_tune} OF {PAIRS} PAIRS IN TUNE"),
        x,
        y,
        1,
        palette::dim(tune_col, fade * 0.8),
    );
    y += 16;

    // ── Work ─────────────────────────────────────────────────
    render_hline(s, x, y, PANEL_W - 20, fade * 0.3);
    y += 6;
    let rows: Vec<(String, u32)> = match p.scene {
        0 => vec![
            ("GOLD: IN TUNE.  RED: OUT OF TUNE.".into(), palette::CHROME),
            (
                "x3: NEXT-DOOR ORBITS. x2: ONE APART.".into(),
                palette::CHROME,
            ),
            ("ORBITS AND SPIN DO NOT SCORE.".into(), palette::CHROME),
        ],
        1 => vec![
            (format!("NOTES CHANGED:    {}", p.changes), palette::CHROME),
            (
                format!(
                    "PAIRS RE-CHECKED: {}",
                    palette::format_count(p.changes * PAIRS)
                ),
                palette::CLASSICAL_RED,
            ),
            (
                format!("{PAIRS} PAIRS AFTER EVERY CHANGE"),
                palette::CLASSICAL_RED,
            ),
        ],
        3 | 4 => solver_rows(p),
        _ => Vec::new(),
    };
    section(s, "WORK", x, y, fade);
    y += 13;
    for (text, col) in rows {
        font::draw_text(s, &text, x, y, 1, palette::dim(col, fade * 0.85));
        y += 12;
    }
}

fn solver_rows(p: &Panel) -> Vec<(String, u32)> {
    let sv = p.solver;
    let mut rows = vec![
        (
            format!("CHANGES TRIED:  {}", palette::format_count(sv.total_moves)),
            palette::CHROME,
        ),
        (
            format!(
                "CHANGES KEPT:   {}",
                palette::format_count(sv.accepted_moves)
            ),
            palette::CHROME,
        ),
    ];
    if p.scene == 3 && p.slow {
        rows.push(("SLOWED DOWN TO FOLLOW EACH ONE".into(), palette::GOLD));
        if let Some(mv) = sv.history.last() {
            let name = p.orrery.planets[mv.planet_idx].name.to_uppercase();
            rows.push((
                format!(
                    "LAST: {} {:.0} -> {:.0} HZ",
                    name,
                    mv.old_hz(),
                    mv.tried_hz()
                ),
                palette::CHROME,
            ));
            rows.push(if mv.accepted {
                ("NOT WORSE: KEPT".to_string(), palette::EMERALD_400)
            } else {
                ("WORSE: UNDONE".to_string(), palette::RUST)
            });
        }
    } else {
        rows.push((
            format!(
                "PER SECOND:     {}",
                palette::format_count(sv.moves_per_sec() as u64)
            ),
            palette::EMERALD_400,
        ));
    }
    rows.push((
        format!(
            "SERIO CHECKED:  {} PAIRS",
            palette::format_count(sv.total_moves * TOUCHED)
        ),
        palette::SERIO_BLUE,
    ));
    rows.push((
        format!(
            "ALL 21 WOULD BE {} PAIRS",
            palette::format_count(sv.total_moves * PAIRS)
        ),
        palette::CLASSICAL_RED,
    ));
    rows
}

fn section(s: &mut Surface, text: &str, x: i32, y: i32, fade: f32) {
    font::draw_text(
        s,
        text,
        x,
        y,
        1,
        palette::dim(palette::EMERALD_500, fade * 0.8),
    );
}

// ── Panel background ─────────────────────────────────────────

fn render_panel_bg(s: &mut Surface, x: i32, y: i32, w: i32, h: i32, fade: f32) {
    for py in y..(y + h) {
        for px in x..(x + w) {
            if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                let idx = py as usize * s.w + px as usize;
                s.buf[idx] = palette::lerp_color(s.buf[idx], palette::NEAR_BLACK, 0.82 * fade);
            }
        }
    }

    let border = palette::dim(palette::EMERALD_800, fade * 0.5);
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

fn render_hline(s: &mut Surface, x: i32, y: i32, w: i32, fade: f32) {
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
