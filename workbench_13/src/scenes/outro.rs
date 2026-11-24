// The close: the mark draws itself, Cordelia at ease beside it, the story
// cards say what this is and what every plan gets, and the address and the
// install line stay up until the picture fades.
use demofx::logo::{draw as logo, LogoState};

use super::{dot_grid, Faces, JADE, NIGHT, PARCH, SAGE};
use crate::gfx::{Frame, H, W};
use crate::timeline::{ease_out, span, Now};

const SCALE: f32 = 1.05;

pub fn draw(f: &mut Frame, faces: &Faces, n: Now, t: f32) {
    let local = n.local;
    let end = 1.0 - span(local, n.len - 1.2, n.len - 0.1);
    f.rect(0, 0, W as i32, H as i32, NIGHT, 1.0);
    dot_grid(f, JADE, 0.14 * end, t);

    let grow = span(local, 0.0, 0.9);
    f.glow(520.0, 300.0, 300.0, JADE, 0.10 * grow * end);
    logo(f, &LogoState::sized(520.0, 300.0, 180.0, grow, end));

    // Cordelia, drawn in by a scanline.
    let img = &faces.ease;
    let (w, h) = ((img.w as f32 * SCALE) as i32, (img.h as f32 * SCALE) as i32);
    let (x, y) = (W as i32 - 60 - w, 100);
    let line = y as f32 + h as f32 * ease_out(span(local, 0.4, 1.2));
    img.draw(
        f,
        x,
        y,
        SCALE,
        |sy| if (sy as f32) < line { end } else { 0.0 },
        |_| 0.0,
    );
    if line < (y + h) as f32 {
        f.rect(
            x - 8,
            line as i32 - 1,
            x + w + 8,
            line as i32 + 2,
            JADE,
            end,
        );
    }

    let addr = span(local, 1.0, 1.4) * end;
    f.text_centered(
        W as i32 / 2,
        900,
        "github.com/blackopsrepl/hermes-solverforge-workbench",
        2,
        PARCH,
        addr,
    );
    let tag = span(local, 1.4, 1.8) * end;
    // Typed in, one letter a frame.
    let install = "hermes plugins install blackopsrepl/hermes-solverforge-workbench --enable";
    let typed: String = install
        .chars()
        .take(((local - 1.4) * 60.0).max(0.0) as usize)
        .collect();
    let x0 = W as i32 / 2 - install.len() as i32 * crate::gfx::advance(2) / 2;
    f.text(x0, 950, &typed, 2, SAGE, tag);
}
