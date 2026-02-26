// ═══════════════════════════════════════════════════════════════
// SOLVERFORGE PALETTE -- Brand colors meeting Amiga demoscene
// ═══════════════════════════════════════════════════════════════
#![allow(dead_code)]

/// Pack RGB into u32 ARGB (minifb format: 0x00RRGGBB)
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
    let r = (((color >> 16) & 0xFF) as f32 * factor) as u8;
    let g = (((color >> 8) & 0xFF) as f32 * factor) as u8;
    let b = ((color & 0xFF) as f32 * factor) as u8;
    rgb(r, g, b)
}

/// Add two colors with clamping
#[inline(always)]
pub fn add_color(a: u32, b: u32) -> u32 {
    let r = (((a >> 16) & 0xFF) + ((b >> 16) & 0xFF)).min(255) as u8;
    let g = (((a >> 8) & 0xFF) + ((b >> 8) & 0xFF)).min(255) as u8;
    let b = ((a & 0xFF) + (b & 0xFF)).min(255) as u8;
    rgb(r, g, b)
}

// ── SolverForge Brand Colors ─────────────────────────────────
pub const EMERALD_300: u32 = rgb_const(110, 231, 183); // #6ee7b7
pub const EMERALD_400: u32 = rgb_const(52, 211, 153); // #34d399
pub const EMERALD_500: u32 = rgb_const(16, 185, 129); // #10b981  PRIMARY
pub const EMERALD_600: u32 = rgb_const(5, 150, 105); // #059669
pub const EMERALD_700: u32 = rgb_const(4, 120, 87); // #047857
pub const EMERALD_800: u32 = rgb_const(6, 95, 70); // #065f46
pub const GREEN_500: u32 = rgb_const(34, 197, 94); // #22c55e  (logo snake)

// ── Neutrals ─────────────────────────────────────────────────
pub const BLACK: u32 = 0x000000;
pub const NEAR_BLACK: u32 = rgb_const(10, 14, 26); // deep midnight
pub const MIDNIGHT: u32 = rgb_const(17, 24, 39); // #111827
pub const GRAY_700: u32 = rgb_const(55, 65, 81); // #374151
pub const WHITE: u32 = 0xFFFFFF;
pub const CHROME: u32 = rgb_const(220, 220, 235); // near-white chrome

// ── Synthwave Accent Colors (complement to emerald) ──────────
pub const CYAN_400: u32 = rgb_const(34, 211, 238); // electric cyan
pub const CYAN_500: u32 = rgb_const(6, 182, 212); // #06b6d4
pub const PURPLE_600: u32 = rgb_const(124, 58, 237); // deep purple
pub const PURPLE_400: u32 = rgb_const(167, 139, 250); // lavender
pub const MAGENTA: u32 = rgb_const(236, 72, 153); // hot magenta accent
pub const AMBER_400: u32 = rgb_const(251, 191, 36); // golden accent

// ── Brand Rust (from landing page --accent-rust: #CE422B) ────
pub const RUST: u32 = rgb_const(206, 66, 43); // #CE422B — brand rust
pub const RUST_DIM: u32 = rgb_const(168, 53, 31); // #A8351F — dimmed rust

// ── Copper bar palette (Amiga copper effect) ─────────────────
pub const COPPER_BARS: [u32; 16] = [
    rgb_const(0, 48, 32),
    rgb_const(0, 72, 48),
    rgb_const(5, 100, 68),
    rgb_const(10, 130, 88),
    rgb_const(16, 160, 110),
    rgb_const(16, 185, 129), // emerald 500 peak
    rgb_const(34, 211, 153),
    rgb_const(52, 230, 170),
    rgb_const(52, 230, 170),
    rgb_const(34, 211, 153),
    rgb_const(16, 185, 129),
    rgb_const(10, 130, 88),
    rgb_const(5, 100, 68),
    rgb_const(0, 72, 48),
    rgb_const(0, 48, 32),
    rgb_const(0, 24, 16),
];

// ── Plasma palette (smooth emerald-to-black gradient) ────────
pub fn plasma_color(v: f32) -> u32 {
    // v in [0.0, 1.0] -- map through emerald spectrum with deep space
    let v = v.clamp(0.0, 1.0);
    if v < 0.15 {
        // Deep space black
        let t = v / 0.15;
        lerp_color(NEAR_BLACK, rgb_const(0, 40, 28), t)
    } else if v < 0.35 {
        // Into deep emerald
        let t = (v - 0.15) / 0.20;
        lerp_color(rgb_const(0, 40, 28), EMERALD_800, t)
    } else if v < 0.55 {
        // Emerald 800 -> 600
        let t = (v - 0.35) / 0.20;
        lerp_color(EMERALD_800, EMERALD_600, t)
    } else if v < 0.72 {
        // Emerald core
        let t = (v - 0.55) / 0.17;
        lerp_color(EMERALD_600, EMERALD_500, t)
    } else if v < 0.86 {
        // Hot emerald -> cyan
        let t = (v - 0.72) / 0.14;
        lerp_color(EMERALD_500, CYAN_400, t)
    } else {
        // Cyan -> chrome white hot
        let t = (v - 0.86) / 0.14;
        lerp_color(CYAN_400, rgb_const(180, 255, 230), t)
    }
}

/// Alternate plasma palette -- deep purple / magenta for variety
pub fn plasma_color_alt(v: f32) -> u32 {
    let v = v.clamp(0.0, 1.0);
    if v < 0.2 {
        let t = v / 0.2;
        lerp_color(NEAR_BLACK, rgb_const(30, 0, 50), t)
    } else if v < 0.5 {
        let t = (v - 0.2) / 0.30;
        lerp_color(rgb_const(30, 0, 50), PURPLE_600, t)
    } else if v < 0.75 {
        let t = (v - 0.5) / 0.25;
        lerp_color(PURPLE_600, EMERALD_500, t)
    } else {
        let t = (v - 0.75) / 0.25;
        lerp_color(EMERALD_500, CYAN_400, t)
    }
}

/// Scrolltext gradient — cycles emerald → amber → rust → amber → emerald.
/// Routes through amber so no murky olive midpoints appear.
/// t in [0.0, 1.0) — wraps automatically.
pub fn scroll_gradient(t: f32) -> u32 {
    let t = ((t % 1.0) + 1.0) % 1.0;
    if t < 0.25 {
        lerp_color(EMERALD_400, AMBER_400, t / 0.25)
    } else if t < 0.50 {
        lerp_color(AMBER_400, RUST, (t - 0.25) / 0.25)
    } else if t < 0.75 {
        lerp_color(RUST, AMBER_400, (t - 0.50) / 0.25)
    } else {
        lerp_color(AMBER_400, EMERALD_400, (t - 0.75) / 0.25)
    }
}

/// Starfield color with intensity
pub fn star_color(brightness: f32) -> u32 {
    // Emerald-tinted stars
    let b = (brightness.clamp(0.0, 1.0) * 255.0) as u8;
    if brightness > 0.85 {
        // Hot white-green center
        WHITE
    } else if brightness > 0.6 {
        rgb(b / 2, b, (b as u32 * 3 / 4) as u8)
    } else {
        // Cool dim emerald
        rgb(0, b / 2, b / 4)
    }
}

/// Fade a color to black (for fade in/out transitions)
pub fn fade(color: u32, alpha: f32) -> u32 {
    dim(color, alpha.clamp(0.0, 1.0))
}

// Const RGB helper (works in const context)
const fn rgb_const(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
