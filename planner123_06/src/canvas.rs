// The screen is a 128x40 ratatui Buffer of 10x18-pixel cells (1280x720).
// Every view draws into that Buffer with ratatui's own widgets or by setting
// cells; post.rs turns cells into pixels. These are the grid constants and
// the 0xRRGGBB colour helpers shared by the views, the post pass and the logo.
use ratatui::style::Color;

pub const COLS: u16 = 128;
pub const ROWS: u16 = 40;
pub const CELL_W: usize = 10;
pub const CELL_H: usize = 18;

pub fn rgb_parts(rgb: u32) -> (f32, f32, f32) {
    (
        ((rgb >> 16) & 0xFF) as f32,
        ((rgb >> 8) & 0xFF) as f32,
        (rgb & 0xFF) as f32,
    )
}

pub fn pack(r: f32, g: f32, b: f32) -> u32 {
    let c = |v: f32| v.round().clamp(0.0, 255.0) as u32;
    (c(r) << 16) | (c(g) << 8) | c(b)
}

pub fn scale_rgb(rgb: u32, gain: f32) -> u32 {
    let (r, g, b) = rgb_parts(rgb);
    pack(r * gain, g * gain, b * gain)
}

pub fn lerp_rgb(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let (ar, ag, ab) = rgb_parts(a);
    let (br, bg, bb) = rgb_parts(b);
    pack(ar + (br - ar) * t, ag + (bg - ag) * t, ab + (bb - ab) * t)
}

/// Sample a colour ramp (evenly spaced stops) at t in [0,1].
pub fn ramp(stops: &[u32], t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
    let i = (t as usize).min(stops.len() - 2);
    lerp_rgb(stops[i], stops[i + 1], t - i as f32)
}

/// 0xRRGGBB as a ratatui colour.
pub fn c(rgb: u32) -> Color {
    Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// A ratatui colour back to 0xRRGGBB, `default` for the non-RGB variants.
pub fn to_rgb(color: Color, default: u32) -> u32 {
    match color {
        Color::Rgb(r, g, b) => ((r as u32) << 16) | ((g as u32) << 8) | b as u32,
        _ => default,
    }
}
