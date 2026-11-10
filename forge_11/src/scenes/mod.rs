// The picture: one module per section, all drawing into the same frame,
// plus the chrome every data section shares (section chip, progress rail),
// the beat-locked cuts between sections and the corner logo.
mod boot;
mod close;
pub mod cordelia;
mod describe;
mod generate;
mod repair;
mod score;
mod solve;

use demofx::{Clip, ClipSpec};

use crate::board::Board;
use crate::capture::Capture;
use crate::code::Listing;
use crate::fx;
use crate::gfx::{clip, mix, Frame, Ink, Size, S1, S2, S3, W};
use crate::script::Script;
use crate::timeline::{beat_pulse, now, span, Section, BAR, DURATION, SECTIONS};

pub const LIME: u32 = 0x50fc00;
pub const EMERALD: u32 = 0x10b981;
pub const MINT: u32 = 0x6ee7b7;
pub const PAPER: u32 = 0xecfdf5;
pub const AMBER: u32 = 0xffb050;
pub const RED: u32 = 0xff3b4a;
pub const CYAN: u32 = 0x8fd0ff;
pub const DIM: u32 = 0x5f7f70;

/// A glyphfx clip placed on the demo clock.
pub struct Timed {
    pub clip: Clip,
    pub from: f32,
    pub speed: f32,
}

impl Timed {
    pub fn new(effect: &str, text: &str, seed: u64, opts: &[&str], from: f32, window: f32) -> Self {
        let cols = text.lines().map(|l| l.chars().count()).max().unwrap_or(1);
        let clip = Clip::render(&ClipSpec {
            effect,
            text,
            cols,
            rows: text.lines().count().max(1),
            seed,
            opts,
        });
        let speed = (clip.seconds() / window.max(0.1)).max(1.0);
        Self { clip, from, speed }
    }

    pub fn draw(&self, f: &mut Frame, t: f32, x: i32, y: i32, size: Size, gain: f32) {
        if t < self.from || gain <= 0.0 {
            return;
        }
        let mut ink = Ink {
            f,
            x,
            y,
            size,
            gain,
            tint: None,
        };
        self.clip
            .draw_at(&mut ink, t - self.from, self.speed, 0, 0, 1.0);
    }
}

pub const LIME_STOPS: &[&str] = &["--final-gradient-stops", "50fc00", "e8ffd6"];
pub const RED_STOPS: &[&str] = &["--final-gradient-stops", "ff3b4a", "ffd0d4"];

const TITLES: [(Section, &str, &str); 4] = [
    (Section::Describe, "01 DESCRIBE", "decrypt"),
    (Section::Generate, "02 GENERATE", "burn"),
    (Section::Solve, "03 SOLVE", "slide"),
    (Section::Repair, "04 REPAIR", "unstable"),
];

pub struct Demo {
    pub cap: Capture,
    pub script: Script,
    board: Board,
    listings: Vec<Listing>,
    titles: Vec<(Section, Timed)>,
    boot: boot::Boot,
    repair: repair::Repair,
    close: close::Close,
}

impl Demo {
    pub fn new(cap: Capture) -> Self {
        let script = Script::new(&cap);
        let board = Board::new(&cap);
        let listings = cap
            .code
            .iter()
            .enumerate()
            .map(|(i, c)| Listing::new(c, i))
            .collect();
        let titles = TITLES
            .iter()
            .enumerate()
            .map(|(i, &(s, text, effect))| {
                let (a, _) = span(s);
                (
                    s,
                    Timed::new(effect, text, 700 + i as u64, LIME_STOPS, a, 0.9),
                )
            })
            .collect();
        let boot = boot::Boot::new();
        let repair = repair::Repair::new(&cap, &script);
        let close = close::Close::new(&cap, &script);
        Self {
            cap,
            script,
            board,
            listings,
            titles,
            boot,
            repair,
            close,
        }
    }

    /// Draw time t; returns the bloom strength for the post pass.
    pub fn frame(&mut self, f: &mut Frame, t: f32) -> f32 {
        let n = now(t);
        let mut bloom = 0.5;
        let mut corner = 1.0;
        match n.section {
            Section::Boot => boot::draw(f, &self.boot, t, n),
            Section::Describe => describe::draw(f, &self.cap, &self.script, &self.board, t, n),
            Section::Generate => {
                bloom = 0.6;
                generate::draw(f, &self.cap, &self.script, &self.listings, t, n)
            }
            Section::Solve => solve::draw(f, &self.cap, &self.script, &self.board, t, n),
            Section::Repair => {
                repair::draw(f, &self.cap, &self.script, &self.board, &self.repair, t, n)
            }
            Section::Close => {
                bloom = 0.55;
                corner = close::draw(f, &self.cap, &self.close, t, n);
            }
        }
        if !matches!(n.section, Section::Boot | Section::Close) {
            self.chrome(f, t, n.section);
        }
        cuts(f, t);
        // The demo fades in from black and out at the very end.
        let fade = (t / 0.6).min(1.0) * ((DURATION - t) / 1.2).clamp(0.0, 1.0);
        if fade < 1.0 {
            f.dim(0, 0, W as i32, crate::gfx::H as i32, fade);
        }
        demofx::logo::corner(f, t, corner * fade);
        bloom
    }

