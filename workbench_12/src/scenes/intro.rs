// Intro: the flight into the stars while the title is beamed in, then the
// plugin's five models land one per beat (timeline::PANEL_AT): the four stock
// models, each with the demo it ships and its rules by level counting up, and
// the empty slot for a model the user builds. The panels fade as the stars
// warp into the first downbeat, where the live window opens.
use super::{EMERALD, LIME, MINT, PAPER};
use crate::gfx::{advance, lerp_px, mix, par_rows, Frame, H, W};
use crate::models::MODELS;
use crate::timeline::{smooth, Now, BAR, BEAT, PANEL_AT};

const PANEL_W: i32 = 340;
const GAP: i32 = 24;
const TOP: i32 = 360;
const BOTTOM: i32 = 800;
const DIM: u32 = 0x3f6b5a;
/// Hard, medium and soft, as the plugin's pages colour them.
const LEVELS: [(u32, &str); 3] = [(0xf87171, "HARD"), (0xfbbf24, "MEDIUM"), (0x6ee7b7, "SOFT")];

pub fn draw(f: &mut Frame, n: Now) -> f32 {
    f.gradient(&[0x000000, 0x02080a, 0x041410, 0x000000]);
    starfield(f, travelled(n.local, n.len), smooth(n.local / 1.2), MINT);
    let out = smooth((n.len - n.local - 0.1) / 0.6);
    // A dark plate under the panels, so no star runs through them.
    plate(
        f,
        TOP - 40,
        BOTTOM + 40,
        0.85 * smooth((n.local - PANEL_AT[0] + 0.4) / 0.6) * out,
    );
    let x0 = (W as i32 - (5 * PANEL_W + 4 * GAP)) / 2;
    for (i, m) in MODELS.iter().enumerate() {
        let age = n.local - PANEL_AT[i];
        if age < 0.0 {
            continue;
        }
        // Each panel drops a short way into place and fades in.
        let land = smooth(age / 0.25);
        let a = land * out;
        let x = x0 + i as i32 * (PANEL_W + GAP);
        let y = TOP - (24.0 * (1.0 - land)) as i32;
        let mine = m.rules == [0, 0, 0];
        // The landing flashes the frame; the user's own slot stays open.
        let flash = (-age * 6.0).exp();
        let edge = if mine {
            DIM
        } else {
            mix(EMERALD, 0xffffff, flash * 0.6)
        };
        f.rect(x, y, x + PANEL_W, y + (BOTTOM - TOP), 0x02100a, 0.9 * a);
        if mine {
            dashed(f, x, y, x + PANEL_W, y + (BOTTOM - TOP), edge, a);
        } else {
            f.frame_rect(x, y, x + PANEL_W, y + (BOTTOM - TOP), edge, a);
            f.frame_rect(
                x - 1,
                y - 1,
                x + PANEL_W + 1,
                y + (BOTTOM - TOP) + 1,
                EMERALD,
                0.4 * a,
            );
        }
        let tx = x + 20;
        f.text(tx, y + 24, m.name[0], 2, PAPER, a);
        f.text(tx, y + 60, m.name[1], 2, PAPER, a);
        f.text(tx, y + 116, m.demo, 2, if mine { DIM } else { LIME }, a);
        f.text(tx, y + 152, m.detail[0], 2, MINT, a * 0.8);
        f.text(tx, y + 184, m.detail[1], 2, MINT, a * 0.8);
        if mine {
            // A cursor blinking on the beat where the rules will be.
            if ((n.local / BEAT) as i32) % 2 == 0 {
                f.rect(tx, y + 250, tx + advance(3), y + 280, LIME, a);
            }
            continue;
        }
        // The rules count up over the first 0.8 s, and the bar fills by level.
        let total: u32 = m.rules.iter().sum();
        let shown = (total as f32 * smooth(age / 0.8)).round() as u32;
        f.text(tx, y + 244, &format!("{shown} RULES"), 3, PAPER, a);
        let (bx, by, bw) = (tx, y + 300, PANEL_W - 40);
        let mut cx = bx;
        let fill = smooth((age - 0.1) / 0.8);
        for (k, (rgb, label)) in LEVELS.iter().enumerate() {
            let w = (bw as f32 * m.rules[k] as f32 / total as f32 * fill).round() as i32;
            f.rect(cx, by, cx + w, by + 14, *rgb, 0.9 * a);
            cx += w;
            f.text(
                tx,
                y + 334 + k as i32 * 30,
                &format!("{} {label}", m.rules[k]),
                2,
                *rgb,
                a * fill,
            );
        }
    }
    smooth(n.local / 1.0)
}

