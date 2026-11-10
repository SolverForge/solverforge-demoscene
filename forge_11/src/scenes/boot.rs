// BOOT: the forge lights. A warp starfield, copper bars dropping in, the
// SOLVERFORGE banner beamed in by glyphfx, and the four verbs of the story landing
// one per beat into the first downbeat.
use super::{Timed, DIM, EMERALD, LIME, MINT};
use crate::fx;
use crate::gfx::{size, smooth, Frame, Size, S2, W};

/// The banner's cell: 91 columns at 18 px leave a margin either side.
const BANNER_CELL: Size = size(18, 33);
use crate::timeline::{Now, BEAT};

const BANNER: &str = "\
███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝";

const VERBS: [&str; 4] = ["DESCRIBE", "GENERATE", "SOLVE", "REPAIR"];

pub struct Boot {
    banner: Timed,
}

impl Boot {
    pub fn new() -> Self {
        Self {
            banner: Timed::new(
                "beams",
                BANNER,
                1101,
                &["--final-gradient-stops", "10b981", "50fc00", "e8ffd6"],
                0.25,
                2.2,
            ),
        }
    }
}

pub fn draw(f: &mut Frame, b: &Boot, t: f32, n: Now) {
    f.gradient(&[0x000000, 0x02080a, 0x041410, 0x000000]);
    fx::starfield(f, t, 0.35 + 0.5 * smooth(n.local / n.len), 1.0, 0x6ee7b7);
    fx::copper(
        f,
        t,
        640.0 - 400.0 * smooth(t / 2.5),
        1080.0,
        &[EMERALD, LIME, 0x0ea5a4],
        5,
        0.35 * smooth((t - 0.6) / 1.2),
    );
    let cols = BANNER.lines().map(|l| l.chars().count()).max().unwrap_or(0) as i32;
    let c = BANNER_CELL;
    let x = (W as i32 - cols * c.w) / 2;
    f.dim(x - 40, 300, x + cols * c.w + 40, 300 + 6 * c.h + 30, 0.35);
    b.banner.draw(f, t, x, 315, c, 1.0);

    // The verbs land on beats 4..7, into the first downbeat.
    let y = 610;
    let gap = 3 * S2.w;
    let total: i32 = VERBS.iter().map(|v| v.len() as i32 * S2.w).sum::<i32>() + gap * 3;
    let mut vx = (W as i32 - total) / 2;
    for (i, v) in VERBS.iter().enumerate() {
        let at = (4 + i) as f32 * BEAT - 0.02;
        let k = smooth((t - at) / 0.12);
        let col = if t - at < 0.15 { 0xffffff } else { MINT };
        f.text_shadow(vx, y, v, S2, col, k);
        if i < 3 {
            f.text(vx + v.len() as i32 * S2.w + S2.w, y, "/", S2, DIM, k);
        }
        vx += v.len() as i32 * S2.w + gap;
    }
}
