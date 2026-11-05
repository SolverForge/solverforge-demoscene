// Live: the Desktop take plays in a window cut out of the demo, and the
// advisor -- the Nous Girl pet, in the manner of SimCity's advisor on the
// SNES -- speaks the narration from a column beside it. The demo draws
// everything around the holes: copper bars rolling behind phosphor bezels.
// It leaves the take's window, the portrait and the dialog box black for the
// compose step, which lays the take, the sprite and the typed words in at
// exactly HOLE, PORTRAIT and DIALOG.
use super::{EMERALD, LIME, MINT};
use crate::gfx::{mix, Frame, H, W};
use crate::statue::Statue;
use crate::timeline::{smooth, Now, BEAT};

/// The take's window on the 1920x1080 frame: 16:9, 1440x810.
pub const HOLE: (i32, i32, i32, i32) = (48, 135, 1488, 945);
/// The advisor's portrait box and, under it, her dialog box.
pub const PORTRAIT: (i32, i32, i32, i32) = (1528, 135, 1880, 503);
pub const DIALOG: (i32, i32, i32, i32) = (1528, 543, 1880, 945);

const COPPER: [u32; 4] = [0x50fc00, 0x10b981, 0x0ea5e9, 0x6ee7b7];

pub fn draw(f: &mut Frame, statue: &Statue, n: Now, t: f32) -> f32 {
    let (x0, _, x1, y1) = HOLE;
    let open = smooth(n.local / 0.8) * smooth((n.len - n.local) / 0.8);

    // Copper bars: horizontal bands of light that roll on the beat.
    f.clear(0x010402);
    // The statue under everything, the rays passing over it.
    statue.draw(f, MINT, 0.03, t);
    for (k, rgb) in COPPER.iter().enumerate() {
        let phase = t * 0.35 + k as f32 * 0.25;
        let cy = (phase.fract() * (H as f32 + 240.0)) - 120.0;
        let beat = 1.0 + 0.25 * (-(t % BEAT) * 8.0).exp();
        for y in (cy - 60.0) as i32..(cy + 60.0) as i32 {
            if y < 0 || y >= H as i32 {
                continue;
            }
            let d = 1.0 - ((y as f32 - cy) / 60.0).abs();
            let c = mix(0x000000, *rgb, d * d * 0.55 * beat);
            for x in 0..W as i32 {
                f.add(x, y, c, 0.6);
            }
        }
    }

    // The bezels grow open from their middles, the advisor's a beat after
    // the take's; the holes stay black.
    bezel(f, HOLE, open);
    let side = smooth((n.local - 0.4) / 0.8) * smooth((n.len - n.local) / 0.8);
    bezel(f, PORTRAIT, side);
    bezel(f, DIALOG, side);

    // Under the window: what this is.
    let label = smooth((n.local - 0.8) / 0.5) * smooth((n.len - n.local) / 0.8);
    f.text_centered(
        (x0 + x1) / 2,
        y1 + 24,
        "LIVE IN HERMES DESKTOP  //  NOTHING STAGED",
        2,
        MINT,
        label,
    );
    1.0
}

/// A phosphor bezel around `rect`, grown open to `open` (0..1) from its
/// middle, with the inside left black.
fn bezel(f: &mut Frame, rect: (i32, i32, i32, i32), open: f32) {
    if open <= 0.0 {
        return;
    }
    let (x0, y0, x1, y1) = rect;
    let (cx, cy) = ((x0 + x1) / 2, (y0 + y1) / 2);
    let hw = ((x1 - x0) as f32 / 2.0 * open) as i32;
    let hh = ((y1 - y0) as f32 / 2.0 * open) as i32;
    let (bx0, by0, bx1, by1) = (cx - hw, cy - hh, cx + hw, cy + hh);
    for pad in 1..=10 {
        f.frame_rect(
            bx0 - pad,
            by0 - pad,
            bx1 + pad,
            by1 + pad,
            EMERALD,
            0.5 / pad as f32,
        );
    }
    f.frame_rect(bx0 - 2, by0 - 2, bx1 + 2, by1 + 2, LIME, 0.9);
    f.rect(bx0, by0, bx1, by1, 0x000000, 1.0);
}
