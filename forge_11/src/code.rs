// Rust excerpts in the editor: layout, syntax colours and the glyphfx
// entrance. The excerpt is rendered by glyphfx on a canvas exactly its own
// size, so the settled clip and the highlighted listing share every cell.
use demofx::{Clip, ClipSpec};

use crate::capture::Code;
use crate::gfx::{size, Size};

/// The listing's text area inside the editor panel.
pub const CODE_BOX: (i32, i32, i32, i32) = (712, 250, 1840, 950);
/// Columns reserved for the line-number gutter.
pub const GUTTER: usize = 4;

pub const EFFECTS: [&str; 6] = ["decrypt", "beams", "sweep", "burn", "slide", "middleout"];
const HOT_STOPS: &[&str] = &["--final-gradient-stops", "ff7a00", "ffd88a"];

pub struct Listing {
    pub lines: Vec<String>,
    pub size: Size,
    pub pitch: i32,
    pub clip: Clip,
}

/// Cell size and line pitch for an excerpt: the largest that fits.
pub fn fit(lines: &[String]) -> (Size, i32) {
    let (x0, y0, x1, y1) = CODE_BOX;
    let cols = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) + GUTTER;
    let rows = lines.len().max(1) as i32;
    for s in [
        size(18, 32),
        size(16, 30),
        size(14, 26),
        size(13, 24),
        size(12, 22),
        size(11, 20),
        size(10, 18),
    ] {
        let pitch = s.h + s.h / 4;
        if cols as i32 * s.w <= x1 - x0 && rows * pitch <= y1 - y0 {
            return (s, pitch);
        }
    }
    (size(10, 18), 20)
}

impl Listing {
    pub fn new(code: &Code, i: usize) -> Self {
        // Lines wider than the editor at 1x are clipped (marked '>').
        let max_cols = ((CODE_BOX.2 - CODE_BOX.0) / 10) as usize - GUTTER;
        let lines: Vec<String> = code
            .lines
            .iter()
            .map(|l| crate::gfx::clip(l.trim_end(), max_cols))
            .collect();
        let (size, pitch) = fit(&lines);
        let cols = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);
        let clip = Clip::render(&ClipSpec {
            effect: EFFECTS[i % EFFECTS.len()],
            text: &lines.join("\n"),
            cols,
            rows: lines.len(),
            seed: 1100 + i as u64,
            opts: HOT_STOPS,
        });
        Self {
            lines,
            size,
            pitch,
            clip,
        }
    }
}

pub const KEYWORD: u32 = 0x50fc00;
pub const TYPE: u32 = 0x6ee7b7;
pub const STRING: u32 = 0xffc070;
pub const NUMBER: u32 = 0x8fd0ff;
pub const COMMENT: u32 = 0x5f7f70;
pub const PLAIN: u32 = 0xe4f2ea;
pub const PUNCT: u32 = 0xa0b4aa;

const KEYWORDS: [&str; 28] = [
    "fn", "pub", "let", "use", "impl", "struct", "enum", "for", "if", "else", "match", "return",
    "mut", "move", "self", "Self", "where", "mod", "const", "static", "crate", "super", "as", "in",
    "true", "false", "dyn", "ref",
];

/// Colour of every character of a Rust line, by a small lexer: comments,
/// strings, numbers, keywords, capitalised types, punctuation.
pub fn colours(line: &str) -> Vec<u32> {
    let ch: Vec<char> = line.chars().collect();
    let mut out = vec![PLAIN; ch.len()];
    let mut i = 0;
    while i < ch.len() {
        let c = ch[i];
        if c == '/' && ch.get(i + 1) == Some(&'/') {
            out[i..].fill(COMMENT);
            break;
        }
        if c == '"' {
            let start = i;
            i += 1;
            while i < ch.len() && ch[i] != '"' {
                if ch[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            let end = (i + 1).min(ch.len());
            out[start..end].fill(STRING);
            i = end;
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            while i < ch.len() && (ch[i].is_ascii_alphanumeric() || ch[i] == '_' || ch[i] == '.') {
                i += 1;
            }
            out[start..i].fill(NUMBER);
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < ch.len() && (ch[i].is_alphanumeric() || ch[i] == '_') {
                i += 1;
            }
            let word: String = ch[start..i].iter().collect();
            let col = if KEYWORDS.contains(&word.as_str()) {
                KEYWORD
            } else if word.chars().next().is_some_and(char::is_uppercase) {
                TYPE
            } else {
                PLAIN
            };
            out[start..i].fill(col);
            continue;
        }
        if !c.is_whitespace() {
            out[i] = PUNCT;
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::Capture;

    #[test]
    fn settled_clip_matches_the_listing_cell_for_cell() {
        let mut caps =
            vec![Capture::parse(include_str!("../assets/capture.placeholder.json")).unwrap()];
        let real = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");
        if std::path::Path::new(real).exists() {
            caps.push(Capture::load(real).unwrap());
        }
        for (i, code) in caps.iter().flat_map(|c| c.code.iter()).enumerate() {
            let l = Listing::new(code, i);
            let mut seen = 0;
            for (col, row, cell) in l.clip.final_cells() {
                let line: Vec<char> = l.lines[row as usize].chars().collect();
                assert_eq!(
                    line.get(col as usize),
                    Some(&cell.ch),
                    "excerpt {i} row {row} col {col}"
                );
                seen += 1;
            }
            let total: usize = l
                .lines
                .iter()
                .map(|s| s.chars().filter(|c| *c != ' ').count())
                .sum();
            assert_eq!(seen, total, "every character of excerpt {i} settles");
        }
    }

    #[test]
    fn highlighter_marks_comments_strings_and_keywords() {
        let c = colours("pub fn x() { \"s\" } // note");
        assert_eq!(c[0], KEYWORD);
        assert_eq!(c[13], STRING);
        assert_eq!(*c.last().unwrap(), COMMENT);
    }
}
