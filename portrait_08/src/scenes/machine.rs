// MACHINE: his computer, as it is: a Sway desktop with a Waybar, tmux
// with three panes -- an editor on this demo's own source, a terminal
// running trex (his tmux session manager), another building this demo --
// and below it a mechanical keyboard whose keys go down with each letter
// typed on screen.
use super::typed;
use crate::gfx::{advance, Frame, W};
use crate::timeline::{smooth, Now};

const BG: u32 = 0x0b0d0c;
const PANE: u32 = 0x0f1211;
const BORDER: u32 = 0x2b3a31;
const GREENC: u32 = 0x8be28b;
const TEXT: u32 = 0xd4dcd6;
const DIM: u32 = 0x6b7a70;
const KEY: u32 = 0x1a1d1c;

/// Lines of the editor: strings_at from this demo's score.rs, as it is in
/// the repository (return type elided, indents shown at two spaces).
const CODE: [&str; 11] = [
    "pub fn strings_at(notes: &[Note], t: f32) -> ...",
    "  let mut out = [(0.0f32, 0.0f32); 6];",
    "  let end = notes.partition_point(|n| n.at <= t);",
    "  for n in notes[..end].iter().rev() {",
    "    let s = n.string.min(5);",
    "    if out[s].1 > 0.0 {",
    "      continue;",
    "    }",
    "    let since = t - n.at;",
    "    out[s] = (n.vel * decay(n, since), n.midi);",
    "  }",
];

/// What gets typed, where and when: (start s, chars a second, text, pane).
const TYPING: [(f32, f32, &str, usize); 4] = [
    (1.0, 10.0, "trex", 1),
    (6.0, 14.0, "cargo build --release", 2),
    (11.2, 16.0, "// one clock for picture and sound", 0),
    (15.6, 12.0, "make render", 2),
];

const SESSIONS: [&str; 5] = ["solverforge", "demoscene", "planner123", "org", "glyphfx"];

pub fn draw(f: &mut Frame, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.clear(BG);
    let l = n.local;
    waybar(f, l, fade);
    editor(f, l, t, fade);
    trex(f, l, t, fade);
    build(f, l, t, fade);
    tmux_status(f, fade);
    keyboard(f, l, fade);
    1.0
}

fn waybar(f: &mut Frame, _l: f32, a: f32) {
    f.rect(0, 400, W as i32, 444, 0x070908, a);
    for (i, ws) in ["1", "2", "3", "4"].iter().enumerate() {
        let x = 20 + i as i32 * 44;
        let on = i == 1;
        f.rect(
            x,
            406,
            x + 36,
            438,
            if on { GREENC } else { 0x1a201c },
            if on { 0.35 * a } else { a },
        );
        f.text(x + 8, 413, ws, 2, if on { 0xffffff } else { DIM }, a);
    }
    f.text(220, 413, "tmux: demoscene", 2, TEXT, a);
    f.text(
        W as i32 - 20 - 12 * advance(2),
        413,
        "SWAY  WAYLAND",
        2,
        DIM,
        a,
    );
}

fn pane(f: &mut Frame, x0: i32, y0: i32, x1: i32, y1: i32, active: bool, a: f32) {
    f.rect(x0, y0, x1, y1, PANE, a);
    f.frame_rect(x0, y0, x1, y1, if active { GREENC } else { BORDER }, a);
}

fn active_pane(l: f32) -> usize {
    TYPING
        .iter()
        .rev()
        .find(|(at, _, _, _)| l >= *at - 0.4)
        .map(|p| p.3)
        .unwrap_or(1)
}

