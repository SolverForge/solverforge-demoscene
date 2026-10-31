// The story, told on cards timed to the picture. Each card is one or two
// plain sentences about what the route board shows at that moment: what a
// route is to SolverForge, how a new stop gets inserted, what is re-checked, and
// how a blocked road is repaired. Every window is long enough to read (demofx
// enforces it); the scene boundaries in scene.rs are laid out to fit them.
use demofx::{card, CardSpec, Cards, Place};

/// The empty sky above the route board (lines up to 83 chars).
const TOP: Place = Place::new(0, 34, 1280, 15, 27);

const COLD: &[&str] = &["--final-gradient-stops", "a9c8ee", "ffffff"];
const GOLD: &[&str] = &["--final-gradient-stops", "ffc94a", "fff3c4"];
const BLUE: &[&str] = &["--final-gradient-stops", "7dd3fc", "e0f2fe"];
const RED: &[&str] = &["--final-gradient-stops", "ff7a5c", "ffe0d6"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];

pub const CARDS: [CardSpec; 17] = [
    // INVOCATION
    card(
        0.60,
        9.40,
        "SOLVERFORGE PLANS ROUTES: A VEHICLE LEAVES ITS START\nAND VISITS A LIST OF STOPS, ONE AFTER ANOTHER.",
        "print",
        COLD,
        TOP,
    ),
    // STATIONS
    card(
        10.60,
        18.60,
        "TO SOLVERFORGE THE ROUTE IS A LIST.\nEACH NUMBER IS A STOP'S PLACE IN IT: 0, 1, 2, AND ON.",
        "slide",
        GOLD,
        TOP,
    ),
    card(
        18.90,
        25.90,
        "THE ORDER IS THE PLAN:\nIT DECIDES WHERE THE VEHICLE GOES FIRST, AND WHEN IT ARRIVES.",
        "middleout",
        GOLD,
        TOP,
    ),
    // SUCCESSORS
    card(
        27.10,
        35.23,
        "EACH STOP POINTS TO THE NEXT: 0 TO 1, 1 TO 2, AND ON.\nTHE MOVING ARROWS ARE THOSE LINKS.",
        "decrypt",
        COLD,
        TOP,
    ),
    card(
        35.52,
        43.40,
        "MOST OF A ROUTE'S COST LIVES IN ITS LINKS:\nHOW FAR EACH HOP IS, AND HOW LONG IT TAKES.",
        "wipe",
        COLD,
        TOP,
    ),
    // REQUIRED STATION
    card(
        44.60,
        51.60,
        "A NEW STOP COMES IN THAT MUST BE VISITED.\nIT IS NOT ON THE ROUTE YET.",
        "beams",
        BLUE,
        TOP,
    ),
    card(
        51.90,
        59.40,
        "WHERE SHOULD IT GO? IT CAN BE SLOTTED INTO\nANY GAP BETWEEN TWO STOPS THAT FOLLOW EACH OTHER.",
        "slide",
        BLUE,
        TOP,
    ),
    // INSERTION SEARCH
    card(
        60.60,
        67.60,
        "SOLVERFORGE TRIES EVERY GAP. EACH NUMBER IS THE EXTRA\nCOST OF THAT DETOUR: LOWER IS BETTER.",
        "decrypt",
        GOLD,
        TOP,
    ),
    card(
        67.90,
        74.90,
        "THE CHEAPEST GAP WINS: BETWEEN STOP 3 AND STOP 4,\nFOR AN EXTRA COST OF 18.",
        "beams",
        GOLD,
        TOP,
    ),
    // SPLICE
    card(
        76.10,
        84.47,
        "INSERTING IT BREAKS ONE LINK, 3 TO 4, AND MAKES TWO:\n3 TO THE NEW STOP, THE NEW STOP TO 4.",
        "middleout",
        GREEN,
        TOP,
    ),
    card(
        84.77,
        91.40,
        "EVERY STOP AFTER IT MOVES ONE PLACE DOWN THE LIST.\nWATCH THEIR NUMBERS SHIFT.",
        "slide",
        GREEN,
        TOP,
    ),
    // CONSTRAINT REPAIR
    card(
        92.60,
        99.85,
        "THEN THE RULES ARE RE-CHECKED, ONLY WHERE THE ROUTE CHANGED:\nARRIVAL TIME, LOAD, VISIT ORDER, DISTANCE.",
        "decrypt",
        COLD,
        TOP,
    ),
    card(
        100.15,
        107.40,
        "THE UNCHANGED REST OF THE ROUTE IS SKIPPED.\nTHAT IS HOW SOLVERFORGE TRIES MILLIONS OF EDITS.",
        "wipe",
        COLD,
        TOP,
    ),
    // REROUTE
    card(
        108.60,
        115.10,
        "NOW A ROAD ON THE ROUTE CLOSES:\nTHE LINK MARKED BLOCK CANNOT BE DRIVEN.",
        "burn",
        RED,
        TOP,
    ),
    card(
        115.40,
        122.90,
        "THE SAME MOVE REPAIRS IT: ONE DETOUR STOP IS SLOTTED IN,\nAND ONLY THE BROKEN LINK IS REDRAWN.",
        "slide",
        GREEN,
        TOP,
    ),
    // FINAL ROUTE
    card(
        124.10,
        130.90,
        "THE FINISHED ROUTE: EVERY REQUIRED STOP IS VISITED,\nIN ORDER, AROUND THE CLOSED ROAD.",
        "beams",
        GOLD,
        TOP,
    ),
    // SCROLLTEXT
    card(
        132.10,
        146.90,
        "SOLVERFORGE CALLS THIS A LIST VARIABLE:\nAN ORDERED LIST THAT IT EDITS ONE SPLICE AT A TIME.",
        "decrypt",
        GREEN,
        TOP,
    ),
];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 40)
}
