// PONTE NOSSA: the furnace, the ingot, the water wheel, the trip hammer,
// sparks and steam.
use std::f32::consts::{PI, TAU};

use crate::canvas::{ramp, scale_rgb, Cell, TextCanvas, COLS, ROWS};
use crate::noise::{h1, h3, pick};
use crate::timeline::{bars, last_strike, smooth, BEAT, HAMMER_STOP};

pub const HOT: [u32; 5] = [0x2a0600, 0xa32000, 0xff7a00, 0xffd88a, 0xffffff];
const IRON: u32 = 0x8a929c;
const WOOD: u32 = 0x9a6a3a;

/// Banner cells of the ingot (taken from a glyphfx clip's last frame).
pub type Ingot = Vec<(u16, u16, Cell)>;

/// Glowing coals along the bottom of the forge.
pub fn furnace(canvas: &mut TextCanvas, t: f32, gain: f32) {
    let tq = (t * 15.0) as i32;
    for r in (ROWS as i32 - 6)..ROWS as i32 {
        let depth = (r - (ROWS as i32 - 6)) as f32 / 6.0;
        for c in 0..COLS as i32 {
            let v = h3(c, r, tq);
            let lick = ((c as f32 * 0.3 + t * 3.0).sin() * 0.5 + 0.5) * 0.4;
            let heat = depth * 0.7 + lick + v * 0.3 - 0.35;
            if heat > 0.05 {
                let ch = pick(&['^', '*', '"', '\'', '.', '^'], v);
                canvas.set(
                    c,
                    r,
                    Cell::new(ch, scale_rgb(ramp(&HOT, heat.min(1.0) * 0.8), gain)),
                );
            }
        }
    }
    // Embers drifting up.
    for i in 0..60 {
        let period = 2.5 + h1(i) * 3.0;
        let age = (t + h1(i + 1) * period).rem_euclid(period);
        let x = h1(i + 2) * COLS as f32 + (age * 2.0 + i as f32).sin() * 3.0;
        let y = ROWS as f32 - 4.0 - age * 6.0;
        let a = (1.0 - age / period) * gain;
        canvas.set(
            x as i32,
            y as i32,
            Cell::new('.', scale_rgb(ramp(&HOT, 0.6 + 0.3 * a), a)),
        );
    }
}

/// Where the k-th strike lands along the ingot.
fn strike_col(k: usize) -> f32 {
    28.0 + ((k * 23) % 72) as f32
}

/// The ingot at forging heat, flashing white under each blow.
pub fn ingot(canvas: &mut TextCanvas, ingot: &Ingot, t: f32, base_heat: f32) {
    let strike = last_strike(t).filter(|_| t < bars(HAMMER_STOP));
    for &(c, r, cell) in ingot {
        let mut heat = base_heat + 0.04 * ((c as f32 * 0.4 + t * 2.0).sin());
        if let Some((ts, k)) = strike {
            let age = t - ts;
            let dx = (c as f32 - strike_col(k)) / 10.0;
            heat += 0.5 * (-age / 0.2).exp() * (-dx * dx).exp() + 0.12 * (-age / 0.1).exp();
        }
        canvas.set(
            c as i32,
            r as i32,
            Cell::new(cell.ch, ramp(&HOT, heat.clamp(0.0, 1.0))),
        );
    }
}

fn line(canvas: &mut TextCanvas, (x0, y0): (f32, f32), (x1, y1): (f32, f32), rgb: u32) {
    let steps = ((x1 - x0).abs().max((y1 - y0).abs() * 1.8) as usize).max(1);
    let slope = (y1 - y0) / (x1 - x0 + 1e-3) * 1.8;
    let ch = if slope.abs() < 0.4 {
        '='
    } else if slope.abs() > 2.5 {
        '|'
    } else if slope > 0.0 {
        '\\'
    } else {
        '/'
    };
    for i in 0..=steps {
        let f = i as f32 / steps as f32;
        canvas.set(
            (x0 + (x1 - x0) * f).round() as i32,
            (y0 + (y1 - y0) * f).round() as i32,
            Cell::new(ch, rgb),
        );
    }
}

/// The water wheel, bottom left, and the race that drives it.
pub fn wheel(canvas: &mut TextCanvas, t: f32) {
    let (cx, cy, rx, ry) = (10.0f32, 32.0f32, 15.0f32, 8.3f32);
    let angle = t * (TAU / 8.0) / BEAT;
    for i in 0..96 {
        let a = i as f32 / 96.0 * TAU;
        canvas.set(
            (cx + a.cos() * rx).round() as i32,
            (cy + a.sin() * ry).round() as i32,
            Cell::new('#', WOOD),
        );
    }
    for s in 0..8 {
        let a = angle + s as f32 / 8.0 * TAU;
        line(
            canvas,
            (cx, cy),
            (cx + a.cos() * rx, cy + a.sin() * ry),
            scale_rgb(WOOD, 0.8),
        );
        let (px, py) = (cx + a.cos() * (rx + 1.5), cy + a.sin() * (ry + 0.8));
        canvas.set(
            px.round() as i32,
            py.round() as i32,
            Cell::new('█', scale_rgb(WOOD, 1.1)),
        );
    }
    canvas.set(cx as i32, cy as i32, Cell::new('O', IRON));
    // The race: water falling onto the paddles from top left.
    let tq = (t * 14.0) as i32;
    for r in 14..24 {
        for c in 0..5 {
            let v = h3(c, r - (t * 22.0) as i32, 1);
            if v < 0.6 {
                canvas.set(
                    c + 8,
                    r,
                    Cell::new(
                        pick(&['|', '!', ':'], h3(c, r, tq)),
                        scale_rgb(0x8fd0ff, 0.5 + v * 0.5),
                    ),
                );
            }
        }
    }
}

