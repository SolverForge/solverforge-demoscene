// MINE: up behind the falls, through the old iron gallery of Bondione,
// where the ore was dug for the forges down the valley. Timber frames,
// rails, lanterns swaying on their chains. Ore veins thread the wall; light
// enters behind them and spreads along the veins at the speed of the
// shortest way (Dijkstra's wavefront, veins.rs), knot by knot, until it
// finds the geode ahead: then the shortest path burns gold and the crystal
// in its niche turns, a vector octahedron in the old Amiga way.
use std::f32::consts::TAU;

use super::party::{self, Light, PACE};
use super::stage::{cell_px, Stage};
use crate::ascii::{COLS, ROWS};
use crate::fx::{h2, noise, window};
use crate::gfx::{mix, scale};
use crate::veins::Veins;

const FEET: i32 = 50;
const CEIL: i32 = 11;
const FLOOR: i32 = 51;
/// The light enters at FRONT.0 and the front moves at a speed that brings
/// it to the geode at FRONT.1.
pub const FRONT: (f32, f32) = (1.5, 9.5);
/// The geode, in world cells.
pub const GEODE: (f32, f32) = (232.0, 24.0);

pub fn veins() -> Veins {
    Veins::grow(
        240,
        380.0,
        (CEIL as f32 + 2.0, FLOOR as f32 - 3.0),
        (4.0, 30.0),
        GEODE,
        5,
    )
}

/// Draw a vein between two cells as slope characters.
fn vein(g: &mut crate::ascii::Grid, a: (f32, f32), b: (f32, f32), rgb: u32) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let steps = dx.abs().max(dy.abs() * 1.0).ceil().max(1.0) as i32;
    let slope = dy / dx.abs().max(0.01) * 1.8;
    let ch = if slope.abs() < 0.4 {
        '-'
    } else if slope.abs() > 2.5 {
        '|'
    } else if (dy > 0.0) == (dx > 0.0) {
        '\\'
    } else {
        '/'
    };
    for s in 0..=steps {
        let k = s as f32 / steps as f32;
        let (c, r) = ((a.0 + dx * k).round() as i32, (a.1 + dy * k).round() as i32);
        if r > CEIL && r < FLOOR {
            g.put(c, r, ch, rgb);
        }
    }
}

