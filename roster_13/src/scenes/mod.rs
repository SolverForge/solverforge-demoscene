// The picture: the Desktop takes (live.rs) between the demo's own scenes
// (intro, the call, outro), then the story cards and the logo on top. Each
// scene returns how much of the phosphor finish (post.rs) it wants: the
// takes none, so the plugin's text reaches the screen as recorded.
mod call;
mod intro;
mod live;
mod outro;

use demofx::clip::{Clip, ClipSpec};
use demofx::{Cards, PixelSink, Place};

use crate::cordelia::Cordelia;
use crate::cut::Kind;
use crate::footage::Footage;
use crate::gfx::{Frame, H, W};
use crate::story;
use crate::timeline::{hash, now, smooth};

pub const LIME: u32 = 0x50fc00;
pub const EMERALD: u32 = 0x10b981;
pub const MINT: u32 = 0x6ee7b7;
pub const PAPER: u32 = 0xecfdf5;
pub const JADE: u32 = 0x2ee6c4;
pub const BLUE: u32 = 0x38bdf8;
pub const AMBER: u32 = 0xfbbf24;

/// The corner logo's place over the takes: in the Desktop's empty sidebar,
/// above the chapter title.
pub const LOGO_AT: (f32, f32) = (160.0, 806.0);

pub struct Demo {
    cards: Cards,
    footage: Footage,
    cordelia: Cordelia,
    call_words: call::Words,
    masks: Vec<(char, demofx::font::Mask)>,
}

impl Demo {
    pub fn new(footage: Footage) -> Self {
        Self {
            cards: story::cards(),
            footage,
            cordelia: Cordelia::load(),
            call_words: call::Words::new(),
            masks: Vec::new(),
        }
    }

    /// Draw the frame at `t`; returns the strength of the phosphor finish.
    pub fn frame(&mut self, f: &mut Frame, t: f32) -> f32 {
        let n = now(t);
        // The logo rides the takes, in the Desktop's sidebar; the demo's
        // own scenes carry the name themselves.
        let (finish, logo) = match n.kind {
            Kind::Take => live::draw(f, &mut self.footage, n, t),
            Kind::Intro => intro::draw(f, n, t),
            Kind::Call => call::draw(f, &self.cordelia, &self.call_words, &mut self.masks, n, t),
            Kind::Outro => outro::draw(f, n),
        };
        self.cards.draw(f, t, 1.0);
        demofx::logo::corner_at(f, LOGO_AT, t, logo);
        finish
    }
}

/// A faint dot every 40 pixels: the graph paper the scenes sit on.
pub fn dot_grid(f: &mut Frame, rgb: u32, a: f32) {
    if a <= 0.0 {
        return;
    }
    for y in (20..H as i32).step_by(40) {
        for x in (20..W as i32).step_by(40) {
            f.blend(x, y, rgb, a);
            f.blend(x + 1, y, rgb, a * 0.5);
        }
    }
}

/// The cell dissolve between the demo and the Desktop: the frame in cells
/// of 40 px, each switching at its own moment from black with a running
/// glyph to the picture underneath, with a lime flash as it turns. At
/// `p` = 0 every cell is black, at 1 the picture is whole. Cells nearer
/// `from` turn first.
pub fn mosaic(f: &mut Frame, p: f32, from: (f32, f32), t: f32) {
    if p >= 1.0 {
        return;
    }
    const C: i32 = 40;
    const GLYPHS: [char; 8] = ['0', '1', ':', '/', '+', '*', '#', '-'];
    let far = ((W as f32).powi(2) + (H as f32).powi(2)).sqrt();
    let tick = (t * 20.0) as u32;
    for cy in 0..(H as i32 / C + 1) {
        for cx in 0..(W as i32 / C) {
            let id = (cy * 64 + cx) as u32;
            let (mx, my) = ((cx * C + C / 2) as f32, (cy * C + C / 2) as f32);
            let d = ((mx - from.0).powi(2) + (my - from.1).powi(2)).sqrt() / far;
            // Each cell's turn, in 0.05..0.9 of the dissolve.
            let turn = 0.05 + 0.6 * d + 0.25 * hash(id);
            let (x0, y0) = (cx * C, cy * C);
            if p < turn {
                f.rect(x0, y0, x0 + C, y0 + C, 0x000000, 1.0);
                let g = GLYPHS[(hash(id ^ tick.wrapping_mul(2654435761)) * 8.0) as usize % 8];
                let near = smooth(1.0 - (turn - p) / 0.25);
                f.char_at(x0 + 10, y0 + 12, g, 2, LIME, 0.15 + 0.6 * near);
            } else {
                let flash = 1.0 - smooth((p - turn) / 0.08);
                if flash > 0.0 {
                    f.rect(x0, y0, x0 + C, y0 + C, LIME, 0.45 * flash);
                }
            }
        }
    }
}

/// A glyphfx clip drawn through a pixel sink at `place`, `idx` frames in.
pub fn draw_clip(
    f: &mut Frame,
    masks: &mut Vec<(char, demofx::font::Mask)>,
    clip: &Clip,
    place: Place,
    idx: usize,
    gain: f32,
) {
    let mut sink = PixelSink::new(f, place, masks);
    clip.draw(&mut sink, idx, 0, 0, gain);
}

pub fn clip(effect: &str, text: &str, cols: usize, seed: u64, opts: &[&str]) -> Clip {
    let c = Clip::render(&ClipSpec {
        effect,
        text,
        cols,
        rows: 1,
        seed,
        opts,
    });
    assert!(!c.is_empty(), "glyphfx left {text} empty");
    c
}
