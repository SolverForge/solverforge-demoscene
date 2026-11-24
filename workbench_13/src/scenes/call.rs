// The call: Cordelia takes the decision. The picture turns emerald night;
// she is drawn in by a scanline in concern, and the words say what changed.
// On the next downbeat a red line cuts in from the left and she dissolves to
// defiance through a raster tear, an emerald glint lighting in her eyes; the
// schedule geometry of the character language (three stepped bars) draws in
// red under the words. Then the plan: keep what has started, replan the
// rest. She dissolves to resolve on the downbeat and the bars fill jade, left
// to right: emerald resolves.
//
// Her keyframes are new edits of the dark mode's approved artwork, made on
// one framing (assets/cordelia/provenance.json), so they dissolve into each
// other with the face in place.
use super::{dot_grid, hash, Faces, JADE, NIGHT, PARCH, PICTURE_H, RED, SAGE};
use crate::gfx::{mix, Frame, W};
use crate::timeline::{ease_out, pulse, span, Now, BAR};

/// The keyframes are 690x570; drawn at 1.2 they fill the picture's height.
pub const SCALE: f32 = 1.2;
const RIGHT: i32 = 40;
/// Iris centres in the keyframes' pixels, measured on the defiance frame.
pub const EYES: [(f32, f32); 2] = [(440.0, 345.0), (516.0, 368.0)];

pub fn draw(f: &mut Frame, faces: &Faces, n: Now, t: f32) {
    let local = n.local;
    f.rect(0, 0, W as i32, PICTURE_H, NIGHT, 1.0);
    circuit(f, t, span(local, 0.0, 1.0));
    dot_grid(f, JADE, 0.10, t);

    let w = (faces.concern.w as f32 * SCALE) as i32;
    let h = (faces.concern.h as f32 * SCALE) as i32;
    let (x, y) = (W as i32 - RIGHT - w, (PICTURE_H - h) / 2);

    // Closing: a scanline collapses her into the next shot.
    let close = span(local, n.len - 0.45, n.len - 0.05);
    let close_y = y as f32 + h as f32 * close;
    let shown = move |sy: i32| if (sy as f32) < close_y { 0.0 } else { 1.0 };

    // The entrance: concern, drawn in top to bottom by a jade line.
    let enter = ease_out(span(local, 0.0, 0.6));
    let enter_y = y as f32 + h as f32 * enter;
    // Each change lands on a downbeat: a dissolve through a raster tear.
    let keys = [
        (BAR, &faces.defiance, RED),
        (3.0 * BAR, &faces.resolve, JADE),
    ];
    let k = keys.iter().rposition(|(at, _, _)| local >= *at);
    let (base, top, mixk, flash) = match k {
        None => (&faces.concern, None, 0.0, JADE),
        Some(i) => {
            let (at, face, rgb) = keys[i];
            let prev = if i == 0 {
                &faces.concern
            } else {
                keys[i - 1].1
            };
            (prev, Some(face), span(local, at, at + 0.35), rgb)
        }
    };
    let since = k.map_or(9.0, |i| local - keys[i].0);
    let tear_k = (1.0 - since / 0.3).clamp(0.0, 1.0);
    let tear = move |sy: i32| {
        if tear_k <= 0.0 {
            return 0.0;
        }
        let band = (sy / 18) as u32;
        let jolt = hash(band * 7919 + (t * 30.0) as u32) - 0.5;
        if jolt.abs() > 0.3 {
            jolt * 60.0 * tear_k
        } else {
            0.0
        }
    };
    base.draw(
        f,
        x,
        y,
        SCALE,
        |sy| {
            if (sy as f32) < enter_y {
                shown(sy)
            } else {
                0.0
            }
        },
        tear,
    );
    if let Some(face) = top {
        if mixk > 0.0 {
            face.draw(f, x, y, SCALE, |sy| mixk * shown(sy), tear);
        }
    }
    if enter < 1.0 {
        f.rect(
            x - 8,
            enter_y as i32 - 1,
            x + w + 8,
            enter_y as i32 + 2,
            JADE,
            1.0,
        );
        f.glow(
            (x + w / 2) as f32,
            enter_y,
            380.0,
            JADE,
            0.22 * (1.0 - enter),
        );
    }
    // The beat's flash along her frame, and the glint in her eyes from
    // defiance on.
    if tear_k > 0.0 {
        f.glow(
            (x + w / 2) as f32,
            (y + h / 2) as f32,
            520.0,
            flash,
            0.18 * tear_k,
        );
    }
    if local >= BAR {
        let g = 0.35 + 0.65 * pulse(t);
        let rgb = mix(0x7dffc4, JADE, span(local, 3.0 * BAR, 3.0 * BAR + 0.6));
        for (ex, ey) in EYES {
            let (cx, cy) = (x as f32 + ex * SCALE, y as f32 + ey * SCALE);
            f.glow(cx, cy, 14.0, rgb, 0.55 * g * (1.0 - close));
        }
    }
    if close > 0.0 && close < 1.0 {
        f.rect(
            x - 8,
            close_y as i32 - 1,
            x + w + 8,
            close_y as i32 + 2,
            PARCH,
            1.0,
        );
    }
    brackets(
        f,
        x - 14,
        y.max(12) - 2,
        x + w + 14,
        (y + h).min(PICTURE_H - 12) + 2,
        mix(JADE, PARCH, 0.2 * pulse(t)),
        span(local, 0.2, 0.6) * (1.0 - close),
    );
    f.text(
        x + 20,
        PICTURE_H - 44,
        "CORDELIA",
        2,
        SAGE,
        span(local, 0.4, 0.9) * (1.0 - close),
    );

    // The conflict: a red line cuts in from the left on the second bar.
    let cut = ease_out(span(local, BAR - 0.35, BAR));
    if cut > 0.0 {
        let ly = 520.0;
        let x1 = 60.0 + (x as f32 - 90.0) * cut;
        let rgb = mix(RED, JADE, span(local, 3.0 * BAR, 3.0 * BAR + 0.6));
        f.line(60.0, ly, x1, ly, 2.0, rgb, 1.0 - close);
        f.glow(x1, ly, 30.0, rgb, 0.6 * (1.0 - close));
    }
    // The schedule geometry: three stepped bars, drawn in red, filled jade
    // when the decision is made.
    let draw_in = span(local, 2.0 * BAR, 2.0 * BAR + 0.8);
    let fill = span(local, 3.0 * BAR + 0.1, 3.0 * BAR + 1.2);
    for i in 0..3 {
        let bx = 60 + i * 90;
        let by = 580 + i * 70;
        let bw = 520;
        let grow = ease_out((draw_in * 3.0 - i as f32).clamp(0.0, 1.0));
        let x1 = bx + (bw as f32 * grow) as i32;
        if grow <= 0.0 {
            continue;
        }
        let edge = mix(RED, JADE, fill);
        f.frame_rect(bx, by, x1, by + 40, edge, 1.0 - close);
        // Jade fills from the left as the plan resolves.
        let fx = bx + ((x1 - bx) as f32 * fill) as i32;
        f.rect(
            bx + 2,
            by + 2,
            fx.max(bx + 2) - 2,
            by + 38,
            JADE,
            0.35 * (1.0 - close),
        );
        // The step at each bar's end, as in the character language's
        // schedule mark.
        f.rect(x1 - 2, by, x1, by + 40, edge, 1.0 - close);
    }
    // A beat tick in the corner keeps the time with the music.
    let tick = pulse(t);
    f.rect(
        60,
        860,
        60 + (40.0 * tick) as i32,
        862,
        JADE,
        0.6 * (1.0 - close),
    );
}

