// GUITAR: Vittorio's guitar, an Epiphone Les Paul Black Beauty, drawn from
// its parts at the real proportions of a Les Paul (measured in inches and
// scaled by the 24.75" scale length): the single-cut black body with its
// pointed horn and cream binding, the raised pickguard, three gold
// humbuckers, the tune-o-matic and stopbar, four gold knobs and the toggle,
// the bound board with block inlays, the open-book headstock with its
// split-diamond inlay and six kidney tuners. The strings ring with the
// notes the soundtrack plays (score.rs). The camera starts on the pickups,
// climbs the neck to the headstock, then pulls back to the whole guitar.
//
// The painted guitar is rasterised once at startup into a mip chain; each
// frame only samples it (and slides the gloss across it), so the camera
// moves at full frame rate. The strings are drawn live on top.
use std::sync::OnceLock;

use crate::gfx::{mix, Frame, H, W};
use crate::score::{strings_at, Note};
use crate::sdf::{circle, cover, ellipse, rbox, smax, smin};
use crate::timeline::{smooth, Now};

pub const NUT_Y: f32 = -720.0;
pub const SCALE_LEN: f32 = 952.0;
pub const BRIDGE_Y: f32 = NUT_Y + SCALE_LEN;
/// Guitar units per inch.
const IN: f32 = SCALE_LEN / 24.75;
/// The fretboard ends at the 22nd fret.
const BOARD_END: f32 = NUT_Y + SCALE_LEN * 0.719_4;
/// The top of the body, where the neck joins it at the 16th fret.
const STOP_Y: f32 = BRIDGE_Y + 1.6 * IN;

const BLACK: u32 = 0x0b0b0e;
const CREAM: u32 = 0xe9dfc4;
const GOLD: u32 = 0xd8b25a;
const GOLD_DARK: u32 = 0x8a6a28;
const BOARD: u32 = 0x17110d;
const PEARL: u32 = 0xe8e6dc;
const FRET: u32 = 0xb8b8b0;
const STRING: u32 = 0xd6d6d0;
const GLOSS: u32 = 0xc8d0dc;

/// Guitar-space point (gx, gy) shown at screen (sx, sy), `s` pixels a unit.
#[derive(Clone, Copy, Debug)]
pub struct Cam {
    pub gx: f32,
    pub gy: f32,
    pub sx: f32,
    pub sy: f32,
    pub s: f32,
}

impl Cam {
    pub fn to_screen(self, p: (f32, f32)) -> (f32, f32) {
        (
            (p.0 - self.gx) * self.s + self.sx,
            (p.1 - self.gy) * self.s + self.sy,
        )
    }
    fn lerp(a: Cam, b: Cam, k: f32) -> Cam {
        let l = |x: f32, y: f32| x + (y - x) * k;
        // Scale interpolates in log space so zooms feel even.
        Cam {
            gx: l(a.gx, b.gx),
            gy: l(a.gy, b.gy),
            sx: l(a.sx, b.sx),
            sy: l(a.sy, b.sy),
            s: (a.s.ln() + (b.s.ln() - a.s.ln()) * k).exp(),
        }
    }
}

/// The whole guitar, fitted to the frame under the story cards.
pub const FULL: Cam = Cam {
    gx: 0.0,
    gy: -230.0,
    sx: 540.0,
    sy: 1110.0,
    s: 0.9,
};

