// The textmode world: a 192x60 grid of 10x18 cells over the 1920x1080
// frame. Every scene draws its picture here as characters, each with its
// own ink and its own cell colour, the way a colour text screen does; the
// grid is then blitted once per frame. Amiga effects (plasma, tunnel,
// rotozoom, Mode 7 floors, fire) are shaders evaluated per cell and turned
// into characters by brightness.
use demofx::font::mask;
use demofx::{CELL_H, CELL_W};

use crate::gfx::{Frame, H, W};

pub const CW: i32 = CELL_W as i32;
pub const CH: i32 = CELL_H as i32;
pub const COLS: i32 = W as i32 / CW;
pub const ROWS: i32 = H as i32 / CH;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: u32,
    pub bg: u32,
}

impl Cell {
    pub const EMPTY: Cell = Cell {
        ch: ' ',
        fg: 0,
        bg: 0,
    };
}

/// A cell colour that lets the pixel layer under the grid show (the sky
/// gradient, the moon): never produced by colour arithmetic.
pub const CLEAR: u32 = 0xFF00_0000;

/// Brightness ramp, dark to bright.
pub const RAMP: [char; 12] = [' ', '.', '`', ':', '-', '~', '=', '+', '*', '#', '%', '@'];

pub fn ramp_char(l: f32) -> char {
    let i = (l.clamp(0.0, 0.999) * RAMP.len() as f32) as usize;
    RAMP[i]
}

#[derive(Clone)]
pub struct Grid {
    pub cells: Vec<Cell>,
    /// The sky colour behind each row, for cells that are CLEAR.
    pub sky: Vec<u32>,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::EMPTY; (COLS * ROWS) as usize],
            sky: vec![0; ROWS as usize],
        }
    }

    pub fn clear(&mut self, bg: u32) {
        self.cells.fill(Cell { ch: ' ', fg: 0, bg });
        self.sky.fill(if bg == CLEAR { 0 } else { bg });
    }

    /// The colour behind a cell: its own, or the sky's where it is clear.
    #[inline]
    pub fn behind(&self, c: i32, r: i32) -> u32 {
        let cell = self.at(c, r);
        if cell.bg == CLEAR {
            self.sky[r.clamp(0, ROWS - 1) as usize]
        } else {
            cell.bg
        }
    }

    /// Make a cell opaque (resolving CLEAR to the sky behind it), for
    /// passes that blend into its colour.
    #[inline]
    pub fn solid(&mut self, c: i32, r: i32) -> Option<&mut Cell> {
        if !Self::inside(c, r) {
            return None;
        }
        let sky = self.sky[r as usize];
        let cell = &mut self.cells[(r * COLS + c) as usize];
        if cell.bg == CLEAR {
            cell.bg = sky;
        }
        Some(cell)
    }

    #[inline]
    pub fn inside(c: i32, r: i32) -> bool {
        (0..COLS).contains(&c) && (0..ROWS).contains(&r)
    }

    #[inline]
    pub fn at(&self, c: i32, r: i32) -> Cell {
        if Self::inside(c, r) {
            self.cells[(r * COLS + c) as usize]
        } else {
            Cell::EMPTY
        }
    }

    #[inline]
    pub fn cell(&mut self, c: i32, r: i32) -> Option<&mut Cell> {
        if Self::inside(c, r) {
            Some(&mut self.cells[(r * COLS + c) as usize])
        } else {
            None
        }
    }

    /// Ink a character, keeping the cell colour.
    #[inline]
    pub fn put(&mut self, c: i32, r: i32, ch: char, fg: u32) {
        if let Some(cell) = self.cell(c, r) {
            cell.ch = ch;
            cell.fg = fg;
        }
    }

    #[inline]
    pub fn set(&mut self, c: i32, r: i32, ch: char, fg: u32, bg: u32) {
        if let Some(cell) = self.cell(c, r) {
            *cell = Cell { ch, fg, bg };
        }
    }

    /// Ink a character only where it is brighter than what is there: light
    /// sources (stars, fireflies, sparks) never darken the scene under them.
    pub fn light(&mut self, c: i32, r: i32, ch: char, fg: u32) {
        if let Some(cell) = self.cell(c, r) {
            if cell.ch == ' ' || luma(fg) >= luma(cell.fg) {
                cell.ch = ch;
                cell.fg = fg;
            }
        }
    }

    /// Mix another grid in, cell by cell: where `pick(c, r)` is true the
    /// other grid's cell wins (a dissolve), colours always mix by `k`.
    pub fn dissolve(&mut self, other: &Grid, k: f32, seed: i32) {
        for r in 0..ROWS {
            for c in 0..COLS {
                let i = (r * COLS + c) as usize;
                let mut o = other.cells[i];
                if o.bg == CLEAR {
                    o.bg = other.sky[r as usize];
                }
                let n = crate::fx::h2(c * 7 + seed, r * 13 - seed);
                let sky = self.sky[r as usize];
                let cell = &mut self.cells[i];
                if cell.bg == CLEAR {
                    cell.bg = sky;
                }
                let win = n < k;
                cell.bg = crate::gfx::mix(cell.bg, o.bg, k);
                if win {
                    cell.ch = o.ch;
                    cell.fg = crate::gfx::mix(cell.fg, o.fg, (k * 1.5).min(1.0));
                } else {
                    cell.fg = crate::gfx::mix(cell.fg, o.fg, k * 0.5);
                }
            }
        }
    }
}

