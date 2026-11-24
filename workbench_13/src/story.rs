// The frame's words, all in one table, each brought in by the glyphfx effect
// that belongs to its moment. The band titles every chapter; the native
// scenes title themselves and name their lanes; the call and the close say
// the few things the narration leaves to the picture.
use demofx::{card, CardSpec, Cards, Place};

use crate::cut::SECTIONS;
use crate::scenes::board::{LANE_H, LANE_Y0, PEOPLE_GAP};

const fn from(i: usize) -> f32 {
    SECTIONS[i].2
}
const fn to(i: usize) -> f32 {
    SECTIONS[i].3
}

/// The band's chapter title, right of the words.
const BAND: Place = Place::new(1440, 963, 360, 18, 34);
/// A native scene's own title, top left of the picture.
const HEAD: Place = Place::new(36, 40, 600, 30, 54);
/// The intro's title and its line under the panels.
const TITLE: Place = Place::new(0, 96, 1920, 40, 72);
const UNDER: Place = Place::new(0, 846, 1920, 20, 36);
/// The call's words, left of Cordelia.
const CALL: Place = Place::new(60, 300, 900, 26, 48);
/// The close.
const CLOSE: Place = Place::new(60, 560, 940, 26, 48);
/// The lane names, left of the board: machines, then people.
const MACHINES: Place = Place::new(28, LANE_Y0, 266, 14, LANE_H as usize);
const PEOPLE: Place = Place::new(
    28,
    LANE_Y0 + 16 * LANE_H + PEOPLE_GAP,
    266,
    14,
    LANE_H as usize,
);

const BEAM: &[&str] = &["--final-gradient-stops", "3dbb8a", "efeae0"];
const PARCHMENT: &[&str] = &["--final-gradient-stops", "8fa79c", "efeae0"];
const JADE: &[&str] = &["--final-gradient-stops", "3dbb8a", "a7e8cc"];
const RED: &[&str] = &["--final-gradient-stops", "dc4f42", "efeae0"];
const COPPER: &[&str] = &[
    "--bar-colors",
    "3dbb8a",
    "4a9bd1",
    "efeae0",
    "--unlit-color",
    "1d3a30",
    "--sweep-color",
    "efeae0",
    "--final-gradient-stops",
    "8fa79c",
    "efeae0",
];
const COMET: &[&str] = &[
    "--comet-color",
    "4a9bd1",
    "--final-gradient-stops",
    "3dbb8a",
    "efeae0",
];
const WAVE: &[&str] = &[
    "--highlight-color",
    "efeae0",
    "--final-gradient-stops",
    "3dbb8a",
    "efeae0",
];
/// The intro's banner: the title in the series' own font, every font pixel a
/// block, so rotozoom can turn it as a picture.
pub const BANNER_LINES: [&str; 2] = ["SOLVERFORGE", "WORKBENCH"];
pub const BANNER_COLS: usize = 104;
pub const BANNER_ROWS: usize = 20;
pub const BANNER: Place = Place::new(24, 270, 1872, 18, 18);
const ROTO: &[&str] = &[
    "--turns",
    "1.25",
    "--final-gradient-stops",
    "3dbb8a",
    "efeae0",
    "3dbb8a",
];

pub fn banner() -> demofx::Clip {
    let mut text = String::new();
    for (i, line) in BANNER_LINES.iter().enumerate() {
        if i > 0 {
            text.push('\n');
        }
        for row in 0..8 {
            for ch in line.chars() {
                let m = demofx::font::mask(ch);
                let bits = m[1 + row * 2];
                for col in 1..9 {
                    text.push(if (bits >> col) & 1 == 1 { '█' } else { ' ' });
                }
                text.push(' ');
            }
            text.push('\n');
        }
    }
    demofx::Clip::render(&demofx::ClipSpec {
        effect: "rotozoom",
        text: text.trim_end_matches('\n'),
        cols: BANNER_COLS,
        rows: BANNER_ROWS,
        seed: 7,
        opts: ROTO,
    })
}
const STORM: &[&str] = &[
    "--lightning-color",
    "dc4f42",
    "--glowing-text-color",
    "efeae0",
    "--storm-time",
    "2",
    "--final-gradient-stops",
    "dc4f42",
    "efeae0",
];
const ETCH: &[&str] = &[
    "--laser-gradient-stops",
    "ffffff",
    "3dbb8a",
    "--spark-gradient-stops",
    "efeae0",
    "3dbb8a",
    "--final-gradient-stops",
    "3dbb8a",
    "efeae0",
];

