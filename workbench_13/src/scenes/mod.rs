// The picture: one function per kind of section, all drawing into the same
// frame, then the band under the picture, the story cards and the matte.
//
// The frame is 1920x1080. The picture is the top 1920x920: a recorded
// Desktop take or a native scene. The band is the bottom 160 rows, the same
// in every section between the intro and the outro: the Nous Girl's place
// (her sprite is laid in by the compose step), her words typed as she says
// them, the chapter title and the mark.
//
// Colours are the SolverForge character language in its dark mode: emerald
// night, silver parchment, jade; blue explores, red flags a conflict,
// emerald resolves.
mod band;
pub mod board;
mod call;
pub mod intro;
mod outro;
pub mod powder;
mod repair;
mod week;

use demofx::font::Mask;
use demofx::Cards;

use crate::data::Data;
use crate::gfx::{Frame, Image, H, W};
use crate::story;

use crate::timeline::{now, Kind, Now};
pub use band::PORTRAIT;

pub const NIGHT: u32 = 0x07110e;
pub const PARCH: u32 = 0xefeae0;
pub const SAGE: u32 = 0x8fa79c;
pub const JADE: u32 = 0x3dbb8a;
pub const BLUE: u32 = 0x4a9bd1;
pub const RED: u32 = 0xdc4f42;

/// The picture's height: the band takes the rows below it.
pub const PICTURE_H: i32 = 920;

pub struct Faces {
    pub concern: Image,
    pub defiance: Image,
    pub resolve: Image,
    pub ease: Image,
}

impl Faces {
    fn load() -> Self {
        Self {
            concern: Image::ppm(include_bytes!("../../assets/cordelia/concern.ppm")),
            defiance: Image::ppm(include_bytes!("../../assets/cordelia/defiance.ppm")),
            resolve: Image::ppm(include_bytes!("../../assets/cordelia/resolve.ppm")),
            ease: Image::ppm(include_bytes!("../../assets/cordelia/ease.ppm")),
        }
    }
}

pub struct Demo {
    cards: Cards,
    data: Data,
    faces: Faces,
    masks: Vec<(char, Mask)>,
    banner: demofx::Clip,
}

impl Demo {
    pub fn new() -> Self {
        Self {
            cards: story::cards(),
            data: Data::load(),
            faces: Faces::load(),
            masks: Vec::new(),
            banner: story::banner(),
        }
    }

    pub fn frame(&mut self, f: &mut Frame, t: f32) {
        let n = now(t);
        match n.kind {
            Kind::Intro => intro::draw(f, n, t, &self.banner, &mut self.masks),
            Kind::Footage => footage_ground(f, n, t),
            Kind::Week => week::draw(f, &self.data, n, t),
            Kind::Powder => powder::draw(f, &self.data, n, t, &mut self.masks),
            Kind::Call => call::draw(f, &self.faces, n, t),
            Kind::Repair => repair::draw(f, &self.data, n, t, &mut self.masks),
            Kind::Outro => outro::draw(f, &self.faces, n, t),
        }
        if !matches!(n.kind, Kind::Intro | Kind::Outro) {
            band::draw(f, n, t, &mut self.masks);
        }
        self.cards.draw(f, t, 1.0);
        if matches!(n.kind, Kind::Footage) {
            wipe_edge(f, n, t);
        }
    }

    /// How much of the recorded take shows at each pixel (0..255): all of
    /// the picture inside a footage section, nothing elsewhere, and on the
    /// way in and out a raster transition of its own per section.
    pub fn matte(&self, t: f32, out: &mut [u8]) {
        let n = now(t);
        out.fill(0);
        if n.kind != Kind::Footage {
            return;
        }
        for y in 0..PICTURE_H as usize {
            for x in 0..W {
                out[y * W + x] = (open(n, x as i32, y as i32) * 255.0).round() as u8;
            }
        }
    }
}

/// Seconds a footage section takes to open and to close.
const OPEN: f32 = 0.55;

/// The transition each footage section opens and closes with.
fn style(index: usize) -> usize {
    index % 4
}

