// One text cell: a character and its 24-bit colour. glyphfx frames, story
// cards and the textmode demos are all made of these.

/// Pixel size of one cell at scale 1: the 8x8 font, double height, padded.
pub const CELL_W: usize = 10;
pub const CELL_H: usize = 18;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub rgb: u32,
    /// The cell's background colour, or `NO_BG` where the effect paints none
    /// (glyphfx's originals light whole cells: copper's bars, comet's and
    /// burnout's glow).
    pub bg: u32,
}

/// No background: the picture under the cell shows.
pub const NO_BG: u32 = u32::MAX;

impl Cell {
    pub const BLANK: Cell = Cell {
        ch: ' ',
        rgb: 0,
        bg: NO_BG,
    };

    pub fn new(ch: char, rgb: u32) -> Cell {
        Cell { ch, rgb, bg: NO_BG }
    }

    pub fn with_bg(ch: char, rgb: u32, bg: u32) -> Cell {
        Cell { ch, rgb, bg }
    }

    pub fn scaled(self, gain: f32) -> Cell {
        Cell {
            ch: self.ch,
            rgb: scale_rgb(self.rgb, gain),
            bg: if self.bg == NO_BG {
                NO_BG
            } else {
                scale_rgb(self.bg, gain)
            },
        }
    }
}

/// Anything cells can be written into: a text grid, or a pixel card.
pub trait CellSink {
    fn set(&mut self, col: i32, row: i32, cell: Cell);
}

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
