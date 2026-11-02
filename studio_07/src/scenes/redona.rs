// REDONA: offer four, in Redona Broadcasting's own colours (Bergamo's civic
// red and yellow). Three shop phones film; their clips fly to the Redona TV
// (yellow, octagon screen, the moustache); the video plays, is checked, and
// goes out.
use crate::gfx::{ease_out, hash, Frame, H, W};
use crate::timeline::{smooth, Now};

const RED: u32 = 0xb4121f;
const RED_DARK: u32 = 0x8f0d18;
const YELLOW: u32 = 0xffc61a;
const INK: u32 = 0x1c0a0c;
const CREAM: u32 = 0xfff4dc;

const TV: (f32, f32) = (900.0, 360.0);
/// When each phone's clip leaves for the TV (seconds into the section).
const SENDS: [f32; 3] = [2.0, 3.4, 4.8];
const PLAY_FROM: f32 = 6.2;
const CHECKED: f32 = 10.0;

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.clear(RED);
    // A slow sunburst behind the TV.
    let spin = t * 0.08;
    for y in (0..H as i32).step_by(2) {
        for x in (0..W as i32).step_by(2) {
            let (dx, dy) = (x as f32 - TV.0, y as f32 - TV.1);
            let ang = dy.atan2(dx) + spin;
            if (ang * 18.0 / std::f32::consts::TAU).rem_euclid(1.0) < 0.5 {
                f.rect(x, y, x + 2, y + 2, RED_DARK, 0.55 * fade);
            }
        }
    }

    // Phones.
    for (i, send) in SENDS.iter().enumerate() {
        let bob = (t * 1.6 + i as f32 * 1.3).sin() * 6.0;
        let (px, py) = (90.0 + i as f32 * 150.0, 250.0 + i as f32 * 40.0 + bob);
        phone(f, px, py, i, n.local, t, fade);
        // The clip flies along an arc to the TV.
        let k = (n.local - send) / 1.2;
        if (0.0..1.0).contains(&k) {
            let e = ease_out(k);
            let (sx, sy) = (px + 55.0, py + 100.0);
            let (ex, ey) = (TV.0, TV.1);
            let x = sx + (ex - sx) * e;
            let y = sy + (ey - sy) * e - 140.0 * (std::f32::consts::PI * e).sin();
            let s = 1.0 - 0.5 * e;
            f.rect(
                (x - 26.0 * s) as i32,
                (y - 40.0 * s) as i32,
                (x + 26.0 * s) as i32,
                (y + 40.0 * s) as i32,
                CREAM,
                fade,
            );
            f.rect(
                (x - 22.0 * s) as i32,
                (y - 36.0 * s) as i32,
                (x + 22.0 * s) as i32,
                (y + 36.0 * s) as i32,
                footage(i),
                fade,
            );
            f.glow(x, y, 50.0, YELLOW, 0.35 * fade);
        }
    }

    tv(f, n.local, t, fade);

    // The wordmark.
    let a = smooth((n.local - 0.6) / 0.8) * fade;
    f.text_centered(TV.0 as i32 + 4, 584, "REDONA", 6, INK, 0.5 * a);
    f.text_centered(TV.0 as i32, 580, "REDONA", 6, YELLOW, a);
    let (rx0, rx1) = (TV.0 as i32 - 150, TV.0 as i32 + 150);
    f.rect(rx0, 640, rx1, 668, CREAM, a);
    f.text_centered(TV.0 as i32, 646, "BROADCASTING", 2, RED, a);
    f.text(90, 640, "BERGAMO", 3, CREAM, a);
    1.0
}

fn footage(i: usize) -> u32 {
    [0xd9893a, 0xc46a2a, 0xe6a85a][i]
}

/// A phone filming a shop counter: warm light, a loaf, the REC dot.
#[allow(clippy::too_many_arguments)]
fn phone(f: &mut Frame, x: f32, y: f32, i: usize, local: f32, t: f32, a: f32) {
    let (w, h) = (110.0, 210.0);
    f.rect(x as i32, y as i32, (x + w) as i32, (y + h) as i32, INK, a);
    let (sx0, sy0, sx1, sy1) = (x + 7.0, y + 18.0, x + w - 7.0, y + h - 18.0);
    // Footage: a warm gradient that pans, and a loaf on the counter.
    for yy in sy0 as i32..sy1 as i32 {
        let k = (yy as f32 - sy0) / (sy1 - sy0);
        let c = crate::gfx::mix(footage(i), 0x3a1a0c, k);
        f.rect(sx0 as i32, yy, sx1 as i32, yy + 1, c, a);
    }
    let pan = (t * 0.6 + i as f32).sin() * 12.0;
    let (lx, ly) = ((sx0 + sx1) * 0.5 + pan, sy1 - 52.0);
    for dy in -14..14 {
        let half = 34.0 * (1.0 - (dy as f32 / 14.0).powi(2)).max(0.0).sqrt();
        f.rect(
            (lx - half) as i32,
            (ly + dy as f32) as i32,
            (lx + half) as i32,
            (ly + dy as f32) as i32 + 1,
            0x8a4a1c,
            a,
        );
    }
    for s in 0..3 {
        let sx = lx - 16.0 + s as f32 * 14.0;
        f.line(sx, ly - 8.0, sx + 8.0, ly + 2.0, 2.0, 0xf2c48a, a, 1.0);
    }
    f.rect(
        sx0 as i32,
        (sy1 - 32.0) as i32,
        sx1 as i32,
        sy1 as i32,
        0x2a120a,
        0.8 * a,
    );
    let sent = local > SENDS[i];
    if !sent && (t * 2.0).fract() < 0.6 {
        f.disc(sx0 + 12.0, sy0 + 12.0, 5.0, 0xff2d3d, a);
    }
    if sent {
        f.text(sx0 as i32 + 6, sy0 as i32 + 6, "SENT", 2, CREAM, a);
    }
    f.disc(x + w * 0.5, y + h - 9.0, 4.0, 0x3a2a2c, a);
}