/// The body outline in guitar units, traced from a front photograph of a
/// black Les Paul Custom (Wikimedia Commons, CC0), registered on its nut and
/// bridge and scaled to the 13" x 17.25" body; clockwise from the bass side
/// of the neck joint.
const BODY: [(f32, f32); 256] = [
    (-195.4, 190.1),
    (-190.3, 184.0),
    (-185.1, 177.9),
    (-179.9, 171.8),
    (-174.6, 165.8),
    (-169.7, 159.5),
    (-165.6, 152.7),
    (-162.1, 145.4),
    (-158.9, 138.1),
    (-155.5, 130.9),
    (-152.1, 123.6),
    (-149.5, 116.1),
    (-147.9, 108.2),
    (-147.7, 100.3),
    (-148.4, 92.3),
    (-150.2, 84.5),
    (-152.7, 76.9),
    (-155.7, 69.5),
    (-158.7, 62.1),
    (-161.2, 54.5),
    (-162.9, 46.7),
    (-164.4, 38.8),
    (-166.2, 31.0),
    (-168.6, 23.4),
    (-171.3, 15.9),
    (-173.7, 8.2),
    (-175.6, 0.5),
    (-177.0, -7.4),
    (-178.0, -15.3),
    (-178.3, -23.3),
    (-177.9, -31.3),
    (-176.8, -39.2),
    (-175.1, -47.0),
    (-172.6, -54.6),
    (-169.5, -62.0),
    (-165.8, -69.1),
    (-161.5, -75.8),
    (-156.6, -82.2),
    (-151.3, -88.1),
    (-145.5, -93.7),
    (-139.4, -98.8),
    (-133.0, -103.6),
    (-126.2, -107.9),
    (-119.3, -111.8),
    (-112.4, -116.0),
    (-105.4, -119.8),
    (-97.8, -122.2),
    (-89.9, -123.6),
    (-82.0, -124.3),
    (-74.0, -125.4),
    (-66.1, -126.2),
    (-58.1, -126.4),
    (-50.1, -126.5),
    (-42.1, -126.4),
    (-34.1, -125.8),
    (-26.2, -124.7),
    (-18.3, -123.5),
    (-10.3, -122.7),
    (-2.3, -122.4),
    (5.7, -122.5),
    (13.6, -122.9),
    (21.6, -124.0),
    (29.6, -124.3),
    (36.7, -121.2),
    (39.9, -114.0),
    (40.6, -106.1),
    (40.8, -98.1),
    (41.2, -90.1),
    (41.5, -82.1),
    (42.2, -74.1),
    (43.8, -66.3),
    (46.4, -58.7),
    (50.2, -51.7),
    (54.7, -45.1),
    (60.1, -39.2),
    (66.1, -33.9),
    (72.8, -29.6),
    (80.0, -26.1),
    (87.7, -23.8),
    (95.6, -22.6),
    (103.6, -22.5),
    (111.5, -23.3),
    (119.3, -25.2),
    (126.6, -28.4),
    (133.4, -32.6),
    (139.5, -37.8),
    (144.9, -43.7),
    (150.0, -49.8),
    (155.6, -55.5),
    (162.6, -59.2),
    (170.2, -57.6),
    (175.0, -51.4),
    (177.5, -43.8),
    (178.5, -35.9),
    (178.7, -27.9),
    (178.2, -19.9),
    (177.1, -12.0),
    (175.4, -4.2),
    (173.2, 3.5),
    (171.0, 11.2),
    (169.2, 19.0),
    (167.7, 26.9),
    (166.0, 34.7),
    (163.5, 42.3),
    (160.4, 49.6),
    (157.0, 56.9),
    (153.5, 64.1),
    (150.0, 71.3),
    (146.5, 78.5),
    (142.9, 85.6),
    (139.6, 92.9),
    (137.8, 100.6),
    (138.0, 108.6),
    (140.1, 116.3),
    (143.1, 123.7),
    (147.1, 130.6),
    (152.1, 136.9),
    (157.5, 142.8),
    (162.7, 148.8),
    (167.5, 155.2),
    (172.1, 161.7),
    (177.0, 168.1),
    (182.3, 174.1),
    (187.5, 180.1),
    (192.7, 186.3),
    (197.6, 192.5),
    (202.5, 198.9),
    (207.2, 205.4),
    (211.8, 211.9),
    (216.1, 218.6),
    (220.2, 225.5),
    (224.1, 232.5),
    (227.6, 239.7),
    (230.9, 247.0),
    (233.8, 254.4),
    (236.3, 262.0),
    (238.6, 269.7),
    (240.6, 277.4),
    (242.4, 285.2),
    (244.0, 293.1),
    (245.2, 301.0),
    (246.1, 308.9),
    (246.7, 316.9),
    (247.0, 324.9),
    (246.9, 332.9),
    (246.4, 340.9),
    (245.6, 348.8),
    (244.5, 356.8),
    (243.2, 364.7),
    (241.5, 372.5),
    (239.6, 380.2),
    (237.4, 387.9),
    (234.9, 395.5),
    (232.0, 403.0),
    (228.9, 410.4),
    (225.7, 417.7),
    (222.5, 425.0),
    (219.3, 432.4),
    (216.0, 439.7),
    (212.8, 447.0),
    (209.4, 454.3),
    (205.1, 461.0),
    (199.8, 466.9),
    (193.6, 471.9),
    (187.0, 476.6),
    (180.5, 481.2),
    (174.0, 485.8),
    (167.4, 490.4),
    (160.9, 495.0),
    (154.4, 499.6),
    (147.7, 504.0),
    (140.8, 508.1),
    (133.7, 511.9),
    (126.5, 515.3),
    (119.2, 518.6),
    (111.8, 521.7),
    (104.4, 524.5),
    (96.8, 527.1),
    (89.2, 529.5),
    (81.5, 531.8),
    (73.7, 533.7),
    (65.9, 535.3),
    (57.9, 536.3),
    (50.0, 536.9),
    (42.0, 537.0),
    (34.0, 537.0),
    (26.0, 537.0),
    (18.0, 537.0),
    (10.0, 537.0),
    (2.0, 537.0),
    (-6.0, 537.0),
    (-14.0, 537.0),
    (-22.0, 537.0),
    (-30.0, 537.0),
    (-38.0, 537.0),
    (-46.0, 537.0),
    (-54.0, 537.0),
    (-62.0, 537.0),
    (-70.0, 537.0),
    (-78.0, 536.5),
    (-85.9, 535.5),
    (-93.8, 533.9),
    (-101.5, 531.9),
    (-109.2, 529.6),
    (-116.8, 527.1),
    (-124.3, 524.4),
    (-131.8, 521.4),
    (-139.1, 518.2),
    (-146.3, 514.7),
    (-153.3, 510.9),
    (-160.2, 506.8),
    (-166.8, 502.4),
    (-173.1, 497.4),
    (-178.9, 491.9),
    (-184.3, 486.0),
    (-189.6, 480.0),
    (-194.9, 474.0),
    (-200.2, 468.0),
    (-205.4, 462.0),
    (-210.5, 455.8),
    (-215.5, 449.5),
    (-220.1, 443.0),
    (-224.5, 436.3),
    (-228.5, 429.4),
    (-232.3, 422.3),
    (-235.8, 415.2),
    (-239.1, 407.9),
    (-242.0, 400.4),
    (-244.4, 392.8),
    (-246.6, 385.1),
    (-248.5, 377.3),
    (-250.0, 369.5),
    (-251.2, 361.6),
    (-252.1, 353.6),
    (-252.7, 345.6),
    (-253.0, 337.6),
    (-252.9, 329.6),
    (-252.5, 321.6),
    (-251.8, 313.7),
    (-250.6, 305.8),
    (-249.3, 297.9),
    (-247.7, 290.0),
    (-245.8, 282.3),
    (-243.7, 274.6),
    (-241.2, 267.0),
    (-238.3, 259.5),
    (-235.1, 252.2),
    (-231.7, 244.9),
    (-228.1, 237.8),
    (-224.2, 230.8),
    (-220.2, 223.9),
    (-215.8, 217.2),
    (-211.2, 210.6),
    (-206.5, 204.2),
    (-201.6, 197.8),
    (-196.6, 191.6),
];