pub fn draw(st: &mut Stage, t: f32, local: f32, v: &Veins) {
    let cam = local * PACE;
    // Rock everywhere; the gallery wall darker, hatched.
    for r in 0..ROWS {
        for c in 0..COLS {
            let wc = c as f32 + cam;
            let n = noise(wc * 0.12, r as f32 * 0.2);
            let wall = r > CEIL && r < FLOOR;
            let base = if wall {
                mix(0x0b0a09, 0x1a1612, n)
            } else {
                mix(0x16120e, 0x2e2620, n)
            };
            let ch = if h2(wc as i32, r) < if wall { 0.05 } else { 0.14 } {
                ['.', ',', '\''][(h2(r, wc as i32) * 3.0) as usize % 3]
            } else {
                ' '
            };
            st.g.set(c, r, ch, scale(base, 2.2), base);
        }
    }
    // The rock's lip over the gallery: drips and teeth along the ceiling,
    // rubble along the floor.
    for c in 0..COLS {
        let wc = (c as f32 + cam) as i32;
        let lip = (noise(wc as f32 * 0.2, 4.0) * 2.5) as i32;
        for r in CEIL + 1..CEIL + 1 + lip {
            st.g.set(
                c,
                r,
                if r == CEIL + lip { 'v' } else { ' ' },
                0x4a3e32,
                0x1e1914,
            );
        }
        if h2(wc, 7) < 0.05 {
            st.g.put(c, CEIL + 1 + lip, '|', 0x5a6a70);
            st.g.put(c, CEIL + 2 + lip, '.', 0x9ab4c0);
        }
        if h2(wc, 8) < 0.12 {
            st.g.set(
                c,
                FLOOR - 1,
                if h2(wc, 9) < 0.5 { 'o' } else { '.' },
                0x5a4a3a,
                0x0e0b09,
            );
        }
    }
    // The veins: dull rust until the front passes, then emerald; the
    // shortest way to the geode gold once the front arrives.
    let speed = v.dist[v.target] / (FRONT.1 - FRONT.0);
    let front = (local - FRONT.0) * speed;
    let arrived = local >= FRONT.1;
    let flare = window(local, FRONT.1, FRONT.1 + 0.6);
    for &(a, b) in &v.edges {
        let (pa, pb) = (v.knots[a], v.knots[b]);
        let (ca, cb) = ((pa.0 - cam, pa.1), (pb.0 - cam, pb.1));
        if ca.0.max(cb.0) < -2.0 || ca.0.min(cb.0) > COLS as f32 + 2.0 {
            continue;
        }
        let lit = ((front - v.dist[a].min(v.dist[b])) / 6.0).clamp(0.0, 1.0);
        let gold = arrived && v.on_path(a, b);
        let rgb = if gold {
            mix(0x5aff9a, 0xffd25a, flare)
        } else {
            mix(0x3a2416, 0x3dbb8a, lit)
        };
        vein(st.g, ca, cb, rgb);
        if gold {
            let (x0, y0) = cell_px(ca.0, ca.1);
            let (x1, y1) = cell_px(cb.0, cb.1);
            st.line((x0, y0), (x1, y1), 2.0, 0xffc040, 0.5 * flare);
        }
    }
    // The wavefront's glints: knots lighting as it passes them.
    for (i, &(c, r)) in v.knots.iter().enumerate() {
        let sc = c - cam;
        if !(-2.0..COLS as f32 + 2.0).contains(&sc) {
            continue;
        }
        let age = front - v.dist[i];
        if age > 0.0 {
            let (x, y) = cell_px(sc, r);
            let fresh = (-age / 12.0).exp();
            st.g.put(
                sc.round() as i32,
                r.round() as i32,
                '+',
                mix(0x3dbb8a, 0xd9fff0, fresh),
            );
            st.glow(x, y, 28.0 + 30.0 * fresh, 0x3dbb8a, 0.15 + 0.5 * fresh);
        }
    }
    // The source: light comes in behind them.
    let (sc, sr) = v.knots[v.source];
    let (x, y) = cell_px(sc - cam, sr);
    st.glow(
        x,
        y,
        140.0,
        0x6affc8,
        0.5 * window(local, FRONT.0 - 0.5, FRONT.0 + 0.5),
    );

    // Timber frames, the rails, the lanterns.
    let spacing = 38.0;
    let first = (cam / spacing).floor() as i32;
    for i in first..first + 7 {
        let c = (i as f32 * spacing - cam).round() as i32;
        for r in CEIL..FLOOR {
            st.g.set(c, r, '|', 0x8a6a44, 0x3a2614);
            st.g.set(c + 1, r, '|', 0x6a4e30, 0x2a1a0c);
        }
        for dc in -6..8 {
            st.g.set(c + dc, CEIL, '=', 0x9a7a50, 0x3a2614);
        }
        // A lantern at mid-span, swaying on its chain.
        let lc = c + (spacing / 2.0) as i32;
        let sway = (1.4 * (t * 1.6 + i as f32).sin()).round() as i32;
        for r in CEIL + 1..CEIL + 4 {
            st.g.put(lc + if r > CEIL + 2 { sway } else { 0 }, r, ':', 0x6a6050);
        }
        st.g.set(lc + sway - 1, CEIL + 4, '[', 0x9a8a60, 0x2a1a08);
        st.g.set(lc + sway, CEIL + 4, '*', 0xfff0b0, 0x8a5a10);
        st.g.set(lc + sway + 1, CEIL + 4, ']', 0x9a8a60, 0x2a1a08);
        let (x, y) = cell_px((lc + sway) as f32, CEIL as f32 + 4.0);
        let flick = 0.85 + 0.15 * (t * 9.0 + i as f32 * 3.0).sin();
        st.glow(x, y, 220.0, 0xff9a3a, 0.35 * flick);
        st.glow(x, y, 30.0, 0xfff0c0, 0.5 * flick);
    }
    for c in 0..COLS {
        let wc = c + cam as i32;
        st.g.set(
            c,
            FLOOR,
            if wc % 4 == 0 { '#' } else { '=' },
            0x7a7068,
            0x1a1410,
        );
    }

    // The geode in its niche, and its crystal turning.
    let (gc, gr) = (GEODE.0 - cam, GEODE.1);
    if gc > -20.0 && gc < COLS as f32 + 20.0 {
        for dr in -4..=4i32 {
            for dc in -8..=8i32 {
                if (dc as f32 / 8.0).powi(2) + (dr as f32 / 4.0).powi(2) <= 1.0 {
                    st.g.set(gc as i32 + dc, gr as i32 + dr, ' ', 0, 0x050404);
                }
            }
        }
        let (x, y) = cell_px(gc, gr);
        let k = 0.35 + 0.65 * flare;
        crystal(
            st,
            x,
            y,
            56.0,
            t * 0.7,
            t * 0.45,
            mix(0x3dbb8a, 0xffe08a, flare),
            k,
        );
        st.glow(x, y, 120.0 + 160.0 * flare, 0x3dbb8a, 0.25 + 0.4 * flare);
    }

    let light = Light {
        tint: 0xffc890,
        k: 0.35,
        rim: Some((0xffb060, -1, 0.5)),
    };
    party::victor(st, t, 100.0, FEET, true, light, 1.0);
    party::cordelia(st, t, 60.0, FEET, true, light, 1.0);
}

/// A wireframe octahedron (with its inner star) rotating about two axes,
/// drawn as additive vector lines.
#[allow(clippy::too_many_arguments)]
fn crystal(st: &mut Stage, cx: f32, cy: f32, size: f32, a: f32, b: f32, rgb: u32, k: f32) {
    let verts = [
        (1.0, 0.0, 0.0),
        (-1.0, 0.0, 0.0),
        (0.0, 1.4, 0.0),
        (0.0, -1.4, 0.0),
        (0.0, 0.0, 1.0),
        (0.0, 0.0, -1.0),
    ];
    let p: Vec<(f32, f32, f32)> = verts
        .iter()
        .map(|&(x, y, z): &(f32, f32, f32)| {
            let (x, z) = (x * a.cos() - z * a.sin(), x * a.sin() + z * a.cos());
            let (y, z) = (y * b.cos() - z * b.sin(), y * b.sin() + z * b.cos());
            let persp = 3.0 / (3.0 + z);
            (cx + x * size * persp, cy + y * size * persp, z)
        })
        .collect();
    let edges = [
        (0, 2),
        (0, 3),
        (0, 4),
        (0, 5),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (2, 4),
        (2, 5),
        (3, 4),
        (3, 5),
    ];
    for (i, j) in edges {
        let depth = 0.55 + 0.45 * (1.0 - (p[i].2 + p[j].2) * 0.25);
        st.line((p[i].0, p[i].1), (p[j].0, p[j].1), 2.2, rgb, k * depth);
    }
    for (i, v) in p.iter().enumerate() {
        let tw = 0.5 + 0.5 * (a * 3.0 + i as f32 * TAU / 6.0).sin();
        st.glow(v.0, v.1, 12.0, 0xffffff, k * 0.4 * tw);
    }
}
