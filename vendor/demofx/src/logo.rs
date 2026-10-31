// The SolverForge ouroboros, drawn from the brand SVG
// (solverforge-ui/static/sf/img/ouroboros.svg, viewBox 0 0 200 200) with
// anti-aliased signed distances. The SVG's tail stub paints nothing (its
// gradient has a zero-width bounding box), so it is not drawn. The black
// crosshair, target ring and squares become pale ink so they read on the
// demo's dark screen.
use std::f32::consts::TAU;

use crate::Pixels;

type P = (f32, f32);

const OUTER: [P; 8] = [
    (100.0, 25.0),
    (150.0, 50.0),
    (165.0, 95.0),
    (150.0, 150.0),
    (100.0, 175.0),
    (50.0, 150.0),
    (35.0, 95.0),
    (50.0, 50.0),
];
const INNER: [P; 8] = [
    (100.0, 55.0),
    (130.0, 70.0),
    (140.0, 95.0),
    (130.0, 120.0),
    (100.0, 135.0),
    (70.0, 120.0),
    (60.0, 95.0),
    (70.0, 70.0),
];
const HEAD: [P; 6] = [
    (0.0, 0.0),
    (-10.0, -12.0),
    (-5.0, -6.0),
    (0.0, -10.0),
    (5.0, -6.0),
    (10.0, -12.0),
];
const SQUARES: [P; 6] = [
    (123.0, 47.0),
    (155.0, 92.0),
    (123.0, 148.0),
    (73.0, 148.0),
    (41.0, 92.0),
    (73.0, 47.0),
];

const GRADIENT: [u32; 3] = [0x22c55e, 0x10b981, 0x059669];
const EMERALD: u32 = 0x10b981;
const INK: u32 = 0xecfdf5;
const RING_WIDTH: f32 = 14.0;

#[derive(Clone, Copy, Debug)]
pub struct LogoState {
    /// Center in pixels.
    pub cx: f32,
    pub cy: f32,
    /// Pixels per SVG unit.
    pub scale: f32,
    /// 0..1: the body draws itself clockwise from the tail to the head.
    pub progress: f32,
    /// 0..1: inner ring, accents, crosshair and brackets fade in.
    pub detail: f32,
    pub gain: f32,
}

impl LogoState {
    /// A logo whose body fits a circle of `radius` pixels (the SVG ring's
    /// 75-unit extent), drawing itself in over `progress` 0..1 with the
    /// detail fading in as the ring closes.
    pub fn sized(cx: f32, cy: f32, radius: f32, progress: f32, gain: f32) -> Self {
        Self {
            cx,
            cy,
            scale: radius / 75.0,
            progress,
            detail: ((progress - 0.85) / 0.15).clamp(0.0, 1.0),
            gain,
        }
    }
}

/// The mark every demo carries in its top-left corner, sized to the frame
/// height. It draws itself in over the demo's first seconds; `gain` fades it,
/// and 0 hides it (while a demo's own large end logo is up, for example).
pub fn corner(pixels: &mut impl Pixels, t: f32, gain: f32) {
    let r = corner_radius(pixels);
    corner_at(pixels, (r * 1.6, r * 1.7), t, gain);
}

/// The corner mark at a demo's own spot, for frames whose top-left corner
/// is taken (a TUI's title bar, say).
pub fn corner_at(pixels: &mut impl Pixels, center: (f32, f32), t: f32, gain: f32) {
    if gain <= 0.0 {
        return;
    }
    let r = corner_radius(pixels);
    let k = ((t - 0.5) / 2.0).clamp(0.0, 1.0);
    let progress = k * k * (3.0 - 2.0 * k);
    draw(
        pixels,
        &LogoState::sized(center.0, center.1, r, progress, gain.min(1.0) * 0.85),
    );
}

fn corner_radius(pixels: &impl Pixels) -> f32 {
    pixels.size().1 as f32 * 0.04
}

fn lerp_rgb(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ch = |v: u32, s: u32| ((v >> s) & 0xFF) as f32;
    let mix = |s: u32| {
        let v = ch(a, s) + (ch(b, s) - ch(a, s)) * t;
        (v.round().clamp(0.0, 255.0) as u32) << s
    };
    mix(16) | mix(8) | mix(0)
}