/// The pickguard, traced from the same photograph: its bound edge out
/// round the cutaway and back under the bridge pickup, then closed along
/// the pickups.
const GUARD: [(f32, f32); 14] = [
    (45.9, -31.5),
    (66.3, -20.0),
    (83.3, -7.8),
    (90.3, 32.9),
    (107.5, 87.0),
    (128.1, 134.4),
    (152.1, 181.8),
    (162.5, 208.9),
    (166.0, 232.6),
    (155.9, 244.1),
    (118.6, 244.1),
    (71.1, 236.0),
    (68.0, 236.0),
    (68.0, -31.5),
];

/// The open-book headstock in inches: x across, y up from the nut; it
/// runs on under the fretboard below the nut so the two read as one piece.
const HEAD_IN: [(f32, f32); 20] = [
    (-0.8, -1.2),
    (-0.8, -0.3),
    (-0.95, 0.0),
    (-1.74, 6.55),
    (-1.76, 6.85),
    (-1.68, 7.08),
    (-1.48, 7.2),
    (-1.1, 7.2),
    (-0.6, 7.1),
    (-0.25, 6.98),
    (0.25, 6.98),
    (0.6, 7.1),
    (1.1, 7.2),
    (1.48, 7.2),
    (1.68, 7.08),
    (1.76, 6.85),
    (1.74, 6.55),
    (0.95, 0.0),
    (0.8, -0.3),
    (0.8, -1.2),
];

