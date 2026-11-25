// glyphfx in a film without words: every clip here animates glyph ART,
// never text. The vendored engine (through demofx) runs each effect once at
// startup on its own canvas; the frames land in the scene's text grid.
//   owl      decrypt   the owl resolves out of noise on its branch (the
//                      owl sprite read as text, then the sprite itself)
//   runes    matrix    the standing stones' runes wake in falling code
//   falls    pour      the released water pours down the three drops
//   kindling burn      the kindling catches
//   ring     swarm     the fireflies swarm and settle into a ring at dawn
use demofx::{Cell, CellSink, Clip, ClipSpec, NO_BG};

use crate::ascii::Grid;

pub const KINDLING: &str = "  /\\/\\/\\  
 <XXXXXXX> 
=#=#=#=#=#=";

/// The water of one drop: columns of falling strokes.
pub const DROP: &str = "|!|:|!|
!|:|!:|
|:!|:!|
:|!:|!:
|!|:|!|
!:|!|:!
|:!|:|!
!|:!|!:
|!:|:!|
:|!|!:|";

/// Rune faces for the standing stones, one per stone.
pub const RUNES: [&str; 6] = [
    "|>\n-+\n<|\n/\\\n=|\n+-",
    "/|\n|<\n+=\n|/\n>-\n=+",
    "\\/\n=|\n|+\n<>\n-|\n|=",
    "|+\n<|\n/-\n|>\n=/\n+|",
    "<|\n-=\n|\\\n+|\n|<\n/=",
    "=|\n|/\n>+\n|-\n<|\n\\=",
];

/// A ring of points, 31 columns by 17 rows (round on screen: cells are
/// 10x18 pixels).
pub fn ring() -> String {
    let (w, h) = (31usize, 17usize);
    let mut rows = vec![vec![' '; w]; h];
    for i in 0..48 {
        let a = i as f32 / 48.0 * std::f32::consts::TAU;
        let (x, y) = (15.0 + 14.6 * a.cos(), 8.0 + 8.0 * a.sin() * 0.99);
        let (c, r) = (x.round() as usize, y.round() as usize);
        if r < h && c < w {
            rows[r][c] = '*';
        }
    }
    rows.into_iter()
        .map(|r| r.into_iter().collect::<String>().trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn dims(art: &str) -> (usize, usize) {
    let rows = art.lines().count();
    let cols = art.lines().map(|l| l.chars().count()).max().unwrap_or(1);
    (cols, rows)
}

fn clip(effect: &str, art: &str, pad: (usize, usize), seed: u64, opts: &[&str]) -> Clip {
    let (c, r) = dims(art);
    Clip::render(&ClipSpec {
        effect,
        text: art,
        cols: c + pad.0,
        rows: r + pad.1,
        seed,
        opts,
    })
}

pub struct Glyphs {
    pub owl: Clip,
    pub runes: Vec<Clip>,
    pub falls: Clip,
    pub kindling: Clip,
    pub ring: Clip,
}

impl Glyphs {
    pub fn render(cast: &crate::sprites::Cast) -> Self {
        let owl_text = cast.get(crate::sprites::Id::Owl).as_text();
        let owl = clip(
            "decrypt",
            &owl_text,
            (0, 0),
            14,
            &[
                "--typing-speed",
                "3",
                "--ciphertext-colors",
                "1f3a33",
                "2c5c4f",
                "3dbb8a",
                "--final-gradient-stops",
                "9aa39c",
                "d8d2c0",
                "--final-gradient-direction",
                "vertical",
            ],
        );
        let runes = RUNES
            .iter()
            .enumerate()
            .map(|(i, art)| {
                clip(
                    "matrix",
                    art,
                    (0, 0),
                    40 + i as u64,
                    &[
                        "--rain-time",
                        "1",
                        "--rain-color-gradient",
                        "0b3a2c",
                        "3dbb8a",
                        "--highlight-color",
                        "d9fff0",
                        "--final-gradient-stops",
                        "3dbb8a",
                        "aef5d2",
                    ],
                )
            })
            .collect();
        let falls = clip(
            "pour",
            DROP,
            (0, 0),
            22,
            &[
                "--pour-direction",
                "down",
                "--pour-speed",
                "3",
                "--starting-color",
                "ffffff",
                "--final-gradient-stops",
                "dff4ff",
                "6fa9c9",
            ],
        );
        let kindling = clip(
            "burn",
            KINDLING,
            (0, 1),
            38,
            &[
                "--starting-color",
                "5a4632",
                "--burn-colors",
                "ffffff",
                "fff75d",
                "fe650d",
                "8a1c0c",
                "3a1206",
                "--final-gradient-stops",
                "ff9b3a",
                "6a2a0c",
            ],
        );
        let ring_art = ring();
        let ring = clip(
            "swarm",
            &ring_art,
            (40, 10),
            50,
            &[
                "--base-color",
                "d7ff7a",
                "9cf26a",
                "--flash-color",
                "ffffff",
                "--final-gradient-stops",
                "c8ff8a",
                "3dbb8a",
            ],
        );
        Self {
            owl,
            runes,
            falls,
            kindling,
            ring,
        }
    }
}

/// Lands clip cells in the grid at a cell offset. Cells with their own
/// background paint it; the rest keep the scene's.
pub struct GridSink<'a> {
    pub g: &'a mut Grid,
    pub col: i32,
    pub row: i32,
    pub gain: f32,
}

impl CellSink for GridSink<'_> {
    fn set(&mut self, col: i32, row: i32, cell: Cell) {
        if cell.ch == ' ' && cell.bg == NO_BG {
            return;
        }
        let fg = demofx::cell::scale_rgb(cell.rgb, self.gain);
        let (c, r) = (self.col + col, self.row + row);
        if let Some(dst) = self.g.cell(c, r) {
            if cell.ch != ' ' {
                dst.ch = cell.ch;
                dst.fg = fg;
            }
            if cell.bg != NO_BG {
                dst.bg = demofx::cell::lerp_rgb(dst.bg, cell.bg, self.gain);
            }
        }
    }
}

/// Draw a clip at time `since` its start, at `speed`.
pub fn play(g: &mut Grid, c: &Clip, since: f32, speed: f32, col: i32, row: i32, gain: f32) {
    if since < 0.0 || gain <= 0.0 {
        return;
    }
    let mut sink = GridSink { g, col, row, gain };
    c.draw_at(&mut sink, since, speed, 0, 0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_clip_runs_and_settles_on_its_art() {
        let g = Glyphs::render(&crate::sprites::Cast::load());
        for (name, c) in [
            ("owl", &g.owl),
            ("falls", &g.falls),
            ("kindling", &g.kindling),
            ("ring", &g.ring),
        ] {
            assert!(c.len() > 10, "{name}: {} frames", c.len());
            assert!(!c.final_cells().is_empty(), "{name}: empty final frame");
        }
        assert_eq!(g.runes.len(), RUNES.len());
    }
}
