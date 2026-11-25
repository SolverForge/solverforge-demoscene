// Intro: out of the cold open's take, the demo takes over. Over rolling
// copper, the month to plan rises as a wave: one glyph for every shift of
// the dataset (536, by ward and day), breathing on the beat. glyphfx beams in
// the title; the three numbers of the problem land one per bar, each counted
// up with a flash. On the last bar the stars streak and the wave rushes past
// the camera into the first take.
use super::{dot_grid, BLUE, EMERALD, JADE, LIME, MINT, PAPER};
use crate::gfx::{advance, mix, pack_px, par_rows, parts, Frame, H, W};
use crate::timeline::{hash, smooth, Now, BAR, BEAT};

/// The problem as the dataset states it ("A month of shifts").
pub const STATS: [(u32, &str); 3] = [(50, "EMPLOYEES"), (3, "WARDS"), (536, "SHIFTS")];
/// Shifts by ward, the wave's three bands over the 28 days of March: a
/// weekday's shifts and a weekend day's, as the dataset has them.
const WARDS: [(usize, usize, u32); 3] = [(8, 4, LIME), (6, 4, JADE), (8, 4, BLUE)];
const DAYS: usize = 28;
const COPPER: [u32; 3] = [0x10b981, 0x0ea5e9, 0x50fc00];

pub fn draw(f: &mut Frame, n: Now, t: f32) -> (f32, f32) {
    let out = smooth((n.len - n.local) / 0.5);
    let warp = smooth((n.local - (n.len - BAR)) / BAR);
    f.gradient(&[0x000000, 0x020806, 0x03120c, 0x000000]);
    copper(f, t, 0.5 * smooth(n.local / 1.0) * out);
    dot_grid(f, EMERALD, 0.18 * smooth(n.local / 0.8) * out);
    stars(f, n.local, warp);
    wave(f, n.local, warp, smooth((n.local - 0.3) / 1.2) * out);

    // The three numbers, one per beat pair from the second bar.
    for (i, (value, label)) in STATS.iter().enumerate() {
        let at = BAR + i as f32 * 2.0 * BEAT;
        let age = n.local - at;
        if age < 0.0 {
            continue;
        }
        let a = smooth(age / 0.2) * (1.0 - warp) * out;
        let shown = (*value as f32 * smooth(age / 0.6)).round() as u32;
        let cx = W as i32 / 2 + (i as i32 - 1) * 440;
        let flash = (-age * 7.0).exp();
        f.glow(cx as f32, 470.0, 220.0, LIME, 0.25 * flash * a);
        let s = shown.to_string();
        f.text_centered(cx, 410, &s, 12, mix(PAPER, 0xffffff, flash), a);
        f.text_centered(cx, 548, label, 3, MINT, a);
    }
    (1.0, 0.0)
}

/// Bands of light rolling up the frame, pulsing on the beat.
fn copper(f: &mut Frame, t: f32, k: f32) {
    if k <= 0.0 {
        return;
    }
    let beat = 1.0 + 0.3 * (-(t % BEAT) * 8.0).exp();
    let bands: Vec<(f32, u32)> = COPPER
        .iter()
        .enumerate()
        .map(|(i, rgb)| {
            let phase = t * 0.3 + i as f32 / 3.0;
            ((1.0 - phase.fract()) * (H as f32 + 240.0) - 120.0, *rgb)
        })
        .collect();
    par_rows(&mut f.px, |y, row| {
        for &(cy, rgb) in &bands {
            let d = 1.0 - ((y as f32 - cy) / 70.0).abs();
            if d <= 0.0 {
                continue;
            }
            let (br, bg, bb) = parts(rgb);
            let a = d * d * 0.35 * beat * k;
            for p in row.iter_mut() {
                let (ar, ag, ab) = parts(*p);
                *p = pack_px(ar + br * a, ag + bg * a, ab + bb * a);
            }
        }
    });
}

/// Stars streaking out of the centre over the last bar.
fn stars(f: &mut Frame, local: f32, warp: f32) {
    if warp <= 0.0 {
        return;
    }
    let (cx, cy) = (W as f32 * 0.5, H as f32 * 0.5);
    for i in 0..420u32 {
        let ang = hash(i) * std::f32::consts::TAU;
        let z = (hash(i + 7919) - local * (0.4 + 2.5 * warp))
            .rem_euclid(1.0)
            .max(0.03);
        let r = 50.0 / z;
        let len = (18.0 * warp / z).min(260.0);
        for s in 0..(len as i32 / 6).max(1) {
            let rr = r - s as f32 * 6.0;
            let (x, y) = (cx + ang.cos() * rr, cy + ang.sin() * rr * 0.7);
            f.blend(
                x as i32,
                y as i32,
                mix(MINT, 0xffffff, 0.5),
                warp * (1.0 - s as f32 * 6.0 / len),
            );
            f.blend(x as i32 + 1, y as i32, MINT, 0.5 * warp);
        }
    }
}

/// The month as a wave of glyphs: x the day, depth the ward's shifts, the
/// height a ripple running along the days. On the warp it rushes forward.
fn wave(f: &mut Frame, local: f32, warp: f32, a: f32) {
    if a <= 0.0 {
        return;
    }
    let (cx, horizon) = (W as f32 * 0.5, 610.0);
    let mut row = 0usize;
    for &(rows, weekend_rows, rgb) in WARDS.iter() {
        for r in 0..rows {
            // Ward A nearest, ward C at the back.
            let z = 1.5 + row as f32 * 0.24 - warp * warp * 8.0;
            row += 1;
            if z <= 0.4 {
                continue;
            }
            for d in 0..DAYS {
                // Weekends run thinner: every ward staffs fewer shifts.
                let weekend = d % 7 >= 5;
                if weekend && r >= weekend_rows {
                    continue;
                }
                let x = d as f32 - 13.5;
                let ripple = (d as f32 * 0.45 - local * 3.2 + row as f32 * 0.3).sin()
                    * 0.35
                    * (1.0 + 0.4 * (-(local % BEAT) * 6.0).exp());
                let y = 1.3 + ripple;
                let (px, py) = (cx + x * 130.0 / z, horizon + y * 260.0 / z);
                if px < 0.0 || py < 0.0 || px >= W as f32 || py >= H as f32 {
                    continue;
                }
                let near = (2.0 / z).min(1.6);
                let scale = if near > 1.1 {
                    4
                } else if near > 0.6 {
                    3
                } else {
                    2
                };
                let ch = if weekend {
                    '.'
                } else if ripple > 0.15 {
                    '#'
                } else {
                    '+'
                };
                f.char_at(
                    px as i32 - advance(scale) / 2,
                    py as i32,
                    ch,
                    scale,
                    rgb,
                    a * (0.35 + 0.5 * near),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The wave holds exactly the dataset's shifts: 3 wards x 28 days,
    /// fewer at weekends.
    #[test]
    fn the_wave_draws_every_shift_once() {
        let mut n = 0;
        for &(rows, weekend_rows, _) in WARDS.iter() {
            for r in 0..rows {
                for d in 0..DAYS {
                    if !(d % 7 >= 5 && r >= weekend_rows) {
                        n += 1;
                    }
                }
            }
        }
        assert_eq!(n, STATS[2].0 as usize);
    }
}
