// READING: three books on a desk under a lamp -- Dune, A Game of Thrones,
// Fleet Tactics -- then Dune opens, and the page gives way to a desert
// under two moons, with something large moving under the sand.
use crate::gfx::{ease_out, hash, mix, Frame, H, W};
use crate::timeline::{smooth, Now};

const OPEN_AT: f32 = 3.4;
const DESERT_AT: f32 = 6.2;

/// (title, author, cover colour, width)
const BOOKS: [(&str, &str, u32, i32); 3] = [
    ("FLEET TACTICS", "HUGHES", 0x1f2a3a, 700),
    ("A GAME OF THRONES", "MARTIN", 0x3a1f1f, 760),
    ("DUNE", "HERBERT", 0x4a3418, 660),
];

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    let l = n.local;
    if l < DESERT_AT + 0.8 {
        desk(f, l, t, fade);
    }
    let d = smooth((l - DESERT_AT) / 0.8);
    if d > 0.0 {
        desert(f, l - DESERT_AT, t, fade * d);
    }
    1.0
}

fn desk(f: &mut Frame, l: f32, t: f32, a: f32) {
    f.gradient(&[0x0b0806, 0x120d09, 0x0a0705, 0x050403]);
    // The lamp's pool of light on the desk.
    f.glow(540.0, 1150.0, 700.0, 0x6a4a22, 0.5 * a);
    f.rect(0, 1380, W as i32, H as i32, 0x1a120c, a);
    f.rect(0, 1376, W as i32, 1382, 0x2a1d13, a);
    // The lamp: base, arm, shade.
    f.disc(900.0, 1370.0, 50.0, 0x0e0a08, a);
    f.line(900.0, 1360.0, 840.0, 760.0, 8.0, 0x0e0a08, a, 1.0);
    f.line(840.0, 760.0, 700.0, 640.0, 8.0, 0x0e0a08, a, 1.0);
    f.poly_fill(
        &[
            (620.0, 600.0),
            (760.0, 600.0),
            (820.0, 700.0),
            (560.0, 700.0),
        ],
        0x1a1410,
        a,
    );
    f.glow(690.0, 710.0, 120.0, 0xffd89a, 0.5 * a);

    // The stack. The top book (Dune) lifts and opens.
    let open = ease_out((l - OPEN_AT) / 1.6);
    let mut y = 1376;
    for (i, (title, author, col, w)) in BOOKS.iter().enumerate() {
        let h = 92;
        let x0 = 540 - w / 2 + (i as i32 % 2) * 24 - 12;
        if i == 2 && open > 0.0 {
            let zoom = smooth((l - OPEN_AT - 1.0) / 1.8);
            open_book(f, open, zoom, t, a);
            continue;
        }
        y -= h;
        f.rect(x0, y, x0 + w, y + h, *col, a);
        f.rect(
            x0 + 10,
            y + 8,
            x0 + w - 10,
            y + 10,
            mix(*col, 0xffffff, 0.2),
            a,
        );
        f.rect(
            x0 + 10,
            y + h - 10,
            x0 + w - 10,
            y + h - 8,
            mix(*col, 0xffffff, 0.2),
            a,
        );
        f.text(x0 + 30, y + 24, title, 3, 0xe8dcc0, a);
        f.text(
            x0 + w - 30 - author.len() as i32 * 20,
            y + 54,
            author,
            2,
            0xb8a888,
            a,
        );
    }
}

