// 05 CLOSE: three sentences in the depot manager's voice, one at a time
// (scenes/captions.rs draws them), each with a line from the capture under
// it; then the ouroboros draws itself in the middle of the frame and the
// address holds to the last frame over the final E minor.
use demofx::logo::{draw as logo, LogoState};

use super::{Timed, EMERALD, LIME, LIME_STOPS, MINT, PAPER};
use crate::capture::Capture;
use crate::fx;
use crate::gfx::{size, smooth, Frame, S3, W};
use crate::script::Script;
use crate::timeline::{Now, DURATION};

const LOGO: (f32, f32) = (960.0, 450.0);
const SUB: crate::gfx::Size = size(15, 27);

pub struct Close {
    url: Timed,
}

impl Close {
    pub fn new(_c: &Capture, s: &Script) -> Self {
        Self {
            url: Timed::new("print", "solverforge.org", 1599, LIME_STOPS, s.url, 0.8),
        }
    }
}

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
    // Each sentence is read alone; its line from the capture appears
    // under it only after that, and both leave together.
    for (i, (sub, at)) in s.subs.iter().enumerate() {
        let to = s
            .captions
            .iter()
            .filter(|c| c.place == crate::script::Place::Close)
            .nth(i)
            .map_or(*at + 3.0, |c| c.to);
        let k = smooth((t - at) / 0.3) * (1.0 - smooth((t - to + 0.4) / 0.4)) * end;
        if k > 0.0 {
            let y = super::captions::close_bottom(s, i) + 30;
            let w = sub.len() as i32 * SUB.w;
            f.text_shadow((W as i32 - w) / 2, y, sub, SUB, PAPER, k);
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
