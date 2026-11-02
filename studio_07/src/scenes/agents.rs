// AGENTS: offer three. A tiling desktop: a planner calendar, a terminal
// mail client, and an agent's log. The windows split as they open, and the
// agent's pointer works through the same windows its log describes.
use super::PAPER;
use crate::gfx::{ease_out, mix, Frame, W};
use crate::timeline::{smooth, Now};

const DESK: u32 = 0x0c0a14;
const PANE: u32 = 0x13111d;
const BORDER: u32 = 0x3b3554;
const FOCUS: u32 = 0xa78bfa;
const TEXT: u32 = 0xc9c3e6;
const DIMTXT: u32 = 0x6f6890;

type Rect = (f32, f32, f32, f32);

const AREA: Rect = (24.0, 120.0, 1256.0, 672.0);
const SPLIT_1: f32 = 3.6;
const SPLIT_2: f32 = 7.4;

const LOG: [(f32, &str); 9] = [
    (8.4, "> screenshot"),
    (9.0, "  3 windows, 1 focused"),
    (9.8, "> focus franking"),
    (10.8, "> click row 2"),
    (11.8, "> focus planner123"),
    (12.6, "> type standup 10:30"),
    (13.6, "> key Return"),
    (14.4, "  placed: thu 10:30"),
    (15.2, "> done"),
];

/// Calendar events: (day, start hour from 9, hours, colour, label).
const EVENTS: [(usize, f32, f32, u32, &str); 7] = [
    (0, 0.0, 1.5, 0x34d399, "deep work"),
    (0, 4.0, 1.0, 0x38bdf8, "client"),
    (1, 1.0, 2.0, 0xfbbf24, "review"),
    (2, 0.5, 1.0, 0x38bdf8, "call"),
    (2, 3.0, 2.5, 0x34d399, "deep work"),
    (4, 2.0, 1.5, 0xf472b6, "redona"),
    (1, 5.0, 1.0, 0x38bdf8, "sync"),
];
/// To-do items the planner schedules: (when, day, hour, len, label).
const TODOS: [(f32, usize, f32, f32, &str); 3] = [
    (1.4, 3, 0.0, 1.5, "write docs"),
    (2.2, 4, 4.5, 1.0, "invoice"),
    (3.0, 1, 3.5, 1.0, "bench run"),
];

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.clear(DESK);
    // Wallpaper: faint diagonal hatching.
    for y in (0..720).step_by(4) {
        for x in (0..W as i32).step_by(4) {
            if (x + y) % 24 == 0 {
                f.blend(x, y, 0x1c1830, fade);
            }
        }
    }
    let (planner, mail, log) = layout(n.local);
    let focus = if n.local > 11.8 {
        0
    } else if n.local > 9.8 {
        1
    } else if n.local > SPLIT_2 {
        2
    } else if n.local > SPLIT_1 {
        1
    } else {
        0
    };

    window(f, planner, "planner123", focus == 0, fade);
    planner_body(f, planner, n.local, fade);
    let mail_in = smooth((n.local - SPLIT_1) / 0.5);
    if mail_in > 0.0 {
        window(f, mail, "franking", focus == 1, fade * mail_in);
        mail_body(f, mail, n.local, fade * mail_in);
    }
    let log_in = smooth((n.local - SPLIT_2) / 0.5);
    if log_in > 0.0 {
        window(f, log, "computer-use-sway", focus == 2, fade * log_in);
        log_body(f, log, n.local, t, fade * log_in);
    }
    pointer(f, planner, mail, n.local, fade);
    bar(f, fade);
    1.0
}

fn lerp_rect(a: Rect, b: Rect, k: f32) -> Rect {
    (
        a.0 + (b.0 - a.0) * k,
        a.1 + (b.1 - a.1) * k,
        a.2 + (b.2 - a.2) * k,
        a.3 + (b.3 - a.3) * k,
    )
}

fn layout(local: f32) -> (Rect, Rect, Rect) {
    let (x0, y0, x1, y1) = AREA;
    let gap = 10.0;
    let mid = x0 + (x1 - x0) * 0.6;
    let k1 = ease_out((local - SPLIT_1) / 0.6);
    let k2 = ease_out((local - SPLIT_2) / 0.6);
    let planner = lerp_rect(AREA, (x0, y0, mid - gap * 0.5, y1), k1);
    let right = (mid + gap * 0.5, y0, x1, y1);
    let ymid = y0 + (y1 - y0) * 0.5;
    let mail = lerp_rect(right, (right.0, y0, x1, ymid - gap * 0.5), k2);
    let log = (right.0, ymid + gap * 0.5, x1, y1);
    let mail = lerp_rect((x1, y0, x1, y1), mail, k1);
    (planner, mail, log)
}

