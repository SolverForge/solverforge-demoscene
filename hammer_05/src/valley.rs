// VALBONDIONE: the Orobie wall at night, the Barbellino water released over
// it in three falls, mist, stars. All glyphs.
use crate::canvas::{lerp_rgb, ramp, scale_rgb, Cell, TextCanvas, COLS, ROWS};
use crate::noise::{fbm, h1, h2, h3, pick, shade_char};

const WATER: [u32; 4] = [0x1c3c6c, 0x6fb6ff, 0xe8f6ff, 0xffffff];
const FALL_CENTER: f32 = 63.0;

fn far_ridge(c: f32) -> f32 {
    11.0 + 3.0 * (c * 0.07 + 1.0).sin() + 2.0 * (c * 0.19 + 0.4).sin() + 1.2 * (c * 0.41).sin()
}

fn wall_top(c: f32) -> f32 {
    let dx = (c - FALL_CENTER).abs();
    let base = if dx < 5.0 {
        7.0
    } else if dx < 25.0 {
        7.0 - (dx - 5.0) * 0.25
    } else {
        2.0 + (dx - 25.0) * 0.27
    };
    base + 0.8 * (c * 0.9).sin() + 0.5 * (c * 2.3 + 1.0).sin()
}

fn grass(c: f32, r: f32) -> bool {
    (c < 60.0 && r > 25.0 + c * 0.24) || (c > 80.0 && r > 31.0 + (127.0 - c) * 0.11)
}

/// The three falls: (center col, lean in cols per row, top row, bottom row,
/// half width at full flow).
const FALLS: [(f32, f32, f32, f32, f32); 3] = [
    (63.0, 0.0, 7.0, 17.0, 4.6),
    (67.0, 0.05, 19.0, 27.0, 3.8),
    (66.0, -0.45, 29.0, 38.0, 3.4),
];