/// Head position of the trip hammer: (center col, bottom row).
fn hammer_head(t: f32) -> (f32, f32) {
    const STRIKE_ROW: f32 = 16.0;
    const LIFT: f32 = 5.0;
    let t_stop = bars(HAMMER_STOP);
    let Some((ts, k)) = last_strike(t) else {
        return (strike_col(0), STRIKE_ROW - LIFT);
    };
    if t >= t_stop {
        // Lifted clear while the ingot is thrown apart.
        let up = smooth((t - t_stop) / BEAT);
        return (strike_col(k), STRIKE_ROW - LIFT * up.max(0.0));
    }
    let p = ((t - ts) / BEAT).clamp(0.0, 1.0);
    let lift = if p < 0.82 {
        smooth(p / 0.82)
    } else {
        1.0 - ((p - 0.82) / 0.18).powi(2)
    };
    let x = strike_col(k) + (strike_col(k + 1) - strike_col(k)) * smooth((p - 0.1) / 0.5);
    (x, STRIKE_ROW - LIFT * lift)
}

pub fn hammer(canvas: &mut TextCanvas, t: f32) {
    let (x, bottom) = hammer_head(t);
    let pivot = (124.0, 10.0);
    line(canvas, (x + 8.0, bottom - 2.5), pivot, scale_rgb(WOOD, 1.1));
    line(
        canvas,
        (x + 8.0, bottom - 1.5),
        (pivot.0, pivot.1 + 1.0),
        scale_rgb(WOOD, 0.8),
    );
    canvas.set(pivot.0 as i32, pivot.1 as i32, Cell::new('O', IRON));
    for r in 0..4 {
        for c in -8..=8 {
            let ch = if r == 0 { '▄' } else { '█' };
            let shade = 0.6 + 0.4 * (1.0 - (c as f32 / 8.0).abs()) - 0.1 * r as f32;
            canvas.set(
                (x + c as f32).round() as i32,
                (bottom - 3.0 + r as f32).round() as i32,
                Cell::new(ch, scale_rgb(IRON, shade)),
            );
        }
    }
}

/// Sparks thrown by the most recent blows (ballistic, no state).
pub fn sparks(canvas: &mut TextCanvas, t: f32) {
    let Some((_, last)) = last_strike(t) else {
        return;
    };
    for k in last.saturating_sub(2)..=last {
        let ts = bars(crate::timeline::HAMMER_FROM) + k as f32 * BEAT;
        let age = t - ts;
        if !(0.0..1.1).contains(&age) {
            continue;
        }
        let (x0, y0) = (strike_col(k), 16.5);
        let count = if k % 4 == 0 { 46 } else { 26 };
        for i in 0..count {
            let seed = (k * 131 + i) as i32;
            let a = PI * (0.08 + 0.84 * h1(seed));
            let speed = 18.0 + 50.0 * h1(seed + 1);
            let x = x0 + a.cos() * speed * age * 1.6;
            let y = y0 - a.sin() * speed * age * 0.55 + 30.0 * age * age;
            let life = 0.4 + 0.7 * h1(seed + 2);
            if age > life {
                continue;
            }
            let heat = 1.0 - age / life;
            let ch = if heat > 0.66 {
                '*'
            } else if heat > 0.33 {
                '+'
            } else {
                '.'
            };
            canvas.set(
                x.round() as i32,
                y.round() as i32,
                Cell::new(ch, ramp(&HOT, 0.35 + 0.65 * heat)),
            );
        }
    }
}

/// Steam boiling off the quenched ingot.
pub fn steam(canvas: &mut TextCanvas, ingot: &Ingot, t: f32, gain: f32) {
    let Some(&(_, top, _)) = ingot.iter().min_by_key(|(_, r, _)| *r) else {
        return;
    };
    for i in 0..220 {
        let period = 1.6 + h1(i) * 2.4;
        let age = (t + h1(i + 1) * period).rem_euclid(period);
        let x = 18.0 + h1(i + 2) * 92.0 + (age * 1.7 + i as f32).sin() * 2.5;
        let y = top as f32 + 3.0 - age * 7.5;
        let a = (PI * age / period).sin() * gain;
        if a > 0.06 {
            let ch = pick(&['~', '.', '\'', ':', '~'], h1(i + 3));
            canvas.set(
                x as i32,
                y as i32,
                Cell::new(ch, scale_rgb(0xd8e2ea, a * 0.75)),
            );
        }
    }
}
