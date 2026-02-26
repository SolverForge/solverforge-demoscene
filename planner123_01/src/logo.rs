// ═══════════════════════════════════════════════════════════════
// SOLVERFORGE OUROBOROS LOGO
//
// ViewBox: 0 0 200 200. Center: (100,100). Max extent: 75px.
//
// WHAT THE SVG ACTUALLY IS:
//   1. A CLOSED 8-point polygon (stroke only, fill=none), stroke-width=14.
//      The Z closes back to M — it's a full ring. Ouroboros.
//   2. A forked snake HEAD at vertex 0 (100,25) pointing straight up.
//   3. A TAIL stub: M100 25 L100 40 — straight down from head, w=10,
//      drawn over the body so the tail feeds into the head.
//   4. Inner 8-point polygon ring, stroke-width=2, opacity=0.4.
//   5. Accent circles (r=5, opacity=0.3) at all 8 outer vertices.
//   6. Center crosshair + ring + dot + corner brackets.
//   7. Six small black squares at edge midpoints.
//
// All 8 outer vertices (SVG coords → normalized, scale=75):
//   v0:(100, 25)→( 0.000,-1.000)  v1:(150, 50)→( 0.667,-0.667)
//   v2:(165, 95)→( 0.867,-0.067)  v3:(150,150)→( 0.667, 0.667)
//   v4:(100,175)→( 0.000, 1.000)  v5:( 50,150)→(-0.667, 0.667)
//   v6:( 35, 95)→(-0.867,-0.067)  v7:( 50, 50)→(-0.667,-0.667)
// ═══════════════════════════════════════════════════════════════

use crate::palette::{self, EMERALD_300, EMERALD_400, EMERALD_500, EMERALD_800, GREEN_500};

// ── Outer 8-point polygon vertices, normalized (÷75 from SVG) ──
const VERTS: [(f32, f32); 8] = [
    (0.000, -1.000),  // v0 top
    (0.667, -0.667),  // v1 upper-right
    (0.867, -0.067),  // v2 right
    (0.667, 0.667),   // v3 lower-right
    (0.000, 1.000),   // v4 bottom
    (-0.667, 0.667),  // v5 lower-left
    (-0.867, -0.067), // v6 left
    (-0.667, -0.667), // v7 upper-left
];

// ── Inner 8-point polygon vertices ────────────────────────────
// SVG: M100 55 L130 70 L140 95 L130 120 L100 135 L70 120 L60 95 L70 70 Z
// Relative to center (100,100), ÷75:
const INNER_VERTS: [(f32, f32); 8] = [
    (0.000, -0.600),  // (100-100, 55-100)/75
    (0.400, -0.400),  // (130-100, 70-100)/75
    (0.533, -0.067),  // (140-100, 95-100)/75
    (0.400, 0.267),   // (130-100,120-100)/75
    (0.000, 0.467),   // (100-100,135-100)/75
    (-0.400, 0.267),  // ( 70-100,120-100)/75
    (-0.533, -0.067), // ( 60-100, 95-100)/75
    (-0.400, -0.400), // ( 70-100, 70-100)/75
];

// ── Six midpoint squares from SVG (rect centers, rel to (100,100), ÷75) ──
// SVG rects: x=123,y=47 → center(125,49); x=155,y=92 → center(157,94); etc.
const MIDPOINT_SQUARES: [(f32, f32); 6] = [
    (0.333, -0.680),  // (125-100, 49-100)/75
    (0.760, -0.080),  // (157-100, 94-100)/75
    (0.333, 0.667),   // (125-100,150-100)/75
    (-0.333, 0.667),  // ( 75-100,150-100)/75
    (-0.760, -0.080), // ( 43-100, 94-100)/75
    (-0.333, -0.680), // ( 75-100, 49-100)/75
];

