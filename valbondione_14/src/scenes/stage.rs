// What a scene draws with: the text grids it paints (the world, and a
// front layer for what passes in front of the figures), the pixel work it
// asks for under and over the world (the sky gradient and the moon under;
// sprites, glows, lines, beams, copper and the mark over), and the shared
// assets.
use crate::amiga::Fire;
use crate::ascii::Grid;
use crate::glyphs::Glyphs;
use crate::post::Look;
use crate::sprites::{Cast, Id};
use crate::tour::Tour;
use crate::world;

/// Pixel-space work.
#[derive(Clone, Copy, Debug)]
pub enum Px {
    /// The sky's smooth gradient down to `to_y`.
    Sky {
        stops: [u32; 4],
        n: usize,
        to_y: f32,
    },
    /// The moon: a soft disc with its seas, and its halo.
    Moon { x: f32, y: f32, r: f32, k: f32 },
    /// A character sprite, anchored at its feet.
    Sprite {
        id: Id,
        x: i32,
        y: i32,
        light: u32,
        light_k: f32,
        rim: Option<(u32, i32, f32)>,
        alpha: f32,
        bob: i32,
    },
    /// Additive soft light.
    Glow {
        x: f32,
        y: f32,
        r: f32,
        rgb: u32,
        k: f32,
    },
    /// Additive line.
    Line {
        a: (f32, f32),
        b: (f32, f32),
        w: f32,
        rgb: u32,
        k: f32,
    },
    /// Copper bars swinging through rows y0..y1.
    Copper { y0: f32, y1: f32, k: f32 },
    /// A searchlight cone.
    Beam {
        x: f32,
        y: f32,
        dir: f32,
        width: f32,
        len: f32,
        rgb: u32,
        k: f32,
    },
    /// The SolverForge mark drawing itself.
    Mark {
        cx: f32,
        cy: f32,
        r: f32,
        progress: f32,
        gain: f32,
    },
}

pub struct Stage<'a> {
    pub g: &'a mut Grid,
    pub front: &'a mut Grid,
    pub under: &'a mut Vec<Px>,
    pub px: &'a mut Vec<Px>,
    pub cast: &'a Cast,
    pub glyphs: &'a Glyphs,
    pub fire: &'a mut Fire,
    pub tour: &'a Tour,
    pub look: Look,
    /// The corner logo's gain this frame.
    pub corner: f32,
}

impl Stage<'_> {
    /// Sky down to `to_row`: a smooth pixel gradient through `stops` (up to
    /// four) behind clear cells.
    pub fn sky(&mut self, stops: &[u32], to_row: i32) {
        world::sky(self.g, stops, to_row);
        let mut s = [0u32; 4];
        for (d, v) in s.iter_mut().zip(stops) {
            *d = *v;
        }
        self.under.push(Px::Sky {
            stops: s,
            n: stops.len().min(4),
            to_y: to_row as f32 * 18.0,
        });
    }

    /// The moon at cell (col, row), `r` pixels across.
    pub fn moon(&mut self, col: f32, row: f32, r: f32, k: f32) {
        let (x, y) = cell_px(col, row);
        self.under.push(Px::Moon { x, y, r, k });
    }

    pub fn glow(&mut self, x: f32, y: f32, r: f32, rgb: u32, k: f32) {
        if k > 0.0 {
            self.px.push(Px::Glow { x, y, r, rgb, k });
        }
    }

    pub fn line(&mut self, a: (f32, f32), b: (f32, f32), w: f32, rgb: u32, k: f32) {
        if k > 0.0 {
            self.px.push(Px::Line { a, b, w, rgb, k });
        }
    }

    /// A sprite with its feet at pixel (x, y).
    #[allow(clippy::too_many_arguments)]
    pub fn sprite(
        &mut self,
        id: Id,
        x: f32,
        y: f32,
        light: u32,
        light_k: f32,
        rim: Option<(u32, i32, f32)>,
        alpha: f32,
        bob: i32,
    ) {
        if alpha > 0.0 {
            self.px.push(Px::Sprite {
                id,
                x: x as i32,
                y: y as i32,
                light,
                light_k,
                rim,
                alpha,
                bob,
            });
        }
    }
}

/// Pixel centre of a cell.
pub fn cell_px(c: f32, r: f32) -> (f32, f32) {
    (c * 10.0 + 5.0, r * 18.0 + 9.0)
}
