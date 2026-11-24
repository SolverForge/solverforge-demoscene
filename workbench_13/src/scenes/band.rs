// The band: the 160 rows under the picture, the same from the first take to
// the last native scene. Left, the Nous Girl's place (the compose step lays
// her sprite in at PORTRAIT); in the middle, her words, typed as she says
// them; right, the chapter's title (a story card) and the mark. Along its
// top edge runs the film's rail: one segment per chapter, the past ones
// jade, the current one lit with a glint at how far it has played.
use super::{JADE, PARCH, PICTURE_H, SAGE};
use crate::cut::{SECTIONS, SPEECH};
use crate::gfx::{advance, Frame, H, W};
use crate::timeline::{span, Kind, Now};

/// Where the compose step lays the Nous Girl: x, y, width, height.
pub const PORTRAIT: (i32, i32, i32, i32) = (40, 936, 150, 132);
const WORDS_X: i32 = 232;
const WORDS_W: i32 = 1180;
const LINE_H: i32 = 28;
const SCALE: i32 = 2;
const GROUND: u32 = 0x050d0b;

pub fn draw(f: &mut Frame, n: Now, t: f32, _masks: &mut [(char, demofx::font::Mask)]) {
    // Opens with the first take, closes with the last native scene.
    let first = SECTIONS.iter().position(|s| s.0 != Kind::Intro).unwrap();
    let last = SECTIONS.iter().rposition(|s| s.0 != Kind::Outro).unwrap();
    let a = if n.index == first {
        span(n.local, 0.0, 0.5)
    } else if n.index == last {
        1.0 - span(n.local, n.len - 0.5, n.len)
    } else {
        1.0
    };
    f.rect(0, PICTURE_H, W as i32, H as i32, GROUND, a);
    rail(f, t, first, last, a);
    f.rect(214, 940, 215, 1064, JADE, 0.3 * a);
    f.rect(1424, 940, 1425, 1064, JADE, 0.3 * a);
    let (px, py, pw, ph) = PORTRAIT;
    corners(f, px - 8, py - 4, px + pw + 8, py + ph + 4, JADE, 0.7 * a);

    // The chapter count under its title.
    let count = format!("{:02} / {:02}", n.index - first + 1, last - first + 1);
    f.text(
        1440 + 180 - count.len() as i32 * advance(1) / 2,
        1044,
        &count,
        1,
        SAGE,
        a,
    );
    demofx::logo::corner_at(f, (1862.0, 1000.0), t, a);

    words(f, t, a);
}

/// The rail: chapter segments along the band's top edge.
fn rail(f: &mut Frame, t: f32, first: usize, last: usize, a: f32) {
    let (t0, t1) = (SECTIONS[first].2, SECTIONS[last].3);
    let x_of = |s: f32| ((s - t0) / (t1 - t0) * W as f32) as i32;
    for s in &SECTIONS[first..=last] {
        let (x0, x1) = (x_of(s.2) + 1, x_of(s.3) - 1);
        let rgb = if t >= s.3 {
            JADE
        } else if t >= s.2 {
            PARCH
        } else {
            0x16302a
        };
        let k = if t >= s.2 && t < s.3 { 1.0 } else { 0.6 };
        f.rect(x0, PICTURE_H, x1, PICTURE_H + 3, rgb, a * k);
    }
    let x = x_of(t.clamp(t0, t1)) as f32;
    f.glow(x, PICTURE_H as f32 + 1.0, 60.0, PARCH, 0.5 * a);
    f.rect(
        x as i32 - 1,
        PICTURE_H,
        x as i32 + 2,
        PICTURE_H + 5,
        PARCH,
        a,
    );
}

/// The line being said, typed at the pace it is spoken, the newest letters
/// brightest; it holds a moment after she stops and clears before the next.
fn words(f: &mut Frame, t: f32, a: f32) {
    let Some(k) = SPEECH.iter().rposition(|s| s.1 <= t) else {
        return;
    };
    let (text, t0, t1) = SPEECH[k];
    let next = SPEECH.get(k + 1).map_or(f32::MAX, |s| s.1);
    let hold_end = (t1 + 1.6).min(next - 0.15);
    if t >= hold_end {
        return;
    }
    let fade = 1.0 - span(t, hold_end - 0.3, hold_end);
    let typed =
        (text.chars().count() as f32 * ((t - t0) / (t1 - t0 - 0.15).max(0.1)).min(1.0)) as usize;
    let lines = wrap(text, (WORDS_W / advance(SCALE)) as usize);
    let total_lines = lines.len() as i32;
    let y0 = 1000 - total_lines * LINE_H / 2;
    let mut seen = 0usize;
    for (row, line) in lines.iter().enumerate() {
        let y = y0 + row as i32 * LINE_H;
        for (col, ch) in line.chars().enumerate() {
            if seen >= typed {
                // The cursor: a block where the next letter goes.
                if seen == typed && ((t * 4.0) as i32) % 2 == 0 {
                    let x = WORDS_X + col as i32 * advance(SCALE);
                    f.rect(x, y + 2, x + advance(SCALE) - 4, y + 18, JADE, a * fade);
                }
                return;
            }
            let age = typed - seen;
            let rgb = if age <= 2 { 0xffffff } else { PARCH };
            let x = WORDS_X + col as i32 * advance(SCALE);
            f.char_at(x, y, ch, SCALE, rgb, a * fade);
            if age <= 1 {
                f.glow(x as f32 + 10.0, y as f32 + 9.0, 18.0, JADE, 0.25 * a * fade);
            }
            seen += 1;
        }
        seen += 1; // the space the wrap took
    }
}

fn wrap(text: &str, cols: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        if !line.is_empty() && line.len() + 1 + word.len() > cols {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn corners(f: &mut Frame, x0: i32, y0: i32, x1: i32, y1: i32, rgb: u32, a: f32) {
    let l = 18;
    f.rect(x0, y0, x0 + l, y0 + 2, rgb, a);
    f.rect(x0, y0, x0 + 2, y0 + l, rgb, a);
    f.rect(x1 - l, y0, x1, y0 + 2, rgb, a);
    f.rect(x1 - 2, y0, x1, y0 + l, rgb, a);
    f.rect(x0, y1 - 2, x0 + l, y1, rgb, a);
    f.rect(x0, y1 - l, x0 + 2, y1, rgb, a);
    f.rect(x1 - l, y1 - 2, x1, y1, rgb, a);
    f.rect(x1 - 2, y1 - l, x1, y1, rgb, a);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_fits_the_band() {
        let cols = (WORDS_W / advance(SCALE)) as usize;
        for (text, ..) in SPEECH {
            let lines = wrap(text, cols);
            assert!(lines.len() <= 4, "{text}");
            assert!(lines.iter().all(|l| l.len() <= cols));
        }
    }
}
