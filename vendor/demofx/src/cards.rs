// Story cards for the pixel demos: one or two plain sentences, timed to the
// picture. A glyphfx effect that belongs to the moment brings the words in,
// they hold still to be read, then burn out letter by letter. A soft scrim
// darkens the picture behind them so they stay legible over anything.
use crate::cell::{lerp_rgb, pack, rgb_parts, Cell, CellSink, CELL_H, CELL_W};
use crate::clip::{Clip, ClipSpec};
use crate::font::{mask, Mask};

/// Share of the card's window the entrance may take, and its ceiling in
/// seconds: the rest of the window the words hold still to be read.
const ENTRANCE: f32 = 0.35;
const ENTRANCE_MAX: f32 = 1.4;
/// Seconds the words take to burn out.
const EXIT: f32 = 0.7;
/// Reading time once the words have settled: a beat to find them, then a
/// slow reading pace (240 words a minute).
const READ_BASE: f32 = 0.8;
const READ_PER_WORD: f32 = 0.25;

/// The shortest window a card with this text may have: entrance, reading
/// time and exit. Cards::new refuses anything shorter.
pub fn reading_window(text: &str) -> f32 {
    ENTRANCE_MAX + READ_BASE + READ_PER_WORD * text.split_whitespace().count() as f32 + EXIT
}
/// Scrim strength and its soft edge in pixels.
const SCRIM: f32 = 0.78;
const SCRIM_EDGE: f32 = 26.0;

/// A 0x00RRGGBB pixel surface.
pub trait Pixels {
    fn size(&self) -> (usize, usize);
    fn get(&self, x: usize, y: usize) -> u32;
    fn put(&mut self, x: usize, y: usize, rgb: u32);
}

/// A plain 0x00RRGGBB buffer (minifb's format).
pub struct RgbBuffer<'a> {
    pub buf: &'a mut [u32],
    pub w: usize,
    pub h: usize,
}

impl Pixels for RgbBuffer<'_> {
    fn size(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    fn get(&self, x: usize, y: usize) -> u32 {
        self.buf[y * self.w + x]
    }
    fn put(&mut self, x: usize, y: usize, rgb: u32) {
        self.buf[y * self.w + x] = rgb;
    }
}

/// Where a card sits: the top-left pixel of its text block, the block's
/// width in pixels (the words are centred in it) and the size of one cell.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub cell_w: usize,
    pub cell_h: usize,
}

impl Place {
    pub const fn new(x: i32, y: i32, w: i32, cell_w: usize, cell_h: usize) -> Self {
        Self {
            x,
            y,
            w,
            cell_w,
            cell_h,
        }
    }

    pub fn cols(&self) -> usize {
        self.w as usize / self.cell_w
    }
}

pub struct CardSpec {
    /// Seconds on the demo clock.
    pub from: f32,
    pub to: f32,
    /// One to three lines, '\n' separated.
    pub text: &'static str,
    pub effect: &'static str,
    pub opts: &'static [&'static str],
    pub place: Place,
}

pub const fn card(
    from: f32,
    to: f32,
    text: &'static str,
    effect: &'static str,
    opts: &'static [&'static str],
    place: Place,
) -> CardSpec {
    CardSpec {
        from,
        to,
        text,
        effect,
        opts,
        place,
    }
}

struct Card {
    spec: &'static CardSpec,
    clip: Clip,
    speed: f32,
    rows: usize,
    /// Columns covered by the settled words.
    span: (i32, i32),
}

pub struct Cards {
    cards: Vec<Card>,
    masks: Vec<(char, Mask)>,
}

impl Cards {
    /// Render every card's glyphfx clip up front (deterministic: seeded).
    pub fn new(specs: &'static [CardSpec], seed: u64) -> Self {
        let cards = specs
            .iter()
            .enumerate()
            .map(|(i, spec)| {
                let cols = spec.place.cols();
                let rows = spec.text.lines().count().max(1);
                for line in spec.text.lines() {
                    assert!(
                        line.chars().count() + 2 <= cols,
                        "card line too long for {cols} columns: {line}"
                    );
                }
                let window = spec.to - spec.from;
                let need = reading_window(spec.text);
                assert!(
                    window + 0.01 >= need,
                    "card window {window:.2}s is too short to read; needs {need:.2}s: {}",
                    spec.text
                );
                let clip = Clip::render(&ClipSpec {
                    effect: spec.effect,
                    text: spec.text,
                    cols,
                    rows,
                    seed: seed + i as u64,
                    opts: spec.opts,
                });
                let cells = clip.final_cells();
                let lo = cells.iter().map(|c| c.0).min().unwrap_or(0) as i32;
                let hi = cells.iter().map(|c| c.0).max().unwrap_or(0) as i32 + 1;
                let entrance = (window * ENTRANCE).min(ENTRANCE_MAX);
                Card {
                    speed: (clip.seconds() / entrance).max(1.0),
                    clip,
                    spec,
                    rows,
                    span: (lo, hi),
                }
            })
            .collect();
        Self {
            cards,
            masks: Vec::new(),
        }
    }

