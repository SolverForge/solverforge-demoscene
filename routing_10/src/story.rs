// The recorded workflow carries all narration. Only the short closing title
// belongs to the demoscene frame.
use demofx::{card, CardSpec, Cards, Place};

use crate::timeline::{DURATION, LIVE_TO};

const TOP: Place = Place::new(0, 40, 1920, 20, 36);
const LIME: &[&str] = &["--final-gradient-stops", "50fc00", "e8ffd6"];

pub const CARDS: [CardSpec; 1] = [card(
    LIVE_TO + 0.1,
    DURATION - 0.1,
    "SOLVERFORGE ROUTING",
    "slide",
    LIME,
    TOP,
)];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 9)
}
