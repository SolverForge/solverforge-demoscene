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
