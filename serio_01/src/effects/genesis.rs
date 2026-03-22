// ═══════════════════════════════════════════════════════════════
// GENESIS -- Scene 0: The orrery materializes from the void
//
// Sequence:
//   0.0s  Black. Then a point of light ignites (the sun).
//   1.5s  Greek text fades in: "ἁρμονία κόσμου"
//   3.0s  Planets materialize one by one, each adding a tone.
//   6.0s  Initial chaotic constraint arcs fill in (red web).
//   8.0s  "MUSICA UNIVERSALIS" title reveals.
//   9.0s  Subtitle: "SERIO INCREMENTAL SCORING ENGINE"
// ═══════════════════════════════════════════════════════════════

use crate::effects::orrery_render::{self, fill_circle, OrreryRenderOpts, TrailBuffer};
use crate::font;
use crate::logo;
use crate::orrery::model::{Orrery, PLANET_COUNT};
use crate::palette;

pub fn render(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    orrery: &Orrery,
    trail: &mut TrailBuffer,
    scene_t: f64, // seconds since scene start [0, 10]
    fade: f32,
    time: f64,
) {
    let cx = 640.0f64;
    let cy = 360.0f64;

    // ── Sun ignition (0.0 - 1.5s) ────────────────────────────
    let sun_alpha = smoothstep(0.0, 1.5, scene_t) as f32 * fade;

    // Render partial sun glow
    if sun_alpha > 0.01 {
        let glow_scale = (scene_t / 1.5).min(1.0) as f32;
        render_sun_birth(buffer, width, height, cx, cy, sun_alpha, glow_scale, time);
    }

    // ── Planets materialize one by one (3.0 - 8.0s) ─────────
    // Each planet appears 0.7s apart
    let planets_started = (scene_t - 3.0).max(0.0);
    let planets_visible = ((planets_started / 0.7) as usize).min(PLANET_COUNT);

    if planets_visible > 0 {
        // Build a partial orrery with only visible planets rendered
        // We use the same positions but fade per-planet
        let full_opts = OrreryRenderOpts {
            cx,
            cy,
            fade: sun_alpha,
            show_labels: planets_visible >= PLANET_COUNT,
            show_arcs: scene_t > 6.0,
            highlighted_pairs: None,
            flash_all: false,
            flash_alpha: 0.0,
            show_rings: sun_alpha > 0.3,
        };

        // Render full orrery with scaled fade
        orrery_render::render(buffer, width, height, orrery, trail, &full_opts, time);

        // Overlay individual planet materialization effect
        for (i, planet) in orrery.planets.iter().enumerate().take(planets_visible) {
            let planet_t = (planets_started - i as f64 * 0.7).clamp(0.0, 0.6);
            let planet_alpha = smoothstep(0.0, 0.6, planet_t) as f32 * fade;

            if planet_alpha < 0.01 {
                continue;
            }

            let (px, py) = orrery.planet_screen_pos(planet, cx, cy);

            // Materialization ring burst
            if planet_t < 0.3 {
                let ring_r = (planet_t / 0.3 * 30.0) as i32 + planet.visual_radius as i32;
                let ring_alpha = (1.0 - planet_t as f32 / 0.3) * planet_alpha * 0.7;
                let ring_col = palette::dim(planet.color, ring_alpha);
                orrery_render::ring_circle(
                    buffer, width, height, px as i32, py as i32, ring_r, 2, ring_col,
                );
            }
        }
    }

    // ── SolverForge logo (top-left corner, small) ────────────
    if scene_t > 1.0 {
        let logo_alpha = smoothstep(1.0, 3.0, scene_t) as f32 * fade;
        let logo_progress = smoothstep(1.0, 3.0, scene_t) as f32;
        logo::draw_logo(
            buffer,
            width,
            height,
            80.0,
            55.0,
            45.0,
            logo_progress,
            time,
            logo_alpha * 0.8,
        );
    }

    // ── Greek title ───────────────────────────────────────────
    // "harmonia kosmou" (harmony of the cosmos) — ASCII approximation
    if scene_t > 1.5 {
        let greek_alpha = smoothstep(1.5, 3.5, scene_t) as f32 * fade;
        // Use Latin transliteration since we only have ASCII font
        font::draw_text_centered_glow(
            buffer,
            width,
            height,
            "HARMONIA KOSMOU",
            640,
            80,
            3,
            palette::dim(palette::GOLD, greek_alpha),
            palette::dim(palette::AMBER_400, greek_alpha * 0.5),
        );
        font::draw_text_centered(
            buffer,
            width,
            height,
            "Harmony of the Cosmos",
            640,
            110,
            1,
            palette::dim(palette::CHROME, greek_alpha * 0.6),
        );
    }

    // ── "MUSICA UNIVERSALIS" title (8.0s) ─────────────────────
    if scene_t > 8.0 {
        let title_alpha = smoothstep(8.0, 9.0, scene_t) as f32 * fade;
        font::draw_text_centered_glow(
            buffer,
            width,
            height,
            "MUSICA UNIVERSALIS",
            640,
            height as i32 - 100,
            3,
            palette::dim(palette::EMERALD_400, title_alpha),
            palette::dim(palette::EMERALD_800, title_alpha * 0.5),
        );
    }

    // ── SERIO subtitle (9.0s) ─────────────────────────────────
    if scene_t > 9.0 {
        let sub_alpha = smoothstep(9.0, 10.0, scene_t) as f32 * fade;
        font::draw_text_centered(
            buffer,
            width,
            height,
            "SERIO INCREMENTAL SCORING ENGINE",
            640,
            height as i32 - 72,
            1,
            palette::dim(palette::EMERALD_500, sub_alpha),
        );
        font::draw_text_centered(
            buffer,
            width,
            height,
            "SCORING ENGINE FOR REAL-TIME INCREMENTAL OPTIMIZATION",
            640,
            height as i32 - 58,
            1,
            palette::dim(palette::CHROME, sub_alpha * 0.5),
        );
    }
}

