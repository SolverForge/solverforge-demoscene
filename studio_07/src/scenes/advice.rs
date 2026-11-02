// ADVICE: offer two, on blueprint paper: advisory for any software project.
// The five steps of an engagement light up in turn, and the sketch of a
// system below them sharpens from a wobbly draft into clean lines.
use super::{typed, PAPER};
use crate::gfx::{hash, Frame, H, W};
use crate::timeline::{smooth, Now};

const PAPER_BLUE: u32 = 0x061226;
const MINOR: u32 = 0x0b2142;
const MAJOR: u32 = 0x12305c;
const LINE: u32 = 0x7dd3fc;
const DIM: u32 = 0x2b5d8a;

const STEPS: [(&str, &str); 5] = [
    ("DISCOVERY", "IS IT WORTH BUILDING, AND IN WHAT SHAPE"),
    ("DESIGN", "LANGUAGES, RUNTIMES, SERVICES, DATA"),
    ("REVIEW", "CODE, APIS, PROTOTYPES, MIGRATIONS"),
    ("TUNING", "WHAT IS SLOW, FRAGILE OR EXPENSIVE, AND WHY"),
    ("NEXT STEP", "A SHARPER NEXT STEP, BEFORE EXPENSIVE CODE"),
];
const FIRST: f32 = 1.4;
const EACH: f32 = 2.6;

const BOX_W: i32 = 200;
const GAP: i32 = 40;
const ROW_Y: i32 = 230;
const BOX_H: i32 = 70;

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.clear(PAPER_BLUE);
    for x in (0..W as i32).step_by(32) {
        let c = if x % 128 == 0 { MAJOR } else { MINOR };
        f.rect(x, 0, x + 1, H as i32, c, fade);
    }
    for y in (0..H as i32).step_by(32) {
        let c = if y % 128 == 0 { MAJOR } else { MINOR };
        f.rect(0, y, W as i32, y + 1, c, fade);
    }

    let stage = (n.local - FIRST) / EACH;
    let x0 = (W as i32 - (5 * BOX_W + 4 * GAP)) / 2;
    for (i, (name, _)) in STEPS.iter().enumerate() {
        let bx = x0 + i as i32 * (BOX_W + GAP);
        let draw_in = smooth((n.local - 0.2 - i as f32 * 0.25) / 0.9);
        let lit = smooth((stage - i as f32) * 3.0);
        let active = (stage - i as f32).clamp(0.0, 1.0) < 1.0 && stage >= i as f32;
        let c = if lit > 0.0 { LINE } else { DIM };
        traced_box(f, bx, ROW_Y, BOX_W, BOX_H, c, fade, draw_in);
        if lit > 0.0 {
            f.rect(
                bx + 1,
                ROW_Y + 1,
                bx + BOX_W - 1,
                ROW_Y + BOX_H - 1,
                LINE,
                0.12 * lit * fade,
            );
        }
        if active {
            let pulse = 0.5 + 0.5 * (t * 6.0).sin();
            f.frame_rect(
                bx - 4,
                ROW_Y - 4,
                bx + BOX_W + 4,
                ROW_Y + BOX_H + 4,
                LINE,
                0.4 * pulse * fade,
            );
        }
        let label = mix_label(lit);
        f.text_centered(bx + BOX_W / 2, ROW_Y + 26, name, 2, label, draw_in * fade);
        // Step number tick above the box, as on a drawing.
        f.text(
            bx,
            ROW_Y - 22,
            &format!("{}", i + 1),
            1,
            DIM,
            draw_in * fade,
        );

        if i + 1 < STEPS.len() {
            let (ax0, ax1, ay) = (bx + BOX_W + 4, bx + BOX_W + GAP - 4, ROW_Y + BOX_H / 2);
            let go = smooth((stage - i as f32 - 0.6) * 2.5);
            let c = if go > 0.0 { LINE } else { DIM };
            f.line(
                ax0 as f32,
                ay as f32,
                ax1 as f32,
                ay as f32,
                2.0,
                c,
                draw_in * fade,
                1.0,
            );
            f.line(
                ax1 as f32,
                ay as f32,
                ax1 as f32 - 7.0,
                ay as f32 - 6.0,
                2.0,
                c,
                draw_in * fade,
                1.0,
            );
            f.line(
                ax1 as f32,
                ay as f32,
                ax1 as f32 - 7.0,
                ay as f32 + 6.0,
                2.0,
                c,
                draw_in * fade,
                1.0,
            );
            // A pulse runs along the arrow as the next step lights.
            let k = (stage - i as f32 - 0.6) * 2.5;
            if (0.0..1.0).contains(&k) {
                let px = ax0 as f32 + (ax1 - ax0) as f32 * k;
                f.glow(px, ay as f32, 14.0, 0xffffff, 0.9 * fade);
            }
        }
    }

    // What the current step covers.
    let idx = stage.floor().clamp(0.0, 4.0) as usize;
    if stage >= 0.0 {
        let since = n.local - FIRST - idx as f32 * EACH;
        let text = typed(STEPS[idx].1, 0.0, 40.0, since);
        f.text_centered(640, ROW_Y + BOX_H + 50, &text, 2, PAPER, 0.9 * fade);
    }

    // The sketch below sharpens as the steps go by.
    let clarity = smooth(stage / 5.0);
    sketch(f, clarity, t, fade * smooth((n.local - 0.8) / 1.0));
    1.0
}

