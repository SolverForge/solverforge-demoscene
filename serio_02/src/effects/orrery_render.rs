// ═══════════════════════════════════════════════════════════════
// ORRERY RENDERER -- The beating heart of Musica Universalis
//
// Renders:
//   - Sun (radial glow, pulsing)
//   - Orbital rings (concentric dashed circles)
//   - Planets (glow halos + filled circles with trails)
//   - Harmonic arcs (color-coded by consonance, labeled)
//   - Planet names
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::orrery::harmony::harmonic_deviation;
use crate::orrery::model::{Orrery, Planet, ORBITAL_RADII, PLANET_COUNT};
// Note: ORBITAL_RADII in serio_02 is [f64; PLANET_COUNT] (fixed per-planet radii)
use crate::palette::{self, harmonic_color, Surface};

/// Rendering options for the orrery
pub struct OrreryRenderOpts {
    /// Center x, y in pixels
    pub cx: f64,
    pub cy: f64,
    /// Overall brightness multiplier [0.0, 1.0]
    pub fade: f32,
    /// Whether to show arc labels (interval names)
    pub show_labels: bool,
    /// How many arcs to show: 0.0 = none, 1.0 = all, fractional = partial reveal
    pub show_arcs: f32,
    /// Whether to highlight specific planet pairs (for dual-panel / retract-insert anim)
    pub highlighted_pairs: Option<Vec<(usize, usize)>>,
    /// Whether this is the "full eval" side (all arcs flash) or SERIO side
    pub flash_all: bool,
    /// Flash intensity [0.0, 1.0] — for the "classical" full-eval strobe
    pub flash_alpha: f32,
    /// Show orbital slot rings
    pub show_rings: bool,
    /// Which planet was just moved (for ring highlight)
    pub moved_planet: Option<usize>,
    /// Ring highlight intensity [0.0, 1.0] — decays after move
    pub move_ring_alpha: f32,
    /// Evaluation sweep frontier: 0.0 = no arcs evaluated yet, 21.0 = all done.
    /// Arcs below the frontier show at consonance color.
    /// The arc AT the frontier strobes white (being evaluated NOW).
    /// Arcs above the frontier are invisible.
    /// Negative or >= 21.0 means "no sweep active" (all arcs render normally).
    pub sweep_frontier: f32,
}

impl Default for OrreryRenderOpts {
    fn default() -> Self {
        OrreryRenderOpts {
            cx: 640.0,
            cy: 360.0,
            fade: 1.0,
            show_labels: true,
            show_arcs: 1.0,
            highlighted_pairs: None,
            flash_all: false,
            flash_alpha: 0.0,
            show_rings: true,
            moved_planet: None,
            move_ring_alpha: 0.0,
            sweep_frontier: -1.0,
        }
    }
}

/// Trail buffer for phosphor persistence (separate from main buffer).
pub struct TrailBuffer {
    pub buf: Vec<u32>,
    width: usize,
    height: usize,
}

impl TrailBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        TrailBuffer {
            buf: vec![0u32; width * height],
            width,
            height,
        }
    }

    /// Fade trail buffer toward black (phosphor decay)
    pub fn decay(&mut self, amount: u32) {
        for px in self.buf.iter_mut() {
            let r = ((*px >> 16) & 0xFF).saturating_sub(amount);
            let g = ((*px >> 8) & 0xFF).saturating_sub(amount);
            let b = (*px & 0xFF).saturating_sub(amount);
            *px = (r << 16) | (g << 8) | b;
        }
    }

    /// Write trail positions for a planet
    pub fn write_planet(&mut self, x: i32, y: i32, color: u32, radius: i32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let px = x + dx;
                    let py = y + dy;
                    if px >= 0 && py >= 0 && px < self.width as i32 && py < self.height as i32 {
                        let idx = py as usize * self.width + px as usize;
                        self.buf[idx] = palette::add_color(self.buf[idx], color);
                    }
                }
            }
        }
    }

    /// Composite trail buffer onto main buffer (additive)
    pub fn composite(&self, buffer: &mut [u32]) {
        for (px, &trail) in buffer.iter_mut().zip(self.buf.iter()) {
            *px = palette::add_color(*px, trail);
        }
    }
}