pub fn draw(canvas: &mut TextCanvas, t: f32, flow: f32, light: f32) {
    let tq = (t * 12.0) as i32;
    for r in 0..ROWS as i32 {
        for c in 0..COLS as i32 {
            let (cf, rf) = (c as f32, r as f32);
            let cell = if rf >= wall_top(cf) {
                if grass(cf, rf) {
                    let v = h2(c, r);
                    let ch = pick(&['"', ',', '\'', '.', '"', ';'], v);
                    Cell::new(ch, scale_rgb(0x3f7a33, light * (0.45 + 0.4 * v)))
                } else {
                    // Rock: ASCII-shaded relief, lit from the left and by the water.
                    if rf < wall_top(cf) + 1.0 {
                        canvas.set(c, r, Cell::new('^', scale_rgb(0x8a96aa, light)));
                        continue;
                    }
                    let relief = fbm(cf * 0.09, rf * 0.16, 3);
                    let ridge = fbm(cf * 0.09 + 0.15, rf * 0.16 - 0.1, 3);
                    let lit = (0.5 + (relief - ridge) * 4.0).clamp(0.0, 1.0);
                    let near = (-((cf - FALL_CENTER) / 12.0).powi(2)).exp() * flow;
                    let v = (relief * 0.6 + lit * 0.5 - 0.15 + near * 0.08).clamp(0.0, 1.0);
                    let ch = shade_char(v);
                    if ch == ' ' {
                        continue;
                    }
                    let base = lerp_rgb(0x56606e, 0x6a86aa, near * 0.5);
                    Cell::new(ch, scale_rgb(base, light * (0.45 + 0.75 * v)))
                }
            } else if rf >= far_ridge(cf) {
                let v = h2(c, r);
                Cell::new(
                    pick(&['.', ':', '.', '\''], v),
                    scale_rgb(0x2a3650, light * (0.5 + 0.4 * v)),
                )
            } else {
                let v = h2(c, r);
                if v < 0.03 {
                    let tw = 0.55 + 0.45 * (t * (1.0 + v * 60.0) + v * 400.0).sin();
                    let ch = if v < 0.006 { '+' } else { '.' };
                    Cell::new(ch, scale_rgb(0xc8d8ff, tw * (1.0 - light * 0.4)))
                } else {
                    continue;
                }
            };
            canvas.set(c, r, cell);
        }
    }

    // Rock shelves between the drops.
    for &(center, lean, top, bottom, width) in &FALLS[..2] {
        let cx = center + lean * (bottom - top);
        let reach = width + 7.0;
        for c in (cx - reach) as i32..=(cx + reach + 4.0) as i32 {
            let v = h2(c, 99);
            canvas.set(
                c,
                bottom as i32 + 1,
                Cell::new(
                    if v < 0.7 { '▀' } else { '=' },
                    scale_rgb(0x8a94a6, light * (0.7 + 0.3 * v)),
                ),
            );
        }
    }

    // The three falls.
    for (k, &(center, lean, top, bottom, width)) in FALLS.iter().enumerate() {
        let half = 0.6 + width * flow;
        for r in top as i32..=bottom as i32 {
            let cx = center + lean * (r as f32 - top);
            for c in (cx - half - 1.0) as i32..=(cx + half + 1.0) as i32 {
                let x = (c as f32 - cx) / half;
                if x.abs() > 1.0 {
                    continue;
                }
                let core = (1.0 - x * x).sqrt();
                let speed = 20.0 + h1(c * 3 + k as i32) * 14.0;
                let len = 3.0 + h1(c + 40 * k as i32) * 5.0;
                let phase = ((r as f32 - t * speed) / len + h1(c * 11 + k as i32)).rem_euclid(1.0);
                let streak = (1.0 - phase).powf(2.5);
                let level =
                    (0.3 + 0.7 * flow.min(1.0)) * (0.55 + 0.45 * core) * (0.6 + 0.4 * streak);
                if level < 0.1 {
                    continue;
                }
                let ch = if core > 0.4 && streak > 0.35 {
                    '|'
                } else {
                    pick(&['!', ':', '\'', '|', '.'], h3(c, r, tq))
                };
                canvas.set(c, r, Cell::new(ch, ramp(&WATER, level)));
            }
        }
        // Splash where each fall lands.
        let cx = center + lean * (bottom - top);
        let spread = 2.0 + 5.0 * flow;
        for r in bottom as i32..=bottom as i32 + 1 {
            for c in (cx - spread) as i32..=(cx + spread) as i32 {
                let v = h3(c, r, tq + k as i32 * 31);
                let fall_off = 1.0 - ((c as f32 - cx).abs() / spread);
                if v < fall_off * flow * 0.9 {
                    let ch = pick(&['~', '*', 'o', '.', '~'], h3(c, r, tq + 5));
                    canvas.set(c, r, Cell::new(ch, ramp(&WATER, 0.4 + 0.6 * v)));
                }
            }
        }
    }

    // Plunge pool.
    for c in 48..80 {
        let wave = ((c as f32 * 0.6 - t * 5.0).sin() * 0.5 + 0.5) * flow.max(0.2);
        canvas.set(
            c,
            39,
            Cell::new(
                if wave > 0.5 { '~' } else { '-' },
                ramp(&WATER, 0.2 + wave * 0.5),
            ),
        );
    }

    // Mist rising off the ledges.
    for i in 0..170 {
        let period = 3.0 + h1(i) * 3.0;
        let age = (t + h1(i + 7) * period).rem_euclid(period);
        let (center, lean, top, bottom, _) = FALLS[i as usize % 3];
        let ox = center + lean * (bottom - top);
        let x = ox
            + (h1(i + 3) - 0.5) * 18.0
            + (age * 1.3 + i as f32).sin() * 2.0
            + age * (h1(i + 5) - 0.5) * 7.0;
        let y = bottom + 1.0 - age * 2.4;
        let alpha = flow * (std::f32::consts::PI * age / period).sin();
        if alpha > 0.08 {
            let ch = pick(&['.', '\'', '~', ':', '.'], h1(i + 9));
            canvas.set(
                x as i32,
                y as i32,
                Cell::new(ch, scale_rgb(0xbcd8f0, alpha * 0.7)),
            );
        }
    }
}
