// ═══════════════════════════════════════════════════════════════
// SERIO_01 PALETTE -- Musica Universalis color system
// Brand emerald + celestial gold + cosmic deep space
// ═══════════════════════════════════════════════════════════════
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
pub const EMERALD_300: u32 = rgb_const(110, 231, 183);
pub const EMERALD_400: u32 = rgb_const(52, 211, 153);
pub const EMERALD_500: u32 = rgb_const(16, 185, 129); // PRIMARY
pub const EMERALD_600: u32 = rgb_const(5, 150, 105);
pub const EMERALD_700: u32 = rgb_const(4, 120, 87);
pub const EMERALD_800: u32 = rgb_const(6, 95, 70);
pub const GREEN_500: u32 = rgb_const(34, 197, 94);

// ── Neutrals ─────────────────────────────────────────────────
pub const NEAR_BLACK: u32 = rgb_const(6, 8, 18); // deep space
pub const MIDNIGHT: u32 = rgb_const(12, 16, 32);
pub const WHITE: u32 = 0xFFFFFF;
pub const CHROME: u32 = rgb_const(220, 220, 235);

// ── Celestial / Pythagorean Gold ─────────────────────────────
pub const GOLD: u32 = rgb_const(255, 215, 0); // pure gold
pub const AMBER_400: u32 = rgb_const(251, 191, 36); // amber
pub const AMBER_600: u32 = rgb_const(217, 119, 6); // deep amber

// ── Planetary Colors (classical 7) ───────────────────────────
pub const LUNA: u32 = rgb_const(210, 215, 230); // silver-white
pub const MERCURY_COL: u32 = rgb_const(180, 165, 140); // warm grey
pub const VENUS_COL: u32 = rgb_const(240, 210, 120); // golden-cream
pub const EARTH_COL: u32 = rgb_const(60, 160, 200); // blue-green
pub const MARS_COL: u32 = rgb_const(200, 80, 50); // rust-red
pub const JUPITER_COL: u32 = rgb_const(220, 170, 100); // amber-tan
pub const SATURN_COL: u32 = rgb_const(210, 195, 140); // pale gold

// ── Harmonic arc colors (consonance gradient) ────────────────
// Maps consonance [0.0=dissonant, 1.0=perfect] to color
pub fn harmonic_color(consonance: f32) -> u32 {
    let c = consonance.clamp(0.0, 1.0);
    if c > 0.9 {
        // Perfect interval: radiant gold
        let t = (c - 0.9) / 0.1;
        lerp_color(AMBER_400, GOLD, t)
    } else if c > 0.7 {
        // Good consonance: emerald
        let t = (c - 0.7) / 0.2;
        lerp_color(EMERALD_500, AMBER_400, t)
    } else if c > 0.4 {
        // Mediocre: dim emerald to amber
        let t = (c - 0.4) / 0.3;
        lerp_color(EMERALD_800, EMERALD_500, t)
    } else {
        // Dissonant: deep red
        let t = c / 0.4;
        lerp_color(rgb_const(120, 20, 20), rgb_const(200, 50, 30), t)
    }
}

// ── Score delta colors ───────────────────────────────────────
pub const SCORE_IMPROVE: u32 = rgb_const(80, 220, 120); // green: improving
pub const SCORE_WORSEN: u32 = rgb_const(220, 70, 50); // red: worsening
pub const SCORE_NEUTRAL: u32 = rgb_const(180, 180, 180); // grey: no change

// ── SERIO panel label colors ─────────────────────────────────
pub const SERIO_BLUE: u32 = rgb_const(60, 160, 255); // SERIO side label
pub const CLASSICAL_RED: u32 = rgb_const(220, 80, 60); // classical side label

// ── Accent ───────────────────────────────────────────────────
pub const RUST: u32 = rgb_const(206, 66, 43);

// ── Plasma background (space-black / deep blue-green) ────────
pub fn plasma_color(v: f32) -> u32 {
    let v = v.clamp(0.0, 1.0);
    if v < 0.2 {
        let t = v / 0.2;
        lerp_color(NEAR_BLACK, rgb_const(0, 20, 40), t)
    } else if v < 0.5 {
        let t = (v - 0.2) / 0.3;
        lerp_color(rgb_const(0, 20, 40), rgb_const(0, 60, 80), t)
    } else if v < 0.75 {
        let t = (v - 0.5) / 0.25;
        lerp_color(rgb_const(0, 60, 80), EMERALD_800, t)
    } else {
        let t = (v - 0.75) / 0.25;
        lerp_color(EMERALD_800, EMERALD_600, t)
    }
}

/// Scrolltext gradient — emerald → gold → amber → gold → emerald
pub fn scroll_gradient(t: f32) -> u32 {
    let t = ((t % 1.0) + 1.0) % 1.0;
    if t < 0.25 {
        lerp_color(EMERALD_400, GOLD, t / 0.25)
    } else if t < 0.50 {
        lerp_color(GOLD, AMBER_400, (t - 0.25) / 0.25)
    } else if t < 0.75 {
        lerp_color(AMBER_400, GOLD, (t - 0.50) / 0.25)
    } else {
        lerp_color(GOLD, EMERALD_400, (t - 0.75) / 0.25)
    }
}

/// Starfield color — cool blue-white for deep space
pub fn star_color(brightness: f32) -> u32 {
    let b = (brightness.clamp(0.0, 1.0) * 255.0) as u8;
    if brightness > 0.85 {
        WHITE
    } else if brightness > 0.5 {
        rgb(b / 2, b / 2 + 20, b)
    } else {
        rgb(0, 0, b / 3)
    }
}

// Const RGB helper
const fn rgb_const(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
