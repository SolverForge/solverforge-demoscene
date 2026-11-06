// Intro: the install line typed into a shell, then a trip sketch. Ten stops
// land on the road web one by one and join in the given order while the
// kilometres tick with the drawn polyline; the loop then redraws into the
// solved order and the counter settles, with the same functions drawing and
// counting both.
use super::{dot_grid, typed, EMERALD, LIME};
use crate::gfx::{advance, ease_out, Frame, W};
use crate::statue::Statue;
use crate::timeline::{smooth, Now, BEAT};
use crate::trip::{self, hop_pixels, POINTS, ROADS, SOLVED, STOPS};
use demofx::CELL_H;

const INSTALL: &str = "$ hermes plugins install blackopsrepl/hermes-solverforge-routing";

const MAP_IN: f32 = 2.6;
const DROP_AT: f32 = 3.4;
const SOLVE_AT: f32 = 8.6;

fn drop_time(i: usize) -> f32 {
    DROP_AT + i as f32 * BEAT
}

pub fn landings() -> Vec<f32> {
    (0..STOPS.len()).map(|i| drop_time(i) + 0.35).collect()
}

fn solved_time(i: usize) -> f32 {
    SOLVE_AT + i as f32 * 0.6
}

/// One drawn hop: its endpoints and progress (0..1).
type Hop = ((f32, f32), (f32, f32), f32);

/// The polyline the trip shows at time t: the naive joins as the stops land,
/// then one hop at a time redrawing into the solved loop.
fn edges(t: f32) -> Vec<Hop> {
    let mut out = Vec::new();
    let stops_on = |t: f32| (0..STOPS.len()).filter(|i| t > drop_time(*i)).count();
    let on = stops_on(t);
    if t < SOLVE_AT {
        for hop in 0..on.saturating_sub(1) {
            out.push((point(hop), point(hop + 1), 1.0));
        }
        return out;
    }
    for hop in 0..(SOLVED.len() - 1) {
        let ready = smooth((t - solved_time(hop)) / 0.45);
        if ready > 0.0 {
            let a = point(SOLVED[hop]);
            let b = point(SOLVED[hop + 1]);
            out.push((a, b, ready));
        }
    }
    if t > solved_time(SOLVED.len() - 1) + 0.45 {
        let ready = smooth((t - solved_time(SOLVED.len() - 1) - 0.45) / 0.45);
        if ready > 0.0 {
            let a = point(SOLVED[SOLVED.len() - 1]);
            let b = point(SOLVED[0]);
            out.push((a, b, ready));
        }
    }
    out
}

fn point(stop: usize) -> (f32, f32) {
    trip::point(stop)
}

pub fn draw(f: &mut Frame, statue: &Statue, n: Now, t: f32) -> f32 {
    let _ = n;
    f.gradient(&[0x020503, 0x03100a, 0x020503]);
    dot_grid(f, EMERALD, 0.18 * smooth(t / 1.0));
    statue.draw(f, 0x6ee7b7, 0.03 * smooth(t / 3.0), t);
    let fade_out = smooth((n.len - n.local) / 0.5);

    let shell = 1.0 - smooth((t - MAP_IN) / 0.6);
    if shell > 0.0 {
        let line = typed(INSTALL, 0.3, 38.0, t);
        let w = INSTALL.chars().count() as i32 * advance(2);
        let x = (W as i32 - w) / 2;
        let alpha = shell;
        f.text(x, 500, &line, 2, LIME, alpha);
        if (t * 2.0).fract() < 0.5 {
            let cx = x + line.chars().count() as i32 * advance(2);
            f.rect(
                cx,
                498,
                cx + advance(2) - 4,
                500 + 2 * CELL_H as i32,
                LIME,
                alpha,
            );
        }
        if t > 2.0 {
            f.text_centered(
                W as i32 / 2,
                556,
                "solverforge-routing 0.2.2 enabled",
                2,
                0x6ee7b7,
                alpha * smooth((t - 2.0) / 0.3),
            );
        }
    }

    let map = smooth((t - MAP_IN) / 0.8) * fade_out;
    if map <= 0.0 {
        return 1.0;
    }

    // The road web: thin lines, the stops off them light up like towns.
    for (from, to) in ROADS.iter() {
        let (ax, ay) = (POINTS[*from].0, POINTS[*from].1);
        let (bx, by) = (POINTS[*to].0, POINTS[*to].1);
        f.line(ax, ay, bx, by, 0x10b981, 0.28 * map);
    }

    // The trip's polylines: red while the given order crisscrosses, lime
    // while the solved loop draws.
    for (a, b, ready) in edges(t) {
        let solved_phase = t > SOLVE_AT;
        let rgb = if solved_phase { LIME } else { 0xf43f5e };
        let steps = (ready * 24.0) as i32;
        let (mut px, mut py) = a;
        for step in 1..=steps {
            let k = step as f32 / 24.0;
            let nx = a.0 + (b.0 - a.0) * k;
            let ny = a.1 + (b.1 - a.1) * k;
            f.line(px as i32, py as i32, nx as i32, ny as i32, rgb, 0.85 * map);
            px = nx;
            py = ny;
        }
        if ready > 0.35 {
            f.glow((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0, 60.0, rgb, 0.06 * map);
        }
    }

    // The stops: numbered circles that drop in one by one, a beat each.
    for (i, stop) in STOPS.iter().enumerate() {
        let born = smooth((t - drop_time(i)) / 0.35);
        if born <= 0.0 {
            continue;
        }
        let (x, y) = POINTS[*stop];
        let ease = ease_out(born);
        let radius = (14.0 + 6.0 * (1.0 - ease)) as i32;
        f.glow(x as f32, y as f32, 52.0, 0x6ee7b7, 0.28 * map * born);
        f.frame_rect(
            x - radius,
            y - radius,
            x + radius,
            y + radius,
            0xecfdf5,
            map * born,
        );
        f.frame_rect(
            x - radius + 2,
            y - radius + 2,
            x + radius - 2,
            y + radius - 2,
            0xecfdf5,
            0.5 * map * born,
        );
        let label = format!("{}", i + 1);
        f.text(x - 5, y - 7, &label, 2, 0xecfdf5, map * born);
    }

    // The kilometres, from the polylines on screen. The given loop's total
    // shows while it draws, red; the solved total settles, lime.
    let given = trip::given_km();
    let solved = trip::solved_km();
    let all_in = t > solved_time(SOLVED.len() - 1) + 0.9;
    let shown = if t < SOLVE_AT {
        // Kilometres as the polyline actually draws: the sum of the hops on
        // screen, through the same factor everything counts through.
        let on = (0..STOPS.len()).filter(|i| t > drop_time(*i)).count();
        let walked: f32 = (0..on.saturating_sub(1)).map(|hop| hop_pixels(hop, hop + 1)).sum();
        (walked * trip::ROAD_KM_PER_PX) as i32
    } else {
        let settle = smooth((t - SOLVE_AT) / (SOLVED.len() as f32 * 0.6));
        given + ((solved - given) as f32 * settle).round() as i32
    };
    let rgb = if t < SOLVE_AT { 0xf43f5e } else { LIME };
    f.text(420, 1030, &format!("{shown} KM"), 3, rgb, map);
    if all_in {
        let label = format!("{}% SHORTER, BEST FOUND", trip::shorter_percent());
        let x = 1840 - label.chars().count() as i32 * advance(3);
        f.text(x, 1030, &label, 3, 0xecfdf5, map);
    }
    1.0
}
