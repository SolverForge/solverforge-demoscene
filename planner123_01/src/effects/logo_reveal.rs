// ═══════════════════════════════════════════════════════════════
// LOGO REVEAL -- SolverForge entrance sequence
// Particle spray, scanline sweep, brand identity moment.
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::logo;
use crate::palette::{self, EMERALD_300, EMERALD_400, EMERALD_500, EMERALD_600, GREEN_500, WHITE};

const NUM_PARTICLES: usize = 200;

#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    color: u32,
    size: i32,
}

impl Particle {
    fn dead() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.0,
            max_life: 1.0,
            color: 0,
            size: 1,
        }
    }
}

pub struct LogoReveal {
    particles: Vec<Particle>,
    scanline_y: f32,
    pub phase: f32,
    spawn_timer: f32,
    particle_seed: usize,
}

impl LogoReveal {
    pub fn new() -> Self {
        Self {
            particles: vec![Particle::dead(); NUM_PARTICLES],
            scanline_y: 0.0,
            phase: 0.0,
            spawn_timer: 0.0,
            particle_seed: 0,
        }
    }

    fn pseudo_rand(&mut self) -> f32 {
        self.particle_seed = self
            .particle_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        let x = (self.particle_seed >> 16) as u16;
        x as f32 / 65535.0
    }

    pub fn update(&mut self, dt: f64, width: usize, height: usize) {
        let dt = dt as f32;
        self.phase = (self.phase + dt * 0.35).min(1.0);

        // Scanline sweep: moves top-to-bottom during reveal
        if self.phase < 0.6 {
            self.scanline_y = (self.phase / 0.6) * height as f32;
        }

        // Spawn particles from logo center
        self.spawn_timer += dt;
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;

        if self.spawn_timer > 0.02 && self.phase > 0.1 && self.phase < 0.95 {
            self.spawn_timer = 0.0;
            // Pre-compute all random values before borrowing particles
            let angle = self.pseudo_rand() * std::f32::consts::TAU;
            let speed = self.pseudo_rand() * 180.0 + 40.0;
            let lifetime = self.pseudo_rand() * 1.2 + 0.4;
            let ci_rand = self.pseudo_rand();
            let vx_rand = self.pseudo_rand();
            let vy_rand = self.pseudo_rand();
            let size_rand = self.pseudo_rand();

            let radius = height.min(width) as f32 * 0.22;
            let ox = angle.cos() * radius;
            let oy = angle.sin() * radius;
            let colors = [EMERALD_300, EMERALD_400, EMERALD_500, GREEN_500];
            let ci = (ci_rand * colors.len() as f32) as usize;
            let chosen_color = colors[ci.min(colors.len() - 1)];
            let chosen_size = if size_rand > 0.7 { 2 } else { 1 };

            // Find a dead particle slot and spawn
            for p in &mut self.particles {
                if p.life <= 0.0 {
                    *p = Particle {
                        x: cx + ox,
                        y: cy + oy,
                        vx: angle.cos() * speed + (vx_rand - 0.5) * 60.0,
                        vy: angle.sin() * speed + (vy_rand - 0.5) * 60.0,
                        life: lifetime,
                        max_life: lifetime,
                        color: chosen_color,
                        size: chosen_size,
                    };
                    break;
                }
            }
        }

        // Update particles
        for p in &mut self.particles {
            if p.life <= 0.0 {
                continue;
            }
            p.life -= dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += 40.0 * dt; // subtle gravity
            p.vx *= 0.98; // air resistance
        }
    }

