// 05 CLOSE: three sentences in the depot manager's voice, one at a time
// (scenes/captions.rs draws them), each with a line from the capture under
// it; then the ouroboros draws itself in the middle of the frame and the
// address holds to the last frame over the final E minor.
use demofx::logo::{draw as logo, LogoState};

use super::{Timed, EMERALD, LIME, LIME_STOPS, MINT, PAPER};
use crate::capture::{num, Capture};
use crate::fx;
use crate::gfx::{clip, size, smooth, Frame, S3, W};
use crate::script::Script;
use crate::timeline::{Now, BAR, DURATION};

const LOGO: (f32, f32) = (960.0, 450.0);
const SUB: crate::gfx::Size = size(15, 27);

pub struct Close {
    subs: [String; 3],
    url: Timed,
}

impl Close {
    pub fn new(c: &Capture, s: &Script) -> Self {
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
            format!("ordinary Rust: {dir}")
        } else {
            format!("ordinary Rust: {base}/{dir}")
        };
        let machine = format!(
            "first plan in {} ms, the repair in {} ms",
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
        let cols = (1500 / SUB.w) as usize;
        Self {
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
                s.logo + BAR,
                0.8,
            ),
        }
    }
}

/// Returns the corner logo's gain (it steps aside for the large mark).
pub fn draw(f: &mut Frame, s: &Script, cl: &Close, t: f32, _n: Now) -> f32 {
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
    let ends: Vec<f32> = s
        .captions
        .iter()
        .filter(|c| c.place == crate::script::Place::Close)
        .map(|c| c.to)
        .collect();
    for (i, &at) in s.close.iter().enumerate() {
        let len = ends.get(i).map_or(3.0 * BAR, |e| e - at);
        let k = smooth((t - at - 1.2) / 0.4) * (1.0 - smooth((t - at - len + 0.4) / 0.4)) * end;
        if k > 0.0 {
            let y = super::captions::close_bottom(s, i) + 30;
            let w = cl.subs[i].len() as i32 * SUB.w;
            f.text_shadow((W as i32 - w) / 2, y, &cl.subs[i], SUB, PAPER, k);
        }
    }

    let grow = smooth((t - s.logo) / 1.4);
    if grow > 0.0 {
        f.glow(LOGO.0, LOGO.1, 380.0, LIME, 0.10 * grow * end);
        logo(f, &LogoState::sized(LOGO.0, LOGO.1, 230.0, grow, end));
    }
    let w = 15 * S3.w;
    cl.url.draw(f, t, LOGO.0 as i32 - w / 2, 760, S3, end);
    1.0 - grow
}
