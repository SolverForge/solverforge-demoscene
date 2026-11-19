// The frame's words, all in one table. The opening says what the plugin
// holds; over the take, each chapter is titled with the step of the work it
// shows, typed in by glyphfx; the close says what every plan gets and where
// to find the plugin. The narration itself is the Nous Girl's, in the dialog.
use demofx::{card, CardSpec, Cards, Place};

use crate::cut::CHAPTERS;
use crate::timeline::{DURATION, LIVE_FROM, LIVE_TO, PANEL_AT};

const TITLE: Place = Place::new(0, 150, 1920, 36, 64);
const TOP: Place = Place::new(0, 40, 1920, 20, 36);
/// Over the take's window (x 48..1488), above it.
const STAGE: Place = Place::new(48, 100, 1440, 20, 36);
const BEAM: &[&str] = &["--final-gradient-stops", "10b981", "50fc00", "e8ffd6"];
const LIME: &[&str] = &["--final-gradient-stops", "50fc00", "e8ffd6"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];

/// A chapter's title, up while the chapter plays.
const fn stage(i: usize, text: &'static str) -> CardSpec {
    card(
        LIVE_FROM + CHAPTERS[i].1 + 0.2,
        LIVE_FROM + CHAPTERS[i].2 - 0.2,
        text,
        "decrypt",
        LIME,
        STAGE,
    )
}

pub const CARDS: [CardSpec; 18] = [
    card(
        0.3,
        LIVE_FROM - 0.2,
        "SOLVERFORGE WORKBENCH",
        "beams",
        BEAM,
        TITLE,
    ),
    card(
        1.0,
        PANEL_AT[4] - 0.2,
        "FOUR PLANNING MODELS, EACH WITH A REAL DEMO.",
        "decrypt",
        GREEN,
        TOP,
    ),
    card(
        PANEL_AT[4] - 0.1,
        LIVE_FROM - 0.2,
        "AND ROOM FOR THE MODEL YOU BUILD.",
        "decrypt",
        GREEN,
        TOP,
    ),
    stage(0, "01  THE MODELS"),
    stage(1, "02  PICK A DATASET"),
    stage(2, "03  READ AND EDIT THE DATA"),
    stage(3, "04  SOLVE"),
    stage(4, "05  SEE THE PLAN"),
    stage(5, "06  SEE WHY"),
    stage(6, "07  FIND A PLACE"),
    stage(7, "08  REPLAN"),
    stage(8, "09  TUNE A PROFILE"),
    stage(9, "10  COMPARE"),
    stage(10, "11  FOLLOW THE TRENDS"),
    stage(11, "12  ASK HERMES"),
    stage(12, "13  BRING YOUR OWN MODEL"),
    card(
        LIVE_TO + 0.3,
        LIVE_TO + 5.8,
        "EVERY PLAN CHECKED AGAINST EVERY RULE,\nAND EXPLAINED.",
        "decrypt",
        GREEN,
        TOP,
    ),
    card(
        LIVE_TO + 5.9,
        DURATION - 0.1,
        "SOLVERFORGE WORKBENCH, A PLUGIN FOR HERMES AGENT.",
        "slide",
        LIME,
        TOP,
    ),
];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 12)
}

#[cfg(test)]
mod tests {
    /// demofx refuses a card too short to read, or two in one place at once.
    #[test]
    fn every_card_can_be_read() {
        super::cards();
    }
}