/// A dashed outline: the slot that is still empty.
fn dashed(f: &mut Frame, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
    let mut x = x0;
    while x < x1 {
        f.rect(x, y0, (x + 12).min(x1), y0 + 2, rgb, a);
        f.rect(x, y1 - 2, (x + 12).min(x1), y1, rgb, a);
        x += 22;
    }
    let mut y = y0;
    while y < y1 {
        f.rect(x0, y, x0 + 2, (y + 12).min(y1), rgb, a);
        f.rect(x1 - 2, y, x1, (y + 12).min(y1), rgb, a);
        y += 22;
    }
}

/// Darken the rows y0..y1, softly at the top and bottom edges.
fn plate(f: &mut Frame, y0: i32, y1: i32, k: f32) {
    if k <= 0.0 {
        return;
    }
    par_rows(&mut f.px, |y, row| {
        let y = y as i32;
        if y < y0 || y >= y1 {
            return;
        }
        let a = (k * smooth((y - y0) as f32 / 40.0) * smooth((y1 - y) as f32 / 40.0)).min(1.0);
        if a > 0.0 {
            for p in row.iter_mut() {
                *p = lerp_px(*p, 0x000000, a);
            }
        }
    });
}

fn hash(i: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

/// Stars rushing out of the centre; `d` is the distance flown so far.
fn starfield(f: &mut Frame, d: f32, k: f32, tint: u32) {
    if k <= 0.0 {
        return;
    }
    let (cx, cy) = (W as f32 * 0.5, H as f32 * 0.5);
    for i in 0..480u32 {
        let ang = hash(i) * std::f32::consts::TAU;
        let z = (hash(i + 7919) - d * (0.6 + 0.4 * hash(i + 104_729)))
            .rem_euclid(1.0)
            .max(0.02);
        let r = 40.0 / z;
        if r < 120.0 {
            continue;
        }
        let (x, y) = (cx + ang.cos() * r, cy + ang.sin() * r * 0.75);
        if x < 0.0 || y < 0.0 || x >= W as f32 || y >= H as f32 {
            continue;
        }
        let b = ((1.0 - z) * 1.2).clamp(0.0, 1.0);
        let ch = if b > 0.8 {
            '*'
        } else if b > 0.5 {
            '+'
        } else {
            '.'
        };
        f.char_at(
            x as i32,
            y as i32,
            ch,
            2,
            mix(tint, 0xffffff, b * 0.5),
            k * b,
        );
    }
}

/// Forward speed of the flight: a cruise that keeps gaining, then the warp
/// over the last bar. Never decreasing.
fn speed(t: f32, len: f32) -> f32 {
    let rush = smooth((t - (len - BAR)) / BAR);
    0.12 + 0.02 * t + 1.8 * rush * rush
}

/// Distance flown by t: the integral of `speed`, so it only grows.
fn travelled(t: f32, len: f32) -> f32 {
    let steps = (t / 0.01).ceil().max(1.0) as usize;
    let dt = t / steps as f32;
    (0..steps)
        .map(|i| speed((i as f32 + 0.5) * dt, len) * dt)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flight_only_goes_forward() {
        let len = crate::timeline::LIVE_FROM;
        let mut last = -1.0;
        let mut t = 1.0 / 60.0;
        while t < len {
            let d = travelled(t, len);
            assert!(d > last, "the stars move backwards at {t}");
            assert!(speed(t + 1.0 / 60.0, len) >= speed(t, len));
            last = d;
            t += 1.0 / 60.0;
        }
    }

    #[test]
    fn the_panels_fit_the_frame() {
        let w = 5 * PANEL_W + 4 * GAP;
        assert!(w <= W as i32 - 80);
        // The longest line at scale 2 inside a panel, with its 20 px inset.
        assert!(15 * advance(2) + 40 <= PANEL_W);
    }
}
