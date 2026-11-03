// HOME: a window in Bergamo at night. Tiled roofs and chimneys outside,
// the upper town lit on its hill, and a cat on the sill beside a tin of
// its food. Then his reflection comes up in the glass: his own photo,
// printed as a halftone.
use crate::gfx::{hash, mix, Frame, H, W};
use crate::portrait::Portrait;
use crate::timeline::{smooth, Now};

const FRAME_C: u32 = 0x0c0a0f;
const ROOF: u32 = 0x2a1714;
const LIGHT: u32 = 0xffc977;

const WIN: (i32, i32, i32, i32) = (90, 420, 990, 1480);

pub fn draw(f: &mut Frame, portrait: &Portrait, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.9, 0.9);
    f.clear(0x07060a);
    outside(f, t, fade);
    // The reflection, on the glass.
    let r = smooth((n.local - 7.6) / 2.0) * fade;
    if r > 0.0 {
        let p = 4.0;
        let (pw, ph) = (portrait.w as f32 * p, portrait.h as f32 * p);
        portrait_reflection(f, portrait, (540.0 - pw / 2.0, 560.0), p, r * 0.55, t, ph);
    }
    window_frame(f, fade);
    sill(f, n.local, t, fade);
    1.0
}

fn outside(f: &mut Frame, t: f32, a: f32) {
    let (x0, y0, x1, y1) = WIN;
    // Sky.
    for y in y0..y1 {
        let k = (y - y0) as f32 / (y1 - y0) as f32;
        f.rect(x0, y, x1, y + 1, mix(0x0e1230, 0x2a2140, k), a);
    }
    for i in 0..160 {
        let (x, y) = (
            x0 as f32 + hash(i, 1) * (x1 - x0) as f32,
            y0 as f32 + hash(i, 2) * 400.0,
        );
        f.blend(x as i32, y as i32, 0xdfe8ff, 0.5 * hash(i, 3) * a);
    }
    f.glow(780.0, 560.0, 200.0, 0x7a7aa8, 0.4 * a);
    f.disc(780.0, 560.0, 46.0, 0xf2eedd, a);
    // The upper town on its hill, far off: towers and warm windows.
    for x in x0..x1 {
        let u = (x as f32 - 540.0) / 420.0;
        let top = 860.0 - 110.0 * (1.0 - u * u).max(0.0);
        f.rect(x, top as i32, x + 1, y1, 0x14101e, a);
    }
    for (tx, th, tw) in [
        (470.0, 140.0, 22.0),
        (560.0, 190.0, 26.0),
        (640.0, 120.0, 20.0),
    ] {
        let base = 860.0 - 110.0 * (1.0 - ((tx - 540.0) / 420.0f32).powi(2));
        f.rect(
            (tx - tw / 2.0) as i32,
            (base - th) as i32,
            (tx + tw / 2.0) as i32,
            base as i32,
            0x14101e,
            a,
        );
        f.blend(tx as i32, (base - th + 20.0) as i32, LIGHT, 0.9 * a);
    }
    for i in 0..220 {
        let x = 180.0 + hash(i, 4) * 720.0;
        let u = (x - 540.0) / 420.0;
        let top = 860.0 - 110.0 * (1.0 - u * u).max(0.0);
        let y = top + 10.0 + hash(i, 5) * 120.0;
        let tw = 0.8 + 0.2 * (t * 2.0 + i as f32).sin();
        f.blend(x as i32, y as i32, LIGHT, 0.7 * hash(i, 6) * tw * a);
    }
    // Near roofs: sloped tiles, chimneys.
    let roofs = [(x0 - 40, 1060, 420), (380, 1000, 380), (700, 1080, 360)];
    for (rx, ry, rw) in roofs {
        for dx in 0..rw {
            let x = rx + dx;
            let peak = ry - (rw / 2 - (dx - rw / 2).abs()) / 3;
            f.rect(x, peak, x + 1, y1, ROOF, a);
            if (dx / 14) % 2 == 0 {
                f.rect(x, peak, x + 1, peak + 3, 0x4a2a22, a);
            }
        }
        for row in 0..12 {
            let y = ry + 30 + row * 26;
            f.rect(rx, y, rx + rw, y + 2, 0x1c0f0d, a);
        }
        f.rect(rx + rw / 3, ry - 70, rx + rw / 3 + 30, ry, 0x1c0f0d, a);
    }
    // A lit window across the street.
    f.rect(470, 1210, 520, 1280, LIGHT, 0.55 * a);
    f.glow(495.0, 1245.0, 70.0, LIGHT, 0.2 * a);
}

