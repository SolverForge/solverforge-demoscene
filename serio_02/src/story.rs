// The story, told on cards timed to the picture. Each card is one or two
// plain sentences about what is on screen at that moment: the problem, how a
// SolverForge scores a change, what SERIO skips, and what it finds. Every
// window is long enough to read (demofx enforces it); the scene boundaries in
// main.rs are laid out to fit these cards.
use demofx::{card, CardSpec, Cards, Place};

/// Under the orrery, left of the telemetry panel (lines up to 56 chars).
const LOW: Place = Place::new(10, 610, 900, 15, 27);
/// Genesis: under the orbits, clear of the title.
const MID: Place = Place::new(10, 508, 900, 15, 27);
/// Split screen: across the top, between the panel labels and the orreries.
const TOP: Place = Place::new(0, 58, 1280, 15, 27);

const COLD: &[&str] = &["--final-gradient-stops", "a9c8ee", "ffffff"];
const GOLD: &[&str] = &["--final-gradient-stops", "ffc94a", "fff3c4"];
const RED: &[&str] = &["--final-gradient-stops", "ff7a5c", "ffe0d6"];
const BLUE: &[&str] = &["--final-gradient-stops", "3ca0ff", "d8ecff"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];

pub const CARDS: [CardSpec; 21] = [
    // GENESIS
    card(
        0.60,
        8.42,
        "SEVEN PLANETS CIRCLE THE SUN. EACH ONE SINGS\nA SINGLE NOTE: A PITCH, WRITTEN UNDER ITS NAME IN HZ.",
        "print",
        COLD,
        MID,
    ),
    card(
        8.72,
        15.53,
        "PYTHAGORAS: TWO NOTES SOUND RIGHT TOGETHER\nWHEN THEIR PITCHES FORM A SIMPLE RATIO, LIKE 3:2.",
        "middleout",
        GOLD,
        MID,
    ),
    card(
        15.83,
        25.40,
        "THE RATIO IS ONE PITCH DIVIDED BY THE OTHER:\nVENUS 165 HZ / LUNA 110 HZ = 1.5, A PURE 3:2.\nTHAT PAIR IS IN TUNE.",
        "decrypt",
        GOLD,
        MID,
    ),
    // RE-CHECK EVERYTHING
    card(
        26.60,
        33.57,
        "THE PUZZLE: GIVE EVERY PLANET A NOTE\nSO THAT EVERY PAIR OF PLANETS IS IN TUNE.",
        "slide",
        COLD,
        LOW,
    ),
    card(
        33.87,
        42.09,
        "THE ORBITS AND THE SPINNING ARE ONLY THE STAGE: WHERE A\nPLANET IS NEVER CHANGES THE SCORE. ONLY ITS NOTE DOES.",
        "wipe",
        COLD,
        LOW,
    ),
    card(
        42.39,
        49.86,
        "ONE RULE USES THE ORBITS: NEIGHBOURS MUST AGREE MOST.\nPAIRS ON NEXT-DOOR ORBITS COUNT THREE TIMES AS MUCH.",
        "slide",
        GOLD,
        LOW,
    ),
    card(
        50.16,
        57.88,
        "EACH LINE IS ONE PAIR: 21 IN ALL. SOLVERFORGE CHANGES\nONE NOTE AT A TIME; THE RING MARKS IT.",
        "decrypt",
        COLD,
        LOW,
    ),
    card(
        58.18,
        65.40,
        "TO SCORE THE CHANGE, THE SIMPLE WAY RE-CHECKS\nALL 21 PAIRS, ONE BY ONE. WATCH THE COUNTER.",
        "wipe",
        RED,
        LOW,
    ),
    // SPLIT SCREEN
    card(
        66.60,
        73.32,
        "BUT ONLY 6 PAIRS TOUCH THE PLANET THAT CHANGED.\nTHE OTHER 15 CANNOT HAVE CHANGED.",
        "middleout",
        COLD,
        TOP,
    ),
    card(
        73.62,
        80.35,
        "LEFT: RE-CHECK EVERYTHING, 21 PAIRS PER CHANGE.\nRIGHT: SERIO RE-CHECKS ONLY THE 6 THAT MOVED.",
        "decrypt",
        BLUE,
        TOP,
    ),
    card(
        80.65,
        87.12,
        "SERIO IS THE SCORING ENGINE INSIDE SOLVERFORGE.\nBOTH SIDES ALWAYS AGREE ON THE SCORE.",
        "beams",
        BLUE,
        TOP,
    ),
    card(
        87.43,
        94.40,
        "HERE THAT IS 3.5 TIMES LESS WORK.\nWITH 100 PLANETS IT WOULD BE 50 TIMES LESS.",
        "slide",
        GREEN,
        TOP,
    ),
    // THE SOLVER: SLOW MOTION
    card(
        95.60,
        103.35,
        "NOW SOLVERFORGE SOLVES IT FOR REAL, SLOWED DOWN.\nIT STARTS FROM NOTES THAT CLASH: RED IS OUT OF TUNE.",
        "burn",
        RED,
        LOW,
    ),
    card(
        103.65,
        111.40,
        "A CHANGE THAT MAKES THE SCORE WORSE IS UNDONE.\nA CHANGE THAT HELPS IS KEPT, AND RED TURNS GOLD.",
        "decrypt",
        GOLD,
        LOW,
    ),
    // THE SOLVER: FULL SPEED
    card(
        112.60,
        119.92,
        "FULL SPEED: HUNDREDS OF THOUSANDS OF CHANGES A SECOND,\nEACH SCORED BY SERIO ON JUST 6 PAIRS.",
        "beams",
        BLUE,
        LOW,
    ),
    card(
        120.22,
        128.78,
        "THE SCORE ADDS UP HOW FAR EACH PAIR'S RATIO IS FROM PURE.\nWITHIN 1.5% COUNTS AS IN TUNE; ZERO MEANS ALL ARE.",
        "slide",
        COLD,
        LOW,
    ),
    card(
        129.08,
        136.40,
        "TWO PLANETS ON ONE NOTE IS FORBIDDEN:\nTHAT IS THE HARD SCORE. IT MUST STAY AT ZERO.",
        "wipe",
        COLD,
        LOW,
    ),
    // SOLUTION
    card(
        137.60,
        144.85,
        "THE BEST TUNING IT FOUND: NO NOTE USED TWICE,\nAND GOLD LINES: EVERY PAIR IS IN TUNE.",
        "beams",
        GOLD,
        LOW,
    ),
    card(
        145.15,
        151.40,
        "THE OLD NAME FOR THIS IS MUSICA UNIVERSALIS:\nTHE HARMONY OF THE SPHERES.",
        "sweep",
        GOLD,
        LOW,
    ),
    // OUTRO
    card(
        152.60,
        159.22,
        "SOLVERFORGE PLANS SHIFTS, ROUTES AND TIMETABLES\nTHE SAME WAY: SMALL CHANGES, EACH ONE SCORED.",
        "decrypt",
        GREEN,
        Place::new(0, 40, 1280, 15, 27),
    ),
    card(
        159.53,
        165.40,
        "ONLY WHAT CHANGED GETS RE-CHECKED.\nTHAT IS WHAT MAKES IT FAST.",
        "print",
        GREEN,
        Place::new(0, 40, 1280, 15, 27),
    ),
];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 50)
}