/// Corner brackets around a rectangle.
fn brackets(f: &mut Frame, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
    if a <= 0.0 {
        return;
    }
    let l = 36;
    for (cx, cy, dx, dy) in [
        (x0, y0, 1, 1),
        (x1, y0, -1, 1),
        (x0, y1, 1, -1),
        (x1, y1, -1, -1),
    ] {
        let (xa, xb) = if dx > 0 { (cx, cx + l) } else { (cx - l, cx) };
        let (ya, yb) = if dy > 0 { (cy, cy + l) } else { (cy - l, cy) };
        f.rect(
            xa,
            cy - (dy < 0) as i32 * 2,
            xb,
            cy + 2 - (dy < 0) as i32 * 2,
            rgb,
            a,
        );
        f.rect(
            cx - (dx < 0) as i32 * 2,
            ya,
            cx + 2 - (dx < 0) as i32 * 2,
            yb,
            rgb,
            a,
        );
    }
}

/// The character language's backdrop: thin diagonal circuit lines in jade,
/// a red one here and there, drifting.
fn circuit(f: &mut Frame, t: f32, a: f32) {
    if a <= 0.0 {
        return;
    }
    for i in 0..26u32 {
        let x = hash(i * 13 + 1) * W as f32;
        let y = hash(i * 29 + 5) * PICTURE_H as f32;
        let len = 60.0 + hash(i * 7 + 3) * 180.0;
        let drift = (t * 8.0 + hash(i) * 50.0) % 60.0;
        let rgb = if i % 9 == 4 { RED } else { JADE };
        let x0 = x + drift;
        f.line(x0, y, x0 + len * 0.6, y - len * 0.6, 1.0, rgb, a * 0.22);
        // A short dotted run off the end, like the atlas's guides.
        for k in 0..6 {
            f.blend(
                (x0 + len * 0.6 + 8.0 + k as f32 * 8.0) as i32,
                (y - len * 0.6) as i32,
                SAGE,
                a * 0.25,
            );
        }
    }
}