fn render_sun_birth(
    buf: &mut [u32],
    w: usize,
    h: usize,
    cx: f64,
    cy: f64,
    alpha: f32,
    glow_scale: f32,
    time: f64,
) {
    let pulse = ((time * 3.0).sin() as f32 * 0.15 + 0.85).abs();
    let core_r = (12.0 * glow_scale * pulse) as i32;
    let cxi = cx as i32;
    let cyi = cy as i32;

    // Birth glow — expanding ring
    let ring_r = (glow_scale * 80.0) as i32;
    if ring_r > 1 {
        let ring_alpha = (1.0 - glow_scale) * alpha * 0.5;
        if ring_alpha > 0.01 {
            orrery_render::ring_circle(
                buf,
                w,
                h,
                cxi,
                cyi,
                ring_r,
                3,
                palette::dim(palette::rgb(255, 230, 100), ring_alpha),
            );
        }
    }

    // Inner glow
    for gy in -50i32..=50 {
        for gx in -50i32..=50 {
            let d2 = (gx * gx + gy * gy) as f32;
            if d2 < 2500.0 {
                let d = d2.sqrt();
                let a = (1.0 - d / 50.0).powf(2.0) * alpha * glow_scale * 0.6;
                if a < 0.01 {
                    continue;
                }
                let gc = palette::rgb((255.0 * a) as u8, (200.0 * a) as u8, (60.0 * a) as u8);
                let px = cxi + gx;
                let py = cyi + gy;
                if px >= 0 && py >= 0 && px < w as i32 && py < h as i32 {
                    buf[py as usize * w + px as usize] =
                        palette::add_color(buf[py as usize * w + px as usize], gc);
                }
            }
        }
    }

    fill_circle(
        buf,
        w,
        h,
        cxi,
        cyi,
        core_r,
        palette::dim(palette::rgb(255, 245, 160), alpha),
    );
    fill_circle(
        buf,
        w,
        h,
        cxi,
        cyi,
        (core_r * 2 / 3).max(1),
        palette::dim(palette::WHITE, alpha),
    );
}

/// Smooth Hermite interpolation
fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
