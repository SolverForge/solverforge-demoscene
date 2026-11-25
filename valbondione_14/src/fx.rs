// Small deterministic helpers: integer hashes, value noise and fbm, easing.
// Every "random" value in the demo is a hash of an index, a seed and the
// time, so `--render` is bit-identical run to run.

pub fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// 0..1 from two integers.
pub fn h2(a: i32, b: i32) -> f32 {
    let v = hash((a as u32).wrapping_mul(0x9E37_79B1) ^ hash((b as u32).wrapping_add(0x85EB_CA6B)));
    (v >> 8) as f32 / (1u32 << 24) as f32
}

pub fn h1(a: i32) -> f32 {
    h2(a, 0x2C1B)
}

/// Smooth value noise in 0..1.
pub fn noise(x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (fx, fy) = (x - xi, y - yi);
    let (xi, yi) = (xi as i32, yi as i32);
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = h2(xi, yi);
    let b = h2(xi + 1, yi);
    let c = h2(xi, yi + 1);
    let d = h2(xi + 1, yi + 1);
    let top = a + (b - a) * ux;
    let bot = c + (d - c) * ux;
    top + (bot - top) * uy
}

/// Fractal noise, `oct` octaves, 0..1.
pub fn fbm(x: f32, y: f32, oct: u32) -> f32 {
    let (mut sum, mut amp, mut norm, mut f) = (0.0, 0.5, 0.0, 1.0);
    for i in 0..oct {
        sum += amp * noise(x * f + i as f32 * 17.3, y * f - i as f32 * 9.1);
        norm += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    sum / norm
}

pub fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// 0 before `a`, 1 after `b`, smooth between.
pub fn window(x: f32, a: f32, b: f32) -> f32 {
    smooth((x - a) / (b - a))
}

pub fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

pub fn ease_in_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
    }
}