/// A closed polygon smoothed with Catmull-Rom, in guitar units.
fn smooth_outline(
    pts: &[(f32, f32)],
    map: impl Fn((f32, f32)) -> (f32, f32),
    sub: usize,
) -> Vec<(f32, f32)> {
    let p: Vec<(f32, f32)> = pts.iter().map(|&q| map(q)).collect();
    let n = p.len();
    let mut out = Vec::with_capacity(n * sub);
    for i in 0..n {
        let (p0, p1, p2, p3) = (p[(i + n - 1) % n], p[i], p[(i + 1) % n], p[(i + 2) % n]);
        for k in 0..sub {
            let t = k as f32 / sub as f32;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * (2.0 * b
                    + (c - a) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
                    + (3.0 * b - a - 3.0 * c + d) * t3)
            };
            out.push((f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    out
}

struct Outline {
    pts: Vec<(f32, f32)>,
    lo: (f32, f32),
    hi: (f32, f32),
}

impl Outline {
    fn new(pts: Vec<(f32, f32)>) -> Self {
        let lo = pts
            .iter()
            .fold((f32::MAX, f32::MAX), |a, p| (a.0.min(p.0), a.1.min(p.1)));
        let hi = pts
            .iter()
            .fold((f32::MIN, f32::MIN), |a, p| (a.0.max(p.0), a.1.max(p.1)));
        Self { pts, lo, hi }
    }

    /// Signed distance: negative inside.
    fn sdf(&self, p: (f32, f32)) -> f32 {
        let out = (self.lo.0 - p.0)
            .max(p.0 - self.hi.0)
            .max(self.lo.1 - p.1)
            .max(p.1 - self.hi.1);
        if out > 30.0 {
            return out;
        }
        let mut d2 = f32::MAX;
        let mut inside = false;
        let n = self.pts.len();
        for i in 0..n {
            let (a, b) = (self.pts[i], self.pts[(i + 1) % n]);
            let (ex, ey) = (b.0 - a.0, b.1 - a.1);
            let (wx, wy) = (p.0 - a.0, p.1 - a.1);
            let h = ((wx * ex + wy * ey) / (ex * ex + ey * ey)).clamp(0.0, 1.0);
            let (dx, dy) = (wx - ex * h, wy - ey * h);
            d2 = d2.min(dx * dx + dy * dy);
            if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) / (b.1 - a.1) * ex {
                inside = !inside;
            }
        }
        if inside {
            -d2.sqrt()
        } else {
            d2.sqrt()
        }
    }
}

struct Shapes {
    body: Outline,
    guard: Outline,
    head: Outline,
}

fn shapes() -> &'static Shapes {
    static S: OnceLock<Shapes> = OnceLock::new();
    S.get_or_init(|| Shapes {
        body: Outline::new(BODY.to_vec()),
        guard: Outline::new(smooth_outline(&GUARD, |p| p, 6)),
        // Straight sides, a few points round the top: a spline would
        // bulge where the head narrows into the neck. Below the nut it is
        // narrower than the board, which covers it.
        head: Outline::new(
            HEAD_IN
                .iter()
                .map(|&(x, y)| (x * IN, NUT_Y - y * IN))
                .collect(),
        ),
    })
}

/// The neck and its board: 1.69" at the nut, 2.2" at the end of the board.
fn board(p: (f32, f32)) -> f32 {
    let k = ((p.1 - NUT_Y) / (BOARD_END - NUT_Y)).clamp(0.0, 1.0);
    let w = (0.845 + (1.1 - 0.845) * k) * IN;
    let dx = p.0.abs() - w;
    let dy = (p.1 - (NUT_Y + BOARD_END) * 0.5).abs() - (BOARD_END - NUT_Y) * 0.5;
    dx.max(dy)
}

pub fn fret_y(n: f32) -> f32 {
    NUT_Y + SCALE_LEN * (1.0 - 2f32.powf(-n / 12.0))
}

/// String i (0 = low E) at the nut, the saddle, and its tuner post.
pub fn string_points(i: usize) -> ((f32, f32), (f32, f32), (f32, f32)) {
    let k = i as f32 - 2.5;
    let nut = (k * 10.4, NUT_Y);
    let saddle = (k * 16.0, BRIDGE_Y);
    // Low E, A, D on the bass side and high E, B, G on the treble side,
    // each from the nut outward: the outer strings take the nearest posts,
    // so no string crosses another. Posts sit 0.42" inside the edge.
    let side = if i < 3 { -1.0 } else { 1.0 };
    let row = if i < 3 { i } else { 5 - i };
    let up = 2.0 + row as f32 * 1.75;
    let edge = 0.95 + (1.74 - 0.95) * up / 6.5;
    let post = (side * (edge - 0.42) * IN, NUT_Y - up * IN);
    (nut, saddle, post)
}

/// Pickup centres: the neck pickup just past the board, the bridge pickup
/// just ahead of the bridge, the middle one between.
const PICKUPS: [f32; 3] = [
    BOARD_END + 1.15 * IN,
    (BOARD_END + 1.15 * IN + BRIDGE_Y - 1.1 * IN) * 0.5,
    BRIDGE_Y - 1.1 * IN,
];
/// Volume and tone knobs on the treble lower bout, where the photograph
/// has them.
const KNOBS: [(f32, f32); 4] = [
    (182.0, 277.0),
    (106.0, 314.0),
    (175.0, 371.0),
    (104.0, 410.0),
];
const INLAYS: [f32; 9] = [3.0, 5.0, 7.0, 9.0, 12.0, 15.0, 17.0, 19.0, 21.0];

/// The colour of the guitar at guitar-space p, its coverage, and how much
/// of the moving gloss it takes. `px` is one texel in guitar units.
fn shade(p: (f32, f32), px: f32) -> (u32, f32, f32) {
    let mut col = 0u32;
    let mut a = 0.0f32;
    let mut gloss = 0.0f32;
    let mut paint = |c: u32, k: f32, g: f32| {
        if k > 0.0 {
            col = if a <= 0.0 { c } else { mix(col, c, k) };
            a = a.max(k);
            gloss = gloss * (1.0 - k) + g * k;
        }
    };
    let s = shapes();

    // Body: a cream binding, then gloss black with a lit rim along the
    // upper-left edge.
    let d = s.body.sdf(p);
    if d < px {
        paint(CREAM, cover(d, px), 0.0);
        let inner = d + 6.0;
        if inner < px {
            let (gx, gy) = (
                s.body.sdf((p.0 + 2.0, p.1)) - d,
                s.body.sdf((p.0, p.1 + 2.0)) - d,
            );
            let nl = (gx * -0.6 + gy * -0.8).max(0.0) * 0.5;
            let rim = nl * (inner / 18.0).exp();
            paint(
                mix(BLACK, GLOSS, 0.5 * rim),
                cover(inner, px),
                0.16 * smooth(-inner / 40.0),
            );
        }
    }
    // Pickguard: black with a thin cream edge, raised (a shadow below).
    let shadow = s.guard.sdf((p.0 - 3.0, p.1 - 4.0));
    if shadow < px {
        paint(0x000000, 0.6 * cover(shadow, px), 0.0);
    }
    let dg = s.guard.sdf(p);
    if dg < px {
        paint(CREAM, cover(dg, px), 0.0);
        if dg + 2.5 < px {
            paint(0x101013, cover(dg + 2.5, px), 0.1);
        }
    }
    // Headstock first: the neck and board are painted over its lower end.
    let dh = s.head.sdf(p);
    if dh < px + 4.0 {
        paint(CREAM, cover(dh - 4.0, px), 0.0);
        if dh < px {
            paint(BLACK, cover(dh, px), 0.12);
            let (dx, dy) = (p.0.abs(), p.1 - (NUT_Y - 5.1 * IN));
            let diamond = dx * 1.55 + dy.abs() - 0.95 * IN;
            if diamond < px && dx > 2.5 && (dy + 0.25 * IN).abs() > 2.0 {
                paint(PEARL, cover(diamond, px), 0.0);
            }
        }
    }
    // Neck, board binding, board, frets, inlays.
    let dn = board(p);
    if dn < px + 3.0 {
        paint(CREAM, cover(dn - 3.0, px), 0.0);
        if dn < px {
            paint(BOARD, cover(dn, px), 0.0);
            if p.1 > NUT_Y && p.1 < BOARD_END {
                for n in 1..=22 {
                    let fy = fret_y(n as f32);
                    if (p.1 - fy).abs() < 2.2 + px {
                        paint(
                            mix(FRET, 0xffffff, 0.2),
                            cover((p.1 - fy).abs() - 1.6, px),
                            0.0,
                        );
                    }
                }
                for n in INLAYS {
                    let (y0, y1) = (fret_y(n - 1.0), fret_y(n));
                    let k = (((y0 + y1) * 0.5 - NUT_Y) / (BOARD_END - NUT_Y)).clamp(0.0, 1.0);
                    let w = (0.845 + 0.255 * k) * IN;
                    let r = rbox(p, (0.0, (y0 + y1) * 0.5), (w * 0.66, (y1 - y0) * 0.3), 1.5);
                    if r < px {
                        let pearl = mix(PEARL, 0xc9d8e8, 0.5 + 0.5 * (p.0 * 0.2 + p.1 * 0.1).sin());
                        paint(pearl, cover(r, px), 0.0);
                    }
                }
            }
        }
    }
    // Nut.
    let dnut = rbox(p, (0.0, NUT_Y - 3.0), (0.86 * IN, 4.0), 1.0);
    if dnut < px {
        paint(0xefe9d6, cover(dnut, px), 0.0);
    }
    for i in 0..6 {
        let (_, _, post) = string_points(i);
        let side = post.0.signum();
        // Kidney-shaped buttons on short shafts, out from each side.
        let bx = post.0 + side * 1.0 * IN;
        let kidney = smin(
            ellipse(p, (bx, post.1 - 5.0), (12.0, 9.0)),
            ellipse(p, (bx, post.1 + 5.0), (12.0, 9.0)),
            6.0,
        );
        let kidney = smax(kidney, -circle(p, (bx + side * 15.0, post.1), 5.0), 2.0);
        let shaft = rbox(p, (post.0 + side * 0.68 * IN, post.1), (0.2 * IN, 3.5), 1.0);
        let housing = rbox(
            p,
            (post.0 + side * 0.72 * IN, post.1),
            (0.3 * IN, 13.0),
            4.0,
        );
        let washer = circle(p, post, 9.0);
        for (dd, c) in [
            (shaft, GOLD_DARK),
            (housing, GOLD_DARK),
            (kidney, GOLD),
            (washer, GOLD),
        ] {
            if dd < px {
                paint(c, cover(dd, px), 0.0);
            }
        }
        let hole = circle(p, post, 3.5);
        if hole < px {
            paint(GOLD_DARK, cover(hole, px), 0.0);
        }
    }
    // Pickups: black rings, gold covers, a row of pole screws.
    for &y in &PICKUPS {
        let ring = rbox(p, (0.0, y), (1.7 * IN, 0.95 * IN), 7.0);
        if ring < px {
            paint(0x050506, cover(ring, px), 0.0);
            let cap = rbox(p, (0.0, y), (1.38 * IN, 0.7 * IN), 4.0);
            if cap < px {
                let g = mix(
                    GOLD,
                    0xfff1c2,
                    0.25 * (-((p.0 - 20.0) / 40.0).powi(2)).exp(),
                );
                paint(g, cover(cap, px), 0.0);
                for s in 0..6 {
                    let pole = circle(p, ((s as f32 - 2.5) * 15.0, y - 10.0), 3.4);
                    if pole < px {
                        paint(GOLD_DARK, cover(pole, px), 0.0);
                    }
                }
            }
        }
    }
    // Tune-o-matic with its saddles and two posts, then the stopbar.
    for x in [-1.55 * IN, 1.55 * IN] {
        let post = circle(p, (x, BRIDGE_Y), 7.0);
        if post < px {
            paint(GOLD_DARK, cover(post, px), 0.0);
        }
    }
    let br = rbox(p, (0.0, BRIDGE_Y), (1.5 * IN, 9.0), 3.0);
    if br < px {
        paint(GOLD, cover(br, px), 0.0);
        for s in 0..6 {
            let sd = rbox(p, ((s as f32 - 2.5) * 16.0, BRIDGE_Y), (5.0, 6.0), 1.0);
            if sd < px {
                paint(0xf2d48a, cover(sd, px), 0.0);
            }
        }
    }
    for x in [-1.6 * IN, 1.6 * IN] {
        let stud = circle(p, (x, STOP_Y), 10.0);
        if stud < px {
            paint(GOLD_DARK, cover(stud, px), 0.0);
        }
    }
    let stop = rbox(p, (0.0, STOP_Y), (1.6 * IN, 9.0), 7.0);
    if stop < px {
        paint(GOLD, cover(stop, px), 0.0);
    }
    // Knobs: gold top hats with a lighter crown.
    for &c in &KNOBS {
        let k = circle(p, c, 0.45 * IN);
        if k < px {
            paint(GOLD_DARK, cover(k, px), 0.0);
            let top = circle(p, (c.0 - 2.0, c.1 - 2.0), 0.3 * IN);
            if top < px {
                paint(GOLD, cover(top, px), 0.0);
            }
        }
    }
    // Toggle switch on the upper bass bout: its cream ring, a gold tip.
    let tc = (-123.0, -52.0);
    let tring = circle(p, tc, 13.0);
    if tring < px {
        paint(CREAM, cover(tring, px), 0.0);
        let tip = circle(p, tc, 6.0);
        if tip < px {
            paint(GOLD, cover(tip, px), 0.0);
        }
    }
    (col, a, gloss)
}

/// The guitar-space area the raster covers.
const AREA: (f32, f32, f32, f32) = (-270.0, -1040.0, 270.0, 540.0);
/// Texels a guitar unit at the sharpest level (the closest camera is 2.9).
const TOP_SCALE: f32 = 3.0;
const LEVELS: usize = 4;

/// One mip level: premultiplied RGB, coverage and gloss weight per texel,
/// each 0..255.
struct Level {
    scale: f32,
    w: usize,
    h: usize,
    px: Vec<[u8; 5]>,
    /// Per texel row, the covered texels' first and one-past-last column.
    spans: Vec<(usize, usize)>,
}

fn spans(px: &[[u8; 5]], w: usize, h: usize) -> Vec<(usize, usize)> {
    (0..h)
        .map(|y| {
            let row = &px[y * w..(y + 1) * w];
            let first = row.iter().position(|t| t[3] > 0).unwrap_or(w);
            let last = row.iter().rposition(|t| t[3] > 0).map_or(0, |i| i + 1);
            (first, last.max(first))
        })
        .collect()
}

impl Level {
    /// Bilinear sample at guitar-space (gx, gy), as 0..255 floats.
    fn sample(&self, gx: f32, gy: f32) -> [f32; 5] {
        let u = (gx - AREA.0) * self.scale - 0.5;
        let v = (gy - AREA.1) * self.scale - 0.5;
        if u < -1.0 || v < -1.0 || u >= self.w as f32 || v >= self.h as f32 {
            return [0.0; 5];
        }
        let (x0, y0) = (u.floor(), v.floor());
        let (fx, fy) = (u - x0, v - y0);
        let (x0, y0) = (x0 as i64, y0 as i64);
        let at = |x: i64, y: i64| -> [u8; 5] {
            if x < 0 || y < 0 || x >= self.w as i64 || y >= self.h as i64 {
                [0; 5]
            } else {
                self.px[y as usize * self.w + x as usize]
            }
        };
        let (a, b, c, d) = (
            at(x0, y0),
            at(x0 + 1, y0),
            at(x0, y0 + 1),
            at(x0 + 1, y0 + 1),
        );
        if a[3] == 0 && b[3] == 0 && c[3] == 0 && d[3] == 0 {
            return [0.0; 5];
        }
        let mut out = [0.0; 5];
        for k in 0..5 {
            let top = a[k] as f32 + (b[k] as f32 - a[k] as f32) * fx;
            let bot = c[k] as f32 + (d[k] as f32 - c[k] as f32) * fx;
            out[k] = top + (bot - top) * fy;
        }
        out
    }

    /// The next level down: a 2x2 box filter.
    fn half(&self) -> Level {
        let (w, h) = (self.w.div_ceil(2), self.h.div_ceil(2));
        let mut px = vec![[0u8; 5]; w * h];
        for y in 0..h {
            for x in 0..w {
                let mut acc = [0u32; 5];
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = ((2 * x + dx).min(self.w - 1), (2 * y + dy).min(self.h - 1));
                    let t = self.px[sy * self.w + sx];
                    for k in 0..5 {
                        acc[k] += t[k] as u32;
                    }
                }
                px[y * w + x] = acc.map(|v| ((v + 2) / 4) as u8);
            }
        }
        Level {
            scale: self.scale * 0.5,
            w,
            h,
            spans: spans(&px, w, h),
            px,
        }
    }
}

