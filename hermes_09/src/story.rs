// The story around the take: what the problem is, then what the plugin
// guarantees and where to get it. One or two plain sentences per card about
// what is on screen; demofx refuses any card too short to read. The live
// take carries its own narration and captions, so it has no cards.
use demofx::{card, CardSpec, Cards, Place};

use crate::timeline::{bars, INTRO_BARS, LIVE_BARS};

/// The band across the top of the frame (lines up to 96 chars).
const TOP: Place = Place::new(0, 40, 1920, 20, 36);

const LIME: &[&str] = &["--final-gradient-stops", "50fc00", "e8ffd6"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];
const GOLD: &[&str] = &["--final-gradient-stops", "ffc94a", "fff3c4"];

const OUTRO: f32 = bars(INTRO_BARS + LIVE_BARS);

pub const CARDS: [CardSpec; 4] = [
    card(
        2.7,
        7.1,
        "YOUR WEEK IS A CONSTRAINT PROBLEM.",
        "decrypt",
        LIME,
        TOP,
    ),
    card(
        7.2,
        11.9,
        "SOLVERFORGE FITS YOUR TASKS AROUND YOUR MEETINGS.",
        "beams",
        GOLD,
        TOP,
    ),
    card(
        OUTRO + 0.4,
        OUTRO + 5.9,
        "0HARD: EVERY HARD RULE KEPT.\nNOTHING WRITTEN WITHOUT YOUR YES.",
        "decrypt",
        GREEN,
        TOP,
    ),
    card(
        OUTRO + 6.0,
        OUTRO + 11.0,
        "SOLVERFORGE PLANNER, A PLUGIN FOR HERMES AGENT.",
        "slide",
        LIME,
        TOP,
    ),
];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 9)
}
