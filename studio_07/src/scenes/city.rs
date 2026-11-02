// CITY: Bergamo's upper town at night -- the Venetian walls, the Campanone,
// the Duomo, Santa Maria Maggiore, the Rocca, San Vigilio on its hill, the
// funicular -- above the lights of the lower town. SolverForge's mark rises
// over it.
use demofx::logo::{self, LogoState};

use super::PAPER;
use crate::gfx::{hash, mix, Frame, H, W};
use crate::timeline::{smooth, Now};

const INK: u32 = 0x07060d;
const WALL: u32 = 0x2a2338;
const WINDOW: u32 = 0xffcf6b;
const HORIZON: f32 = 600.0;

/// Top of the ground at x: the upper town's plateau and San Vigilio.
fn hill(x: f32) -> f32 {
    let u = ((x - 650.0) / 470.0).clamp(-1.0, 1.0);
    let plateau = 125.0 * (1.0 - u.powi(4)).max(0.0);
    let vigilio = 205.0 * (-((x - 190.0) / 115.0).powi(2)).exp();
    HORIZON - plateau.max(vigilio)
}

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(1.0, 0.9);
    f.gradient(&[0x02030c, 0x070b22, 0x150f33, 0x2b1638, 0x14091a]);

    // Stars and the moon.
    for i in 0..280 {
        let (x, y) = (hash(i, 1) * W as f32, hash(i, 2) * 520.0);
        let tw = 0.5 + 0.5 * (t * (1.0 + hash(i, 3) * 3.0) + hash(i, 4) * 9.0).sin();
        let a = (0.25 + 0.6 * hash(i, 5)) * tw * fade;
        f.blend(x as i32, y as i32, 0xdfe8ff, a);
    }
    f.glow(1090.0, 170.0, 70.0, 0x6a6a90, 0.25 * fade);
    f.disc(1090.0, 170.0, 22.0, 0xf2eedd, 0.95 * fade);
    f.disc(1080.0, 164.0, 20.0, 0x0b0c24, 0.92 * fade);

    // Ground silhouette.
    for x in 0..W as i32 {
        let top = hill(x as f32) as i32;
        f.rect(x, top, x + 1, H as i32, INK, fade);
    }

    // The Venetian walls: a lit band of stone with bastions along the slope.
    for x in 300..1010 {
        let xf = x as f32;
        let top = hill(xf) + 6.0;
        let bastion = ((xf - 300.0) / 118.0).fract();
        let depth = if bastion < 0.22 { 36.0 } else { 28.0 };
        let shade = 0.55 + 0.45 * hash(x / 3, 7);
        f.rect(
            x,
            top as i32,
            x + 1,
            (top + depth) as i32,
            mix(INK, WALL, shade),
            fade,
        );
        f.blend(x, top as i32, 0x6d6390, 0.55 * fade);
        if hash(x / 6, 8) > 0.86 && hash(x, 9) > 0.5 {
            f.blend(x, (top + depth * 0.6) as i32, WINDOW, 0.7 * fade);
        }
    }

    // Roofs of the upper town.
    let mut x = 320.0;
    let mut k = 0;
    while x < 1000.0 {
        let w = 12.0 + 22.0 * hash(k, 11);
        let h = 10.0 + 34.0 * hash(k, 12);
        let base = hill(x + w * 0.5) + 8.0;
        building(f, x, base, w, h, k, t, fade);
        x += w + 1.0;
        k += 1;
    }

    // Landmarks.
    tower(f, 585.0, 18.0, 118.0, 21, t, fade); // Torre del Gombito
    campanone(f, 640.0, t, fade);
    tower(f, 722.0, 15.0, 112.0, 23, t, fade); // Santa Maria Maggiore
    spire(f, 722.0, hill(722.0) - 112.0, 22.0, 26.0, fade);
    dome(f, 690.0, 16.0, 46.0, fade); // Cappella Colleoni
    dome(f, 778.0, 30.0, 62.0, fade); // Duomo
    rocca(f, 905.0, fade);
    vigilio(f, 190.0, fade);
    funicular(f, t, fade);

    // Houses on the slopes under the walls, and the lower town's lights.
    for i in 0..500 {
        let x = 120.0 + hash(i, 41) * 1060.0;
        let y = hill(x) + 44.0 + hash(i, 42) * (HORIZON - hill(x) - 40.0).max(0.0);
        if y > HORIZON {
            continue;
        }
        let tw = 0.8 + 0.2 * (t * 1.5 + hash(i, 43) * 20.0).sin();
        f.rect(
            x as i32,
            y as i32,
            x as i32 + 2,
            y as i32 + 2,
            WINDOW,
            0.55 * hash(i, 44) * tw * fade,
        );
    }
    for i in 0..1800 {
        let y = HORIZON + 4.0 + hash(i, 31).powf(0.8) * 116.0;
        let x = hash(i, 32) * W as f32;
        let warm = if hash(i, 33) > 0.3 {
            0xffb35c
        } else {
            0xe8f0ff
        };
        let tw = 0.75 + 0.25 * (t * 2.0 + hash(i, 34) * 30.0).sin();
        let a = (0.45 + 0.55 * hash(i, 35)) * tw * fade;
        let size = if hash(i, 36) > 0.8 { 2 } else { 1 };
        f.rect(
            x as i32,
            y as i32,
            x as i32 + size,
            y as i32 + size,
            warm,
            a,
        );
    }
    // A haze of light over the lower town.
    for y in (HORIZON as i32 - 30)..H as i32 {
        let k = 1.0 - ((y as f32 - HORIZON + 30.0) / 150.0 - 0.4).abs() / 0.6;
        if k > 0.0 {
            for x in (0..W as i32).step_by(1) {
                f.add(x, y, 0x3a1c08, 0.25 * k * fade);
            }
        }
    }

    // SolverForge's mark rises over the town while the cards speak of it.
    let rise = smooth((n.local - 0.8) / 3.0);
    let gone = smooth((n.local - 14.0) / 1.6);
    let gain = rise * (1.0 - gone) * fade;
    if gain > 0.0 {
        let cy = 196.0 + 30.0 * (1.0 - rise);
        f.glow(640.0, cy, 170.0, 0x10b981, 0.18 * gain);
        logo::draw(
            f,
            &LogoState::sized(640.0, cy, 66.0, smooth((n.local - 0.8) / 3.4), gain),
        );
        f.text_centered(640, cy as i32 + 96, "SOLVERFORGE", 2, PAPER, 0.85 * gain);
    }
    1.0 - (rise * (1.0 - gone))
}

