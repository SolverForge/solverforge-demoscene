// Outro: the logo draws itself large in the middle (the corner mark steps
// aside), the card names the plugin, and the address and the install line
// stay up to the last frame.
use demofx::logo::{draw as logo, LogoState};

use super::{dot_grid, EMERALD, LIME, MINT, PAPER};
use crate::gfx::{Frame, W};
use crate::timeline::{smooth, Now};

pub fn draw(f: &mut Frame, n: Now) -> (f32, f32) {
    f.gradient(&[0x020503, 0x04140c, 0x020503]);
    dot_grid(f, EMERALD, 0.2 * smooth(n.local / 0.8));

    let grow = smooth(n.local / 0.9);
    let end = smooth((n.len - n.local) / 1.2);
    f.glow(W as f32 / 2.0, 540.0, 360.0, LIME, 0.10 * grow * end);
    logo(
        f,
        &LogoState::sized(W as f32 / 2.0, 540.0, 210.0, grow, end),
    );

    let addr = smooth((n.local - 0.5) / 0.4) * end;
    f.text_centered(
        W as i32 / 2,
        830,
        "github.com/blackopsrepl/hermes-solverforge-workbench",
        3,
        PAPER,
        addr,
    );
    let tag = smooth((n.local - 0.9) / 0.4) * end;
    f.text_centered(
        W as i32 / 2,
        900,
        "hermes plugins install blackopsrepl/hermes-solverforge-workbench --enable",
        2,
        MINT,
        tag,
    );
    // The large logo is up: no corner mark.
    (1.0, 0.0)
}