/// The painted guitar, rasterised once.
pub struct Guitar {
    levels: Vec<Level>,
}

impl Guitar {
    pub fn paint() -> Self {
        let w = ((AREA.2 - AREA.0) * TOP_SCALE) as usize;
        let h = ((AREA.3 - AREA.1) * TOP_SCALE) as usize;
        let size = 1.0 / TOP_SCALE;
        let mut px = vec![[0u8; 5]; w * h];
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
        let rows = h.div_ceil(threads);
        std::thread::scope(|sc| {
            for (band, chunk) in px.chunks_mut(rows * w).enumerate() {
                sc.spawn(move || {
                    for (i, texel) in chunk.iter_mut().enumerate() {
                        let (x, y) = (i % w, band * rows + i / w);
                        let p = (
                            AREA.0 + (x as f32 + 0.5) * size,
                            AREA.1 + (y as f32 + 0.5) * size,
                        );
                        let (rgb, a, gloss) = shade(p, size);
                        let q = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
                        let ch = |s: u32| ((rgb >> s) & 255) as f32 / 255.0 * a;
                        *texel = [q(ch(16)), q(ch(8)), q(ch(0)), q(a), q(gloss * a)];
                    }
                });
            }
        });
        let mut levels = vec![Level {
            scale: TOP_SCALE,
            w,
            h,
            spans: spans(&px, w, h),
            px,
        }];
        for _ in 1..LEVELS {
            let next = levels[levels.len() - 1].half();
            levels.push(next);
        }
        Self { levels }
    }