#[allow(clippy::too_many_arguments)]
fn building(f: &mut Frame, x: f32, base: f32, w: f32, h: f32, k: i32, t: f32, fade: f32) {
    let (x0, x1) = (x as i32, (x + w) as i32);
    f.rect(x0, (base - h) as i32, x1, base as i32 + 4, INK, fade);
    // A pitched roof on about half of them.
    if hash(k, 13) > 0.5 {
        for dx in 0..(w as i32) {
            let r = (w * 0.5 - (dx as f32 - w * 0.5).abs()) * 0.45;
            f.rect(
                x0 + dx,
                (base - h - r) as i32,
                x0 + dx + 1,
                (base - h) as i32,
                INK,
                fade,
            );
        }
    }
    windows(f, x0, x1, (base - h) as i32 + 4, base as i32, k, t, fade);
}

#[allow(clippy::too_many_arguments)]
fn windows(f: &mut Frame, x0: i32, x1: i32, y0: i32, y1: i32, k: i32, t: f32, fade: f32) {
    let mut y = y0;
    while y + 4 < y1 {
        let mut x = x0 + 3;
        while x + 2 < x1 - 2 {
            let h = hash(k * 131 + x, y);
            if h > 0.72 {
                // A few lights go on and off.
                let on = if h > 0.97 {
                    ((t * 0.4 + h * 10.0).sin() > 0.0) as i32 as f32
                } else {
                    1.0
                };
                f.rect(x, y, x + 2, y + 3, WINDOW, 0.8 * on * fade);
            }
            x += 5;
        }
        y += 7;
    }
}

fn tower(f: &mut Frame, cx: f32, w: f32, h: f32, k: i32, t: f32, fade: f32) {
    let base = hill(cx) + 8.0;
    let (x0, x1) = ((cx - w * 0.5) as i32, (cx + w * 0.5) as i32);
    f.rect(x0, (base - h) as i32, x1, base as i32, INK, fade);
    windows(
        f,
        x0,
        x1,
        (base - h) as i32 + 10,
        (base - h * 0.5) as i32,
        k,
        t,
        fade * 0.6,
    );
}

fn spire(f: &mut Frame, cx: f32, top: f32, w: f32, h: f32, fade: f32) {
    for dy in 0..h as i32 {
        let half = w * 0.5 * (dy as f32 / h);
        f.rect(
            (cx - half) as i32,
            (top - h) as i32 + dy,
            (cx + half) as i32 + 1,
            (top - h) as i32 + dy + 1,
            INK,
            fade,
        );
    }
}

