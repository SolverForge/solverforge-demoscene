// The running order: which world and which glyphfx clip for every moment of
// the 56 bars; story.rs adds the words.
use crate::canvas::{lerp_rgb, scale_rgb, Cell, TextCanvas, CELL_H, CELL_W, COLS, ROWS};
use crate::forge::{self, Ingot};
use crate::logo::LogoState;
use crate::post::{HEIGHT, WIDTH};
use crate::story::Story;
use crate::timeline::{bars, flow, last_strike, now, smooth, Section, BAR, DURATION, HAMMER_STOP};
use crate::{river, valley};
use demofx::{Clip, ClipSpec};

const BANNER: &str = "\
███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝";

struct Timed {
    clip: Clip,
    from: f32,
    speed: f32,
}

impl Timed {
    fn new(spec: ClipSpec, from_bar: f32, to_bar: f32) -> Self {
        let clip = Clip::render(&spec);
        let window = bars(to_bar - from_bar);
        let speed = (clip.seconds() / window).max(1.0);
        eprintln!(
            "[clip] {:<10} {:>5} frames {:>5.1}s native -> window {:.1}s (x{:.2})",
            spec.effect,
            clip.len(),
            clip.seconds(),
            window,
            speed
        );
        Self {
            clip,
            from: bars(from_bar),
            speed,
        }
    }

    fn draw(&self, cv: &mut TextCanvas, t: f32, gain: f32) {
        self.clip.draw_at(cv, t - self.from, self.speed, 0, 0, gain);
    }
}

pub struct Look {
    pub bg_top: u32,
    pub bg_bottom: u32,
    pub bloom: f32,
    pub logo: Option<LogoState>,
}

pub struct Demo {
    story: Story,
    form: Timed,
    burst: Timed,
    quench: Timed,
    etch: Timed,
    glint: Timed,
    ingot: Ingot,
}

fn banner_spec<'a>(effect: &'a str, seed: u64, opts: &'a [&'a str]) -> ClipSpec<'a> {
    ClipSpec {
        effect,
        text: BANNER,
        cols: COLS,
        rows: ROWS,
        seed,
        opts,
    }
}

impl Demo {
    pub fn new() -> Self {
        let story = Story::new();

        let form = Timed::new(
            banner_spec(
                "scattered",
                21,
                &["--final-gradient-stops", "ff7a00", "ffc070"],
            ),
            21.0,
            24.0,
        );
        let burst = Timed::new(
            banner_spec(
                "unstable",
                33,
                &[
                    "--unstable-color",
                    "ffd88a",
                    "--final-gradient-stops",
                    "ff7a00",
                    "ffc070",
                ],
            ),
            33.0,
            36.0,
        );
        let quench = Timed::new(
            banner_spec(
                "smoke",
                36,
                &[
                    "--starting-color",
                    "ff7a00",
                    "--smoke-gradient-stops",
                    "6b7280",
                    "e5e7eb",
                    "--final-gradient-stops",
                    "10b981",
                    "6ee7b7",
                ],
            ),
            36.0,
            40.0,
        );
        let etch = Timed::new(
            banner_spec(
                "laseretch",
                40,
                &["--final-gradient-stops", "10b981", "6ee7b7"],
            ),
            40.0,
            46.0,
        );
        let glint = Timed::new(
            banner_spec(
                "highlight",
                46,
                &["--final-gradient-stops", "10b981", "6ee7b7"],
            ),
            46.0,
            52.0,
        );
        let ingot = form.clip.final_cells();
        Self {
            story,
            form,
            burst,
            quench,
            etch,
            glint,
            ingot,
        }
    }