/// Draw the SolverForge ouroboros logo centered at (cx,cy) with given radius.
/// progress: 0.0→nothing, 1.0→fully drawn. time for pulse. brightness for fade.
pub fn draw_logo(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: f32,
    cy: f32,
    radius: f32,
    progress: f32,
    time: f64,
    brightness: f32,
) {
    let progress = progress.clamp(0.0, 1.0);
    // px-per-SVG-unit: SVG uses 75px max extent = 1.0 normalized
    let s = radius / 75.0;

    // ── 1. Outer body: closed 8-point polygon, stroke-width=14 ──
    // Gradient: #22c55e (head/top) → #10b981 → #059669 (tail)
    let sw = (14.0 * s).max(2.0);
    let segs_drawn = (progress * 8.0).min(8.0);

    for seg in 0..8usize {
        let seg_p = (segs_drawn - seg as f32).clamp(0.0, 1.0);
        if seg_p <= 0.0 {
            break;
        }

        let i0 = seg;
        let i1 = (seg + 1) % 8;
        let (ax, ay) = VERTS[i0];
        let (bx, by) = VERTS[i1];

        let ex = ax + (bx - ax) * seg_p;
        let ey = ay + (by - ay) * seg_p;

        // Gradient: seg 0 = bright green, seg 7 = dark emerald
        let t = seg as f32 / 7.0;
        let seg_color = palette::lerp_color(
            palette::dim(GREEN_500, brightness),
            palette::dim(EMERALD_800, brightness),
            t,
        );

        thick_line(
            buffer,
            width,
            height,
            cx + ax * radius,
            cy + ay * radius,
            cx + ex * radius,
            cy + ey * radius,
            sw,
            seg_color,
        );
    }

    if progress < 0.05 {
        return;
    }

    // ── 2. Tail stub: M100 25 L100 40, stroke-width=10 ──────────
    // Drawn over the body — covers the join so the ring looks seamless.
    {
        let tail_color = palette::dim(GREEN_500, brightness);
        let tail_w = (10.0 * s).max(2.0);
        let tx = cx + VERTS[0].0 * radius;
        let ty0 = cy + VERTS[0].1 * radius;
        let ty1 = ty0 + 15.0 * s;
        thick_line(buffer, width, height, tx, ty0, tx, ty1, tail_w, tail_color);
    }

    // ── 3. Snake head at v0 (top vertex) ────────────────────────
    // SVG local: "M0 0 L-10 -12 L-5 -6 L0 -10 L5 -6 L10 -12 Z"
    // Forked arrowhead pointing UP. Fill=gradient, stroke=black w=1.
    {
        let hx = cx + VERTS[0].0 * radius;
        let hy = cy + VERTS[0].1 * radius;

        let lp = |lx: f32, ly: f32| -> (i32, i32) { ((hx + lx * s) as i32, (hy + ly * s) as i32) };
        let p0 = lp(0.0, 0.0);
        let p1 = lp(-10.0, -12.0);
        let p2 = lp(-5.0, -6.0);
        let p3 = lp(0.0, -10.0);
        let p4 = lp(5.0, -6.0);
        let p5 = lp(10.0, -12.0);

        let hc = palette::dim(GREEN_500, brightness);
        // Fill the fork as 4 triangles
        fill_tri(buffer, width, height, p0, p1, p2, hc);
        fill_tri(buffer, width, height, p0, p2, p3, hc);
        fill_tri(buffer, width, height, p0, p3, p4, hc);
        fill_tri(buffer, width, height, p0, p4, p5, hc);

        // Two eyes at cx=±3, cy=-8 (SVG local), r=1.5
        let er = ((1.5 * s).round() as i32).max(1);
        let (elx, ely) = lp(-3.0, -8.0);
        let (erx, _) = lp(3.0, -8.0);
        fill_circle(buffer, width, height, elx, ely, er, 0x000000);
        fill_circle(buffer, width, height, erx, ely, er, 0x000000);
    }

    if progress < 0.5 {
        return;
    }

    let inner_p = ((progress - 0.5) * 2.0).clamp(0.0, 1.0);

    // ── 4. Inner polygon ring (stroke-width=2, opacity=0.4) ─────
    {
        let ic = palette::dim(EMERALD_500, brightness * 0.4 * inner_p);
        let isw = (2.0 * s).max(1.0);
        for seg in 0..8usize {
            let (ax, ay) = INNER_VERTS[seg];
            let (bx, by) = INNER_VERTS[(seg + 1) % 8];
            thick_line(
                buffer,
                width,
                height,
                cx + ax * radius,
                cy + ay * radius,
                cx + bx * radius,
                cy + by * radius,
                isw,
                ic,
            );
        }
    }

    // ── 5. Accent circles at all 8 outer vertices (r=5, op=0.3) ─
    {
        let vr = ((5.0 * s).round() as i32).max(1);
        let vc = palette::dim(EMERALD_500, brightness * 0.3 * inner_p);
        for &(vx, vy) in &VERTS {
            fill_circle(
                buffer,
                width,
                height,
                (cx + vx * radius) as i32,
                (cy + vy * radius) as i32,
                vr,
                vc,
            );
        }
    }

    if progress < 0.7 {
        return;
    }

    let center_p = ((progress - 0.7) / 0.3).clamp(0.0, 1.0);

    // ── 6. Center node ───────────────────────────────────────────
    let cxi = cx as i32;
    let cyi = cy as i32;

    // Crosshair arms: gap 8..20 SVG units, stroke-width=2.5, black
    {
        let arm_out = (20.0 * s * center_p) as i32;
        let arm_in = (8.0 * s) as i32;
        let cw = (2.5 * s).max(1.0);
        let cc = palette::dim(0x111111, brightness * center_p);
        if arm_out > arm_in {
            thick_line(
                buffer,
                width,
                height,
                cx - arm_out as f32,
                cy,
                cx - arm_in as f32,
                cy,
                cw,
                cc,
            );
            thick_line(
                buffer,
                width,
                height,
                cx + arm_in as f32,
                cy,
                cx + arm_out as f32,
                cy,
                cw,
                cc,
            );
            thick_line(
                buffer,
                width,
                height,
                cx,
                cy - arm_out as f32,
                cx,
                cy - arm_in as f32,
                cw,
                cc,
            );
            thick_line(
                buffer,
                width,
                height,
                cx,
                cy + arm_in as f32,
                cx,
                cy + arm_out as f32,
                cw,
                cc,
            );
        }
    }

    // Ring r=6, stroke-width=2.5, fill=none, stroke=black
    {
        let cr = ((6.0 * s).round() as i32).max(2);
        let cw = ((2.5 * s).round() as i32).max(1);
        ring_circle(
            buffer,
            width,
            height,
            cxi,
            cyi,
            cr,
            cw,
            palette::dim(0x111111, brightness * center_p),
        );
    }

    // Center dot r=3, fill=emerald, with time pulse
    {
        let pulse = ((time * 3.0).sin() as f32 * 0.25 + 0.75).abs();
        let dr = ((3.0 * s).round() as i32).max(1);
        fill_circle(
            buffer,
            width,
            height,
            cxi,
            cyi,
            dr,
            palette::dim(EMERALD_400, brightness * center_p * pulse),
        );
        fill_circle(
            buffer,
            width,
            height,
            cxi,
            cyi,
            (dr / 2).max(1),
            palette::dim(EMERALD_300, brightness * center_p),
        );
    }

    // Corner brackets: "M-12 -12 L-15 -12 L-15 -15" × 4 corners
    // stroke-width=2, stroke=emerald
    {
        let b12 = (12.0 * s) as i32;
        let b15 = (15.0 * s) as i32;
        let bw = (2.0 * s).max(1.0);
        let bc = palette::dim(EMERALD_500, brightness * 0.9 * center_p);
        for &(sx, sy) in &[(-1i32, -1i32), (1, -1), (1, 1), (-1, 1)] {
            let x12 = cx + (sx * b12) as f32;
            let y12 = cy + (sy * b12) as f32;
            let x15 = cx + (sx * b15) as f32;
            let y15 = cy + (sy * b15) as f32;
            thick_line(buffer, width, height, x12, y12, x15, y12, bw, bc);
            thick_line(buffer, width, height, x15, y12, x15, y15, bw, bc);
        }
    }

    // ── 7. Six midpoint squares (4×4 SVG px → 4*s screen px) ────
    if progress > 0.85 {
        let sq_p = ((progress - 0.85) / 0.15).clamp(0.0, 1.0);
        let sq_r = ((2.0 * s).round() as i32).max(1);
        let sq_c = palette::dim(0x111111, brightness * 0.6 * sq_p);
        for &(mx, my) in &MIDPOINT_SQUARES {
            let px = (cx + mx * radius) as i32;
            let py = (cy + my * radius) as i32;
            for dy in -sq_r..=sq_r {
                for dx in -sq_r..=sq_r {
                    let x = px + dx;
                    let y = py + dy;
                    if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                        buffer[y as usize * width + x as usize] = sq_c;
                    }
                }
            }
        }
    }
}