/// Main orrery renderer
pub fn render(
    s: &mut Surface,
    orrery: &Orrery,
    trail: &mut TrailBuffer,
    opts: &OrreryRenderOpts,
    time: f64,
) {
    let cx = opts.cx;
    let cy = opts.cy;
    let fade = opts.fade;

    // ── Sun ──────────────────────────────────────────────────
    render_sun(s, cx, cy, fade, time);

    // ── Orbital slot rings ───────────────────────────────────
    if opts.show_rings {
        render_rings(s, cx, cy, fade);
    }

    // ── Harmonic arcs ────────────────────────────────────────
    if opts.show_arcs > 0.0 {
        render_arcs(s, orrery, cx, cy, fade, opts, time);
    }

    // ── Planet trails (phosphor) ─────────────────────────────
    trail.decay(6);
    for planet in &orrery.planets {
        let (px, py) = orrery.planet_screen_pos(planet, cx, cy);
        let tr = ((planet.visual_radius * 0.5) as i32).max(1);
        trail.write_planet(px as i32, py as i32, palette::dim(planet.color, 0.25), tr);
    }
    trail.composite(s.buf);

    // ── Planets ───────────────────────────────────────────────
    for planet in &orrery.planets {
        let (px, py) = orrery.planet_screen_pos(planet, cx, cy);
        render_planet(s, planet, px, py, fade, time);
    }

    // ── Moved-planet ring highlight ─────────────────────────
    if let Some(pidx) = opts.moved_planet {
        if pidx < orrery.planets.len() && opts.move_ring_alpha > 0.01 {
            let planet = &orrery.planets[pidx];
            let (px, py) = orrery.planet_screen_pos(planet, cx, cy);
            let ring_r = planet.visual_radius as i32 + 12;
            let ring_col = palette::dim(palette::GOLD, opts.move_ring_alpha * fade * 0.8);
            ring_circle(
                s, px as i32, py as i32, ring_r, 2, ring_col,
            );
            // Outer glow ring
            let glow_r = ring_r + 4;
            let glow_col = palette::dim(palette::AMBER_400, opts.move_ring_alpha * fade * 0.3);
            ring_circle(
                s, px as i32, py as i32, glow_r, 1, glow_col,
            );
        }
    }

    // ── Planet name labels ───────────────────────────────────
    for planet in &orrery.planets {
        let (px, py) = orrery.planet_screen_pos(planet, cx, cy);
        let name_x = px as i32 + planet.visual_radius as i32 + 4;
        let name_y = py as i32 - 4;
        let c = palette::dim(planet.color, fade * 0.7);
        font::draw_text(s, planet.name, name_x, name_y, 1, c);
    }
}

// ── Sun ───────────────────────────────────────────────────────

fn render_sun(s: &mut Surface, cx: f64, cy: f64, fade: f32, time: f64) {
    let pulse = ((time * 1.2).sin() as f32 * 0.1 + 0.9).abs();
    let core_r = (18.0 * pulse) as i32;
    let cxi = cx as i32;
    let cyi = cy as i32;

    // Outer glow (large soft halo)
    for gy in -60i32..=60 {
        for gx in -60i32..=60 {
            let d2 = (gx * gx + gy * gy) as f32;
            if d2 < 3600.0 {
                let d = d2.sqrt();
                let alpha = (1.0 - d / 60.0).powf(2.5) * fade * 0.5;
                let glow = palette::rgb(
                    (255.0 * alpha) as u8,
                    (220.0 * alpha) as u8,
                    (80.0 * alpha) as u8,
                );
                let px = cxi + gx;
                let py = cyi + gy;
                if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                    s.buf[py as usize * s.w + px as usize] =
                        palette::add_color(s.buf[py as usize * s.w + px as usize], glow);
                }
            }
        }
    }

    // Mid glow
    for gy in -30i32..=30 {
        for gx in -30i32..=30 {
            let d2 = gx * gx + gy * gy;
            if d2 < 900 {
                let d = (d2 as f32).sqrt();
                let alpha = (1.0 - d / 30.0).powf(1.5) * fade * 0.8;
                let glow = palette::rgb(255, (240.0 * alpha) as u8, (120.0 * alpha) as u8);
                let px = cxi + gx;
                let py = cyi + gy;
                if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                    s.buf[py as usize * s.w + px as usize] =
                        palette::add_color(s.buf[py as usize * s.w + px as usize], glow);
                }
            }
        }
    }

    // Core — filled white-gold circle
    fill_circle(s, cxi, cyi, core_r, palette::dim(palette::rgb(255, 245, 160), fade));
    fill_circle(
        s,
        cxi,
        cyi,
        (core_r * 2 / 3).max(1),
        palette::dim(palette::WHITE, fade),
    );
}

// ── Orbital rings ─────────────────────────────────────────────

fn render_rings(s: &mut Surface, cx: f64, cy: f64, fade: f32) {
    for &r in ORBITAL_RADII.iter() {
        draw_dashed_circle(
            s,
            cx as i32,
            cy as i32,
            r as i32,
            palette::dim(palette::MIDNIGHT, fade * 0.8),
        );
    }
}