    pub fn render(&self, buffer: &mut [u32], width: usize, height: usize, time: f64, global_fade: f32) {
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let logo_radius = height.min(width) as f32 * 0.22;

        // Dark background with vignette
        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 / width as f32 - 0.5;
                let dy = y as f32 / height as f32 - 0.5;
                let dist = (dx * dx + dy * dy).sqrt() * 1.6;
                let vignette = (1.0 - dist.min(1.0)).powf(1.5);
                let bg = palette::dim(palette::NEAR_BLACK, vignette * global_fade);
                buffer[y * width + x] = palette::add_color(buffer[y * width + x], bg);
            }
        }

        // Scanline sweep effect (white line moving down the screen)
        if self.phase < 0.7 {
            let scan_y = self.scanline_y as i32;
            let scan_brightness = (1.0 - self.phase / 0.7) * 0.8 * global_fade;
            for dy in -2..=2i32 {
                let sy = scan_y + dy;
                if sy >= 0 && sy < height as i32 {
                    let brightness = scan_brightness * (1.0 - dy.abs() as f32 * 0.3);
                    for x in 0..width {
                        let sc = palette::dim(EMERALD_400, brightness);
                        buffer[sy as usize * width + x] =
                            palette::add_color(buffer[sy as usize * width + x], sc);
                    }
                }
            }
        }

        // Draw the ouroboros logo (progressive reveal)
        let logo_progress = (self.phase * 2.0).min(1.0);
        logo::draw_logo(
            buffer,
            width,
            height,
            cx,
            cy,
            logo_radius,
            logo_progress,
            time,
            global_fade * logo_progress,
        );

        // Particles
        for p in &self.particles {
            if p.life <= 0.0 {
                continue;
            }
            let life_t = p.life / p.max_life;
            let px = p.x as i32;
            let py = p.y as i32;
            let pc = palette::dim(p.color, life_t * life_t * global_fade);

            for dy in 0..p.size {
                for dx in 0..p.size {
                    let ppx = px + dx;
                    let ppy = py + dy;
                    if ppx >= 0 && ppy >= 0 && ppx < width as i32 && ppy < height as i32 {
                        buffer[ppy as usize * width + ppx as usize] =
                            palette::add_color(buffer[ppy as usize * width + ppx as usize], pc);
                    }
                }
            }
        }

        // Text reveal (fades in after logo is drawn)
        if self.phase > 0.5 {
            let text_alpha = ((self.phase - 0.5) * 3.0).min(1.0) * global_fade;
            let ty = cy as i32 + logo_radius as i32 + 30;

            // "SOLVER" in white, "FORGE" in emerald
            let solver_w = font::text_width("SOLVER", 4);
            let forge_w = font::text_width("FORGE", 4);
            let total_w = solver_w + 4 * 5 + forge_w; // 4 = scale, 5 = char advance
            let tx = cx as i32 - total_w / 2;

            font::draw_text_glow(
                buffer,
                width,
                height,
                "SOLVER",
                tx,
                ty,
                4,
                palette::dim(WHITE, text_alpha),
                palette::dim(EMERALD_300, text_alpha * 0.5),
            );

            font::draw_text_glow(
                buffer,
                width,
                height,
                "FORGE",
                tx + solver_w + 5 * 4,
                ty,
                4,
                palette::dim(EMERALD_500, text_alpha),
                palette::dim(EMERALD_300, text_alpha * 0.7),
            );

            // Subtitle
            if self.phase > 0.75 {
                let sub_alpha = ((self.phase - 0.75) * 4.0).min(1.0) * global_fade;
                let sub_y = ty + 8 * 4 + 16;
                font::draw_text_centered(
                    buffer,
                    width,
                    height,
                    "CONSTRAINT ENGINE",
                    cx as i32,
                    sub_y,
                    2,
                    palette::dim(EMERALD_400, sub_alpha * 0.8),
                );

                if self.phase > 0.88 {
                    let tag_alpha = ((self.phase - 0.88) * 8.0).min(1.0) * global_fade;
                    font::draw_text_centered(
                        buffer,
                        width,
                        height,
                        "PLANNER123",
                        cx as i32,
                        sub_y + 8 * 2 + 24,
                        3,
                        palette::dim(WHITE, tag_alpha),
                    );
                }
            }
        }

        // Corner brackets -- Swiss design frame
        if self.phase > 0.6 {
            let bracket_alpha = ((self.phase - 0.6) * 2.5).min(1.0) * global_fade * 0.5;
            let bc = palette::dim(EMERALD_600, bracket_alpha);
            let margin = 40usize;
            let blen = 80usize;

            // Top-left
            for i in 0..blen {
                if margin + i < width {
                    buffer[margin * width + margin + i] =
                        palette::add_color(buffer[margin * width + margin + i], bc);
                }
                if margin + i < height {
                    buffer[(margin + i) * width + margin] =
                        palette::add_color(buffer[(margin + i) * width + margin], bc);
                }
            }
            // Top-right
            for i in 0..blen {
                if width.saturating_sub(margin + i) < width {
                    buffer[margin * width + width - margin - i - 1] =
                        palette::add_color(buffer[margin * width + width - margin - i - 1], bc);
                }
                if margin + i < height {
                    buffer[(margin + i) * width + width - margin - 1] =
                        palette::add_color(buffer[(margin + i) * width + width - margin - 1], bc);
                }
            }
            // Bottom-left
            for i in 0..blen {
                if margin + i < width {
                    buffer[(height - margin - 1) * width + margin + i] =
                        palette::add_color(buffer[(height - margin - 1) * width + margin + i], bc);
                }
                if i < height.saturating_sub(margin) {
                    buffer[(height - margin - i - 1) * width + margin] =
                        palette::add_color(buffer[(height - margin - i - 1) * width + margin], bc);
                }
            }
            // Bottom-right
            for i in 0..blen {
                if width.saturating_sub(margin + i) < width {
                    buffer[(height - margin - 1) * width + width - margin - i - 1] =
                        palette::add_color(
                            buffer[(height - margin - 1) * width + width - margin - i - 1],
                            bc,
                        );
                }
                if i < height.saturating_sub(margin) {
                    buffer[(height - margin - i - 1) * width + width - margin - 1] =
                        palette::add_color(
                            buffer[(height - margin - i - 1) * width + width - margin - 1],
                            bc,
                        );
                }
            }
        }
    }
}
