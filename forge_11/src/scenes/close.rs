// 05 CLOSE: three claims, one per bar, each backed by a line from the
// capture; then the ouroboros draws itself and the address stays up to the
// last frame.
use demofx::logo::{draw as logo, LogoState};

use super::{Timed, EMERALD, LIME, LIME_STOPS, MINT, PAPER};
use crate::capture::{num, Capture};
use crate::fx;
use crate::gfx::{clip, size, smooth, Frame, S3};
use crate::script::Script;
use crate::timeline::{Now, BAR, DURATION};

const BIG: crate::gfx::Size = size(44, 80);
const LINES: [(&str, &str); 3] = [
    ("YOUR MODEL.", "decrypt"),
    ("YOUR MACHINE.", "beams"),
    ("YOUR SOLVER VERSION.", "laseretch"),
];
const LOGO: (f32, f32) = (1500.0, 480.0);
const SUB: crate::gfx::Size = size(15, 27);

pub struct Close {
    lines: Vec<Timed>,
    subs: [String; 3],
    url: Timed,
    from: f32,
}

impl Close {
    pub fn new(c: &Capture, s: &Script) -> Self {
        let lines = LINES
            .iter()
            .enumerate()
            .map(|(i, &(text, effect))| {
                Timed::new(effect, text, 1500 + i as u64, LIME_STOPS, s.close[i], 0.9)
            })
            .collect();
        let base = c
            .workdir
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();
        let dir = c.code[0]
            .path
            .rsplit_once('/')
            .map(|(d, _)| format!("{d}/"))
            .unwrap_or_default();
        let model = if base.is_empty() {
            format!("ordinary Rust in your repo: {dir}")
        } else {
            format!("ordinary Rust in your repo: {base}/{dir}")
        };
        let machine = format!(
            "solved in {} ms, repaired live in {} ms. No sign-in.",
            num(c.solve.wall_ms.round()),
            num(c.repair.solve.wall_ms.round())
        );
        let mut ver = Vec::new();
        if let Some(v) = c.version("solverforge") {
            ver.push(format!("solverforge {v}"));
        }
        if let Some(v) = c.version("solverforge_cli") {
            ver.push(format!("solverforge-cli {v}"));
        }
        let cols = (1120 / SUB.w) as usize;
        Self {
            lines,
            subs: [
                clip(&model, cols),
                clip(&machine, cols),
                clip(&ver.join("  /  "), cols),
            ],
            url: Timed::new(
                "print",
                "solverforge.org",
                1599,
                LIME_STOPS,
                s.close[0] + 3.0 * BAR,
                0.8,
            ),
            from: s.close[0],
        }
    }
}

/// Returns the corner logo's gain (it steps aside for the large mark).
pub fn draw(f: &mut Frame, _c: &Capture, cl: &Close, t: f32, _n: Now) -> f32 {
    f.gradient(&[0x000000, 0x020a06, 0x04140c, 0x000000]);
    fx::starfield(f, t, 0.08, 0.7, MINT);
    fx::copper(f, t, 120.0, 1040.0, &[EMERALD, LIME, 0x0ea5a4], 5, 0.18);
    fx::river(
        f,
        t,
        1000,
        4,
        &[0x02140c, 0x0b5a3a, EMERALD, 0xd1fae5],
        0.45,
    );

    let end = smooth((DURATION - t) / 1.2);
    let x = 140;
    for (i, line) in cl.lines.iter().enumerate() {
        let y = 190 + i as i32 * 200;
        line.draw(f, t, x, y, BIG, end);
        let k = smooth((t - line.from - 0.8) / 0.4) * end;
        f.text_shadow(x + 4, y + BIG.h + 14, &cl.subs[i], SUB, PAPER, k);
    }

    let grow = smooth((t - (cl.from + 2.5 * BAR)) / 1.4);
    if grow > 0.0 {
        f.glow(LOGO.0, LOGO.1, 380.0, LIME, 0.10 * grow * end);
        logo(f, &LogoState::sized(LOGO.0, LOGO.1, 210.0, grow, end));
    }
    let w = 15 * S3.w;
    cl.url.draw(f, t, LOGO.0 as i32 - w / 2, 760, S3, end);
    1.0 - grow
}
