// The party: Cordelia and, from the knight's scene on, Victor, each in an
// RPG window at the bottom of the screen. A window is a deep emerald
// gradient with a bevelled silver edge (the old console menus, in the
// brand's colours). Inside it a portrait performs: expressions crossfade,
// eyes blink, and the window wobbles in on Amiga raster lines. Under
// Cordelia's portrait an energy gauge empties through the day and fills
// as she rests. A small wordless dialog window beside the party holds
// pictograms: a cup, a slice of pie, a drop, a firefly, a star, a sword.
//
// The portraits are image-to-image edits of the brand direction's approved
// dark artwork (assets/portraits/provenance.json), stored as binary PPMs
// and parsed by hand.
use crate::fx::{ease_out, h1, smooth, window};
use crate::gfx::{mix, Frame, H, W};

pub const PORTRAIT: usize = 272;

macro_rules! portraits {
    ($($id:ident => $file:literal),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Face { $($id),* }
        const FILES: &[(Face, &[u8])] = &[
            $((Face::$id, include_bytes!(concat!("../assets/portraits/", $file, ".ppm")))),*
        ];
    };
}

portraits! {
    Tired => "cordelia-tired",
    TiredBlink => "cordelia-tired-blink",
    Relief => "cordelia-relief",
    Scrutiny => "cordelia-scrutiny",
    Humor => "cordelia-humor",
    Awe => "cordelia-awe",
    Resolve => "cordelia-resolve",
    Drowsy => "cordelia-drowsy",
    Asleep => "cordelia-asleep",
    Laugh => "cordelia-laugh",
    Watch => "victor-watch",
    VBlink => "victor-blink",
    VRelief => "victor-relief",
    VDefiance => "victor-defiance",
    VResolve => "victor-resolve",
    VLaugh => "victor-laugh",
}

impl Face {
    /// The frame this face blinks to, if it blinks.
    fn blink(self) -> Option<Face> {
        match self {
            Face::Tired | Face::Scrutiny | Face::Resolve => Some(Face::TiredBlink),
            Face::Watch | Face::VResolve | Face::VRelief => Some(Face::VBlink),
            _ => None,
        }
    }
}

pub struct Portraits {
    imgs: Vec<(Face, Vec<u32>)>,
}