/// A chapter's title in the band, up while the chapter plays.
const fn band(i: usize, effect: &'static str, opts: &'static [&'static str]) -> CardSpec {
    card(
        from(i) + 0.3,
        to(i) - 0.1,
        SECTIONS[i].1,
        effect,
        opts,
        BAND,
    )
}

pub const CARDS: [CardSpec; 24] = [
    // The intro.
    card(5.7, to(0) - 0.1, "SOLVERFORGE WORKBENCH", "decrypt", BEAM, TITLE),
    card(
        8.3,
        to(0) - 0.1,
        "FOUR PLANNING MODELS FOR HERMES, AND ROOM FOR YOURS.",
        "decrypt",
        PARCHMENT,
        UNDER,
    ),
    // The band, chapter by chapter.
    band(1, "decrypt", PARCHMENT),
    band(3, "slide", JADE),
    band(4, "slice", RED),
    band(6, "unstable", RED),
    band(7, "laseretch", ETCH),
    band(9, "bannerwave", WAVE),
    // The week: its title and its lanes.
    card(from(2) + 0.1, to(2) - 0.1, "THE WEEK", "comet", COMET, HEAD),
    card(
        from(2) + 0.2,
        to(2) - 0.1,
        "LASER 1\nLASER 2\nLASER 3 12KW\nPRESS 135T\nPRESS 320T\nVERTICAL MILL 1\nVERTICAL MILL 2\n5-AXIS MILL 3\n5-AXIS MILL 4\nLATHE 1\nLATHE 2\nLATHE 3\nPOWDER LINE 1\nPOWDER LINE 2\nWELDING ROBOT 1\nWELDING ROBOT 2",
        "decrypt",
        PARCHMENT,
        MACHINES,
    ),
    card(
        from(2) + 0.2,
        to(2) - 0.1,
        "PIETER WELDER\nSANNE WELDER\nJORIS WELDER\nFATIMA ASSEMBLER\nLUCAS ASSEMBLER\nEMMA ASSEMBLER\nNOOR INSPECTOR\nDAAN INSPECTOR",
        "decrypt",
        PARCHMENT,
        PEOPLE,
    ),
    // The powder.
    card(from(5) + 0.1, to(5) - 0.1, "THE POWDER", "copper", COPPER, HEAD),
    // The call.
    card(from(6) + 0.2, from(6) + 4.4, "THE POWDER\nCOMES THURSDAY.", "thunderstorm", STORM, CALL),
    card(
        from(6) + 4.6,
        to(6) - 0.1,
        "KEEP WHAT HAS STARTED.\nREPLAN THE REST.",
        "laseretch",
        ETCH,
        CALL,
    ),
    // The repair: its title and its lanes.
    card(from(8) + 0.1, to(8) - 0.1, "THE REPAIR", "bannerwave", WAVE, HEAD),
    card(
        from(8) + 0.2,
        to(8) - 0.1,
        "LASER 1\nLASER 2\nLASER 3 12KW\nPRESS 135T\nPRESS 320T\nVERTICAL MILL 1\nVERTICAL MILL 2\n5-AXIS MILL 3\n5-AXIS MILL 4\nLATHE 1\nLATHE 2\nLATHE 3\nPOWDER LINE 1\nPOWDER LINE 2\nWELDING ROBOT 1\nWELDING ROBOT 2",
        "decrypt",
        PARCHMENT,
        MACHINES,
    ),
    card(
        from(8) + 0.2,
        to(8) - 0.1,
        "PIETER WELDER\nSANNE WELDER\nJORIS WELDER\nFATIMA ASSEMBLER\nLUCAS ASSEMBLER\nEMMA ASSEMBLER\nNOOR INSPECTOR\nDAAN INSPECTOR",
        "decrypt",
        PARCHMENT,
        PEOPLE,
    ),
    // The close.
    card(from(10) + 0.3, from(10) + 5.6, "SOLVERFORGE\nWORKBENCH", "beams", BEAM, CLOSE),
    card(
        from(10) + 5.8,
        to(10) - 0.05,
        "EVERY PLAN CHECKED\nAGAINST EVERY RULE.",
        "slide",
        JADE,
        CLOSE,
    ),
    // Lines in the header, between a scene's title and its score.
    card(from(5) + 4.0, from(5) + 9.0, "TUE 06:00  ->  THU 18:00", "scattered", PARCHMENT, SHIFT),
    card(from(5) + 9.2, to(5) - 0.1, "PARTS OUT OF STOCK", "unstable", RED, SHIFT),
    card(from(8) + 0.6, from(8) + 5.4, "KEEP BEFORE THU 06:00", "decrypt", PARCHMENT, SHIFT),
    card(from(8) + 15.0, to(8) - 0.1, "96 OPERATIONS MOVED", "slide", JADE, SHIFT),
    card(from(2) + 15.0, to(2) - 0.1, "409 OF 409 PLACED", "slide", JADE, SHIFT),
];

/// One line in the header, between a scene's title and its score.
const SHIFT: Place = Place::new(660, 52, 560, 20, 36);

pub fn cards() -> Cards {
    Cards::new(&CARDS, 13)
}

#[cfg(test)]
mod tests {
    /// demofx refuses a card too short to read, or too wide for its place.
    #[test]
    fn every_card_can_be_read() {
        super::cards();
    }
}