fn window(f: &mut Frame, r: Rect, title: &str, focused: bool, a: f32) {
    let (x0, y0, x1, y1) = (r.0 as i32, r.1 as i32, r.2 as i32, r.3 as i32);
    if x1 - x0 < 8 {
        return;
    }
    f.rect(x0, y0, x1, y1, PANE, a);
    let c = if focused { FOCUS } else { BORDER };
    f.frame_rect(x0, y0, x1, y1, c, a);
    f.frame_rect(x0 + 1, y0 + 1, x1 - 1, y1 - 1, c, 0.5 * a);
    f.rect(x0 + 2, y0 + 2, x1 - 2, y0 + 28, 0x1a1728, a);
    let max = ((x1 - x0 - 24) / 20).max(0) as usize;
    let title: String = title.chars().take(max).collect();
    f.text(
        x0 + 12,
        y0 + 6,
        &title,
        2,
        if focused { PAPER } else { DIMTXT },
        a,
    );
}

fn planner_body(f: &mut Frame, r: Rect, local: f32, a: f32) {
    let (x0, y0, x1, y1) = (r.0 + 12.0, r.1 + 40.0, r.2 - 12.0, r.3 - 12.0);
    let todo_w = 170.0;
    let gx0 = x0 + todo_w;
    let col = (x1 - gx0) / 5.0;
    let hours = 8.0;
    let row = (y1 - (y0 + 26.0)) / hours;
    let gy0 = y0 + 26.0;
    for (d, name) in ["MON", "TUE", "WED", "THU", "FRI"].iter().enumerate() {
        let cx = gx0 + col * (d as f32 + 0.5);
        f.text_centered(cx as i32, y0 as i32, name, 2, TEXT, a);
        f.rect(
            (gx0 + col * d as f32) as i32,
            gy0 as i32,
            (gx0 + col * d as f32) as i32 + 1,
            y1 as i32,
            BORDER,
            a,
        );
    }
    for h in 0..=hours as i32 {
        let y = gy0 + row * h as f32;
        f.rect(gx0 as i32, y as i32, x1 as i32, y as i32 + 1, 0x221f33, a);
        if h < hours as i32 {
            f.text(
                gx0 as i32 - 26,
                y as i32 + 3,
                &format!("{:02}", 9 + h),
                1,
                DIMTXT,
                a,
            );
        }
    }
    let slot = |d: usize, h: f32, len: f32| -> Rect {
        let x = gx0 + col * d as f32 + 3.0;
        let y = gy0 + row * h + 2.0;
        (x, y, x + col - 6.0, y + row * len - 4.0)
    };
    let block = |f: &mut Frame, r: Rect, c: u32, label: &str, a: f32| {
        f.rect(
            r.0 as i32,
            r.1 as i32,
            r.2 as i32,
            r.3 as i32,
            mix(PANE, c, 0.35),
            a,
        );
        f.rect(r.0 as i32, r.1 as i32, r.0 as i32 + 3, r.3 as i32, c, a);
        let max = ((r.2 - r.0 - 10.0) / 10.0) as usize;
        let label: String = label.chars().take(max).collect();
        f.text(r.0 as i32 + 7, r.1 as i32 + 4, &label, 1, PAPER, a);
    };
    for (d, h, len, c, label) in EVENTS {
        block(f, slot(d, h, len), c, label, a);
    }

    // The to-do list, and the items the planner places into free time.
    f.text(x0 as i32, y0 as i32, "TO DO", 2, TEXT, a);
    for (i, (at, d, h, len, label)) in TODOS.iter().enumerate() {
        let home = (
            x0,
            y0 + 34.0 + i as f32 * 30.0,
            x0 + todo_w - 24.0,
            y0 + 58.0 + i as f32 * 30.0,
        );
        let k = ease_out((local - at) / 0.7);
        let r = lerp_rect(home, slot(*d, *h, *len), k);
        block(f, r, 0xa78bfa, label, a);
    }
    // The agent's entry, typed in the log at 12.6.
    let k = ease_out((local - 13.6) / 0.5);
    if k > 0.0 {
        let r = slot(3, 1.5, 0.75);
        block(f, r, 0x50fc00, "standup", a * k);
    }
}

const MAIL: [(&str, &str); 6] = [
    ("anna", "shift plan"),
    ("luca", "40 stops, 3 vans"),
    ("ci", "release green"),
    ("giulia", "new clips"),
    ("marco", "review notes"),
    ("bench", "nightly run"),
];

fn mail_body(f: &mut Frame, r: Rect, local: f32, a: f32) {
    let (x0, y0) = (r.0 + 12.0, r.1 + 40.0);
    let cols = ((r.2 - r.0 - 24.0) / 20.0).max(0.0) as usize;
    let sel = if local > 10.8 { 1 } else { 0 };
    for (i, (from, subj)) in MAIL.iter().enumerate() {
        let y = y0 + i as f32 * 30.0;
        if y + 24.0 > r.3 {
            break;
        }
        if i == sel {
            f.rect(
                r.0 as i32 + 4,
                y as i32 - 4,
                r.2 as i32 - 4,
                y as i32 + 22,
                0x2a2440,
                a,
            );
        }
        let line = format!("{from:<7}{subj}");
        let line: String = line.chars().take(cols).collect();
        let c = if i == sel { PAPER } else { TEXT };
        f.text(x0 as i32, y as i32, &line, 2, c, a);
    }
}

