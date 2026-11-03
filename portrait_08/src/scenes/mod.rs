// The picture: one function per section, all drawing into the same 9:16
// frame, then the story cards and the corner logo on top.
mod challenge;
pub mod guitar;
mod home;
mod machine;
mod name;
mod reading;
mod work;
mod worlds;

use demofx::{Cards, Clip, ClipSpec, Pixels};

use crate::gfx::{Frame, H, W};
use crate::portrait::Portrait;
use crate::score::{score, Note};
use crate::story;
use crate::timeline::{now, Section};

pub struct Demo {
    notes: Vec<Note>,
    guitar: guitar::Guitar,
    portrait: Portrait,
    routes: work::Routes,
    name: Clip,
    cards: Cards,
    logo: Logo,
}

impl Demo {
    pub fn new() -> Self {
        Self {
            notes: score(),
            guitar: guitar::Guitar::paint(),
            portrait: Portrait::load(),
            routes: work::Routes::solve(),
            name: Clip::render(&ClipSpec {
                effect: "decrypt",
                text: "VITTORIO\nDISTEFANO",
                cols: 9,
                rows: 2,
                seed: 8,
                opts: &["--final-gradient-stops", "f2d48a", "fff6e0"],
            }),
            cards: story::cards(),
            logo: Logo::paint(),
        }
    }

    pub fn frame(&mut self, f: &mut Frame, t: f32) {
        let n = now(t);
        let corner = match n.section {
            Section::Guitar => guitar::draw(f, &self.guitar, &self.notes, n, t),
            Section::Worlds => worlds::draw(f, n, t),
            Section::Challenge => challenge::draw(f, n, t),
            Section::Machine => machine::draw(f, n, t),
            Section::Reading => reading::draw(f, n, t),
            Section::Home => home::draw(f, &self.portrait, n, t),
            Section::Work => work::draw(f, &self.notes, &self.routes, n, t),
            Section::Name => name::draw(
                f,
                &self.guitar,
                &self.notes,
                &self.portrait,
                &self.name,
                n,
                t,
            ),
        };
        self.cards.draw(f, t, 1.0);
        // Once it has drawn itself in, the corner mark is the same every
        // frame: composite the cached copy.
        if t >= LOGO_SETTLED && corner >= 1.0 {
            self.logo.draw(f);
        } else {
            demofx::logo::corner(f, t, corner);
        }
    }
}

/// When the corner mark has finished drawing itself in (demofx: 0.5 s + 2 s).
const LOGO_SETTLED: f32 = 2.6;
/// The square the corner mark occupies, from the top-left.
const LOGO_BOX: usize = 300;

/// A full-size surface that keeps only the top-left square, for drawing the
/// corner mark into.
struct Patch(Vec<u32>);

impl Pixels for Patch {
    fn size(&self) -> (usize, usize) {
        (W, H)
    }
    fn get(&self, x: usize, y: usize) -> u32 {
        if x < LOGO_BOX && y < LOGO_BOX {
            self.0[y * LOGO_BOX + x]
        } else {
            0
        }
    }
    fn put(&mut self, x: usize, y: usize, rgb: u32) {
        if x < LOGO_BOX && y < LOGO_BOX {
            self.0[y * LOGO_BOX + x] = rgb;
        }
    }
}

/// The settled corner mark as colour and coverage, recovered by drawing it
/// once over black and once over white.
struct Logo {
    texels: Vec<(usize, [f32; 3], f32)>,
}

impl Logo {
    fn paint() -> Self {
        let mut black = Patch(vec![0; LOGO_BOX * LOGO_BOX]);
        let mut white = Patch(vec![0xffffff; LOGO_BOX * LOGO_BOX]);
        demofx::logo::corner(&mut black, 10.0, 1.0);
        demofx::logo::corner(&mut white, 10.0, 1.0);
        let mut texels = Vec::new();
        for (i, (&b, &w)) in black.0.iter().zip(&white.0).enumerate() {
            let ch = |p: u32, s: u32| ((p >> s) & 255) as f32;
            // Over black: c*a. Over white: c*a + 255*(1-a).
            let a = 1.0
                - (ch(w, 16) - ch(b, 16) + ch(w, 8) - ch(b, 8) + ch(w, 0) - ch(b, 0))
                    / (3.0 * 255.0);
            if a > 0.004 {
                texels.push((i, [ch(b, 16), ch(b, 8), ch(b, 0)], a.min(1.0)));
            }
        }
        Self { texels }
    }

    fn draw(&self, f: &mut Frame) {
        for &(i, c, a) in &self.texels {
            let p = &mut f.px[(i / LOGO_BOX) * W + i % LOGO_BOX];
            let d = *p;
            let ch = |k: usize, s: u32| {
                ((((d >> s) & 255) as f32 * (1.0 - a) + c[k] + 0.5) as u32).min(255) << s
            };
            *p = ch(0, 16) | ch(1, 8) | ch(2, 0);
        }
    }
}

/// The part of `s` typed so far, at `cps` characters a second from `start`.
pub fn typed(s: &str, start: f32, cps: f32, t: f32) -> String {
    let k = ((t - start) * cps).max(0.0) as usize;
    s.chars().take(k).collect()
}