fn mix_label(lit: f32) -> u32 {
    crate::gfx::mix(DIM, 0xffffff, 0.35 + 0.65 * lit)
}

#[allow(clippy::too_many_arguments)]
fn traced_box(f: &mut Frame, x: i32, y: i32, w: i32, h: i32, c: u32, a: f32, k: f32) {
    let (x0, y0, x1, y1) = (x as f32, y as f32, (x + w) as f32, (y + h) as f32);
    let sides = [
        (x0, y0, x1, y0),
        (x1, y0, x1, y1),
        (x1, y1, x0, y1),
        (x0, y1, x0, y0),
    ];
    for (i, s) in sides.iter().enumerate() {
        let part = (k * 4.0 - i as f32).clamp(0.0, 1.0);
        f.line(s.0, s.1, s.2, s.3, 1.6, c, a, part);
    }
}

/// A small system drawing: client, API, queue, worker, data. Wobbly
/// and dashed while the problem is unclear, straight and solid once it is.
fn sketch(f: &mut Frame, clarity: f32, t: f32, a: f32) {
    if a <= 0.0 {
        return;
    }
    let nodes = [
        ("CLIENT", 170, 520),
        ("API", 420, 470),
        ("QUEUE", 420, 600),
        ("WORKER", 700, 535),
        ("DATA", 990, 535),
    ];
    let edges = [(0, 1), (1, 2), (2, 3), (1, 3), (3, 4)];
    let wob = (1.0 - clarity) * 9.0;
    let jitter = |i: i32, k: i32| -> (f32, f32) {
        let ph = t * 2.0 + i as f32;
        (
            wob * (hash(i, k) - 0.5) * 2.0 + wob * 0.3 * ph.sin(),
            wob * (hash(i, k + 7) - 0.5) * 2.0 + wob * 0.3 * ph.cos(),
        )
    };
    for (e, &(i, j)) in edges.iter().enumerate() {
        let (ax, ay) = (nodes[i].1 as f32 + 60.0, nodes[i].2 as f32 + 20.0);
        let (bx, by) = (nodes[j].1 as f32 + 60.0, nodes[j].2 as f32 + 20.0);
        let segs = 14;
        for s in 0..segs {
            // Dashes close up into a solid line as clarity grows.
            let on = clarity > 0.85 || s % 2 == 0;
            if !on {
                continue;
            }
            let (k0, k1) = (s as f32 / segs as f32, (s + 1) as f32 / segs as f32);
            let (j0, j1) = (
                jitter(e as i32 * 31 + s, 1),
                jitter(e as i32 * 31 + s + 1, 1),
            );
            f.line(
                ax + (bx - ax) * k0 + j0.0,
                ay + (by - ay) * k0 + j0.1,
                ax + (bx - ax) * k1 + j1.0,
                ay + (by - ay) * k1 + j1.1,
                1.4,
                LINE,
                (0.35 + 0.5 * clarity) * a,
                1.0,
            );
        }
    }
    for (i, (name, x, y)) in nodes.iter().enumerate() {
        let (jx, jy) = jitter(i as i32, 3);
        let (x, y) = (*x + jx as i32, *y + jy as i32);
        f.rect(x, y, x + 120, y + 40, PAPER_BLUE, a);
        traced_box(f, x, y, 120, 40, LINE, (0.4 + 0.6 * clarity) * a, 1.0);
        f.text_centered(x + 60, y + 11, name, 2, PAPER, (0.5 + 0.5 * clarity) * a);
    }
}
