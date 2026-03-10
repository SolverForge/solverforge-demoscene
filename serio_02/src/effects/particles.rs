// ═══════════════════════════════════════════════════════════════
// PARTICLES -- Retract/Insert arc animation + score delta floaters
//
// When the solver moves a planet:
//   RETRACT: particles dissolve outward along affected arcs (blue)
//   INSERT:  particles bloom inward along new arc positions (gold/emerald)
//
// Delta floaters: "+3" / "-2" drift upward from the moved planet.
// ═══════════════════════════════════════════════════════════════

use crate::font;
use crate::orrery::model::Orrery;
use crate::palette::{self, Surface};

const MAX_PARTICLES: usize = 512;
const PARTICLE_LIFETIME: f32 = 0.4; // seconds
const MAX_FLOATERS: usize = 16;
const FLOATER_LIFETIME: f32 = 1.2; // seconds

#[derive(Clone)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32, // 0.0 = dead, 1.0 = just spawned
    color: u32,
}

impl Particle {
    fn is_alive(&self) -> bool {
        self.life > 0.0
    }
}

#[derive(Clone)]
struct DeltaFloater {
    x: f32,
    y: f32,
    vy: f32,
    life: f32,
    text: String,
    color: u32,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    floaters: Vec<DeltaFloater>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::with_capacity(MAX_PARTICLES),
            floaters: Vec::with_capacity(MAX_FLOATERS),
        }
    }

    /// Called when a solver move happens.
    /// Spawns retract particles (dissolving from old arcs) and
    /// after the move, insert particles (forming new arcs).
    pub fn spawn_move(
        &mut self,
        orrery: &Orrery,
        planet_idx: usize,
        cx: f64,
        cy: f64,
        delta_soft: i64,
        accepted: bool,
    ) {
        let planet = &orrery.planets[planet_idx];
        let (px, py) = orrery.planet_screen_pos(planet, cx, cy);

        // Spawn arc-particle burst from the moved planet toward each neighbor
        for (j, other) in orrery.planets.iter().enumerate() {
            if j == planet_idx {
                continue;
            }
            let (ox, oy) = orrery.planet_screen_pos(other, cx, cy);

            // Direction: from planet toward other
            let dx = (ox - px) as f32;
            let dy = (oy - py) as f32;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            let ndx = dx / dist;
            let ndy = dy / dist;

            // Retract: particles scatter from planet outward
            for k in 0..6 {
                if self.particles.len() >= MAX_PARTICLES {
                    break;
                }
                let spread = (k as f32 - 2.5) * 0.15;
                let speed = 40.0 + k as f32 * 8.0;
                self.particles.push(Particle {
                    x: px as f32,
                    y: py as f32,
                    vx: (ndx + spread * ndy) * speed,
                    vy: (ndy - spread * ndx) * speed,
                    life: 1.0,
                    color: palette::rgb(80, 140, 255), // blue-white retract
                });
            }

            // Insert: particles flow from other planet toward this one
            for k in 0..5 {
                if self.particles.len() >= MAX_PARTICLES {
                    break;
                }
                let spread = (k as f32 - 2.0) * 0.12;
                let speed = 50.0 + k as f32 * 10.0;
                let insert_col = if accepted {
                    palette::EMERALD_400
                } else {
                    palette::dim(palette::RUST, 0.8)
                };
                self.particles.push(Particle {
                    x: ox as f32,
                    y: oy as f32,
                    vx: (-ndx + spread * ndy) * speed,
                    vy: (-ndy - spread * ndx) * speed,
                    life: 1.0,
                    color: insert_col,
                });
            }
        }

        // Score delta floater
        if delta_soft.abs() > 0 && self.floaters.len() < MAX_FLOATERS {
            let sign = if delta_soft > 0 { "+" } else { "" };
            let text = format!("{}{}", sign, delta_soft);
            let color = if delta_soft > 0 {
                palette::SCORE_IMPROVE
            } else if delta_soft < 0 {
                palette::SCORE_WORSEN
            } else {
                palette::SCORE_NEUTRAL
            };
            self.floaters.push(DeltaFloater {
                x: px as f32,
                y: py as f32 - 20.0,
                vy: -25.0,
                life: 1.0,
                text,
                color,
            });
        }
    }

    /// Update particles and floaters each frame.
    pub fn update(&mut self, dt: f64) {
        let dt32 = dt as f32;
        let decay = dt32 / PARTICLE_LIFETIME;
        let fdecay = dt32 / FLOATER_LIFETIME;

        for p in self.particles.iter_mut() {
            if !p.is_alive() {
                continue;
            }
            p.x += p.vx * dt32;
            p.y += p.vy * dt32;
            // Slight drag
            p.vx *= 0.92;
            p.vy *= 0.92;
            p.life = (p.life - decay).max(0.0);
        }

        for f in self.floaters.iter_mut() {
            if f.life <= 0.0 {
                continue;
            }
            f.y += f.vy * dt32;
            f.vy *= 0.97;
            f.life = (f.life - fdecay).max(0.0);
        }

        // Compact dead particles periodically
        if self.particles.len() > MAX_PARTICLES / 2 {
            self.particles.retain(|p| p.is_alive());
        }
        self.floaters.retain(|f| f.life > 0.0);
    }

    /// Render all live particles and floaters.
    pub fn render(&self, s: &mut Surface) {
        for p in &self.particles {
            if !p.is_alive() {
                continue;
            }
            let alpha = p.life * p.life; // quadratic fade
            let col = palette::dim(p.color, alpha);
            let px = p.x as i32;
            let py = p.y as i32;
            if px >= 0 && py >= 0 && px < s.w as i32 && py < s.h as i32 {
                s.buf[py as usize * s.w + px as usize] =
                    palette::add_color(s.buf[py as usize * s.w + px as usize], col);
                // 2x2 for brighter particles
                if alpha > 0.5 && px + 1 < s.w as i32 {
                    s.buf[py as usize * s.w + (px + 1) as usize] = palette::add_color(
                        s.buf[py as usize * s.w + (px + 1) as usize],
                        palette::dim(col, 0.5),
                    );
                }
            }
        }

        for f in &self.floaters {
            if f.life <= 0.0 {
                continue;
            }
            let alpha = f.life * f.life;
            let col = palette::dim(f.color, alpha);
            font::draw_text(
                s,
                &f.text,
                f.x as i32 - (f.text.len() as i32 * 5 / 2),
                f.y as i32,
                1,
                col,
            );
        }
    }
}
