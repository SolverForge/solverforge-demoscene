// Outro: the logo draws itself large in the middle (the corner mark steps
// aside), the guarantees run on the cards, and the address stays up to the
// last frame.
use demofx::logo::{draw as logo, LogoState};

use super::{dot_grid, EMERALD, LIME, MINT, PAPER};
use crate::gfx::{Frame, W};
use crate::timeline::{smooth, Now};

pub fn draw(f: &mut Frame, n: Now) -> f32 {
    f.gradient(&[0x020503, 0x04140c, 0x020503]);
    dot_grid(f, EMERALD, 0.2 * smooth(n.local / 0.8));

    let grow = smooth(n.local / 3.0);
    let end = smooth((n.len - n.local) / 1.2);
    f.glow(W as f32 / 2.0, 560.0, 360.0, LIME, 0.10 * grow * end);
    logo(
        f,
        &LogoState::sized(W as f32 / 2.0, 560.0, 230.0, grow, end),
    );

    let addr = smooth((n.local - 2.5) / 0.6) * end;
    f.text_centered(
        W as i32 / 2,
        860,
        "github.com/blackopsrepl/hermes-solverforge-routing",
        3,
        PAPER,
        addr,
    );
    let tag = smooth((n.local - 3.5) / 0.6) * end;
    f.text_centered(
        W as i32 / 2,
        920,
        "HERMES AGENT PLUGIN HACKATHON  //  @NOUSRESEARCH",
        2,
        MINT,
        tag,
    );
    // The large logo is up: the corner mark steps aside.
    1.0 - smooth(n.local / 0.6)
}
