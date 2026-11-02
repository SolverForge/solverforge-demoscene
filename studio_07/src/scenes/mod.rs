// The picture: one function per section, all drawing into the same frame,
// then the story cards and the corner logo on top.
mod advice;
mod agents;
mod boot;
mod city;
mod contact;
mod devrel;
mod plan;
mod work;

use demofx::{Cards, Clip, ClipSpec};

use crate::gfx::Frame;
use crate::portrait::Portrait;
use crate::routes::Plan;
use crate::story;
use crate::timeline::{now, Section};

pub const LIME: u32 = 0x50fc00;
pub const EMERALD: u32 = 0x10b981;
pub const MINT: u32 = 0x6ee7b7;
pub const PAPER: u32 = 0xecfdf5;

pub struct Demo {
    portrait: Portrait,
    plan: Plan,
    /// When each kept move of the route search appears (the song plays a
    /// blip at the same times).
    pub move_times: Vec<f32>,
    name_tall: Clip,
    name_wide: Clip,
    cards: Cards,
}

impl Demo {
    pub fn new() -> Self {
        let plan = Plan::solve();
        let move_times = plan::move_times(&plan);
        let name = |text: &str, cols, rows, seed| {
            Clip::render(&ClipSpec {
                effect: "decrypt",
                text,
                cols,
                rows,
                seed,
                opts: &["--final-gradient-stops", "50fc00", "e8ffd6"],
            })
        };
        Self {
            portrait: Portrait::load(),
            name_tall: name("VITTORIO\nDISTEFANO", 9, 2, 7),
            name_wide: name("VITTORIO DISTEFANO", 18, 1, 8),
            move_times,
            plan,
            cards: story::cards(),
        }
    }

    pub fn frame(&mut self, f: &mut Frame, t: f32) {
        let n = now(t);
        let corner = match n.section {
            Section::Boot => boot::draw(f, &self.portrait, &self.name_tall, n, t),
            Section::City => city::draw(f, n, t),
            Section::Plan => plan::draw(f, &self.plan, &self.move_times, n, t),
            Section::Advice => advice::draw(f, n, t),
            Section::Agents => agents::draw(f, n, t),
            Section::DevRel => devrel::draw(f, n, t),
            Section::Work => work::draw(f, n, t),
            Section::Contact => contact::draw(f, &self.portrait, &self.name_wide, n, t),
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

/// A faint dot every 32 pixels: the graph paper the personal scenes sit on.
pub fn dot_grid(f: &mut Frame, rgb: u32, a: f32) {
    if a <= 0.0 {
        return;
    }
    for y in (16..crate::gfx::H as i32).step_by(32) {
        for x in (16..crate::gfx::W as i32).step_by(32) {
            f.blend(x, y, rgb, a);
            f.blend(x + 1, y, rgb, a * 0.5);
        }
    }
}