pub fn luma(rgb: u32) -> u32 {
    let (r, g, b) = ((rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255);
    (r * 3 + g * 6 + b) / 10
}

/// Turns grids into pixels. Glyph bitmaps are the series' font (demofx).
pub struct Blitter {
    masks: Vec<[u16; CELL_H]>,
    extra: Vec<(char, [u16; CELL_H])>,
}

impl Blitter {
    pub fn new() -> Self {
        let masks = (0u32..128)
            .map(|c| mask(char::from_u32(c).unwrap_or(' ')))
            .collect();
        let extra = ['█', '▀', '▄', '▌', '▐', '░', '▒', '▓', '│', '─', '╱', '╲']
            .iter()
            .map(|&c| (c, mask(c)))
            .collect();
        Self { masks, extra }
    }

    fn glyph(&self, ch: char) -> [u16; CELL_H] {
        let code = ch as u32;
        if code < 128 {
            self.masks[code as usize]
        } else {
            self.extra
                .iter()
                .find(|(c, _)| *c == ch)
                .map(|(_, m)| *m)
                .unwrap_or_else(|| mask(ch))
        }
    }

    /// Draw the grid over the frame, offset by (dx, dy) pixels. CLEAR cells
    /// leave what is under them. Rows of cells are split across cores.
    pub fn blit(&self, g: &Grid, f: &mut Frame, dx: i32, dy: i32) {
        let cores = std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .clamp(1, 16);
        let rows_per = (ROWS as usize).div_ceil(cores);
        let band_px = rows_per * CH as usize * W;
        std::thread::scope(|s| {
            for (band, chunk) in f.px.chunks_mut(band_px).enumerate() {
                s.spawn(move || {
                    let y_base = band * rows_per * CH as usize;
                    let rows_here = chunk.len() / W;
                    for ly in 0..rows_here {
                        let y = (y_base + ly) as i32 - dy;
                        let r = y.div_euclid(CH);
                        let gy = y.rem_euclid(CH) as usize;
                        let row = &mut chunk[ly * W..(ly + 1) * W];
                        if !(0..ROWS).contains(&r) {
                            continue;
                        }
                        let mut last_c = i32::MIN;
                        let mut bits = 0u16;
                        let mut cell = Cell::EMPTY;
                        for (x, px) in row.iter_mut().enumerate() {
                            let xs = x as i32 - dx;
                            let c = xs.div_euclid(CW);
                            if c != last_c {
                                last_c = c;
                                if (0..COLS).contains(&c) {
                                    cell = g.cells[(r * COLS + c) as usize];
                                    bits = if cell.ch == ' ' {
                                        0
                                    } else {
                                        self.glyph(cell.ch)[gy]
                                    };
                                } else {
                                    cell = Cell::EMPTY;
                                    bits = 0;
                                }
                            }
                            let gx = xs.rem_euclid(CW);
                            if (bits >> gx) & 1 == 1 {
                                *px = cell.fg;
                            } else if cell.bg != CLEAR {
                                *px = cell.bg;
                            }
                        }
                    }
                });
            }
        });
    }
}
