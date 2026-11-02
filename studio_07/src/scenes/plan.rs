// PLAN: offer one. 36 stops, four vans, one depot. The careless plan is
// drawn first, then the local search's kept moves play one after another;
// every number on the panel is computed from the routes on screen with the
// same `length()` the search uses.
use super::{EMERALD, MINT, PAPER};
use crate::gfx::{ease_out, Frame};
use crate::routes::{Plan, BOARD, CAPACITY};
use crate::timeline::{smooth, start_of, Now, Section};

const VAN: [u32; 4] = [0x34d399, 0x38bdf8, 0xfbbf24, 0xf472b6];
const BG: u32 = 0x030c09;
const GRID: u32 = 0x0b2219;
/// Seconds into the section: stops appear, the careless plan traces in,
/// the search runs, the result holds.
const ROUTES_IN: f32 = 2.6;
const SEARCH_FROM: f32 = 4.2;
const SEARCH_TO: f32 = 20.0;

/// The demo-clock time of every kept move after the first plan.
pub fn move_times(plan: &Plan) -> Vec<f32> {
    let start = start_of(Section::Plan) + SEARCH_FROM;
    let n = plan.steps.len().saturating_sub(1).max(1);
    (1..plan.steps.len())
        .map(|k| start + (SEARCH_TO - SEARCH_FROM) * (k as f32 / n as f32).powf(1.0 / 1.6))
        .collect()
}

pub fn draw(f: &mut Frame, plan: &Plan, times: &[f32], n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.clear(BG);

    let (x0, y0, x1, y1) = BOARD;
    let (gx0, gy0, gx1, gy1) = (x0 - 30.0, y0 - 30.0, x1 + 30.0, y1 + 30.0);
    let mut x = gx0;
    while x <= gx1 {
        f.rect(x as i32, gy0 as i32, x as i32 + 1, gy1 as i32, GRID, fade);
        x += 35.0;
    }
    let mut y = gy0;
    while y <= gy1 {
        f.rect(gx0 as i32, y as i32, gx1 as i32, y as i32 + 1, GRID, fade);
        y += 35.0;
    }

    let kept = times.partition_point(|&tk| tk <= t);
    let snap = &plan.steps[kept];
    let trace = smooth((n.local - ROUTES_IN) / 1.4);
    let settled = smooth((n.local - SEARCH_TO) / 1.0);

    // Routes, each with light flowing along it.
    for (v, route) in snap.routes.iter().enumerate() {
        let mut pts = vec![plan.depot];
        pts.extend(route.iter().map(|&s| plan.pos(s)));
        pts.push(plan.depot);
        let total: f32 = pts.windows(2).map(|w| seg(w[0], w[1])).sum();
        let mut walked = 0.0;
        for w in pts.windows(2) {
            let len = seg(w[0], w[1]);
            let upto = ((trace * total - walked) / len).clamp(0.0, 1.0);
            let a = (0.7 + 0.3 * settled) * fade;
            f.line(w[0].0, w[0].1, w[1].0, w[1].1, 2.2, VAN[v], a, upto);
            walked += len;
        }
        if trace >= 1.0 {
            let mut d = (t * 70.0 + v as f32 * 13.0) % 38.0;
            while d < total {
                let p = along(&pts, d);
                f.disc(p.0, p.1, 1.8, 0xffffff, 0.75 * fade);
                d += 38.0;
            }
        }
    }

    // Stops pop in, then take their van's colour.
    let mut owner = [usize::MAX; 64];
    for (v, route) in snap.routes.iter().enumerate() {
        for &s in route {
            owner[s] = v;
        }
    }
    for (s, &p) in plan.stops.iter().enumerate() {
        let pop = ease_out((n.local - 0.3 - s as f32 * 0.05) / 0.4);
        let c = if trace > 0.0 { VAN[owner[s]] } else { MINT };
        f.disc(p.0, p.1, 6.0 * pop, c, fade);
        f.disc(p.0, p.1, 2.4 * pop, BG, fade);
    }
    let (dx, dy) = plan.depot;
    f.glow(dx, dy, 40.0, EMERALD, 0.35 * fade);
    f.rect(
        dx as i32 - 9,
        dy as i32 - 9,
        dx as i32 + 9,
        dy as i32 + 9,
        PAPER,
        fade,
    );
    f.rect(
        dx as i32 - 5,
        dy as i32 - 5,
        dx as i32 + 5,
        dy as i32 + 5,
        BG,
        fade,
    );

    // The stop the last kept move touched.
    if kept > 0 {
        if let Some(stop) = snap.moved {
            let k = ((t - times[kept - 1]) / 0.45).clamp(0.0, 1.0);
            let p = plan.pos(stop);
            f.ring(p.0, p.1, 7.0 + 20.0 * k, 2.0, 0xffffff, (1.0 - k) * fade);
        }
    }

    panel(f, plan, kept, n, fade, settled);
    1.0
}

fn panel(f: &mut Frame, plan: &Plan, kept: usize, n: Now, fade: f32, settled: f32) {
    let a = smooth((n.local - 1.0) / 0.8) * fade;
    if a <= 0.0 {
        return;
    }
    let snap = &plan.steps[kept];
    let (px, py) = (960, 160);
    f.rect(px - 16, py - 18, 1250, 630, 0x041510, 0.85 * a);
    f.frame_rect(px - 16, py - 18, 1250, 630, 0x1f5c45, a);
    f.text(px, py, "ROUTE PLAN", 2, MINT, a);

    let start = plan.length(&plan.steps[0].routes);
    let now = plan.length(&snap.routes);
    let saved = 100.0 * (1.0 - now / start);
    f.text(px, py + 44, "LENGTH", 2, 0x7fa898, a);
    let glow = 0.75 + 0.25 * settled;
    f.text(px, py + 70, &format!("{:>6.0}", now), 4, PAPER, glow * a);
    f.text(
        px,
        py + 124,
        &format!("START {:>6.0}", start),
        2,
        0x7fa898,
        a,
    );
    f.text(
        px,
        py + 152,
        &format!("SAVED {:>5.1}%", saved),
        2,
        0x34d399,
        a,
    );
    f.text(px, py + 180, &format!("MOVES {:>6}", kept), 2, 0x7fa898, a);

    for (v, route) in snap.routes.iter().enumerate() {
        let y = py + 236 + v as i32 * 44;
        f.text(px, y, &format!("VAN {}", v + 1), 2, VAN[v], a);
        for slot in 0..CAPACITY {
            let x = px + 108 + slot as i32 * 13;
            let on = slot < route.len();
            let c = if on { VAN[v] } else { 0x1a2e27 };
            f.rect(x, y + 2, x + 9, y + 16, c, a);
        }
        f.text(
            px + 108,
            y + 22,
            &format!("{:>2}/{} STOPS", route.len(), CAPACITY),
            1,
            0x7fa898,
            a,
        );
    }
}

fn seg(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn along(pts: &[(f32, f32)], mut d: f32) -> (f32, f32) {
    for w in pts.windows(2) {
        let len = seg(w[0], w[1]);
        if d <= len {
            let k = d / len.max(1e-6);
            return (
                w[0].0 + (w[1].0 - w[0].0) * k,
                w[0].1 + (w[1].1 - w[0].1) * k,
            );
        }
        d -= len;
    }
    *pts.last().unwrap()
}
