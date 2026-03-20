// ═══════════════════════════════════════════════════════════════
// PALETTE -- SolverForge screensaver color system
// Only colors and helpers used by the production screensaver.
// ═══════════════════════════════════════════════════════════════

pub struct Surface<'a> {
    pub buf: &'a mut [u32],
    pub w: usize,
    pub h: usize,
}

/// Pack RGB into u32 (minifb format: 0x00RRGGBB)
#[inline(always)]
pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Blend two colors by factor t in [0.0, 1.0]
#[inline(always)]
pub fn lerp_color(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ar = ((a >> 16) & 0xFF) as f32;
    let ag = ((a >> 8) & 0xFF) as f32;
    let ab = (a & 0xFF) as f32;
    let br = ((b >> 16) & 0xFF) as f32;
    let bg = ((b >> 8) & 0xFF) as f32;
    let bb = (b & 0xFF) as f32;
    let r = (ar + (br - ar) * t) as u8;
    let g = (ag + (bg - ag) * t) as u8;
    let b = (ab + (bb - ab) * t) as u8;
    rgb(r, g, b)
}

/// Multiply a color by a scalar brightness [0.0, 1.0]
#[inline(always)]
pub fn dim(color: u32, factor: f32) -> u32 {
    let factor = factor.clamp(0.0, 1.0);
    let r = (((color >> 16) & 0xFF) as f32 * factor) as u8;
    let g = (((color >> 8) & 0xFF) as f32 * factor) as u8;
    let b = ((color & 0xFF) as f32 * factor) as u8;
    rgb(r, g, b)
}

/// Add two colors with clamping.
#[inline(always)]
pub fn add_color(a: u32, b: u32) -> u32 {
    let r = (((a >> 16) & 0xFF) + ((b >> 16) & 0xFF)).min(255) as u8;
    let g = (((a >> 8) & 0xFF) + ((b >> 8) & 0xFF)).min(255) as u8;
    let b = ((a & 0xFF) + (b & 0xFF)).min(255) as u8;
    rgb(r, g, b)
}

// ── SolverForge brand colors ─────────────────────────────────
pub const EMERALD_300: u32 = rgb_const(110, 231, 183);
pub const EMERALD_400: u32 = rgb_const(52, 211, 153);
pub const EMERALD_500: u32 = rgb_const(16, 185, 129);
pub const EMERALD_600: u32 = rgb_const(5, 150, 105);
pub const EMERALD_700: u32 = rgb_const(4, 120, 87);
pub const EMERALD_800: u32 = rgb_const(6, 95, 70);
pub const GREEN_500: u32 = rgb_const(34, 197, 94);

// ── Neutrals ─────────────────────────────────────────────────
pub const NEAR_BLACK: u32 = rgb_const(6, 8, 18);
pub const MIDNIGHT: u32 = rgb_const(12, 16, 32);
pub const WHITE: u32 = 0x00FF_FFFF;
pub const CHROME: u32 = rgb_const(220, 220, 235);

const fn rgb_const(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn bresenham(s: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, col: u32) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && y >= 0 && x < s.w as i32 && y < s.h as i32 {
            s.buf[y as usize * s.w + x as usize] = col;
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