    /// The section chip, top left beside the corner mark, and the progress
    /// rail across the top right.
    fn chrome(&self, f: &mut Frame, t: f32, section: Section) {
        if let Some((_, title)) = self.titles.iter().find(|(s, _)| *s == section) {
            title.draw(f, t, 150, 44, S3, 1.0);
        }
        let x0 = 1330;
        f.text(x0, 34, "SOLVERFORGE", S1, DIM, 0.9);
        for (i, &(s, _, _)) in TITLES.iter().enumerate() {
            let (a, b) = span(s);
            let x = x0 + i as i32 * 132;
            let p = ((t - a) / (b - a)).clamp(0.0, 1.0);
            f.rect(x, 64, x + 124, 72, DIM, 0.35);
            let col = if s == section {
                mix(LIME, 0xffffff, 0.5 * beat_pulse(t, 8.0))
            } else {
                EMERALD
            };
            f.rect(x, 64, x + (124.0 * p) as i32, 72, col, 0.95);
            let label = clip(TITLES[i].1, 12);
            f.text(
                x,
                80,
                &label,
                S1,
                if s == section { PAPER } else { DIM },
                0.8,
            );
        }
    }

    /// Burned into every frame when the capture is the placeholder: the
    /// preview can never pass for the real session.
    pub fn watermark(&self, f: &mut Frame, t: f32) {
        if self.cap.is_release() {
            return;
        }
        let msg = "PLACEHOLDER DATA";
        let k = 0.85 + 0.15 * (t * 3.0).sin();
        let w = msg.len() as i32 * S2.w;
        f.rect(
            W as i32 / 2 - w / 2 - 12,
            2,
            W as i32 / 2 + w / 2 + 12,
            44,
            0x300004,
            0.85,
        );
        f.text_centered(W as i32 / 2, 6, msg, S2, RED, k);
        f.rect(W as i32 - w - 40, 1030, W as i32 - 16, 1074, 0x300004, 0.85);
        f.text(W as i32 - w - 28, 1034, msg, S2, RED, k);
    }
}

/// The cut into every section: a glyph wipe over the last half second of
/// the outgoing section, a flash and a short tear on the downbeat.
fn cuts(f: &mut Frame, t: f32) {
    for &(s, b0, _) in SECTIONS.iter().skip(1) {
        let at = b0 * BAR;
        let p = (t - (at - 0.5)) / 0.8;
        if (0.0..1.0).contains(&p) {
            let col = match s {
                Section::Repair => RED,
                Section::Generate => AMBER,
                _ => LIME,
            };
            fx::glyph_wipe(f, t, p, col);
        }
        let since = t - at;
        if (0.0..0.5).contains(&since) {
            fx::flash(f, 0xffffff, 0.22 * (-since * 9.0).exp());
            fx::glitch(f, t, 0.5 * (1.0 - since / 0.25).max(0.0), b0 as i32);
        }
    }
}

/// Type `s` progressively: the first `n` characters.
pub fn typed(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// The blinking textmode cursor, on for the first half of every beat.
pub fn cursor(f: &mut Frame, x: i32, y: i32, size: Size, rgb: u32, t: f32) {
    if t.rem_euclid(crate::timeline::BEAT) < crate::timeline::BEAT * 0.5 {
        f.char_at(x, y, '█', size, rgb, 0.85);
    }
}

/// "mm:ss" for a transcript timestamp in milliseconds.
pub fn mmss(ms: f64) -> String {
    let s = (ms / 1000.0).max(0.0) as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}

/// The run's command: the one that launches the app's own binary (named
/// after its directory), else one that runs or solves, else the last.
pub fn run_command(c: &Capture) -> Option<&str> {
    let app = c
        .workdir
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("");
    let bin = app.replace('-', "_");
    let launches = |cmd: &str| {
        cmd.split_whitespace()
            .find(|tok| !tok.contains('='))
            .map(|prog| prog.rsplit('/').next().unwrap_or(prog))
            .is_some_and(|name| !app.is_empty() && (name == app || name == bin))
    };
    c.commands
        .iter()
        .find(|s| launches(s))
        .or_else(|| {
            c.commands
                .iter()
                .rev()
                .find(|s| s.contains("cargo run") || s.contains(" solve"))
        })
        .or(c.commands.last())
        .map(String::as_str)
}