fn sub(a: P, b: P) -> P {
    (a.0 - b.0, a.1 - b.1)
}
fn dot(a: P, b: P) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn seg_dist(p: P, a: P, b: P) -> f32 {
    let (pa, ba) = (sub(p, a), sub(b, a));
    let h = (dot(pa, ba) / dot(ba, ba)).clamp(0.0, 1.0);
    let d = (pa.0 - ba.0 * h, pa.1 - ba.1 * h);
    dot(d, d).sqrt()
}

/// Signed distance to a closed polygon (negative inside).
fn sd_poly(p: P, v: &[P]) -> f32 {
    let mut d = dot(sub(p, v[0]), sub(p, v[0]));
    let mut s = 1.0;
    let n = v.len();
    for i in 0..n {
        let j = (i + n - 1) % n;
        let e = sub(v[j], v[i]);
        let w = sub(p, v[i]);
        let h = (dot(w, e) / dot(e, e)).clamp(0.0, 1.0);
        let b = (w.0 - e.0 * h, w.1 - e.1 * h);
        d = d.min(dot(b, b));
        let c = [p.1 >= v[i].1, p.1 < v[j].1, e.0 * w.1 > e.1 * w.0];
        if c.iter().all(|x| *x) || c.iter().all(|x| !*x) {
            s = -s;
        }
    }
    s * d.sqrt()
}

/// Miter-offset a convex polygon outward (d > 0) or inward (d < 0).
fn offset(v: &[P; 8], d: f32) -> [P; 8] {
    let c = v
        .iter()
        .fold((0.0, 0.0), |a, p| (a.0 + p.0 / 8.0, a.1 + p.1 / 8.0));
    let line = |i: usize| {
        let (a, b) = (v[i], v[(i + 1) % 8]);
        let e = sub(b, a);
        let len = dot(e, e).sqrt();
        let mut n = (e.1 / len, -e.0 / len);
        let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        if dot(n, sub(mid, c)) < 0.0 {
            n = (-n.0, -n.1);
        }
        ((a.0 + n.0 * d, a.1 + n.1 * d), e)
    };
    let mut out = [(0.0, 0.0); 8];
    for (i, slot) in out.iter_mut().enumerate() {
        let (p1, d1) = line((i + 7) % 8);
        let (p2, d2) = line(i);
        let cross = d1.0 * d2.1 - d1.1 * d2.0;
        let t = ((p2.0 - p1.0) * d2.1 - (p2.1 - p1.1) * d2.0) / cross;
        *slot = (p1.0 + d1.0 * t, p1.1 + d1.1 * t);
    }
    out
}

/// The brand gradient runs corner to corner over the body's bounding box.
fn gradient(p: P) -> u32 {
    let u = (p.0 - (35.0 - RING_WIDTH / 2.0)) / (130.0 + RING_WIDTH);
    let v = (p.1 - (25.0 - RING_WIDTH / 2.0)) / (150.0 + RING_WIDTH);
    let t = ((u + v) / 2.0).clamp(0.0, 1.0) * 2.0;
    if t < 1.0 {
        lerp_rgb(GRADIENT[0], GRADIENT[1], t)
    } else {
        lerp_rgb(GRADIENT[1], GRADIENT[2], t - 1.0)
    }
}

fn blend(px: &mut u32, rgb: u32, a: f32) {
    if a <= 0.0 {
        return;
    }
    *px = lerp_rgb(*px, rgb, a.min(1.0));
}

/// Coverage from a signed distance in SVG units at `scale` px per unit.
fn cover(sd: f32, scale: f32) -> f32 {
    (0.5 - sd * scale).clamp(0.0, 1.0)
}