    /// The coarsest level still at least as fine as the screen.
    fn level(&self, s: f32) -> &Level {
        self.levels
            .iter()
            .rev()
            .find(|l| l.scale >= s)
            .unwrap_or(&self.levels[0])
    }
}

/// Composite the raster through `cam`, sliding the gloss band, in parallel
/// bands of rows.
fn blit(f: &mut Frame, level: &Level, cam: Cam, sheen: f32, gain: f32) {
    let (x0, y0) = cam.to_screen((AREA.0, AREA.1));
    let (x1, y1) = cam.to_screen((AREA.2, AREA.3));
    let (x0, x1) = (x0.max(0.0) as usize, (x1.ceil().max(0.0) as usize).min(W));
    let (y0, y1) = (y0.max(0.0) as usize, (y1.ceil().max(0.0) as usize).min(H));
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    let inv = 1.0 / cam.s;
    let k = gain.min(1.0);
    let gl = [
        ((GLOSS >> 16) & 255) as f32,
        ((GLOSS >> 8) & 255) as f32,
        (GLOSS & 255) as f32,
    ];
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows = (y1 - y0).div_ceil(threads).max(1);
    std::thread::scope(|sc| {
        for (band, chunk) in f.px[y0 * W..y1 * W].chunks_mut(rows * W).enumerate() {
            sc.spawn(move || {
                for (r, line) in chunk.chunks_mut(W).enumerate() {
                    let y = y0 + band * rows + r;
                    let gy = (y as f32 + 0.5 - cam.sy) * inv + cam.gy;
                    // Only the covered stretch of the two texel rows this
                    // screen row samples.
                    let v = ((gy - AREA.1) * level.scale - 0.5).floor();
                    if v < -1.0 || v >= level.h as f32 {
                        continue;
                    }
                    let (va, vb) = ((v.max(0.0)) as usize, ((v + 1.0) as usize).min(level.h - 1));
                    let (ta, tb) = (level.spans[va], level.spans[vb]);
                    let (t0, t1) = (ta.0.min(tb.0), ta.1.max(tb.1));
                    if t0 >= t1 {
                        continue;
                    }
                    let to_x = |tx: f32| (tx / level.scale + AREA.0 - cam.gx) * cam.s + cam.sx;
                    let sx0 = (to_x(t0 as f32 - 1.0).floor().max(0.0) as usize).max(x0);
                    let sx1 = (to_x(t1 as f32 + 1.0).ceil().max(0.0) as usize).min(x1);
                    for (x, dst) in line.iter_mut().enumerate().take(sx1).skip(sx0) {
                        let gx = (x as f32 + 0.5 - cam.sx) * inv + cam.gx;
                        let s = level.sample(gx, gy);
                        if s[3] < 0.5 {
                            continue;
                        }
                        let a = s[3] / 255.0;
                        // The gloss band slides diagonally across the finish.
                        let g = s[4] / 255.0
                            * (-((gx * 0.55 + gy * 0.83 - sheen) / 46.0).powi(2)).exp();
                        let d = *dst;
                        let mut out = 0u32;
                        for (c, shift) in [16u32, 8, 0].iter().enumerate().map(|(i, &sh)| (i, sh)) {
                            let pc = s[c] + (gl[c] * a - s[c]) * g;
                            let dc = ((d >> shift) & 255) as f32;
                            let v = (dc * (1.0 - a * k) + pc * k).clamp(0.0, 255.0) as u32;
                            out |= v << shift;
                        }
                        *dst = out;
                    }
                }
            });
        }
    });
}

