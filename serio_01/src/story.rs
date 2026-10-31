// The story, told on cards timed to the picture. Each card is one or two
// plain sentences about what is on screen at that moment: the puzzle, how a
// SolverForge scores a move, what SERIO skips, and what it finds. Every
// window is long enough to read (demofx enforces it); the scene boundaries in
// main.rs are laid out to fit these cards.
use demofx::{card, CardSpec, Cards, Place};

/// Under the title, across the screen (lines up to 83 chars).
const TOP: Place = Place::new(0, 112, 1280, 15, 27);
/// Under the orrery.
const LOW: Place = Place::new(0, 622, 1280, 15, 27);
/// Under the orrery, left of the score graph.
const LOW_LEFT: Place = Place::new(10, 622, 840, 15, 27);
/// The solution: under the orrery, above the FEASIBLE line.
const SOLVED: Place = Place::new(0, 586, 1280, 15, 27);
/// The split screen: between the panel labels and the orreries.
const SPLIT: Place = Place::new(0, 76, 1280, 15, 27);

const COLD: &[&str] = &["--final-gradient-stops", "a9c8ee", "ffffff"];
const GOLD: &[&str] = &["--final-gradient-stops", "ffc94a", "fff3c4"];
const RED: &[&str] = &["--final-gradient-stops", "ff7a5c", "ffe0d6"];
const BLUE: &[&str] = &["--final-gradient-stops", "3ca0ff", "d8ecff"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];

pub const CARDS: [CardSpec; 15] = [
    // GENESIS
    card(
        0.60,
        6.85,
        "SEVEN PLANETS CIRCLE THE SUN.\nEACH ONE CAN TAKE ANY OF 21 ORBITS.",
        "print",
        COLD,
        TOP,
    ),
    card(
        7.15,
        14.90,
        "KEPLER LOOKED FOR MUSIC IN THE PLANETS' MOTIONS.\nHERE WE LOOK FOR IT IN THE SIZES OF THEIR ORBITS.",
        "middleout",
        GOLD,
        TOP,
    ),
    // FULL EVALUATION
    card(
        16.10,
        23.58,
        "THE PUZZLE: GIVE EACH PLANET ITS OWN ORBIT, SO THAT\nEVERY PAIR FORMS A PURE RATIO, LIKE 3:2.",
        "slide",
        COLD,
        LOW,
    ),
    card(
        23.88,
        29.87,
        "SOLVERFORGE MOVES ONE PLANET AT A TIME,\nTHEN SCORES THE NEW LAYOUT.",
        "decrypt",
        COLD,
        LOW,
    ),
    card(
        30.17,
        36.40,
        "THE SIMPLE WAY RE-SCORES ALL 21 PAIRS\nAFTER EVERY MOVE: EVERY LINE FLASHES.",
        "wipe",
        RED,
        LOW,
    ),
    // SPLIT SCREEN
    card(
        37.60,
        44.26,
        "BUT A MOVE ONLY AFFECTS 6 PAIRS:\nTHE ONES THAT INCLUDE THE PLANET THAT MOVED.",
        "middleout",
        COLD,
        SPLIT,
    ),
    card(
        44.56,
        50.98,
        "LEFT: ALL 21 PAIRS RE-SCORED. RIGHT: SERIO,\nSOLVERFORGE'S SCORING ENGINE, RE-SCORES ONLY THOSE 6.",
        "decrypt",
        BLUE,
        SPLIT,
    ),
    card(
        51.27,
        57.94,
        "THE OTHER 15 PAIRS KEEP THEIR SCORES,\nSO BOTH SIDES ARRIVE AT THE SAME TOTAL.",
        "beams",
        BLUE,
        SPLIT,
    ),
    card(
        58.24,
        64.40,
        "SAME ANSWER, 3.5 TIMES LESS WORK.\nTHE MORE PLANETS, THE BIGGER THE GAP.",
        "slide",
        GREEN,
        SPLIT,
    ),
    // HARMONY
    card(
        65.60,
        73.92,
        "NOW 480 MOVES A SECOND. IT KEEPS THE MOVES THAT HELP,\nAND A FEW THAT DO NOT, TO ESCAPE DEAD ENDS.",
        "burn",
        BLUE,
        LOW_LEFT,
    ),
    card(
        74.22,
        81.03,
        "GOLD: A PAIR AT A PURE RATIO. GREEN: CLOSE.\nRED: FAR FROM ANY PURE RATIO.",
        "decrypt",
        GOLD,
        LOW_LEFT,
    ),
    card(
        81.33,
        87.90,
        "THE SCORE IS THE TOTAL DISTANCE FROM PURE RATIOS.\nWATCH IT CLIMB TOWARD ZERO.",
        "waves",
        COLD,
        LOW_LEFT,
    ),
    // SOLUTION
    card(
        89.10,
        96.85,
        "THE BEST LAYOUT IT FOUND: EACH PLANET ON ITS OWN ORBIT,\nAS CLOSE TO PURE AS 21 ORBITS ALLOW.",
        "beams",
        GOLD,
        SOLVED,
    ),
    card(
        97.15,
        103.40,
        "THE OLD NAME FOR THIS IS MUSICA UNIVERSALIS:\nTHE HARMONY OF THE SPHERES.",
        "sweep",
        GOLD,
        SOLVED,
    ),
    // OUTRO
    card(
        104.60,
        111.40,
        "SERIO SCORES ONLY WHAT CHANGED.\nTHAT IS HOW SOLVERFORGE STAYS FAST.",
        "decrypt",
        GREEN,
        Place::new(0, 40, 1280, 15, 27),
    ),
];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 70)
}