/// How far the take is open at pixel (x, y), 0..1.
fn open(n: Now, x: i32, y: i32) -> f32 {
    let k_in = (n.local / OPEN).clamp(0.0, 1.0);
    let k_out = ((n.len - n.local) / OPEN).clamp(0.0, 1.0);
    let k = k_in.min(k_out);
    if k >= 1.0 {
        return 1.0;
    }
    let (fx, fy) = (x as f32 / W as f32, y as f32 / PICTURE_H as f32);
    let edge = match style(n.index) {
        // A CRT powering on: a bright line opening into the picture.
        0 => 1.0 - (fy - 0.5).abs() * 2.0,
        // Venetian blinds, 46 rows each.
        1 => 1.0 - ((y % 46) as f32 / 46.0),
        // A diagonal sweep from the left.
        2 => 1.0 - (fx * 0.8 + fy * 0.2),
        // Blocks resolving in a stable random order.
        _ => 1.0 - hash((x / 24) as u32 * 7919 + (y / 24) as u32 * 104_729),
    };
    if edge < 1.0 - k {
        0.0
    } else {
        1.0
    }
}

/// The ground under a take: emerald night, seen only while it opens.
fn footage_ground(f: &mut Frame, n: Now, t: f32) {
    f.rect(0, 0, W as i32, PICTURE_H, NIGHT, 1.0);
    dot_grid(f, JADE, 0.12, t);
    let _ = n;
}

/// The light along the edge of an opening or closing take, drawn where the
/// take does not yet show.
fn wipe_edge(f: &mut Frame, n: Now, _t: f32) {
    let k_in = (n.local / OPEN).clamp(0.0, 1.0);
    let k_out = ((n.len - n.local) / OPEN).clamp(0.0, 1.0);
    let k = k_in.min(k_out);
    if k >= 1.0 || k <= 0.0 {
        return;
    }
    let glow = 1.0 - (k - 0.5).abs() * 2.0;
    match style(n.index) {
        0 => {
            let h = (k * PICTURE_H as f32 * 0.5) as i32;
            for (y, a) in [(PICTURE_H / 2 - h - 2, 1.0), (PICTURE_H / 2 + h, 1.0)] {
                f.rect(0, y, W as i32, y + 2, PARCH, a * (0.4 + glow));
                f.glow(W as f32 / 2.0, y as f32, 700.0, JADE, 0.25 * glow);
            }
        }
        1 => {
            for b in 0..(PICTURE_H / 46 + 1) {
                let y = b * 46 + (46.0 * k) as i32;
                f.rect(0, y, W as i32, y + 1, JADE, 0.8 * glow);
            }
        }
        2 => {
            let x = (k * W as f32 * 1.25) as i32;
            f.rect(x - 2, 0, x + 2, PICTURE_H, PARCH, glow);
            f.glow(x as f32, PICTURE_H as f32 / 2.0, 260.0, JADE, 0.3 * glow);
        }
        _ => {}
    }
}

/// A faint dot every 40 pixels over the picture, drifting slowly.
pub fn dot_grid(f: &mut Frame, rgb: u32, a: f32, t: f32) {
    if a <= 0.0 {
        return;
    }
    let off = ((t * 6.0) as i32).rem_euclid(40);
    for y in (20..PICTURE_H).step_by(40) {
        for x in (20 - off..W as i32).step_by(40) {
            f.blend(x, y, rgb, a);
            f.blend(x + 1, y, rgb, a * 0.5);
        }
    }
}

/// Faint scanlines rolling down the picture area.
pub fn scan(f: &mut Frame, t: f32, a: f32) {
    let y = ((t * 140.0) % (PICTURE_H as f32 + 200.0)) as i32 - 100;
    for k in 0..60 {
        f.rect(
            0,
            y + k,
            W as i32,
            y + k + 1,
            JADE,
            a * (1.0 - k as f32 / 60.0) * 0.08,
        );
    }
}

pub fn hash(i: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    x as f32 / u32::MAX as f32
}

#[allow(dead_code)]
const _: () = assert!(H as i32 - PICTURE_H == 160);