fn editor(f: &mut Frame, l: f32, t: f32, a: f32) {
    let (x0, y0, x1, y1) = (10, 452, W as i32 - 10, 860);
    pane(f, x0, y0, x1, y1, active_pane(l) == 0, a);
    f.text(x0 + 14, y0 + 10, "src/score.rs", 2, DIM, a);
    let comment = TYPING[2];
    let typed_line = typed(comment.2, comment.0, comment.1, l);
    let insert_at = 8usize;
    let mut row = 0;
    for (i, line) in CODE.iter().enumerate() {
        if i == insert_at && !typed_line.is_empty() {
            draw_code_line(f, x0, y0, row, i, &format!("    {typed_line}"), true, a);
            if l < comment.0 + comment.2.len() as f32 / comment.1 + 0.3 && (t * 3.0).fract() < 0.6 {
                let cx = x0 + 70 + (4 + typed_line.chars().count()) as i32 * advance(2);
                f.rect(
                    cx,
                    y0 + 44 + row * 28,
                    cx + 14,
                    y0 + 44 + row * 28 + 20,
                    GREENC,
                    a,
                );
            }
            row += 1;
        }
        draw_code_line(f, x0, y0, row, i, line, false, a);
        row += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_code_line(
    f: &mut Frame,
    x0: i32,
    y0: i32,
    row: i32,
    _i: usize,
    line: &str,
    comment: bool,
    a: f32,
) {
    let y = y0 + 44 + row * 28;
    f.text(x0 + 14, y, &format!("{:>2}", row + 1), 2, 0x3f4a44, a);
    let x = x0 + 70;
    if comment {
        f.text(x, y, line, 2, 0x7f9a86, a);
        return;
    }
    // A little syntax colour: keywords, types, numbers.
    let mut cx = x;
    for word in split_keep(line) {
        let c = match word.trim() {
            "pub" | "fn" | "let" | "mut" | "for" | "in" | "if" | "continue" => 0xc792ea,
            "f32" | "Note" | "usize" => 0xffcb6b,
            w if w.chars().next().is_some_and(|ch| ch.is_ascii_digit()) => 0xf78c6c,
            _ => TEXT,
        };
        f.text(cx, y, word, 2, c, a);
        cx += word.chars().count() as i32 * advance(2);
    }
}

/// Split into words and the separators between them, keeping both.
fn split_keep(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        let w = bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.';
        let next_w = i + 1 < bytes.len()
            && (bytes[i + 1].is_ascii_alphanumeric()
                || bytes[i + 1] == b'_'
                || bytes[i + 1] == b'.');
        if w != next_w || i + 1 == bytes.len() {
            out.push(&s[start..=i]);
            start = i + 1;
        }
    }
    out
}

fn trex(f: &mut Frame, l: f32, t: f32, a: f32) {
    let (x0, y0, x1, y1) = (10, 868, 535, 1230);
    pane(f, x0, y0, x1, y1, active_pane(l) == 1, a);
    let (at, cps, cmd, _) = TYPING[0];
    f.text(
        x0 + 14,
        y0 + 16,
        &format!("$ {}", typed(cmd, at, cps, l)),
        2,
        GREENC,
        a,
    );
    let open = at + cmd.len() as f32 / cps + 0.4;
    if l < open {
        return;
    }
    f.text(x0 + 14, y0 + 56, "TREX  SESSIONS", 2, DIM, a);
    // The selection walks down to "demoscene" and opens it.
    let sel = if l < open + 0.8 { 0 } else { 1 };
    for (i, s) in SESSIONS.iter().enumerate() {
        let y = y0 + 96 + i as i32 * 40;
        if i == sel {
            f.rect(x0 + 8, y - 6, x1 - 8, y + 26, 0x1d3a26, a);
        }
        f.text(x0 + 24, y, s, 2, if i == sel { 0xffffff } else { TEXT }, a);
        f.text(x1 - 120, y, &format!("{} win", 1 + (i * 3) % 4), 2, DIM, a);
    }
    if l > open + 1.4 {
        f.text(
            x0 + 14,
            y1 - 40,
            "attached: demoscene",
            2,
            GREENC,
            a * smooth((l - open - 1.4) / 0.3),
        );
    }
    let _ = t;
}

fn build(f: &mut Frame, l: f32, t: f32, a: f32) {
    let (x0, y0, x1, y1) = (545, 868, W as i32 - 10, 1230);
    pane(f, x0, y0, x1, y1, active_pane(l) == 2, a);
    let mut y = y0 + 16;
    let line = |f: &mut Frame, y: &mut i32, s: &str, c: u32| {
        f.text(x0 + 14, *y, s, 2, c, a);
        *y += 32;
    };
    let (at, cps, cmd, _) = TYPING[1];
    line(f, &mut y, &format!("$ {}", typed(cmd, at, cps, l)), GREENC);
    let done = at + cmd.len() as f32 / cps + 0.3;
    let out = [
        (0.0, "  Compiling demofx", TEXT),
        (0.8, "  Compiling portrait-08", TEXT),
        (2.6, "   Finished release", GREENC),
    ];
    for (dt, s, c) in out {
        if l > done + dt {
            line(f, &mut y, s, c);
        }
    }
    let (at2, cps2, cmd2, _) = TYPING[3];
    if l > at2 - 0.4 {
        line(
            f,
            &mut y,
            &format!("$ {}", typed(cmd2, at2, cps2, l)),
            GREENC,
        );
        if l > at2 + cmd2.len() as f32 / cps2 + 0.3 {
            line(f, &mut y, "[render] 1080x1920", TEXT);
            line(f, &mut y, "  @ 60fps -> ffmpeg", TEXT);
        }
    }
    if (t * 3.0).fract() < 0.6 {
        f.rect(x0 + 14, y, x0 + 28, y + 20, GREENC, 0.7 * a);
    }
    let _ = x1;
    let _ = y1;
}

fn tmux_status(f: &mut Frame, a: f32) {
    f.rect(10, 1236, W as i32 - 10, 1266, 0x1d3a26, a);
    f.text(
        20,
        1241,
        "[demoscene] 0:nvim 1:trex 2:cargo",
        2,
        0xdfffe0,
        a,
    );
}

const ROWS: [&str; 4] = ["1234567890-", "QWERTYUIOP", "ASDFGHJKL;", "ZXCVBNM,./"];

/// The key being pressed at time l, if any: the last letter typed within
/// the last tenth of a second.
fn pressed(l: f32) -> Option<char> {
    for &(at, cps, text, _) in &TYPING {
        let k = ((l - at) * cps).floor();
        if k >= 0.0 && (k as usize) < text.len() {
            let since = (l - at) * cps - k;
            if since < 0.8 {
                return text.chars().nth(k as usize).map(|c| c.to_ascii_uppercase());
            }
        }
    }
    None
}

fn keyboard(f: &mut Frame, l: f32, a: f32) {
    let down = pressed(l);
    let (x0, y0) = (40, 1310);
    f.rect(
        x0 - 20,
        y0 - 18,
        W as i32 - 20,
        y0 + 5 * 60 + 8,
        0x0d0f0e,
        a,
    );
    f.frame_rect(
        x0 - 20,
        y0 - 18,
        W as i32 - 20,
        y0 + 5 * 60 + 8,
        0x26302a,
        a,
    );
    for (r, row) in ROWS.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            let x = x0 + r as i32 * 22 + c as i32 * 88;
            let y = y0 + r as i32 * 60;
            key(f, x, y, 78, ch, down == Some(ch), a);
        }
    }
    // Space bar.
    let y = y0 + 4 * 60;
    key(f, x0 + 220, y, 520, ' ', down == Some(' '), a);
}

fn key(f: &mut Frame, x: i32, y: i32, w: i32, ch: char, down: bool, a: f32) {
    let dy = if down { 6 } else { 0 };
    f.rect(x, y + 6, x + w, y + 54, 0x050606, a);
    f.rect(
        x,
        y + dy,
        x + w,
        y + 48 + dy,
        if down { 0x2c4a34 } else { KEY },
        a,
    );
    f.frame_rect(
        x,
        y + dy,
        x + w,
        y + 48 + dy,
        if down { GREENC } else { 0x2e3431 },
        a,
    );
    if ch != ' ' {
        f.char_at(
            x + 12,
            y + 6 + dy,
            ch,
            2,
            if down { 0xffffff } else { 0x9aa49e },
            a,
        );
    }
    if down {
        f.glow((x + w / 2) as f32, (y + 24) as f32, 60.0, GREENC, 0.25 * a);
    }
}