/// Draw the guitar through `cam`, its strings ringing with `strings`.
pub fn draw_guitar(
    f: &mut Frame,
    guitar: &Guitar,
    cam: Cam,
    strings: [(f32, f32); 6],
    t: f32,
    gain: f32,
) {
    if gain <= 0.0 {
        return;
    }
    let sheen = -400.0 + (t * 70.0) % 1200.0;
    blit(f, guitar.level(cam.s), cam, sheen, gain);
    // Strings: tuner post to nut, nut to saddle, saddle to stopbar. A
    // ringing string blurs into a band, widest at its middle.
    for (i, &(amp, _)) in strings.iter().enumerate() {
        let (nut, saddle, post) = string_points(i);
        let width = (0.9 + (5 - i) as f32 * 0.22) * cam.s;
        // From far away a real string is thinner than a pixel: dim it.
        let thin = (width / 1.6).clamp(0.35, 1.0);
        let stop = (saddle.0 * 0.95, STOP_Y);
        for (a, b) in [(post, nut), (saddle, stop)] {
            let (sa, sb) = (cam.to_screen(a), cam.to_screen(b));
            f.line(
                sa.0,
                sa.1,
                sb.0,
                sb.1,
                width.max(0.8),
                STRING,
                0.85 * thin * gain,
                1.0,
            );
        }
        let segs = 40;
        for k in 0..segs {
            let (u0, u1) = (k as f32 / segs as f32, (k + 1) as f32 / segs as f32);
            let at = |u: f32| {
                (
                    nut.0 + (saddle.0 - nut.0) * u,
                    nut.1 + (saddle.1 - nut.1) * u,
                )
            };
            let (p0, p1) = (cam.to_screen(at(u0)), cam.to_screen(at(u1)));
            let swing = amp * 7.0 * (std::f32::consts::PI * (u0 + u1) * 0.5).sin() * cam.s;
            let c = mix(STRING, 0xffffff, (amp * 1.5).min(1.0));
            f.line(
                p0.0,
                p0.1,
                p1.0,
                p1.1,
                width.max(0.8) + swing,
                c,
                (0.9 - 0.4 * (swing / (swing + 6.0))) * thin * gain,
                1.0,
            );
            if amp > 0.05 {
                f.glow(
                    (p0.0 + p1.0) * 0.5,
                    (p0.1 + p1.1) * 0.5,
                    6.0 + swing * 2.0,
                    0xfff3d0,
                    0.06 * amp * gain,
                );
            }
        }
    }
}