fn draw_dashed_circle(s: &mut Surface, cx: i32, cy: i32, r: i32, col: u32) {
    let steps = (r as f32 * std::f32::consts::TAU) as usize + 64;
    for step in 0..steps {
        // Dashed: skip every other 4th segment
        if (step / 4) % 2 == 1 {
            continue;
        }
        let angle = step as f32 * std::f32::consts::TAU / steps as f32;
        let px = (cx as f32 + angle.cos() * r as f32) as i32;
        let py = (cy as f32 + angle.sin() * r as f32) as i32;
        if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
            s.buf[py as usize * s.w + px as usize] =
                palette::add_color(s.buf[py as usize * s.w + px as usize], col);
        }
    }
}

// ── Harmonic arcs ─────────────────────────────────────────────

fn render_arcs(
    s: &mut Surface,
    orrery: &Orrery,
    cx: f64,
    cy: f64,
    fade: f32,
    opts: &OrreryRenderOpts,
    _time: f64,
) {
    let planets = &orrery.planets;

    // Fractional show_arcs: reveal arcs progressively (0.0=none, 1.0=all)
    let total_pairs = PLANET_COUNT * (PLANET_COUNT - 1) / 2;
    let visible_pairs = if opts.show_arcs >= 1.0 {
        total_pairs
    } else {
        (opts.show_arcs * total_pairs as f32).ceil() as usize
    };
    let mut pair_idx = 0usize;
    let sweep_active = opts.sweep_frontier >= 0.0;

    for i in 0..PLANET_COUNT {
        for j in (i + 1)..PLANET_COUNT {
            pair_idx += 1;
            if pair_idx > visible_pairs {
                return;
            }

            // Sweep frontier: arcs beyond the frontier are invisible
            let arc_index = pair_idx as f32 - 1.0; // 0-based
            if sweep_active && arc_index > opts.sweep_frontier {
                continue; // not evaluated yet
            }

            let pa = &planets[i];
            let pb = &planets[j];

            let f_a = pa.frequency();
            let f_b = pb.frequency();
            let (penalty, interval, dev) = harmonic_deviation(f_a, f_b);
            let cons = 1.0 - (penalty as f64 / 1000.0);

            // Is this pair highlighted (affected by current move)?
            let is_highlighted = opts
                .highlighted_pairs
                .as_ref()
                .map(|pairs| pairs.contains(&(i, j)) || pairs.contains(&(j, i)))
                .unwrap_or(false);

            // Sweep strobe: the arc at the frontier glows white
            let sweep_strobe = if sweep_active {
                let dist = opts.sweep_frontier - arc_index;
                if dist < 1.0 { (1.0 - dist).clamp(0.0, 1.0) } else { 0.0 }
            } else {
                0.0
            };

            // Classical full-eval mode: all arcs flash uniformly
            let effective_fade = if opts.flash_all {
                let flash = opts.flash_alpha;
                fade * (0.2 + flash * 0.8)
            } else if is_highlighted {
                // Highlighted (SERIO) pairs: pulse bright on move, then stay at full
                fade * (1.0 + opts.flash_alpha * 0.8)
            } else if opts.highlighted_pairs.is_some() {
                fade * 0.3
            } else {
                fade
            };

            // Arc color from consonance
            let base_color = harmonic_color(cons as f32);
            let arc_color = if sweep_strobe > 0.01 {
                // Arc being evaluated RIGHT NOW: strobe white
                let strobed = palette::lerp_color(base_color, palette::WHITE, sweep_strobe * 0.85);
                palette::dim(strobed, effective_fade.max(fade * 0.6))
            } else if opts.flash_all && opts.flash_alpha > 0.01 {
                let strobed = palette::lerp_color(base_color, palette::WHITE, opts.flash_alpha * 0.7);
                palette::dim(strobed, effective_fade)
            } else if is_highlighted && opts.flash_alpha > 0.01 {
                // SERIO highlighted arc: flash bright when move fires
                let strobed = palette::lerp_color(base_color, palette::WHITE, opts.flash_alpha * 0.6);
                palette::dim(strobed, effective_fade)
            } else {
                palette::dim(base_color, effective_fade)
            };

            // Get planet screen positions
            let (ax, ay) = orrery.planet_screen_pos(pa, cx, cy);
            let (bx, by) = orrery.planet_screen_pos(pb, cx, cy);

            // Draw quadratic bezier arc (control point = center with offset)
            draw_bezier_arc(s, [(ax, ay), (bx, by), (cx, cy)], arc_color);

            // Label near-perfect intervals
            if opts.show_labels && dev < 0.03 && cons > 0.7 {
                let mid_x = ((ax + bx) / 2.0) as i32;
                let mid_y = ((ay + by) / 2.0) as i32;
                let label_col = palette::dim(palette::GOLD, effective_fade * 0.9);
                font::draw_text(
                    s,
                    interval.name,
                    mid_x - 20,
                    mid_y - 8,
                    1,
                    label_col,
                );
                // Draw ratio label
                let ratio_str = format!("{}:{}", interval.num, interval.den);
                font::draw_text(
                    s,
                    &ratio_str,
                    mid_x - 8,
                    mid_y + 2,
                    1,
                    palette::dim(palette::AMBER_400, effective_fade),
                );
            }

            // Glow ring for near-perfect intervals
            if cons > 0.85 {
                let glow_alpha = ((cons - 0.85) / 0.15) as f32 * effective_fade * 0.5;
                let glow_col = palette::dim(palette::GOLD, glow_alpha);
                draw_bezier_arc(
                    s,
                    [(ax + 1.0, ay + 1.0), (bx + 1.0, by + 1.0), (cx, cy)],
                    glow_col,
                );
            }
        }
    }
}

