// 01 DESCRIBE: the plain-language problem, typed to the coding agent word
// by word on the sixteenth-note grid. The buses in the capture's plan drive
// past on the river road below.
use super::{cursor, AMBER, CYAN, DIM, EMERALD, LIME, MINT, PAPER};
use crate::board::Board;
use crate::capture::Capture;
use crate::fx;
use crate::gfx::{mix, smooth, Frame, S1, S2, W};
use crate::script::{Script, PROMPT_BOX};
use crate::timeline::Now;

const DOMAIN: [&str; 22] = [
    "bus",
    "buses",
    "technician",
    "technicians",
    "skill",
    "skills",
    "part",
    "parts",
    "workshop",
    "due",
    "day",
    "days",
    "sick",
    "maintenance",
    "job",
    "jobs",
    "depot",
    "week",
    "mechanic",
    "mechanics",
    "shift",
    "shifts",
];
const NUMBERS: [&str; 12] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "twelve",
    "twenty",
];

/// Colour per character: domain words in lime, quantities in amber.
fn word_colours(line: &str) -> Vec<u32> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = vec![PAPER; chars.len()];
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_alphanumeric() {
            let s = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-') {
                i += 1;
            }
            let w: String = chars[s..i].iter().collect::<String>().to_lowercase();
            let col = if DOMAIN.contains(&w.as_str()) {
                LIME
            } else if NUMBERS.contains(&w.as_str()) || w.chars().all(|c| c.is_ascii_digit()) {
                AMBER
            } else {
                PAPER
            };
            out[s..i].fill(col);
        } else {
            i += 1;
        }
    }
    out
}

pub fn draw(f: &mut Frame, c: &Capture, s: &Script, board: &Board, t: f32, n: Now) {
    f.gradient(&[0x010306, 0x04101a, 0x061820, 0x020406]);
    fx::copper(f, t, 140.0, 1000.0, &[EMERALD, 0x0ea5a4, LIME], 4, 0.22);
    fx::river(f, t, 978, 5, &[0x041820, 0x1f6f8b, CYAN, 0xffffff], 0.9);
    buses(f, board, t, n.local);

    // Behind the opener only the road runs.
    let open = smooth((t - s.intro_end(crate::timeline::Section::Describe)) / 0.5);
    if open <= 0.0 {
        return;
    }
    let (x0, y0, x1, y1) = (170, 200, 1750, 880);
    fx::panel(f, x0, y0, x1, y1, EMERALD, open);
    if open < 0.5 {
        return;
    }
    // The header, read alone before the prompt types; it then steps
    // back so the prompt is what the eye follows.
    let head = crate::script::prompt_header(c);
    let settle = mix(
        MINT,
        DIM,
        smooth((t - s.typing.words.first().map_or(t, |w| w.0)) / 0.6),
    );
    let hs = if head.len() as i32 * S2.w <= x1 - x0 - 60 {
        S2
    } else {
        S1
    };
    let k = smooth((t - s.header) / 0.3) * open;
    f.text(x0 + 30, y0 + 40, &head, hs, settle, k);
    f.rect(x0 + 20, y0 + 90, x1 - 20, y0 + 91, EMERALD, 0.4 * open);

    let (bx, by, _, by1) = PROMPT_BOX;
    let size = s.prompt_size;
    let pitch = size.h + 4;
    let shown = s.typing.chars_at(t);
    // Lines that fit; if the prompt is taller than the box, it scrolls.
    let fit = ((by1 - by) / pitch).max(1) as usize;
    let mut left = shown;
    let mut rows: Vec<(String, Vec<u32>)> = Vec::new();
    for line in &s.prompt_lines {
        let n = line.chars().count();
        if left == 0 && !rows.is_empty() {
            break;
        }
        let k = left.min(n);
        left -= k;
        rows.push((line.chars().take(k).collect(), word_colours(line)));
    }
    let skip = rows.len().saturating_sub(fit);
    let mut cy = by;
    let mut end = (bx, by);
    for (txt, cols) in rows.iter().skip(skip) {
        for (i, ch) in txt.chars().enumerate() {
            f.char_at(bx + i as i32 * size.w, cy, ch, size, cols[i], 1.0);
        }
        end = (bx + txt.chars().count() as i32 * size.w, cy);
        cy += pitch;
    }
    cursor(f, end.0, end.1, size, LIME, t);
}

/// The plan's buses, one textmode sprite per bus, driving left to right in
/// convoy along the river road. Windows instead of names: moving words
/// would pull the eye off the prompt.
fn buses(f: &mut Frame, board: &Board, t: f32, local: f32) {
    let mut names: Vec<&str> = Vec::new();
    for b in &board.blocks {
        if !names.contains(&b.bus.as_str()) {
            names.push(&b.bus);
        }
    }
    let width = 22 * S1.w;
    let gap = 60;
    let lap = (names.len() as i32 * (width + gap)).max(W as i32 + width);
    let y = 908;
    for i in 0..names.len() {
        let x = ((local * 170.0) as i32 + i as i32 * (width + gap)).rem_euclid(lap) - width;
        let bob = if ((t * 6.0) as i32 + i as i32) % 2 == 0 {
            0
        } else {
            1
        };
        let rows = [
            " .------------------.",
            " | [] [] [] [] [] []|",
            " '-(o)----------(o)-'",
        ];
        for (r, row) in rows.iter().enumerate() {
            let col = CYAN;
            f.text(x, y + r as i32 * S1.h - bob, row, S1, col, 0.9);
        }
    }
}
