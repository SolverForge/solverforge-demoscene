// The demo is a 128x40 text grid plus a 64x20 double-size grid for copy and
// the scroller. Every layer -- mountains, water, sparks, glyphfx clips --
// writes cells; post.rs turns cells into pixels.

pub const COLS: usize = 128;
pub const ROWS: usize = 40;
pub const BIG_COLS: usize = COLS / 2;
pub const BIG_ROWS: usize = ROWS / 2;
pub use demofx::{Cell, CELL_H, CELL_W};

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

pub struct TextCanvas {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Cell>,
}

impl TextCanvas {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![Cell::BLANK; cols * rows],
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::BLANK);
    }

    pub fn set(&mut self, col: i32, row: i32, cell: Cell) {
        if col >= 0 && row >= 0 && (col as usize) < self.cols && (row as usize) < self.rows {
            self.cells[row as usize * self.cols + col as usize] = cell;
        }
    }

    /// Shift every cell sideways (screen shake on hammer hits).
    pub fn shift(&mut self, dx: i32, dy: i32) {
        if dx == 0 && dy == 0 {
            return;
        }
        let old = self.cells.clone();
        let (cols, rows) = (self.cols as i32, self.rows as i32);
        for row in 0..rows {
            for col in 0..cols {
                let (sc, sr) = (col - dx, row - dy);
                let cell = if sc >= 0 && sr >= 0 && sc < cols && sr < rows {
                    old[(sr * cols + sc) as usize]
                } else {
                    Cell::BLANK
                };
                self.cells[(row * cols + col) as usize] = cell;
            }
        }
    }
}

impl demofx::CellSink for TextCanvas {
    fn set(&mut self, col: i32, row: i32, cell: Cell) {
        TextCanvas::set(self, col, row, cell);
    }
}