    /// Draw time `t` into the main grid and the double-size copy grid.
    pub fn frame(&self, cv: &mut TextCanvas, big: &mut TextCanvas, t: f32) -> Look {
        cv.clear();
        big.clear();
        let n = now(t);
        let bar = t / BAR;
        let mut look = Look {
            bg_top: 0x03050c,
            bg_bottom: 0x0a1020,
            bloom: 0.55,
            logo: None,
        };
        match n.section {
            Section::Night => valley::draw(cv, t, flow(t), 0.45),
            Section::Dam => valley::draw(cv, t, flow(t), 0.45 + 0.4 * n.progress),
            Section::Falls => {
                look.bg_bottom = 0x0e1a30;
                look.bloom = 0.75;
                valley::draw(cv, t, flow(t), 0.85);
            }
            Section::River => {
                look.bg_top = 0x0a1426;
                look.bg_bottom = 0x04080e;
                river::draw(cv, t, 0.95);
            }
            Section::Iron => {
                look.bg_top = 0x0c0402;
                look.bg_bottom = 0x1a0803;
                forge::furnace(cv, t, smooth(n.local / 2.0));
                if bar >= 21.0 {
                    self.form.draw(cv, t, 1.0);
                }
            }
            Section::Hammer => {
                look.bg_top = 0x0c0402;
                look.bg_bottom = 0x1c0904;
                look.bloom = 0.7;
                forge::furnace(cv, t, 0.55);
                forge::wheel(cv, t);
                if t < bars(HAMMER_STOP) {
                    forge::ingot(cv, &self.ingot, t, 0.62 - 0.12 * n.progress);
                } else {
                    self.burst.draw(cv, t, 1.0);
                }
                forge::hammer(cv, t);
                forge::sparks(cv, t);
                if let Some((ts, k)) = last_strike(t).filter(|_| t < bars(HAMMER_STOP)) {
                    if t - ts < 0.06 {
                        cv.shift(if k % 2 == 0 { 1 } else { -1 }, 0);
                    }
                }
            }
            Section::Quench => {
                look.bg_top = 0x080a0c;
                look.bg_bottom = 0x14181c;
                forge::furnace(cv, t, 0.3 * (1.0 - n.progress));
                self.quench.draw(cv, t, 1.0);
                forge::steam(cv, &self.ingot, t, 1.0 - 0.6 * n.progress);
            }
            Section::Mark => {
                look.bg_top = 0x010805;
                look.bg_bottom = 0x04140c;
                look.bloom = 0.6;
                if bar < 46.0 {
                    for &(c, r, cell) in &self.ingot {
                        cv.set(c as i32, r as i32, Cell::new(cell.ch, 0x2a3036));
                    }
                    self.etch.draw(cv, t, 1.0);
                } else {
                    self.glint.draw(cv, t, 1.0);
                }
            }
            Section::Close => {
                // The ouroboros draws itself from tail to head, then its centre.
                let reveal = (bar - 53.0) / 1.5;
                let dim = smooth(reveal * 1.5);
                look.bloom = 0.55 - 0.3 * dim;
                valley::draw(cv, t, flow(t), 0.85 * (1.0 - 0.75 * dim));
                if reveal > 0.0 {
                    let (lx, ly) = (WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0 + 24.0);
                    // Clear the valley behind the mark so it stands on black.
                    for (i, cell) in cv.cells.iter_mut().enumerate() {
                        let px = (i % COLS) as f32 * CELL_W as f32 + CELL_W as f32 / 2.0 - lx;
                        let py = (i / COLS) as f32 * CELL_H as f32 + CELL_H as f32 / 2.0 - ly;
                        let r = (px * px + py * py).sqrt();
                        let hole = smooth((230.0 - r) / 60.0) * dim;
                        *cell = cell.scaled(1.0 - hole);
                    }
                    look.logo = Some(LogoState {
                        cx: WIDTH as f32 / 2.0,
                        cy: HEIGHT as f32 / 2.0 + 24.0,
                        scale: 2.4,
                        progress: smooth(reveal),
                        detail: smooth((reveal - 1.0) * 2.5),
                        gain: 1.0,
                    });
                }
            }
        }

        self.story.draw(cv, big, t);

        // Cuts and fades.
        let fade = smooth(t / 1.2)
            * smooth((DURATION - t) / 1.6)
            * dip(t, bars(20.0), 0.35)
            * dip(t, bars(52.0), 0.4);
        if fade < 1.0 {
            for cell in cv.cells.iter_mut().chain(big.cells.iter_mut()) {
                *cell = cell.scaled(fade);
            }
            if let Some(logo) = look.logo.as_mut() {
                logo.gain *= fade;
            }
            look.bg_top = scale_rgb(look.bg_top, fade);
            look.bg_bottom = scale_rgb(look.bg_bottom, fade);
        }
        for hit in [16.0, 24.0, 33.0, 40.0] {
            let age = t - bars(hit);
            if (0.0..0.4).contains(&age) {
                let k = (-age * 9.0).exp();
                look.bg_top = lerp_rgb(look.bg_top, 0x6080a0, k * 0.5);
                look.bg_bottom = lerp_rgb(look.bg_bottom, 0x8090a0, k * 0.5);
            }
        }
        look
    }
}

/// Dip to black and back around a cut at `at`.
fn dip(t: f32, at: f32, half: f32) -> f32 {
    smooth((t - at).abs() / half)
}