/// A binary PPM (P6, maxval 255), parsed by hand.
fn ppm(data: &[u8]) -> (usize, usize, Vec<u32>) {
    let mut fields = Vec::new();
    let mut i = 0;
    while fields.len() < 4 {
        while data[i].is_ascii_whitespace() {
            i += 1;
        }
        if data[i] == b'#' {
            while data[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let start = i;
        while !data[i].is_ascii_whitespace() {
            i += 1;
        }
        fields.push(String::from_utf8_lossy(&data[start..i]).to_string());
    }
    i += 1;
    assert_eq!(fields[0], "P6", "portraits are binary PPMs");
    let w: usize = fields[1].parse().expect("PPM width");
    let h: usize = fields[2].parse().expect("PPM height");
    let px = data[i..i + w * h * 3]
        .as_chunks::<3>()
        .0
        .iter()
        .map(|c| (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32)
        .collect();
    (w, h, px)
}

impl Portraits {
    pub fn load() -> Self {
        let imgs = FILES
            .iter()
            .map(|(face, bytes)| {
                let (w, h, px) = ppm(bytes);
                assert_eq!((w, h), (PORTRAIT, PORTRAIT), "{face:?} is {w}x{h}");
                (*face, px)
            })
            .collect();
        Self { imgs }
    }

    fn get(&self, f: Face) -> &[u32] {
        &self
            .imgs
            .iter()
            .find(|(g, _)| *g == f)
            .expect("every face loads")
            .1
    }
}

/// One character's acting: (time, face) keys, crossfading over `XF`.
pub struct Acting {
    pub keys: &'static [(f32, Face)],
    /// Seed for the blink rhythm.
    pub seed: i32,
}

const XF: f32 = 0.35;

impl Acting {
    /// (from, to, mix) at time t.
    fn faces(&self, t: f32) -> (Face, Face, f32) {
        let mut cur = self.keys[0].1;
        let mut prev = cur;
        let mut since = 99.0;
        for &(at, f) in self.keys {
            if t >= at {
                prev = cur;
                cur = f;
                since = t - at;
            }
        }
        let k = smooth(since / XF);
        // Blinks: a short closed frame every 3-6 seconds while the face holds.
        if k >= 1.0 {
            if let Some(b) = cur.blink() {
                let period = 3.2 + 2.6 * h1(self.seed);
                let n = (t / period).floor() as i32;
                let phase = t - n as f32 * period - period * 0.6 * h1(n * 31 + self.seed);
                if (0.0..0.16).contains(&phase) {
                    return (cur, b, 1.0);
                }
            }
        }
        (prev, cur, k)
    }
}

/// Where and how a party window is this frame.
#[derive(Clone, Copy)]
pub struct Slot {
    pub x: i32,
    pub y: i32,
    /// 0 hidden .. 1 fully in (drives the wobble-in).
    pub show: f32,
    /// Multiplied into the portrait (scene light).
    pub tint: u32,
    pub tint_k: f32,
}

pub const WIN_W: i32 = PORTRAIT as i32 + 28;
pub const WIN_H: i32 = PORTRAIT as i32 + 64;

/// The window chrome: gradient body, bevelled silver border, a jade inner
/// rule. `show` < 1 opens it from its centre line, as menus do.
pub fn window_box(f: &mut Frame, x0: i32, y0: i32, w: i32, h: i32, show: f32, alpha: f32) {
    if show <= 0.0 {
        return;
    }
    let open = ease_out(show);
    let hh = ((h as f32) * open) as i32;
    let y0 = y0 + (h - hh) / 2;
    let h = hh.max(2);
    for y in y0..y0 + h {
        let v = (y - y0) as f32 / h as f32;
        let c = mix(0x0b3a31, 0x03120f, v);
        f.rect(x0, y, x0 + w, y + 1, c, 0.92 * alpha);
    }
    // Bevel: light top-left, dark bottom-right, a silver line, a jade rule.
    f.rect(x0, y0, x0 + w, y0 + 3, 0xdfe9e4, alpha);
    f.rect(x0, y0, x0 + 3, y0 + h, 0xdfe9e4, alpha);
    f.rect(x0, y0 + h - 3, x0 + w, y0 + h, 0x5d6d67, alpha);
    f.rect(x0 + w - 3, y0, x0 + w, y0 + h, 0x5d6d67, alpha);
    f.frame_rect(
        x0 + 6,
        y0 + 6,
        x0 + w - 6,
        y0 + h - 6,
        0x3dbb8a,
        0.55 * alpha,
    );
    // Rivets.
    for (cx, cy) in [
        (x0 + 10, y0 + 10),
        (x0 + w - 11, y0 + 10),
        (x0 + 10, y0 + h - 11),
        (x0 + w - 11, y0 + h - 11),
    ] {
        f.rect(cx - 2, cy - 2, cx + 2, cy + 2, 0xc8d6cf, alpha);
    }
}

pub fn draw_portrait(f: &mut Frame, p: &Portraits, acting: &Acting, slot: Slot, t: f32) {
    if slot.show <= 0.0 {
        return;
    }
    window_box(f, slot.x, slot.y, WIN_W, WIN_H, slot.show * 1.6, 1.0);
    let reveal = window(slot.show, 0.35, 1.0);
    if reveal <= 0.0 {
        return;
    }
    let (a, b, k) = acting.faces(t);
    let (ia, ib) = (p.get(a), p.get(b));
    let (px0, py0) = (slot.x + 14, slot.y + 14);
    let wob = (1.0 - reveal) * 26.0;
    for y in 0..PORTRAIT {
        // Interlaced reveal: odd rows arrive later, every row sways in.
        let row_k = if y % 2 == 0 {
            reveal
        } else {
            window(reveal, 0.25, 1.0)
        };
        if row_k <= 0.0 {
            continue;
        }
        let shift = (wob * ((y as f32) * 0.09 + t * 9.0).sin()) as i32;
        let fy = py0 + y as i32;
        if fy < 0 || fy >= H as i32 {
            continue;
        }
        for x in 0..PORTRAIT {
            let fx = px0 + x as i32 + shift;
            if fx < px0 || fx >= px0 + PORTRAIT as i32 || fx >= W as i32 {
                continue;
            }
            let i = y * PORTRAIT + x;
            let mut c = if k >= 1.0 {
                ib[i]
            } else {
                mix(ia[i], ib[i], k)
            };
            if slot.tint_k > 0.0 {
                c = mix(c, multiply(c, slot.tint), slot.tint_k);
            }
            f.blend(fx, fy, c, row_k);
        }
    }
    // Thin inner frame around the picture.
    f.frame_rect(
        px0 - 1,
        py0 - 1,
        px0 + PORTRAIT as i32 + 1,
        py0 + PORTRAIT as i32 + 1,
        0x0a0f0d,
        reveal,
    );
}

fn multiply(a: u32, b: u32) -> u32 {
    let ch = |s: u32| (((a >> s) & 255) * ((b >> s) & 255) / 255) << s;
    ch(16) | ch(8) | ch(0)
}

/// The energy gauge under a portrait: segmented, red when low, amber, then
/// emerald when full; a shimmer runs along it while it fills.
pub fn gauge(
    f: &mut Frame,
    (x0, y0): (i32, i32),
    w: i32,
    value: f32,
    filling: f32,
    t: f32,
    alpha: f32,
) {
    if alpha <= 0.0 {
        return;
    }
    let h = 16;
    f.rect(x0 - 2, y0 - 2, x0 + w + 2, y0 + h + 2, 0x0a0f0d, alpha);
    f.frame_rect(
        x0 - 3,
        y0 - 3,
        x0 + w + 3,
        y0 + h + 3,
        0x8fa79c,
        alpha * 0.8,
    );
    let seg = 18;
    let n = w / seg;
    let lit = value.clamp(0.0, 1.0) * n as f32;
    let col = if value < 0.25 {
        mix(0xdc4f42, 0xff7a3a, value / 0.25)
    } else if value < 0.6 {
        mix(0xff9f3a, 0xe8c75a, (value - 0.25) / 0.35)
    } else {
        mix(0xa4d65e, 0x3dbb8a, (value - 0.6) / 0.4)
    };
    // Low energy blinks.
    let blink = if value < 0.2 {
        0.55 + 0.45 * (t * 6.0).sin().abs()
    } else {
        1.0
    };
    for i in 0..n {
        let k = (lit - i as f32).clamp(0.0, 1.0);
        if k <= 0.0 {
            continue;
        }
        let sx = x0 + i * seg + 1;
        let shimmer =
            filling * (1.0 - ((i as f32 - (t * 14.0) % (n as f32 + 6.0)).abs() / 3.0).min(1.0));
        let c = mix(col, 0xffffff, shimmer * 0.7);
        for y in 0..h {
            let v = y as f32 / h as f32;
            let shade = 1.15 - 0.5 * v;
            f.rect(
                sx,
                y0 + y,
                sx + seg - 3,
                y0 + y + 1,
                crate::gfx::scale(c, shade),
                alpha * k * blink,
            );
        }
    }
}

// ── Pictograms ───────────────────────────────────────────────────────────────
// 12x12 bitmaps drawn as square pixels: '#' ink, '+' highlight, 'o' accent.

pub const CUP: [&str; 12] = [
    "   +  +     ",
    "  +  +      ",
    "   +  +     ",
    "            ",
    " ########   ",
    " #oooooo### ",
    " #oooooo# # ",
    " #oooooo# # ",
    " #oooooo### ",
    "  ######    ",
    "############",
    " ########## ",
];
pub const PIE: [&str; 12] = [
    "            ",
    "          ++",
    "        ++o#",
    "      ++oo##",
    "    ++oooo##",
    "  ++oooooo##",
    "++oooooooo##",
    "############",
    "#o#o#o#o#o##",
    "############",
    "            ",
    "            ",
];
pub const DROP: [&str; 12] = [
    "     #      ",
    "     #      ",
    "    ###     ",
    "    #o#     ",
    "   #ooo#    ",
    "   #o+o#    ",
    "  #oo+oo#   ",
    "  #oooo+#   ",
    "  #ooooo#   ",
    "   #ooo#    ",
    "    ###     ",
    "            ",
];
pub const FIREFLY: [&str; 12] = [
    "            ",
    "   ##  ##   ",
    "  #  ##  #  ",
    "  #  ##  #  ",
    "   ######   ",
    "    #oo#    ",
    "   +oooo+   ",
    "  +oo++oo+  ",
    "   +oooo+   ",
    "    +oo+    ",
    "     ++     ",
    "            ",
];
pub const STAR: [&str; 12] = [
    "     ++     ",
    "     ++     ",
    "    +oo+    ",
    "    +oo+    ",
    "++++oooo++++",
    " +oooooooo+ ",
    "  +oooooo+  ",
    "  +oooooo+  ",
    " +ooo++ooo+ ",
    " +oo+  +oo+ ",
    "+o+      +o+",
    "            ",
];
pub const MOON: [&str; 12] = [
    "    ####    ",
    "  ##oo#     ",
    " #ooo#      ",
    " #oo#       ",
    "#ooo#       ",
    "#ooo#       ",
    "#ooo#       ",
    "#oooo#      ",
    " #oooo#   # ",
    " #oooooo### ",
    "  ##oooo##  ",
    "    ####    ",
];
pub const SWORD: [&str; 12] = [
    "          ++",
    "         +o+",
    "        +o+ ",
    "       +o+  ",
    "      +o+   ",
    "     +o+    ",
    "  # +o+     ",
    "   #o+      ",
    "    #       ",
    "   # #      ",
    "  #   #     ",
    " #          ",
];
pub const MALLOW: [&str; 12] = [
    "            ",
    "     ##     ",
    "    #++#    ",
    "   #o++o#   ",
    "   #oooo#   ",
    "   #oooo#   ",
    "    ####    ",
    "     #      ",
    "     #      ",
    "     #      ",
    "     #      ",
    "     #      ",
];
pub const GEM: [&str; 12] = [
    "            ",
    "   ######   ",
    "  #+oooo+#  ",
    " #+oooooo+# ",
    "############",
    " #oo+oooo#  ",
    "  #oo+oo#   ",
    "   #oooo#   ",
    "    #oo#    ",
    "     ##     ",
    "            ",
    "            ",
];
pub const HEART: [&str; 12] = [
    "            ",
    " ###   ###  ",
    "#ooo# #ooo# ",
    "#o+oo#ooooo#",
    "#o+oooooooo#",
    "#oooooooooo#",
    " #oooooooo# ",
    "  #oooooo#  ",
    "   #oooo#   ",
    "    #oo#    ",
    "     ##     ",
    "            ",
];

/// A pictogram at (x, y), `px` pixels per bitmap pixel, `appear` 0..1
/// drawing it in row by row.
pub fn pictogram(
    f: &mut Frame,
    bmp: &[&str; 12],
    (x, y): (i32, i32),
    px: i32,
    ink: u32,
    accent: u32,
    appear: f32,
) {
    let rows = (appear.clamp(0.0, 1.0) * 12.0).ceil() as usize;
    for (j, line) in bmp.iter().enumerate().take(rows) {
        for (i, ch) in line.chars().enumerate() {
            let c = match ch {
                '#' => ink,
                '+' => 0xffffff,
                'o' => accent,
                _ => continue,
            };
            let (sx, sy) = (x + i as i32 * px, y + j as i32 * px);
            f.rect(sx, sy, sx + px - 1, sy + px - 1, c, 1.0);
        }
    }
}