/// The Redona TV: yellow body, octagon screen, the moustache.
fn tv(f: &mut Frame, local: f32, t: f32, a: f32) {
    let (cx, cy) = TV;
    let (w, h) = (330.0, 250.0);
    // Antennas.
    f.line(
        cx - 20.0,
        cy - h * 0.5,
        cx - 80.0,
        cy - h * 0.5 - 70.0,
        4.0,
        INK,
        a,
        1.0,
    );
    f.line(
        cx + 20.0,
        cy - h * 0.5,
        cx + 70.0,
        cy - h * 0.5 - 80.0,
        4.0,
        INK,
        a,
        1.0,
    );
    f.disc(cx - 80.0, cy - h * 0.5 - 70.0, 8.0, YELLOW, a);
    f.disc(cx + 70.0, cy - h * 0.5 - 80.0, 8.0, YELLOW, a);
    // Body with a dark outline.
    f.rect(
        (cx - w * 0.5 - 6.0) as i32,
        (cy - h * 0.5 - 6.0) as i32,
        (cx + w * 0.5 + 6.0) as i32,
        (cy + h * 0.5 + 6.0) as i32,
        INK,
        a,
    );
    f.rect(
        (cx - w * 0.5) as i32,
        (cy - h * 0.5) as i32,
        (cx + w * 0.5) as i32,
        (cy + h * 0.5) as i32,
        YELLOW,
        a,
    );
    // Octagon screen.
    let r = 92.0;
    let (scx, scy) = (cx, cy - 18.0);
    let oct = |x: f32, y: f32, r: f32| -> bool {
        let (dx, dy) = ((x - scx).abs(), (y - scy).abs());
        dx < r && dy < r && dx + dy < r * 1.42
    };
    let arrived = SENDS.iter().filter(|s| local > **s + 1.2).count();
    let playing = smooth((local - PLAY_FROM) / 0.4);
    for y in (scy - r - 8.0) as i32..(scy + r + 8.0) as i32 {
        for x in (scx - r - 8.0) as i32..(scx + r + 8.0) as i32 {
            let (xf, yf) = (x as f32, y as f32);
            if oct(xf, yf, r + 7.0) && !oct(xf, yf, r) {
                f.blend(x, y, INK, a);
            } else if oct(xf, yf, r) {
                // Static until the clips arrive, then the edited video.
                let noise = hash(x + (t * 30.0) as i32 * 977, y);
                let base = crate::gfx::mix(0x2a2a2a, 0xd0d0d0, noise);
                let band = ((yf - scy) / (2.0 * r) + 0.5 + t * 0.25).fract();
                let video = crate::gfx::mix(
                    footage((band * 3.0) as usize % 3),
                    0x3a1a0c,
                    (yf - scy + r) / (2.0 * r) * 0.6,
                );
                let c = crate::gfx::mix(base, video, playing.max(arrived as f32 * 0.15));
                f.blend(x, y, c, a);
            }
        }
    }
    if playing > 0.0 {
        // Play triangle, and a progress bar along the bottom of the screen.
        for dy in -18i32..18 {
            let half = (18 - dy.abs()) as f32 * 1.1;
            f.rect(
                (scx - 10.0) as i32,
                (scy + dy as f32) as i32,
                (scx - 10.0 + half) as i32,
                (scy + dy as f32) as i32 + 1,
                CREAM,
                0.85 * playing * a * (1.0 - smooth((local - CHECKED) / 0.3)),
            );
        }
        let prog = ((local - PLAY_FROM) / (CHECKED - PLAY_FROM)).clamp(0.0, 1.0);
        f.rect(
            (scx - 60.0) as i32,
            (scy + 60.0) as i32,
            (scx + 60.0) as i32,
            (scy + 66.0) as i32,
            INK,
            0.6 * a,
        );
        f.rect(
            (scx - 60.0) as i32,
            (scy + 60.0) as i32,
            (scx - 60.0 + 120.0 * prog) as i32,
            (scy + 66.0) as i32,
            YELLOW,
            a,
        );
    }
    // The check stamp.
    let k = ease_out((local - CHECKED) / 0.35);
    if k > 0.0 {
        let s = 1.0 + 0.6 * (1.0 - k);
        f.disc(scx, scy, 44.0 * s, CREAM, a * k);
        f.ring(scx, scy, 44.0 * s, 4.0, INK, a * k);
        f.line(
            scx - 20.0 * s,
            scy,
            scx - 5.0 * s,
            scy + 16.0 * s,
            8.0,
            RED,
            a * k,
            1.0,
        );
        f.line(
            scx - 5.0 * s,
            scy + 16.0 * s,
            scx + 22.0 * s,
            scy - 18.0 * s,
            8.0,
            RED,
            a * k,
            1.0,
        );
    }
    // The moustache under the screen.
    let my = cy + h * 0.5 - 28.0;
    for side in [-1.0f32, 1.0] {
        for i in 0..24 {
            let u = i as f32 / 23.0;
            let x = cx + side * (6.0 + u * 48.0);
            let y = my + 6.0 * (u * 3.0).sin() - 10.0 * u * u;
            f.disc(x, y, 9.0 * (1.0 - u * 0.75), INK, a);
        }
    }
    // Knobs.
    f.disc(cx + w * 0.5 - 26.0, cy - 50.0, 9.0, INK, a);
    f.disc(cx + w * 0.5 - 26.0, cy - 20.0, 9.0, INK, a);
}