/// Dune, opening: two pages spreading from the spine, then filling the
/// frame as we lean in.
fn open_book(f: &mut Frame, open: f32, zoom: f32, t: f32, a: f32) {
    let s = 1.0 + 3.0 * zoom;
    let (cx, cy) = (540.0, 1100.0 - 300.0 * zoom);
    let (pw, ph) = (300.0 * s * open.max(0.05), 420.0 * s);
    // Cover behind, pages on top.
    f.rect(
        (cx - pw - 10.0) as i32,
        (cy - ph / 2.0 - 10.0) as i32,
        (cx + pw + 10.0) as i32,
        (cy + ph / 2.0 + 10.0) as i32,
        0x4a3418,
        a,
    );
    for side in [-1.0f32, 1.0] {
        let (x0, x1) = if side < 0.0 {
            (cx - pw, cx)
        } else {
            (cx, cx + pw)
        };
        f.rect(
            x0 as i32,
            (cy - ph / 2.0) as i32,
            x1 as i32,
            (cy + ph / 2.0) as i32,
            0xe9dfc6,
            a,
        );
        // Lines of text, as grey bars.
        for k in 0..18 {
            let y = cy - ph / 2.0 + ph * (0.1 + k as f32 * 0.045);
            let w = (x1 - x0) * (0.7 + 0.2 * hash(k + side as i32 * 40, 2));
            let lx = x0 + (x1 - x0) * 0.1;
            f.rect(
                lx as i32,
                y as i32,
                (lx + w * 0.85) as i32,
                (y + 3.0 * s) as i32,
                0x9a8f7a,
                a,
            );
        }
    }
    f.line(
        cx,
        cy - ph / 2.0,
        cx,
        cy + ph / 2.0,
        2.0 * s,
        0xb8ab90,
        a,
        1.0,
    );
    // Sand starts to blow off the page.
    for i in 0..60 {
        let ph2 = (t * 0.6 + hash(i, 1)).fract();
        let x = cx - pw + hash(i, 2) * pw * 2.0 + ph2 * 200.0;
        let y = cy + ph * 0.2 - ph2 * 400.0;
        f.disc(x, y, 2.0, 0xe8b86a, (1.0 - ph2) * zoom * a);
    }
}

/// Dunes at dusk under two moons; a ridge of sand rises and runs across
/// the near dune.
fn desert(f: &mut Frame, l: f32, t: f32, a: f32) {
    f.gradient(&[0x1a1030, 0x4a2238, 0xb0563a, 0xe8a060]);
    f.glow(320.0, 560.0, 160.0, 0xfff0d8, 0.3 * a);
    f.disc(320.0, 560.0, 70.0, 0xfbefd8, a);
    f.disc(560.0, 480.0, 28.0, 0xf3e2c8, 0.9 * a);
    let layers = [
        (1000.0, 0x8a4a2c, 0.003, 60.0),
        (1180.0, 0x6a3420, 0.0045, 80.0),
        (1400.0, 0x4a2216, 0.006, 110.0),
    ];
    for (k, &(base, col, freq, amp)) in layers.iter().enumerate() {
        let drift = t * (6.0 + k as f32 * 10.0);
        for x in 0..W as i32 {
            let xf = x as f32 + drift;
            let mut h = (xf * freq).sin() * amp + (xf * freq * 2.3 + k as f32).sin() * amp * 0.4;
            // The thing under the sand: a moving bulge on the nearest dune.
            if k == 2 {
                let at = -200.0 + l * 190.0;
                h += 70.0 * (-((x as f32 - at) / 90.0).powi(2)).exp();
            }
            let top = base - h;
            f.rect(x, top as i32, x + 1, H as i32, col, a);
            // Sunlit crest.
            f.rect(
                x,
                top as i32,
                x + 1,
                top as i32 + 3,
                mix(col, 0xffd0a0, 0.5),
                a,
            );
        }
    }
    // Spray from the moving bulge.
    let at = -200.0 + l * 190.0;
    for i in 0..60 {
        let ph = (t * 1.4 + hash(i, 3)).fract();
        let x = at - 40.0 + hash(i, 4) * 80.0 - ph * 120.0;
        let y = 1300.0 - ph * 160.0 * hash(i, 5);
        f.disc(x, y, 2.5, 0xf0c080, (1.0 - ph) * a);
    }
    // Wind-blown sand across the frame.
    for i in 0..200 {
        let x = (hash(i, 6) * W as f32 + t * (200.0 + 300.0 * hash(i, 7))) % W as f32;
        let y = 900.0 + hash(i, 8) * 800.0;
        f.blend(x as i32, y as i32, 0xffe0b0, 0.4 * a);
    }
}
