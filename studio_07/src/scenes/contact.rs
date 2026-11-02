// CONTACT: the portrait again, the name, the four offers as chips, and
// where to write. The frame fades to black on the last bar.
use demofx::Clip;

use super::{dot_grid, typed, EMERALD, LIME, PAPER};
use crate::gfx::{Frame, W};
use crate::portrait::Portrait;
use crate::timeline::{smooth, Now};

const LINES: [(&str, u32); 5] = [
    ("HELLO@SOLVERFORGE.AI", LIME),
    ("VDISTEFANO.STUDIO", PAPER),
    ("SOLVERFORGE.ORG", PAPER),
    ("GITHUB.COM/BLACKOPSREPL", PAPER),
    ("BERGAMO, ITALY", 0x8fa89a),
];
const OFFERS: [(&str, u32); 4] = [
    ("PLANNING", 0x34d399),
    ("ADVISORY", 0x7dd3fc),
    ("AGENTS", 0xa78bfa),
    ("DEVREL", 0xfb923c),
];
const CREDITS: &str = "STUDIO_07  -  SOLVERFORGE DEMOSCENE  -  BERGAMO MMXXVI";

pub fn draw(f: &mut Frame, portrait: &Portrait, name: &Clip, n: Now, t: f32) -> f32 {
    let fade = smooth(n.local / 0.8) * smooth((n.len - n.local) / 1.8);
    f.clear(0x020503);
    dot_grid(f, LIME, 0.05 * fade);

    portrait.draw(
        f,
        70.0,
        130.0,
        4.0,
        LIME,
        ((n.local - 0.2) / 2.5).clamp(0.0, 1.0),
        fade,
        t,
    );

    let x = 560;
    let local = n.local - 0.6;
    if local > 0.0 {
        let speed = (name.seconds() / 2.2).max(1.0);
        f.clip(name, (local * 60.0 * speed) as usize, x, 140, 3, fade);
    }
    for (i, (line, c)) in LINES.iter().enumerate() {
        let at = 2.0 + i as f32 * 0.7;
        let y = 220 + i as i32 * 40;
        let scale = if i == 0 { 3 } else { 2 };
        let y = if i == 0 { y - 4 } else { y + 12 };
        f.text(x, y, &typed(line, at, 30.0, n.local), scale, *c, fade);
    }

    // The four offers, as chips.
    let mut cx = x;
    for (i, (label, c)) in OFFERS.iter().enumerate() {
        let a = smooth((n.local - 5.8 - i as f32 * 0.3) / 0.4) * fade;
        let w = label.len() as i32 * 20 + 24;
        f.rect(cx, 470, cx + w, 506, *c, 0.18 * a);
        f.frame_rect(cx, 470, cx + w, 506, *c, a);
        f.text(cx + 12, 479, label, 2, *c, a);
        cx += w + 14;
    }

    let a = smooth((n.local - 8.0) / 1.0) * fade;
    f.text_centered(W as i32 / 2, 676, CREDITS, 2, EMERALD, 0.8 * a);
    1.0
}