// ── Primitive drawing helpers ─────────────────────────────────

/// Bresenham 1-pixel line.
fn line(buf: &mut [u32], w: usize, h: usize, x0: i32, y0: i32, x1: i32, y1: i32, col: u32) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1i32 } else { -1 };
    let sy = if y0 < y1 { 1i32 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
            buf[y as usize * w + x as usize] = col;
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

/// Thick line: offset parallel 1px lines along the perpendicular.
fn thick_line(
    buf: &mut [u32],
    w: usize,
    h: usize,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    thickness: f32,
    col: u32,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let nx = -dy / len;
    let ny = dx / len;
    let half = thickness / 2.0;
    let steps = (thickness.ceil() as i32).max(1);
    for i in 0..=steps {
        let t = -half + (i as f32 / steps as f32) * thickness;
        let ox = (nx * t).round() as i32;
        let oy = (ny * t).round() as i32;
        line(
            buf,
            w,
            h,
            x0 as i32 + ox,
            y0 as i32 + oy,
            x1 as i32 + ox,
            y1 as i32 + oy,
            col,
        );
    }
}

/// Filled triangle by scanline rasterization.
fn fill_tri(
    buf: &mut [u32],
    w: usize,
    h: usize,
    a: (i32, i32),
    b: (i32, i32),
    c: (i32, i32),
    col: u32,
) {
    let mut pts = [a, b, c];
    pts.sort_by_key(|p| p.1);
    let (x0, y0) = (pts[0].0, pts[0].1);
    let (x1, y1) = (pts[1].0, pts[1].1);
    let (x2, y2) = (pts[2].0, pts[2].1);
    let lerp = |y: i32, ya: i32, xa: i32, yb: i32, xb: i32| -> i32 {
        if yb == ya {
            xa
        } else {
            xa + (xb - xa) * (y - ya) / (yb - ya)
        }
    };
    for y in y0..=y2 {
        if y < 0 || y >= h as i32 {
            continue;
        }
        let (mut xl, mut xr) = if y < y1 {
            (lerp(y, y0, x0, y1, x1), lerp(y, y0, x0, y2, x2))
        } else {
            (lerp(y, y1, x1, y2, x2), lerp(y, y0, x0, y2, x2))
        };
        if xl > xr {
            std::mem::swap(&mut xl, &mut xr);
        }
        for x in xl.max(0)..=xr.min(w as i32 - 1) {
            buf[y as usize * w + x as usize] = col;
        }
    }
}

/// Filled circle.
fn fill_circle(buf: &mut [u32], w: usize, h: usize, cx: i32, cy: i32, r: i32, col: u32) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
                    buf[y as usize * w + x as usize] = col;
                }
            }
        }
    }
}

/// Circle ring (outline) with pixel thickness.
fn ring_circle(
    buf: &mut [u32],
    w: usize,
    h: usize,
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
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
                    buf[y as usize * w + x as usize] = col;
                }
            }
        }
    }
}