    /// Draw every card live at time t, scaled by `fade` (0..1).
    pub fn draw(&mut self, px: &mut impl Pixels, t: f32, fade: f32) {
        for i in 0..self.cards.len() {
            let card = &self.cards[i];
            let (from, to) = (card.spec.from, card.spec.to);
            if t < from || t >= to || fade <= 0.0 {
                continue;
            }
            let local = t - from;
            let gone = smooth((t - (to - EXIT)) / EXIT);
            let shown = smooth(local / 0.4);
            let place = card.spec.place;
            let (lo, hi) = card.span;
            scrim(
                px,
                place.x + lo * place.cell_w as i32,
                place.y,
                place.x + hi * place.cell_w as i32,
                place.y + (card.rows * place.cell_h) as i32,
                SCRIM * shown * (1.0 - gone) * fade,
            );

            let idx = (local * 60.0 * card.speed) as usize;
            let mut cells = Vec::new();
            card.clip.for_each_cell(idx, |col, row, cell| {
                let threshold = hash(col as i32 * 7 + 3, row as i32 * 13 + 1);
                if gone > threshold {
                    return;
                }
                let cell = if gone > 0.0 && gone > threshold - 0.18 {
                    Cell::new('.', cell.scaled(1.6).rgb)
                } else {
                    cell
                };
                cells.push((col, row, cell.scaled(fade)));
            });
            let mut sink = PixelSink {
                px,
                place,
                masks: &mut self.masks,
            };
            for (col, row, cell) in cells {
                sink.set(col as i32, row as i32, cell);
            }
        }
    }
}

/// Rasterises cells into pixels with a faint phosphor halo.
struct PixelSink<'a, P: Pixels> {
    px: &'a mut P,
    place: Place,
    masks: &'a mut Vec<(char, Mask)>,
}

impl<P: Pixels> CellSink for PixelSink<'_, P> {
    fn set(&mut self, col: i32, row: i32, cell: Cell) {
        if cell.ch == ' ' || cell.rgb == 0 {
            return;
        }
        let m = match self.masks.iter().find(|(c, _)| *c == cell.ch) {
            Some((_, m)) => *m,
            None => {
                let m = mask(cell.ch);
                self.masks.push((cell.ch, m));
                m
            }
        };
        let (cw, ch) = (self.place.cell_w as i32, self.place.cell_h as i32);
        let (x0, y0) = (self.place.x + col * cw, self.place.y + row * ch);
        let (w, h) = self.px.size();
        let lit = |x: i32, y: i32| -> bool {
            if x < 0 || y < 0 || x >= cw || y >= ch {
                return false;
            }
            let mx = x as usize * CELL_W / cw as usize;
            let my = y as usize * CELL_H / ch as usize;
            (m[my] >> mx) & 1 == 1
        };
        for y in -1..=ch {
            for x in -1..=cw {
                let (sx, sy) = (x0 + x, y0 + y);
                if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
                    continue;
                }
                let (sx, sy) = (sx as usize, sy as usize);
                if lit(x, y) {
                    self.px.put(sx, sy, cell.rgb);
                } else {
                    let near = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                        .iter()
                        .filter(|&&(a, b)| lit(a, b))
                        .count();
                    if near > 0 {
                        let k = 0.16 * near as f32;
                        let bg = self.px.get(sx, sy);
                        self.px.put(sx, sy, add(bg, cell.rgb, k));
                    }
                }
            }
        }
    }
}

fn scrim(px: &mut impl Pixels, x0: i32, y0: i32, x1: i32, y1: i32, k: f32) {
    if k <= 0.0 {
        return;
    }
    let (w, h) = px.size();
    let e = SCRIM_EDGE as i32;
    for y in (y0 - e).max(0)..(y1 + e).min(h as i32) {
        let dy = (y0 - y).max(y - y1).max(0) as f32;
        for x in (x0 - e).max(0)..(x1 + e).min(w as i32) {
            let dx = (x0 - x).max(x - x1).max(0) as f32;
            let out = (dx * dx + dy * dy).sqrt() / SCRIM_EDGE;
            let a = k * smooth(1.0 - out);
            if a <= 0.0 {
                continue;
            }
            let (x, y) = (x as usize, y as usize);
            let c = px.get(x, y);
            px.put(x, y, lerp_rgb(c, 0, a));
        }
    }
}

fn add(a: u32, b: u32, k: f32) -> u32 {
    let (ar, ag, ab) = rgb_parts(a);
    let (br, bg, bb) = rgb_parts(b);
    pack(ar + br * k, ag + bg * k, ab + bb * k)
}

pub fn smooth(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Stable 0..1 hash of a cell, for the letter-by-letter exit.
fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    (h & 0xFFFF) as f32 / 65535.0
}