pub fn draw(f: &mut Frame, guitar: &Guitar, notes: &[Note], n: Now, t: f32) -> f32 {
    let fade = n.envelope(1.2, 0.8);
    room(f, t, fade);
    let cam = |gx, gy, sx, sy, s| Cam { gx, gy, sx, sy, s };
    let keys = [
        (0.0, cam(10.0, 150.0, 560.0, 1150.0, 2.9)),
        (3.6, cam(-10.0, 90.0, 540.0, 1100.0, 2.5)),
        (7.5, cam(0.0, -560.0, 540.0, 1050.0, 2.2)),
        (9.5, cam(0.0, -860.0, 540.0, 1000.0, 2.0)),
        (13.5, FULL),
    ];
    let mut at = keys[keys.len() - 1].1;
    for w in keys.windows(2) {
        let ((t0, a), (t1, b)) = (w[0], w[1]);
        if n.local < t1 {
            at = Cam::lerp(a, b, smooth((n.local - t0) / (t1 - t0)));
            break;
        }
    }
    draw_guitar(f, guitar, at, strings_at(notes, t), t, fade);
    1.0
}

/// A dark room: a warm light from above, dust drifting in it.
pub fn room(f: &mut Frame, t: f32, fade: f32) {
    // The gradient and its big warm glow never change: paint them once and
    // copy (or fade between the plain gradient and the lit one).
    static LIT: OnceLock<(Vec<u32>, Vec<u32>)> = OnceLock::new();
    let (plain, lit) = LIT.get_or_init(|| {
        let mut g = Frame::new();
        g.gradient(&[0x0a0807, 0x14100d, 0x0d0a09, 0x050404]);
        let plain = g.px.clone();
        g.glow(540.0, 500.0, 900.0, 0x5a3a1c, 0.35);
        (plain, g.px)
    });
    if fade >= 1.0 {
        f.px.copy_from_slice(lit);
    } else {
        f.px.copy_from_slice(plain);
        f.glow(540.0, 500.0, 900.0, 0x5a3a1c, 0.35 * fade);
    }
    for i in 0..140 {
        let h = |k: i32| crate::gfx::hash(i, k);
        let x = (h(1) * W as f32 + (t * (6.0 + 10.0 * h(2)) + h(3) * 50.0).sin() * 30.0) % W as f32;
        let y = (h(4) * H as f32 - t * (8.0 + 14.0 * h(5))).rem_euclid(H as f32);
        f.blend(x as i32, y as i32, 0xffe2b0, (0.15 + 0.35 * h(6)) * fade);
    }
}
