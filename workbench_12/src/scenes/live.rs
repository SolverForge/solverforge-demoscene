// Live: the Desktop take plays in a window cut out of the demo, titled with
// the step of the work it shows (story.rs) and with the plugin's models
// named under it, the one on screen lit. The
// advisor -- the Nous Girl pet, in the manner of SimCity's advisor on the
// SNES -- speaks the narration from a column beside it. The demo draws
// everything around the holes: copper bars rolling behind phosphor bezels.
// It leaves the take's window, the portrait and the dialog box black for the
// compose step, which lays the take, the sprite and the typed words in at
// exactly HOLE, PORTRAIT and DIALOG.
use super::{EMERALD, LIME, MINT};
use crate::cut::CHAPTERS;
use crate::gfx::{advance, mix, pack_px, par_rows, parts, Frame, H};
use crate::models::{on_screen, MODELS};
use crate::statue::Statue;
use crate::timeline::{smooth, Now, BEAT};

/// The take retains the capture's aspect; the right column belongs to the
/// speaking Nous Girl and her timed dialog, as in the planner demo.
pub const HOLE: (i32, i32, i32, i32) = (48, 158, 1488, 932);
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
    // One row at a time, the bands in order, so each pixel takes the same
    // additions in the same order as band by band.
    let beat = 1.0 + 0.25 * (-(t % BEAT) * 8.0).exp();
    let bands: Vec<(f32, u32)> = COPPER
        .iter()
        .enumerate()
        .map(|(k, rgb)| {
            let phase = t * 0.35 + k as f32 * 0.25;
            ((phase.fract() * (H as f32 + 240.0)) - 120.0, *rgb)
        })
        .collect();
    par_rows(&mut f.px, |y, row| {
        for &(cy, rgb) in &bands {
            if (y as i32) < (cy - 60.0) as i32 || (y as i32) >= (cy + 60.0) as i32 {
                continue;
            }
            let d = 1.0 - ((y as f32 - cy) / 60.0).abs();
            let (br, bg, bb) = parts(mix(0x000000, rgb, d * d * 0.55 * beat));
            for p in row.iter_mut() {
                let (ar, ag, ab) = parts(*p);
                *p = pack_px(ar + br * 0.6, ag + bg * 0.6, ab + bb * 0.6);
            }
        }
    });

    // Open the footage and advisor boxes on their own clocks.
    bezel(f, HOLE, open);
    let side = smooth((n.local - 0.4) / 0.8) * smooth((n.len - n.local) / 0.8);
    bezel(f, PORTRAIT, side);
    bezel(f, DIALOG, side);

    // Under the window: the plugin's models, the one on screen lit.
    let rack = smooth((n.local - 0.8) / 0.5) * smooth((n.len - n.local) / 0.8);
    let lit = chapter_at(n.local).and_then(on_screen);
    let names: Vec<&str> = MODELS.iter().map(|m| m.short).collect();
    let gap = 3 * advance(2);
    let width: i32 = names
        .iter()
        .map(|s| s.len() as i32 * advance(2))
        .sum::<i32>()
        + gap * 4;
    let mut x = (x0 + x1) / 2 - width / 2;
    for (i, name) in names.iter().enumerate() {
        let w = name.len() as i32 * advance(2);
        let on = lit.is_none_or(|k| k == i);
        f.text(
            x,
            y1 + 26,
            name,
            2,
            if on { LIME } else { MINT },
            rack * if on { 1.0 } else { 0.3 },
        );
        if on && lit.is_some() {
            f.rect(x, y1 + 50, x + w, y1 + 53, LIME, rack);
        }
        x += w + gap;
    }
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

/// The chapter of the cut playing `local` seconds into the live section.
fn chapter_at(local: f32) -> Option<&'static str> {
    CHAPTERS
        .iter()
        .find(|(_, a, b)| (*a..*b).contains(&local))
        .map(|(name, _, _)| *name)
}