/// Draw the logo into any frame the demos render to.
pub fn draw(pixels: &mut impl Pixels, s: &LogoState) {
    let (width, height) = pixels.size();
    let ring_out = offset(&OUTER, RING_WIDTH / 2.0);
    let ring_in = offset(&OUTER, -RING_WIDTH / 2.0);
    let head: Vec<P> = HEAD.iter().map(|&(x, y)| (x + 100.0, y + 25.0)).collect();
    let half = 100.0 * s.scale;
    let (x0, x1) = (
        (s.cx - half).max(0.0) as usize,
        ((s.cx + half) as usize).min(width),
    );
    let (y0, y1) = (
        (s.cy - half).max(0.0) as usize,
        ((s.cy + half) as usize).min(height),
    );
    let reveal = s.progress.clamp(0.0, 1.0);
    let detail = s.detail.clamp(0.0, 1.0) * s.gain;
    let head_a = ((reveal - 0.94) / 0.06).clamp(0.0, 1.0) * s.gain;

    for y in y0..y1 {
        for x in x0..x1 {
            // Pixel center in SVG units.
            let p = (
                (x as f32 + 0.5 - s.cx) / s.scale + 100.0,
                (y as f32 + 0.5 - s.cy) / s.scale + 100.0,
            );
            let mut value = pixels.get(x, y);
            let px = &mut value;

            // Body: the ring between the two miter offsets, revealed clockwise
            // from the top.
            let ring = cover(sd_poly(p, &ring_out).max(-sd_poly(p, &ring_in)), s.scale);
            if ring > 0.0 && reveal > 0.0 {
                let angle = (p.0 - 100.0).atan2(100.0 - p.1).rem_euclid(TAU) / TAU;
                let shown = ((reveal * 1.03 - angle) * 40.0).clamp(0.0, 1.0);
                blend(px, gradient(p), ring * shown * s.gain);
            }
            // Head with its black outline and eyes.
            if head_a > 0.0 {
                let sd = sd_poly(p, &head);
                blend(px, 0x000000, cover(sd - 0.5, s.scale) * head_a);
                blend(px, gradient(p), cover(sd + 0.5, s.scale) * head_a);
                for ex in [-3.0, 3.0] {
                    let e = sub(p, (100.0 + ex, 17.0));
                    blend(
                        px,
                        0x000000,
                        cover(dot(e, e).sqrt() - 1.5, s.scale) * head_a,
                    );
                }
            }
            if detail <= 0.0 {
                pixels.put(x, y, value);
                continue;
            }
            // Inner octagon, 2 wide at 40%.
            let inner = (0..8)
                .map(|i| seg_dist(p, INNER[i], INNER[(i + 1) % 8]))
                .fold(f32::MAX, f32::min);
            blend(px, EMERALD, cover(inner - 1.0, s.scale) * 0.4 * detail);
            // Accent circles at the outer vertices (r 5, 30%).
            for v in OUTER {
                let e = sub(p, v);
                blend(
                    px,
                    EMERALD,
                    cover(dot(e, e).sqrt() - 5.0, s.scale) * 0.3 * detail,
                );
            }
            let c = sub(p, (100.0, 100.0));
            // Crosshair arms, 2.5 wide, round caps.
            let arms = [
                ((-20.0, 0.0), (-8.0, 0.0)),
                ((20.0, 0.0), (8.0, 0.0)),
                ((0.0, -20.0), (0.0, -8.0)),
                ((0.0, 20.0), (0.0, 8.0)),
            ];
            let arm = arms
                .iter()
                .map(|&(a, b)| seg_dist(c, a, b))
                .fold(f32::MAX, f32::min);
            blend(px, INK, cover(arm - 1.25, s.scale) * detail);
            // Target ring r 6 (2.5 wide) and the emerald dot r 3.
            let r = dot(c, c).sqrt();
            blend(px, INK, cover((r - 6.0).abs() - 1.25, s.scale) * detail);
            blend(px, EMERALD, cover(r - 3.0, s.scale) * detail);
            // Corner brackets, 2 wide.
            let mut bracket = f32::MAX;
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let (a, k, b) = (
                    (12.0 * sx, 12.0 * sy),
                    (15.0 * sx, 12.0 * sy),
                    (15.0 * sx, 15.0 * sy),
                );
                bracket = bracket.min(seg_dist(c, a, k)).min(seg_dist(c, k, b));
            }
            blend(px, EMERALD, cover(bracket - 1.0, s.scale) * detail);
            // Six small squares at edge midpoints (4x4, 60%).
            for (sx, sy) in SQUARES {
                let d = (p.0 - sx - 2.0).abs().max((p.1 - sy - 2.0).abs()) - 2.0;
                blend(px, INK, cover(d, s.scale) * 0.6 * detail);
            }
            pixels.put(x, y, value);
        }
    }
}
