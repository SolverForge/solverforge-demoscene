// The film's own words, all in one table. Over the takes, each chapter is
// titled in the Desktop's empty sidebar, typed in by glyphfx; the opening
// names the plugin and the plan, the close says where to get it. The call's
// words are drawn by its scene (scenes/call.rs), on its own beats. The
// narration is spoken, never written on screen.
use demofx::{card, CardSpec, Cards, Place};

use crate::cut::{Kind, CHAPTERS, SEGMENTS};

/// The Desktop's sidebar is empty from y 720 down to its status bar: the
/// chapter goes there, under the corner logo (scenes::LOGO_AT).
pub const SIDEBAR: Place = Place::new(14, 876, 292, 12, 22);
const TITLE: Place = Place::new(0, 150, 1920, 36, 64);
const SUB: Place = Place::new(0, 252, 1920, 20, 36);
const TOP: Place = Place::new(0, 120, 1920, 20, 36);
const BEAM: &[&str] = &["--final-gradient-stops", "10b981", "50fc00", "e8ffd6"];
const LIME: &[&str] = &["--final-gradient-stops", "50fc00", "e8ffd6"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];

const fn seg(kind: Kind) -> (f32, f32) {
    let mut i = 0;
    while i < SEGMENTS.len() {
        if SEGMENTS[i].2 as u8 == kind as u8 {
            return (SEGMENTS[i].0, SEGMENTS[i].1);
        }
        i += 1;
    }
    panic!("no such segment");
}

const INTRO: (f32, f32) = seg(Kind::Intro);
const OUTRO: (f32, f32) = seg(Kind::Outro);

/// A chapter's title, up while the chapter plays.
const fn chapter(i: usize) -> CardSpec {
    card(
        CHAPTERS[i].0 + 0.3,
        CHAPTERS[i].1 - 0.15,
        CHAPTERS[i].2,
        "decrypt",
        LIME,
        SIDEBAR,
    )
}

pub static CARDS: [CardSpec; 11] = [
    chapter(0),
    card(
        INTRO.0 + 0.3,
        INTRO.1 - 0.5,
        "SOLVERFORGE WORKBENCH",
        "beams",
        BEAM,
        TITLE,
    ),
    card(
        INTRO.0 + 1.0,
        INTRO.1 - 0.5,
        "A MONTH OF HOSPITAL SHIFTS, PLANNED LIVE IN HERMES",
        "decrypt",
        GREEN,
        SUB,
    ),
    chapter(1),
    chapter(2),
    chapter(3),
    chapter(4),
    chapter(5),
    chapter(6),
    chapter(7),
    card(
        OUTRO.0 + 0.4,
        OUTRO.1 - 0.1,
        "SOLVERFORGE WORKBENCH, A PLUGIN FOR HERMES AGENT.",
        "slide",
        LIME,
        TOP,
    ),
];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 13)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// demofx refuses a card too short to read.
    #[test]
    fn every_card_can_be_read() {
        cards();
    }

    #[test]
    fn every_chapter_is_titled() {
        assert_eq!(CHAPTERS.len(), 8);
    }
}
