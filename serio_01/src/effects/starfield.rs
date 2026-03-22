// ═══════════════════════════════════════════════════════════════
// STARFIELD -- 3D warp tunnel through the cosmos
// Classic demoscene. Flying through the SolverForge universe.
// ═══════════════════════════════════════════════════════════════

use crate::palette;

const NUM_STARS: usize = 800;
const MAX_DEPTH: f32 = 512.0;

#[derive(Clone, Copy)]
struct Star {
    x: f32, // -1.0 to 1.0 (normalized space)
    y: f32,
    z: f32, // depth: MAX_DEPTH (far) to 0 (close)
}

pub struct Starfield {
    stars: Vec<Star>,
    warp_factor: f32,
}

impl Starfield {
    pub fn new() -> Self {
        let mut stars = Vec::with_capacity(NUM_STARS);
        // Simple deterministic init -- no rand dependency
        for i in 0..NUM_STARS {
            stars.push(Star {
                x: Self::pseudo_rand(i * 3 + 0),
                y: Self::pseudo_rand(i * 3 + 1),
                z: Self::pseudo_rand(i * 3 + 2) * MAX_DEPTH,
            });
        }
        Self {
            stars,
            warp_factor: 0.0,
        }
    }

    /// Simple deterministic pseudo-random in [-1, 1]
    fn pseudo_rand(seed: usize) -> f32 {
        let x = (seed.wrapping_mul(2654435761) ^ seed.wrapping_mul(1234567891)) as u32;
        let x = x ^ (x >> 16);
        let x = x.wrapping_mul(0x45d9f3b);
        let x = x ^ (x >> 16);
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    pub fn update(&mut self, dt: f64, warp: f32) {
        self.warp_factor = warp;
        let speed = (30.0 + warp * 300.0) * dt as f32;

        for (i, star) in self.stars.iter_mut().enumerate() {
            star.z -= speed;

            if star.z <= 0.5 {
                // Respawn star at back
                star.x = Self::pseudo_rand(i * 7 + (star.z as usize).wrapping_add(42));
                star.y = Self::pseudo_rand(i * 7 + 1 + (star.z as usize).wrapping_add(99));
                star.z = MAX_DEPTH;
            }
        }
    }

    pub fn render(&self, buffer: &mut [u32], width: usize, height: usize, fade: f32, time: f64) {
        let w2 = width as f32 / 2.0;
        let h2 = height as f32 / 2.0;
        let fov = w2 * 1.2; // field of view

        for star in &self.stars {
            if star.z <= 0.1 {
                continue;
            }

            // Project to screen
            let sx = (star.x * fov / star.z + w2) as i32;
            let sy = (star.y * fov / star.z + h2) as i32;

            if sx < 0 || sy < 0 || sx >= width as i32 || sy >= height as i32 {
                continue;
            }

            // Brightness based on depth (closer = brighter)
            let t = (1.0 - star.z / MAX_DEPTH).powf(1.5);
            let brightness = t * fade;

            // Size based on proximity
            let star_size = if t > 0.9 {
                3
            } else if t > 0.7 {
                2
            } else {
                1
            };

            let color = palette::star_color(brightness);

            // Draw star pixel(s)
            for dy in -(star_size as i32)..=star_size as i32 {
                for dx in -(star_size as i32)..=star_size as i32 {
                    if dx * dx + dy * dy <= star_size * star_size {
                        let px = sx + dx;
                        let py = sy + dy;
                        if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                            let col = palette::dim(color, brightness);
                            buffer[py as usize * width + px as usize] =
                                palette::add_color(buffer[py as usize * width + px as usize], col);
                        }
                    }
                }
            }

            // Streak effect during warp
            if self.warp_factor > 0.3 {
                let streak_len = (self.warp_factor * 60.0 * (1.0 - star.z / MAX_DEPTH)) as i32;

                // Draw streak toward center
                let dx_sign = if sx < width as i32 / 2 { 1 } else { -1 };
                let dy_sign = if sy < height as i32 / 2 { 1 } else { -1 };

                for s in 0..streak_len {
                    let px = sx + dx_sign * s;
                    let py = sy + dy_sign * s;
                    if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                        let fade_s = (1.0 - s as f32 / streak_len as f32) * brightness * 0.5;
                        let sc = palette::dim(palette::EMERALD_500, fade_s);
                        buffer[py as usize * width + px as usize] =
                            palette::add_color(buffer[py as usize * width + px as usize], sc);
                    }
                }
            }
        }

        // Tunnel rings during heavy warp
        if self.warp_factor > 0.6 {
            let ring_alpha = (self.warp_factor - 0.6) * 2.5 * fade;
            self.render_tunnel_rings(buffer, width, height, time, ring_alpha);
        }
    }

    fn render_tunnel_rings(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        time: f64,
        alpha: f32,
    ) {
        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;
        let num_rings = 12;

        for i in 0..num_rings {
            let phase = (i as f64 / num_rings as f64 + time * 0.8) % 1.0;
            let ring_r = phase as f32 * width.min(height) as f32 * 0.5;
            let ring_alpha = (1.0 - phase as f32) * alpha;

            if ring_alpha < 0.01 {
                continue;
            }

            // Draw ring as sequence of points
            let ring_steps = (ring_r * std::f32::consts::TAU) as usize + 32;
            for step in 0..ring_steps {
                let a = step as f32 * std::f32::consts::TAU / ring_steps as f32;
                let px = (cx + a.cos() * ring_r) as i32;
                let py = (cy + a.sin() * ring_r) as i32;
                if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                    let ring_color = palette::dim(palette::EMERALD_400, ring_alpha);
                    buffer[py as usize * width + px as usize] =
                        palette::add_color(buffer[py as usize * width + px as usize], ring_color);
                }
            }
        }
    }
}