/// Draw a quadratic Bezier arc from (ax,ay) to (bx,by).
/// Control point is 40% of the way from the midpoint toward the center,
/// creating a gentle inward curve (toward the sun).
fn draw_bezier_arc(
    s: &mut Surface,
    pts: [(f64, f64); 3],
    color: u32,
) {
    let (ax, ay) = pts[0];
    let (bx, by) = pts[1];
    let (cx, cy) = pts[2];
    if color == 0 {
        return;
    }

    // Control point: pull midpoint 30% toward center
    let mx = (ax + bx) / 2.0;
    let my = (ay + by) / 2.0;
    let cpx = mx + (cx - mx) * 0.3;
    let cpy = my + (cy - my) * 0.3;

    let steps = {
        let dx = bx - ax;
        let dy = by - ay;
        ((dx * dx + dy * dy).sqrt() as usize / 3).clamp(32, 256)
    };

    let mut last_x = ax as i32;
    let mut last_y = ay as i32;

    for step in 1..=steps {
        let t = step as f64 / steps as f64;
        let it = 1.0 - t;
        let px = it * it * ax + 2.0 * it * t * cpx + t * t * bx;
        let py = it * it * ay + 2.0 * it * t * cpy + t * t * by;
        let px = px as i32;
        let py = py as i32;

        bresenham(s, last_x, last_y, px, py, color);
        last_x = px;
        last_y = py;
    }
}

// ── Planet rendering ──────────────────────────────────────────

fn render_planet(
    s: &mut Surface,
    planet: &Planet,
    px: f64,
    py: f64,
    fade: f32,
    time: f64,
) {
    let _ = time;
    let xi = px as i32;
    let yi = py as i32;
    let vr = planet.visual_radius as i32;

    // Glow halo
    let glow_r = vr + 8;
    for dy in -glow_r..=glow_r {
        for dx in -glow_r..=glow_r {
            let d2 = dx * dx + dy * dy;
            if d2 < glow_r * glow_r {
                let d = (d2 as f32).sqrt();
                let alpha = (1.0 - d / glow_r as f32).powf(2.0) * fade * 0.4;
                if alpha < 0.01 {
                    continue;
                }
                let gc = palette::dim(planet.color, alpha);
                let bx2 = xi + dx;
                let by2 = yi + dy;
                if bx2 >= 0 && by2 >= 0 && bx2 < s.w as i32 && by2 < s.h as i32 {
                    s.buf[by2 as usize * s.w + bx2 as usize] =
                        palette::add_color(s.buf[by2 as usize * s.w + bx2 as usize], gc);
                }
            }
        }
    }

    // Planet body
    fill_circle(s, xi, yi, vr, palette::dim(planet.color, fade));

    // Bright specular highlight
    let spec_r = (vr / 3).max(1);
    fill_circle(
        s,
        xi - spec_r / 2,
        yi - spec_r / 2,
        spec_r,
        palette::dim(palette::WHITE, fade * 0.6),
    );
}

// ── Primitive drawing helpers ─────────────────────────────────

pub fn fill_circle(s: &mut Surface, cx: i32, cy: i32, r: i32, col: u32) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                    s.buf[py as usize * s.w + px as usize] = col;
                }
            }
        }
    }
}

pub fn bresenham(
    s: &mut Surface,
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
        if x >= 0 && y >= 0 && x < s.w as i32 && y < s.h as i32 {
            s.buf[y as usize * s.w + x as usize] =
                palette::add_color(s.buf[y as usize * s.w + x as usize], col);
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

/// Draw a circle ring (outline only)
pub fn ring_circle(
    s: &mut Surface,
    cx: i32,
    cy: i32,
    r: i32,
    thickness: i32,
    col: u32,
) {
    let ro = r;
    let ri = (r - thickness).max(0);
    for dy in -ro..=ro {
        for dx in -ro..=ro {
            let d2 = dx * dx + dy * dy;
            if d2 <= ro * ro && d2 >= ri * ri {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                    s.buf[py as usize * s.w + px as usize] =
                        palette::add_color(s.buf[py as usize * s.w + px as usize], col);
                }
            }
        }
    }
}
