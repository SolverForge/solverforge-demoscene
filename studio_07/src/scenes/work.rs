// WORK: the proof. Stars rush past, a wireframe octagon (the shape of
// SolverForge's mark) turns, copper bars roll, and a sine scroller names
// the published projects.
use super::{EMERALD, LIME, MINT};
use crate::gfx::{advance, hash, ramp, Frame, W};
use crate::timeline::Now;

const SCROLL: &str = "SOLVERFORGE  *  SOLVERFORGE-PY  *  PLANNER123  *  FRANKING  *  LUMEN  *  COMPUTER-USE-SWAY  *  SESHAT  *  GLYPHFX  *  TRANCHE  *  ELPHAME  *  RICESPACE  *  MERIDIAN  *  GITNAGA  *  NISABA  *  TREX";
const SCALE: i32 = 3;
const BASE_Y: f32 = 590.0;

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.clear(0x010203);

    // Stars flying out of the centre.
    for i in 0..360 {
        let z = (hash(i, 1) - n.local * 0.22).rem_euclid(1.0).max(0.02);
        let (sx, sy) = ((hash(i, 2) - 0.5) * 2.0, (hash(i, 3) - 0.5) * 2.0);
        let (x, y) = (640.0 + sx / z * 140.0, 300.0 + sy / z * 140.0);
        let a = (1.0 - z) * fade;
        let c = if i % 7 == 0 { MINT } else { 0xdfe8ff };
        f.blend(x as i32, y as i32, c, a);
        if z < 0.25 {
            f.blend(x as i32 + 1, y as i32, c, a * 0.6);
        }
    }

    octagon(f, n.local, fade);

    // Copper bars behind the scroller.
    let bars = [0x10b981, 0x50fc00, 0x38bdf8, 0xa78bfa, 0xffc61a];
    for (i, c) in bars.iter().enumerate() {
        let y = BASE_Y + 10.0 + (t * 1.7 + i as f32 * 0.9).sin() * 70.0;
        for dy in -12..=12 {
            let k = 1.0 - (dy as f32 / 12.0).abs();
            f.rect(
                0,
                y as i32 + dy,
                W as i32,
                y as i32 + dy + 1,
                *c,
                0.42 * k * k * fade,
            );
        }
    }

    // The sine scroller enters from the right over the section.
    let total = SCROLL.chars().count() as f32 * advance(SCALE) as f32 + W as f32;
    let x0 = W as f32 - total * ((n.local - 0.4) / (n.len - 1.2)).clamp(0.0, 1.0);
    for (i, ch) in SCROLL.chars().enumerate() {
        let x = x0 + (i as i32 * advance(SCALE)) as f32;
        if x < -40.0 || x > W as f32 {
            continue;
        }
        let y = BASE_Y - 14.0 + (x * 0.0055 + t * 2.2).sin() * 30.0;
        let c = ramp(
            &[LIME, EMERALD, 0x38bdf8, LIME],
            (x / W as f32 + t * 0.1).fract(),
        );
        f.char_at(x as i32 + 3, y as i32 + 3, ch, SCALE, 0x000000, 0.6 * fade);
        f.char_at(x as i32, y as i32, ch, SCALE, c, fade);
    }
    1.0
}

/// A turning octagonal prism, wireframe.
fn octagon(f: &mut Frame, local: f32, a: f32) {
    let (ry, rx) = (local * 0.7, 0.45 + 0.25 * (local * 0.4).sin());
    let (cx, cy, s) = (640.0, 310.0, 175.0);
    let mut pts = Vec::with_capacity(16);
    for z in [-0.45f32, 0.45] {
        for k in 0..8 {
            let ang = k as f32 * std::f32::consts::TAU / 8.0 + std::f32::consts::PI / 8.0;
            let (x, y) = (ang.cos(), ang.sin());
            // Rotate around y, then x.
            let (x1, z1) = (x * ry.cos() + z * ry.sin(), -x * ry.sin() + z * ry.cos());
            let (y2, z2) = (y * rx.cos() - z1 * rx.sin(), y * rx.sin() + z1 * rx.cos());
            let p = 2.6 / (3.4 + z2);
            pts.push((cx + x1 * s * p, cy + y2 * s * p, z2));
        }
    }
    let edge = |f: &mut Frame, i: usize, j: usize| {
        let (p, q) = (pts[i], pts[j]);
        let depth = 0.55 + 0.45 * (1.0 - (p.2 + q.2) * 0.5).clamp(0.0, 1.0);
        f.line(p.0, p.1, q.0, q.1, 2.0, EMERALD, a * depth, 1.0);
    };
    for k in 0..8 {
        edge(f, k, (k + 1) % 8);
        edge(f, 8 + k, 8 + (k + 1) % 8);
        edge(f, k, 8 + k);
    }
    for p in &pts {
        f.glow(p.0, p.1, 10.0, MINT, 0.5 * a);
    }
}
