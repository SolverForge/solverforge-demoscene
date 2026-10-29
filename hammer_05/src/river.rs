// VAL SERIANA: flying low down the river, the valley walls streaming past.
use crate::canvas::{ramp, scale_rgb, Cell, TextCanvas, COLS, ROWS};
use crate::noise::{h2, h3, pick};

const HORIZON: i32 = 9;
const WATER: [u32; 4] = [0x0d2440, 0x2f6fb0, 0x8fd0ff, 0xf0faff];

fn meander(z: f32) -> f32 {
    2.2 * (z * 0.06).sin() + 1.0 * (z * 0.17 + 1.3).sin()
}

pub fn draw(canvas: &mut TextCanvas, t: f32, light: f32) {
    let travel = t * 9.0;
    // Distant ridges above the horizon.
    for c in 0..COLS as i32 {
        let cf = c as f32;
        let ridge = HORIZON as f32 - 3.0 - 2.5 * (cf * 0.05 + 0.7).sin() - 1.5 * (cf * 0.17).sin();
        for r in ridge.max(0.0) as i32..HORIZON {
            let v = h2(c, r);
            canvas.set(
                c,
                r,
                Cell::new(
                    pick(&['^', '.', ':', '.'], v),
                    scale_rgb(0x3a4a66, light * (0.4 + 0.4 * v)),
                ),
            );
        }
    }
    for r in HORIZON..ROWS as i32 {
        let d = (r - HORIZON) as f32 / (ROWS as i32 - HORIZON) as f32; // 0 far .. 1 near
        let depth = 1.0 / (d + 0.04);
        let z = depth * 3.0 + travel;
        let scale = d * 46.0 + 1.5;
        let shade = (0.3 + 0.7 * d) * light;
        let center = meander(z);
        let zq = (z * 1.5) as i32;
        for c in 0..COLS as i32 {
            let xw = (c as f32 - 64.0) / scale - center;
            let ax = xw.abs();
            let cell = if ax < 1.0 {
                let v = h3((xw * 5.0) as i32, zq, 3);
                let glint = (((z * 2.0 + xw * 3.0).sin() * 0.5 + 0.5) * v).powf(1.5);
                let ch = if glint > 0.55 {
                    '~'
                } else {
                    pick(&['-', '~', '=', '-', '.'], v)
                };
                Cell::new(
                    ch,
                    scale_rgb(ramp(&WATER, 0.25 + 0.75 * glint), shade + 0.25),
                )
            } else if ax < 1.25 {
                Cell::new(
                    pick(&['.', ',', 'o'], h3(c, zq, 9)),
                    scale_rgb(0x8a8f80, shade),
                )
            } else if ax < 4.5 {
                let v = h3((xw * 3.0) as i32, zq, 5);
                Cell::new(
                    pick(&['"', ',', '\'', '.', ';'], v),
                    scale_rgb(0x3f7a33, shade * (0.5 + 0.5 * v)),
                )
            } else {
                let v = h3((xw * 2.0) as i32, zq, 7);
                Cell::new(
                    pick(&['#', '%', '/', '\\', '&'], v),
                    scale_rgb(0x4a5260, shade * (0.4 + 0.5 * v)),
                )
            };
            canvas.set(c, r, cell);
        }
    }
}
