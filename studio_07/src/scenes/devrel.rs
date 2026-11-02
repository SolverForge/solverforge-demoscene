// DEVREL: offer four, developer relations. A launch kit for a project
// assembles tile by tile: the docs, a quickstart that runs, the release
// post, and a demo video -- the kind of thing this video is.
use super::{dot_grid, typed, PAPER};
use crate::gfx::{ease_out, mix, Frame};
use crate::timeline::{smooth, Now};

const BG: u32 = 0x0d0806;
const CORAL: u32 = 0xfb923c;
const PANE: u32 = 0x1a110c;
const BORDER: u32 = 0x5a3a24;
const DIM: u32 = 0x8a6a52;
const TEXT: u32 = 0xf3e2d4;

type Rect = (i32, i32, i32, i32);

const DOCS: Rect = (60, 130, 620, 400);
const QUICK: Rect = (660, 130, 1220, 400);
const RELEASE: Rect = (60, 430, 620, 690);
const VIDEO: Rect = (660, 430, 1220, 690);
/// When each tile drops in (seconds into the section).
const IN: [f32; 4] = [0.6, 2.4, 4.2, 6.0];

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.clear(BG);
    dot_grid(f, CORAL, 0.06 * fade);

    let tiles = [
        (DOCS, "DOCS"),
        (QUICK, "QUICKSTART"),
        (RELEASE, "RELEASE POST"),
        (VIDEO, "DEMO VIDEO"),
    ];
    for (i, (r, title)) in tiles.iter().enumerate() {
        let k = ease_out((n.local - IN[i]) / 0.6);
        if k <= 0.0 {
            continue;
        }
        let dy = ((1.0 - k) * 40.0) as i32;
        let r = (r.0, r.1 + dy, r.2, r.3 + dy);
        let a = k * fade;
        tile(f, r, title, a);
        let local = n.local - IN[i];
        match i {
            0 => docs(f, r, local, a),
            1 => quickstart(f, r, local, t, a),
            2 => release(f, r, local, a),
            _ => video(f, r, local, t, a),
        }
    }
    1.0
}

fn tile(f: &mut Frame, r: Rect, title: &str, a: f32) {
    f.rect(r.0, r.1, r.2, r.3, PANE, 0.92 * a);
    f.frame_rect(r.0, r.1, r.2, r.3, BORDER, a);
    f.rect(r.0 + 1, r.1 + 1, r.2 - 1, r.1 + 30, 0x24170f, a);
    for (k, c) in [0xff5f56, 0xffbd2e, 0x27c93f].iter().enumerate() {
        f.disc(
            r.0 as f32 + 16.0 + k as f32 * 16.0,
            r.1 as f32 + 15.0,
            4.5,
            *c,
            0.8 * a,
        );
    }
    f.text(r.0 + 70, r.1 + 7, title, 2, CORAL, a);
}

/// A docs page: a heading, paragraphs that fill in, a code block.
fn docs(f: &mut Frame, r: Rect, local: f32, a: f32) {
    let (x, y) = (r.0 + 24, r.1 + 46);
    f.text(
        x,
        y,
        &typed("GETTING STARTED", 0.2, 24.0, local),
        3,
        PAPER,
        a,
    );
    for line in 0..4i32 {
        let w = [470, 430, 490, 300][line as usize];
        let k = smooth((local - 0.9 - line as f32 * 0.25) / 0.5);
        f.rect(
            x,
            y + 48 + line * 18,
            x + (w as f32 * k) as i32,
            y + 56 + line * 18,
            DIM,
            0.8 * a,
        );
    }
    let k = smooth((local - 2.0) / 0.4);
    f.rect(x, y + 130, r.2 - 24, y + 205, 0x0a0705, k * a);
    f.rect(x, y + 130, x + 3, y + 205, CORAL, k * a);
    f.text(
        x + 16,
        y + 142,
        &typed("[dependencies]", 2.2, 30.0, local),
        2,
        DIM,
        a,
    );
    f.text(
        x + 16,
        y + 170,
        &typed("yourproject = \"1.0\"", 2.8, 30.0, local),
        2,
        TEXT,
        a,
    );
}

const SHELL: [(f32, &str, bool); 5] = [
    (0.3, "$ git clone yourproject", true),
    (1.4, "$ cd yourproject", true),
    (2.5, "$ make run", true),
    (3.4, "  build ok", false),
    (3.9, "  open localhost:8080", false),
];

