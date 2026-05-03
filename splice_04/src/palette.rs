#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub const VOID: Color = Color::new(3, 5, 6);
pub const STONE: Color = Color::new(22, 27, 30);
pub const STONE_DARK: Color = Color::new(9, 12, 14);
pub const EMERALD_DARK: Color = Color::new(16, 49, 34);
pub const EMERALD: Color = Color::new(46, 200, 124);
pub const MINT: Color = Color::new(188, 255, 230);
pub const GOLD: Color = Color::new(210, 180, 96);
pub const PANIC: Color = Color::new(255, 88, 88);
pub const GLASS: Color = Color::new(102, 180, 220);

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| -> u8 { (x as f32 + (y as f32 - x as f32) * t) as u8 };
    Color::new(lerp(a.r, b.r), lerp(a.g, b.g), lerp(a.b, b.b))
}