/// The Torre Civica: a square tower with a lit belfry and a small cap.
fn campanone(f: &mut Frame, cx: f32, t: f32, fade: f32) {
    let base = hill(cx) + 8.0;
    let top = base - 150.0;
    f.rect(
        (cx - 13.0) as i32,
        top as i32,
        (cx + 13.0) as i32,
        base as i32,
        INK,
        fade,
    );
    // Belfry openings, lit warm.
    for i in 0..2 {
        let x = cx - 9.0 + i as f32 * 11.0;
        f.rect(
            x as i32,
            top as i32 + 8,
            x as i32 + 6,
            top as i32 + 22,
            WINDOW,
            0.75 * fade,
        );
    }
    // The clock face.
    f.disc(cx, top + 40.0, 6.0, 0xffe7a8, 0.8 * fade);
    f.glow(cx, top + 40.0, 26.0, WINDOW, 0.25 * fade);
    // Cap and lantern.
    spire(f, cx, top, 30.0, 12.0, fade);
    f.rect(
        cx as i32 - 3,
        (top - 22.0) as i32,
        cx as i32 + 3,
        (top - 12.0) as i32,
        INK,
        fade,
    );
    // The belfry breathes.
    let pulse = 0.5 + 0.5 * (t * 2.0).sin();
    f.glow(cx, top + 15.0, 18.0, WINDOW, 0.2 * pulse * fade);
}

fn dome(f: &mut Frame, cx: f32, r: f32, drum: f32, fade: f32) {
    let base = hill(cx) + 8.0;
    let top = base - drum;
    f.rect(
        (cx - r * 0.85) as i32,
        top as i32,
        (cx + r * 0.85) as i32,
        base as i32,
        INK,
        fade,
    );
    for dy in 0..r as i32 {
        let half = (r * r - (dy as f32).powi(2)).max(0.0).sqrt();
        let y = top as i32 - dy;
        f.rect(
            (cx - half) as i32,
            y,
            (cx + half) as i32 + 1,
            y + 1,
            INK,
            fade,
        );
    }
    f.rect(
        cx as i32 - 3,
        (top - r - 12.0) as i32,
        cx as i32 + 3,
        (top - r + 1.0) as i32,
        INK,
        fade,
    );
    f.rect(
        cx as i32 - 1,
        (top - r - 22.0) as i32,
        cx as i32 + 1,
        (top - r - 10.0) as i32,
        INK,
        fade,
    );
    f.rect(
        cx as i32 - 5,
        (top - r - 18.0) as i32,
        cx as i32 + 5,
        (top - r - 16.0) as i32,
        INK,
        fade,
    );
}

/// La Rocca: a round keep with crenellations.
fn rocca(f: &mut Frame, cx: f32, fade: f32) {
    let base = hill(cx) + 8.0;
    let top = base - 52.0;
    f.rect(
        (cx - 20.0) as i32,
        top as i32,
        (cx + 20.0) as i32,
        base as i32,
        INK,
        fade,
    );
    for i in 0..5 {
        let x = cx - 20.0 + i as f32 * 9.0;
        f.rect(
            x as i32,
            top as i32 - 6,
            x as i32 + 5,
            top as i32,
            INK,
            fade,
        );
    }
    f.rect(
        cx as i32 - 1,
        top as i32 + 12,
        cx as i32 + 2,
        top as i32 + 18,
        WINDOW,
        0.7 * fade,
    );
}

/// San Vigilio's castle on the higher hill to the west.
fn vigilio(f: &mut Frame, cx: f32, fade: f32) {
    let base = hill(cx) + 6.0;
    f.rect(
        (cx - 34.0) as i32,
        (base - 18.0) as i32,
        (cx + 34.0) as i32,
        base as i32,
        INK,
        fade,
    );
    for dx in [-34.0, 26.0] {
        let x = cx + dx;
        f.rect(
            x as i32,
            (base - 34.0) as i32,
            x as i32 + 9,
            base as i32,
            INK,
            fade,
        );
    }
    for i in 0..7 {
        let x = cx - 30.0 + i as f32 * 9.0;
        f.rect(
            x as i32,
            (base - 23.0) as i32,
            x as i32 + 5,
            (base - 18.0) as i32,
            INK,
            fade,
        );
    }
}

/// The funicular from the lower town to the walls, its car climbing.
fn funicular(f: &mut Frame, t: f32, fade: f32) {
    let (x0, y0) = (835.0, HORIZON + 8.0);
    let (x1, y1) = (875.0, hill(875.0) + 10.0);
    f.line(x0, y0, x1, y1, 1.2, 0x5a4f70, 0.9 * fade, 1.0);
    let k = (t * 0.07).fract();
    let k = if k < 0.5 { k * 2.0 } else { 2.0 - k * 2.0 };
    let (cx, cy) = (x0 + (x1 - x0) * k, y0 + (y1 - y0) * k);
    f.rect(
        cx as i32 - 4,
        cy as i32 - 4,
        cx as i32 + 4,
        cy as i32 + 2,
        0xffe7a8,
        0.9 * fade,
    );
    f.glow(cx, cy, 14.0, WINDOW, 0.3 * fade);
}