fn window_frame(f: &mut Frame, a: f32) {
    let (x0, y0, x1, y1) = WIN;
    f.rect(0, 0, W as i32, y0, FRAME_C, a);
    f.rect(0, y1, W as i32, H as i32, FRAME_C, a);
    f.rect(0, y0, x0, y1, FRAME_C, a);
    f.rect(x1, y0, W as i32, y1, FRAME_C, a);
    // Mullions.
    f.rect((x0 + x1) / 2 - 10, y0, (x0 + x1) / 2 + 10, y1, FRAME_C, a);
    f.rect(x0, 900, x1, 916, FRAME_C, a);
    f.frame_rect(x0, y0, x1, y1, 0x2a2430, a);
}

fn sill(f: &mut Frame, local: f32, t: f32, a: f32) {
    let y = WIN.3;
    f.rect(40, y, W as i32 - 40, y + 40, 0x1e1820, a);
    f.rect(40, y + 40, W as i32 - 40, y + 46, 0x0a080c, a);
    cat(f, 330.0, y as f32, local, t, a);
    tin(f, 760.0, y as f32, a);
}

/// The cat, seen against the window: a sitting silhouette with a tail that
/// curls and flicks. No colour, no markings -- only its shape.
fn cat(f: &mut Frame, x: f32, base: f32, local: f32, t: f32, a: f32) {
    let c = 0x030304;
    // Body.
    for dy in 0..170 {
        let k = dy as f32 / 170.0;
        let half = 40.0 + 55.0 * k.powf(0.7);
        let yy = base - 170.0 + dy as f32;
        f.rect(
            (x - half) as i32,
            yy as i32,
            (x + half) as i32,
            yy as i32 + 1,
            c,
            a,
        );
    }
    // Head, ears; it turns toward the room now and then.
    let turn = (local * 0.4).sin() * 10.0;
    let (hx, hy) = (x + turn, base - 205.0);
    f.disc(hx, hy, 46.0, c, a);
    for side in [-1.0f32, 1.0] {
        for i in 0..20 {
            let u = i as f32 / 19.0;
            let ex = hx + side * (22.0 + 10.0 * u);
            let ey = hy - 30.0 - 34.0 * u;
            f.disc(ex, ey, 14.0 * (1.0 - u) + 1.0, c, a);
        }
    }
    // Tail along the sill, its tip flicking.
    let flick = (t * 2.2).sin() * 0.5 + 0.5;
    for i in 0..40 {
        let u = i as f32 / 39.0;
        let tx = x + 70.0 + u * 150.0;
        let ty = base - 6.0 - (u * u) * (40.0 + 50.0 * flick);
        f.disc(tx, ty, 13.0 - 6.0 * u, c, a);
    }
}

/// The tin of cat food, label and all.
fn tin(f: &mut Frame, x: f32, base: f32, a: f32) {
    let (x0, x1) = (x as i32 - 60, x as i32 + 60);
    let (y0, y1) = (base as i32 - 110, base as i32);
    f.rect(x0, y0, x1, y1, 0x8a8a90, a);
    f.rect(x0, y0 + 20, x1, y1 - 14, 0xd8cfb8, a);
    f.text(x0 + 10, y0 + 32, "CAT", 2, 0x3a2a20, a);
    for k in 0..4 {
        f.rect(
            x0 + 10,
            y0 + 62 + k * 8,
            x1 - 10 - k * 12,
            y0 + 65 + k * 8,
            0x6a5a48,
            a,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn portrait_reflection(
    f: &mut Frame,
    p: &Portrait,
    at: (f32, f32),
    pitch: f32,
    gain: f32,
    t: f32,
    _h: f32,
) {
    p.draw(f, at.0, at.1, pitch, 0xcfd8ff, 1.0, gain, t);
}
