// ═══════════════════════════════════════════════════════════════
// GENESIS -- Scene 0: The orrery materializes from the void
//
// Sequence:
//   0.0s  Black. Then a point of light ignites (the sun).
//   1.5s  "HARMONIA KOSMOU" fades in.
//   3.0s  Planets materialize one by one, each adding a tone.
//   6.0s  Initial chaotic constraint arcs fill in (red web).
//   8.0s  "MUSICA UNIVERSALIS" title reveals.
//   9.0s  Subtitle: "SERIO INCREMENTAL SCORING ENGINE"
// ═══════════════════════════════════════════════════════════════

use crate::effects::orrery_render::{self, fill_circle, OrreryRenderOpts, TrailBuffer};
use crate::font;
use crate::logo;
use crate::orrery::model::{Orrery, PLANET_COUNT};
use crate::palette::{self, Surface};

pub fn render(
    s: &mut Surface,
    orrery: &Orrery,
    trail: &mut TrailBuffer,
    scene_t: f64,
    fade: f32,
    time: f64,
) {
    let cx = 640.0f64;
    let cy = 360.0f64;

    // ── Sun ignition (0.0 - 1.5s) ────────────────────────────
    let sun_alpha = palette::smoothstep(0.0, 1.5, scene_t) as f32 * fade;

    if sun_alpha > 0.01 {
        let glow_scale = (scene_t / 1.5).min(1.0) as f32;
        render_sun_birth(s, cx, cy, sun_alpha, glow_scale, time);
    }

    // ── Planets materialize one by one (3.0 - 8.0s) ─────────
    let planets_started = (scene_t - 3.0).max(0.0);
    let planets_visible = ((planets_started / 0.7) as usize).min(PLANET_COUNT);

    if planets_visible > 0 {
        // Arc reveal: nothing until 10s, first arc at 10-10.5s, rest fade in 10.5-13s
        let arc_reveal = if scene_t < 10.0 {
            0.0_f32
        } else if scene_t < 10.5 {
            // First arc only (1/21 ≈ 0.048)
            0.048
        } else if scene_t < 13.0 {
            // Remaining arcs fade in
            0.048 + 0.952 * palette::smoothstep(10.5, 13.0, scene_t) as f32
        } else {
            1.0
        };

        let full_opts = OrreryRenderOpts {
            cx,
            cy,
            fade: sun_alpha,
            show_labels: planets_visible >= PLANET_COUNT && scene_t > 12.0,
            show_arcs: arc_reveal,
            highlighted_pairs: None,
            flash_all: false,
            flash_alpha: 0.0,
            show_rings: sun_alpha > 0.3,
            moved_planet: None,
            move_ring_alpha: 0.0,
            sweep_frontier: -1.0,
        };

        orrery_render::render(s, orrery, trail, &full_opts, time);

        // Materialization ring burst per planet
        for (i, planet) in orrery.planets.iter().enumerate().take(planets_visible) {
            let planet_t = (planets_started - i as f64 * 0.7).clamp(0.0, 0.6);
            let planet_alpha = palette::smoothstep(0.0, 0.6, planet_t) as f32 * fade;

            if planet_alpha < 0.01 {
                continue;
            }

            let (px, py) = orrery.planet_screen_pos(planet, cx, cy);

            if planet_t < 0.3 {
                let ring_r = (planet_t / 0.3 * 30.0) as i32 + planet.visual_radius as i32;
                let ring_alpha = (1.0 - planet_t as f32 / 0.3) * planet_alpha * 0.7;
                let ring_col = palette::dim(planet.color, ring_alpha);
                orrery_render::ring_circle(
                    s, px as i32, py as i32, ring_r, 2, ring_col,
                );
            }
        }
    }

    // ── SolverForge logo (top-left corner, small) ────────────
    if scene_t > 1.0 {
        let logo_alpha = palette::smoothstep(1.0, 3.0, scene_t) as f32 * fade;
        let logo_progress = palette::smoothstep(1.0, 3.0, scene_t) as f32;
        logo::draw_logo(
            s,
            80.0,
            55.0,
            45.0,
            logo_progress,
            time,
            logo_alpha * 0.8,
        );
    }

    // ── Greek title ───────────────────────────────────────────
    if scene_t > 1.5 {
        let greek_alpha = palette::smoothstep(1.5, 3.5, scene_t) as f32 * fade;
        font::draw_text_centered_glow(
            s,
            "HARMONIA KOSMOU",
            640,
            80,
            3,
            palette::dim(palette::GOLD, greek_alpha),
            palette::dim(palette::AMBER_400, greek_alpha * 0.5),
        );
        font::draw_text_centered(
            s,
            "The Harmony of the Spheres",
            640,
            110,
            1,
            palette::dim(palette::CHROME, greek_alpha * 0.6),
        );
    }

    // ── Demo title (8.0s) ───────────────────────────────────────
    if scene_t > 8.0 {
        let title_alpha = palette::smoothstep(8.0, 9.0, scene_t) as f32 * fade;
        font::draw_text_centered_glow(
            s,
            "MUSICA UNIVERSALIS",
            640,
            s.h as i32 - 100,
            3,
            palette::dim(palette::EMERALD_400, title_alpha),
            palette::dim(palette::EMERALD_800, title_alpha * 0.5),
        );
    }

    // ── SERIO subtitle (9.0s) ─────────────────────────────────
    if scene_t > 9.0 {
        let sub_alpha = palette::smoothstep(9.0, 10.0, scene_t) as f32 * fade;
        font::draw_text_centered(
            s,
            "A Meditation on Incremental Evaluation",
            640,
            s.h as i32 - 72,
            1,
            palette::dim(palette::EMERALD_500, sub_alpha),
        );
        font::draw_text_centered(
            s,
            "SolverForge -- 2026",
            640,
            s.h as i32 - 58,
            1,
            palette::dim(palette::CHROME, sub_alpha * 0.5),
        );
    }
}

fn render_sun_birth(
    s: &mut Surface,
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

    // Birth glow -- expanding ring
    let ring_r = (glow_scale * 80.0) as i32;
    if ring_r > 1 {
        let ring_alpha = (1.0 - glow_scale) * alpha * 0.5;
        if ring_alpha > 0.01 {
            orrery_render::ring_circle(
                s,
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
                if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                    s.buf[py as usize * s.w + px as usize] =
                        palette::add_color(s.buf[py as usize * s.w + px as usize], gc);
                }
            }
        }
    }

    fill_circle(
        s,
        cxi,
        cyi,
        core_r,
        palette::dim(palette::rgb(255, 245, 160), alpha),
    );
    fill_circle(
        s,
        cxi,
        cyi,
        (core_r * 2 / 3).max(1),
        palette::dim(palette::WHITE, alpha),
    );
}

