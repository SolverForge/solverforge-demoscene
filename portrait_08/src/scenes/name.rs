// NAME: the guitar again, dim behind, ringing out the last chord; his
// portrait, the name, four words, and the work in one line.
use demofx::Clip;

use super::guitar::{draw_guitar, room, Guitar, FULL};
use crate::gfx::{Frame, W};
use crate::portrait::Portrait;
use crate::score::{strings_at, Note};
use crate::timeline::{smooth, Now};

const WARM: u32 = 0xf2d48a;
const PAPER: u32 = 0xfff6e0;
const DIM: u32 = 0xb8a88a;

pub fn draw(
    f: &mut Frame,
    guitar: &Guitar,
    notes: &[Note],
    portrait: &Portrait,
    name: &Clip,
    n: Now,
    t: f32,
) -> f32 {
    let fade = smooth(n.local / 0.8) * smooth((n.len - n.local) / 2.0);
    room(f, t, fade * 0.6);
    draw_guitar(f, guitar, FULL, strings_at(notes, t), t, 0.22 * fade);

    let p = 3.4;
    let pw = portrait.w as f32 * p;
    portrait.draw(
        f,
        W as f32 / 2.0 - pw / 2.0,
        420.0,
        p,
        WARM,
        ((n.local - 0.2) / 2.2).clamp(0.0, 1.0),
        fade,
        t,
    );

    let local = n.local - 1.0;
    if local > 0.0 {
        let speed = (name.seconds() / 2.0).max(1.0);
        f.clip(name, (local * 60.0 * speed) as usize, 180, 900, 8, fade);
    }
    let lines: [(f32, &str, i32, u32, i32); 4] = [
        (3.2, "ENGINEER. CREATOR.", 4, PAPER, 1090),
        (3.8, "GAMER. GUITARIST.", 4, PAPER, 1150),
        (4.8, "FOUNDER, SOLVERFORGE.", 3, WARM, 1250),
        (5.6, "VDISTEFANO.STUDIO", 2, DIM, 1320),
    ];
    for (at, s, scale, c, y) in lines {
        let a = smooth((n.local - at) / 0.6) * fade;
        f.text_centered(W as i32 / 2, y, s, scale, c, a);
    }
    1.0
}
