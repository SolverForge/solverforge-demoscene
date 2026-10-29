// Deterministic hashes: every "random" glyph is a pure function of its cell,
// its seed and (quantised) time, so the headless render is bit-identical.

pub fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// 0..1 from up to three integer coordinates.
pub fn h3(a: i32, b: i32, c: i32) -> f32 {
    let v = hash(
        (a as u32).wrapping_mul(0x9E37_79B1)
            ^ hash((b as u32).wrapping_add(0x85EB_CA6B) ^ hash(c as u32)),
    );
    (v >> 8) as f32 / (1u32 << 24) as f32
}

pub fn h2(a: i32, b: i32) -> f32 {
    h3(a, b, 0x51ED)
}

pub fn h1(a: i32) -> f32 {
    h3(a, 0x2C1B, 0x7A3D)
}

pub fn pick(chars: &[char], v: f32) -> char {
    chars[((v * chars.len() as f32) as usize).min(chars.len() - 1)]
}

/// Smooth value noise in 0..1.
pub fn vnoise(x: f32, y: f32, seed: i32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - xi as f32, y - yi as f32);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = h3(xi, yi, seed);
    let b = h3(xi + 1, yi, seed);
    let c = h3(xi, yi + 1, seed);
    let d = h3(xi + 1, yi + 1, seed);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

/// Three octaves of value noise, 0..1.
pub fn fbm(x: f32, y: f32, seed: i32) -> f32 {
    (vnoise(x, y, seed) * 0.55
        + vnoise(x * 2.1, y * 2.1, seed + 1) * 0.3
        + vnoise(x * 4.3, y * 4.3, seed + 2) * 0.15)
        .clamp(0.0, 1.0)
}

/// Classic ASCII-art density ramp, darkest to brightest.
pub const SHADE: [char; 10] = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

pub fn shade_char(v: f32) -> char {
    SHADE[((v.clamp(0.0, 0.999)) * SHADE.len() as f32) as usize]
}