fn log_body(f: &mut Frame, r: Rect, local: f32, t: f32, a: f32) {
    let (x0, y0) = (r.0 + 12.0, r.1 + 40.0);
    let rows = ((r.3 - y0 - 8.0) / 26.0) as usize;
    let shown: Vec<&str> = LOG
        .iter()
        .filter(|(at, _)| local >= *at)
        .map(|(_, s)| *s)
        .collect();
    let skip = shown.len().saturating_sub(rows);
    for (i, line) in shown.iter().skip(skip).enumerate() {
        let c = if line.starts_with('>') {
            0xa78bfa
        } else {
            TEXT
        };
        f.text(x0 as i32, (y0 + i as f32 * 26.0) as i32, line, 2, c, a);
    }
    if (t * 2.5).fract() < 0.6 {
        let y = y0 + (shown.len() - skip) as f32 * 26.0;
        if y + 18.0 < r.3 {
            f.rect(
                x0 as i32,
                y as i32,
                x0 as i32 + 14,
                y as i32 + 18,
                0xa78bfa,
                a,
            );
        }
    }
}

/// The agent's pointer: where the log says it goes, with a ripple on click.
fn pointer(f: &mut Frame, planner: Rect, mail: Rect, local: f32, a: f32) {
    let appear = smooth((local - 8.4) / 0.4) * a;
    if appear <= 0.0 {
        return;
    }
    let row2 = (mail.0 + 160.0, mail.1 + 40.0 + 30.0 + 8.0);
    let gx0 = planner.0 + 12.0 + 170.0;
    let col = (planner.2 - 12.0 - gx0) / 5.0;
    let row = (planner.3 - 12.0 - (planner.1 + 66.0)) / 8.0;
    let slot = (gx0 + col * 3.5, planner.1 + 66.0 + row * 1.85);
    let keys = [
        (8.4, (640.0, 420.0)),
        (9.8, (640.0, 420.0)),
        (10.6, row2),
        (11.8, row2),
        (12.5, slot),
    ];
    let mut p = keys[keys.len() - 1].1;
    for w in keys.windows(2) {
        let ((t0, a0), (t1, b0)) = (w[0], w[1]);
        if local < t1 {
            let k = ease_out(((local - t0) / (t1 - t0)).clamp(0.0, 1.0));
            p = (a0.0 + (b0.0 - a0.0) * k, a0.1 + (b0.1 - a0.1) * k);
            break;
        }
    }
    for click in [10.8, 12.6] {
        let k = (local - click) / 0.5;
        if (0.0..1.0).contains(&k) {
            f.ring(p.0, p.1, 6.0 + 22.0 * k, 2.0, FOCUS, (1.0 - k) * appear);
        }
    }
    // An arrow pointer.
    let pts = [
        (0.0, 0.0),
        (0.0, 22.0),
        (6.0, 17.0),
        (11.0, 27.0),
        (15.0, 25.0),
        (10.0, 15.0),
        (17.0, 15.0),
    ];
    for y in 0..28 {
        for x in 0..18 {
            if inside(&pts, x as f32 + 0.5, y as f32 + 0.5) {
                f.blend(p.0 as i32 + x, p.1 as i32 + y, PAPER, appear);
            }
        }
    }
    for i in 0..pts.len() {
        let (a0, b0) = (pts[i], pts[(i + 1) % pts.len()]);
        f.line(
            p.0 + a0.0,
            p.1 + a0.1,
            p.0 + b0.0,
            p.1 + b0.1,
            1.2,
            0x000000,
            appear,
            1.0,
        );
    }
}

fn inside(poly: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + n - 1) % n]);
        if (a.1 > y) != (b.1 > y) && x < (b.0 - a.0) * (y - a.1) / (b.1 - a.1) + a.0 {
            c = !c;
        }
    }
    c
}

fn bar(f: &mut Frame, a: f32) {
    f.rect(0, 684, W as i32, 720, 0x08070d, a);
    for (i, ws) in ["1", "2", "3"].iter().enumerate() {
        let x = 24 + i as i32 * 36;
        let c = if i == 0 { FOCUS } else { BORDER };
        f.rect(
            x,
            690,
            x + 28,
            714,
            c,
            if i == 0 { 0.35 * a } else { 0.2 * a },
        );
        f.text(x + 4, 693, ws, 2, if i == 0 { PAPER } else { DIMTXT }, a);
    }
    f.text(1256 - 13 * 20, 693, "BERGAMO 09:41", 2, TEXT, a);
}
