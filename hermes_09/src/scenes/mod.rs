// The picture: one function per section, all drawing into the same frame,
// then the story cards and the corner logo on top.
mod intro;
mod live;
mod outro;

pub use live::{DIALOG, HOLE, PORTRAIT};

use demofx::Cards;

use crate::gfx::Frame;
use crate::statue::Statue;
use crate::story;
use crate::timeline::{now, Section};

pub const LIME: u32 = 0x50fc00;
pub const EMERALD: u32 = 0x10b981;
pub const MINT: u32 = 0x6ee7b7;
pub const PAPER: u32 = 0xecfdf5;
pub const RED: u32 = 0xf43f5e;
pub const SLATE: u32 = 0x64748b;

pub struct Demo {
    /// When each task block lands in the intro's week (the song plays a blip
    /// at the same times).
    pub landings: Vec<f32>,
    cards: Cards,
    statue: Statue,
}

impl Demo {
    pub fn new() -> Self {
        Self {
            landings: intro::landings(),
            cards: story::cards(),
            statue: Statue::load(),
        }
    }

    pub fn frame(&mut self, f: &mut Frame, t: f32) {
        let n = now(t);
        let corner = match n.section {
            Section::Intro => intro::draw(f, &self.statue, n, t),
            Section::Live => live::draw(f, &self.statue, n, t),
            Section::Outro => outro::draw(f, n),
        };
        self.cards.draw(f, t, 1.0);
        demofx::logo::corner(f, t, corner);
    }
}

/// The part of `s` typed so far, at `cps` characters a second from `start`.
pub fn typed(s: &str, start: f32, cps: f32, t: f32) -> String {
    let k = ((t - start) * cps).max(0.0) as usize;
    s.chars().take(k).collect()
}

/// A faint dot every 40 pixels: the graph paper the scenes sit on.
pub fn dot_grid(f: &mut Frame, rgb: u32, a: f32) {
    if a <= 0.0 {
        return;
    }
    for y in (20..crate::gfx::H as i32).step_by(40) {
        for x in (20..crate::gfx::W as i32).step_by(40) {
            f.blend(x, y, rgb, a);
            f.blend(x + 1, y, rgb, a * 0.5);
        }
    }
}