/// A quickstart that runs, with a check when it is up.
fn quickstart(f: &mut Frame, r: Rect, local: f32, t: f32, a: f32) {
    let (x, y) = (r.0 + 24, r.1 + 50);
    let mut last = (x, y);
    for (i, (at, line, cmd)) in SHELL.iter().enumerate() {
        if local < *at {
            break;
        }
        let ly = y + i as i32 * 34;
        let shown = if *cmd {
            typed(line, *at, 22.0, local)
        } else {
            line.to_string()
        };
        let c = if *cmd { TEXT } else { 0x86efac };
        f.text(x, ly, &shown, 2, c, a);
        last = (x + shown.chars().count() as i32 * 20, ly);
    }
    if local < 4.4 && (t * 3.0).fract() < 0.6 {
        f.rect(last.0 + 4, last.1, last.0 + 18, last.1 + 18, CORAL, a);
    }
    let k = ease_out((local - 4.4) / 0.4);
    if k > 0.0 {
        let (cx, cy) = (r.2 as f32 - 60.0, r.3 as f32 - 56.0);
        f.disc(cx, cy, 30.0 * k, 0x27c93f, a);
        f.line(cx - 13.0, cy, cx - 3.0, cy + 11.0, 5.0, BG, a * k, 1.0);
        f.line(
            cx - 3.0,
            cy + 11.0,
            cx + 15.0,
            cy - 11.0,
            5.0,
            BG,
            a * k,
            1.0,
        );
    }
}

/// The release post: a version tag, a headline, the changes.
fn release(f: &mut Frame, r: Rect, local: f32, a: f32) {
    let (x, y) = (r.0 + 24, r.1 + 46);
    let k = ease_out(local / 0.4);
    f.rect(x, y, x + 144, y + 30, CORAL, 0.9 * k * a);
    f.text(x + 12, y + 6, "V1.0.0", 2, BG, k * a);
    f.text(
        x + 164,
        y + 6,
        &typed("IS OUT", 0.4, 20.0, local),
        2,
        PAPER,
        a,
    );
    let items = ["FASTER", "SIMPLER API", "NEW EXAMPLES", "MIGRATION GUIDE"];
    for (i, item) in items.iter().enumerate() {
        let k = smooth((local - 1.0 - i as f32 * 0.4) / 0.3);
        let ly = y + 54 + i as i32 * 36;
        f.disc(x as f32 + 6.0, ly as f32 + 9.0, 4.0, CORAL, k * a);
        f.text(x + 22, ly, item, 2, TEXT, k * a);
    }
}

/// A demo video playing: a turning octagon over copper bars, with a
/// progress bar -- a small copy of the scroller scene further on.
fn video(f: &mut Frame, r: Rect, local: f32, t: f32, a: f32) {
    let (x0, y0, x1, y1) = (r.0 + 14, r.1 + 40, r.2 - 14, r.3 - 34);
    f.rect(x0, y0, x1, y1, 0x020203, a);
    let (cx, cy) = (((x0 + x1) / 2) as f32, ((y0 + y1) / 2) as f32);
    for (i, c) in [0x10b981u32, 0x50fc00, 0x38bdf8, 0xa78bfa]
        .iter()
        .enumerate()
    {
        let y = cy + 40.0 + (t * 1.7 + i as f32 * 0.9).sin() * 40.0;
        for dy in -6i32..=6 {
            let k = 1.0 - (dy as f32 / 6.0).abs();
            f.rect(
                x0,
                y as i32 + dy,
                x1,
                y as i32 + dy + 1,
                *c,
                0.4 * k * k * a,
            );
        }
    }
    let rot = t * 0.9;
    let pts: Vec<(f32, f32)> = (0..8)
        .map(|k| {
            let ang = k as f32 * std::f32::consts::TAU / 8.0 + std::f32::consts::PI / 8.0 + rot;
            (cx + ang.cos() * 58.0, cy - 10.0 + ang.sin() * 58.0 * 0.8)
        })
        .collect();
    for k in 0..8 {
        let (p, q) = (pts[k], pts[(k + 1) % 8]);
        f.line(p.0, p.1, q.0, q.1, 2.5, 0x10b981, a, 1.0);
    }
    // Controls.
    let prog = (local / 4.5).fract();
    f.rect(x0, y1 + 12, x1, y1 + 16, BORDER, a);
    f.rect(
        x0,
        y1 + 12,
        x0 + ((x1 - x0) as f32 * prog) as i32,
        y1 + 16,
        CORAL,
        a,
    );
    f.disc(
        x0 as f32 + (x1 - x0) as f32 * prog,
        y1 as f32 + 14.0,
        6.0,
        mix(CORAL, 0xffffff, 0.4),
        a,
    );
}
