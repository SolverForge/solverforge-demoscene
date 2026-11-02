// The story, in Vittorio's own voice: who he is, then his four offers, then
// the proof and the address. One or two plain sentences per card about what
// is on screen; demofx refuses any card too short to read.
use demofx::{card, CardSpec, Cards, Place};

/// The band across the top of the frame (lines up to 83 chars).
const TOP: Place = Place::new(0, 30, 1280, 15, 27);

const LIME: &[&str] = &["--final-gradient-stops", "50fc00", "e8ffd6"];
const COLD: &[&str] = &["--final-gradient-stops", "a9c8ee", "ffffff"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];
const BLUE: &[&str] = &["--final-gradient-stops", "7dd3fc", "e0f2fe"];
const VIOLET: &[&str] = &["--final-gradient-stops", "c4b5fd", "f5f3ff"];
const CORAL: &[&str] = &["--final-gradient-stops", "fb923c", "ffedd5"];
const GOLD: &[&str] = &["--final-gradient-stops", "ffc94a", "fff3c4"];

pub const CARDS: [CardSpec; 15] = [
    // BOOT
    card(
        1.0,
        10.6,
        "I'M VITTORIO DISTEFANO. I BUILD SOFTWARE FOR HARD PROBLEMS,\nFROM BERGAMO, ITALY.",
        "print",
        LIME,
        TOP,
    ),
    // CITY
    card(
        12.8,
        20.4,
        "I FOUNDED SOLVERFORGE, AN OPEN-SOURCE CONSTRAINT SOLVER\nWITH A NATIVE RUST CORE AND A PYTHON SURFACE.",
        "decrypt",
        GREEN,
        TOP,
    ),
    card(
        20.6,
        27.8,
        "SOLVERS ARE ONE PART OF WHAT I BUILD.\nHERE ARE FOUR WAYS TO WORK WITH ME.",
        "slide",
        COLD,
        TOP,
    ),
    // PLAN
    card(
        28.6,
        36.0,
        "ONE: PLANNING SYSTEMS. SCHEDULING, ROUTING AND ALLOCATION,\nMODELED AND BUILT ON SOLVERFORGE.",
        "beams",
        GREEN,
        TOP,
    ),
    card(
        36.3,
        45.4,
        "EVERY FRAME IS A REAL STEP OF A LOCAL SEARCH: A STOP MOVES,\nAND THE MOVE STAYS ONLY IF THE ROUTES GET SHORTER.",
        "decrypt",
        GOLD,
        TOP,
    ),
    card(
        45.7,
        53.6,
        "I TAKE A PLANNING PROBLEM FROM THE SPREADSHEET IT LIVES IN\nTO A SOLVER RUNNING IN PRODUCTION.",
        "middleout",
        GREEN,
        TOP,
    ),
    // ADVICE
    card(
        54.6,
        62.0,
        "TWO: ADVISORY, FOR ANY SOFTWARE PROJECT: DISCOVERY,\nARCHITECTURE, CODE REVIEWS AND PERFORMANCE, WITH ME DIRECTLY.",
        "wipe",
        BLUE,
        TOP,
    ),
    card(
        62.3,
        69.6,
        "A SHORT, RIGOROUS LOOK AT A HARD PROBLEM\nCAN SAVE MONTHS OF BUILDING THE WRONG THING.",
        "slide",
        BLUE,
        TOP,
    ),
    // AGENTS
    card(
        70.6,
        79.0,
        "THREE: AI AGENTS WITH SOLID BOUNDARIES, AND THE TOOLS UNDER THEM:\nMCP SERVERS, TERMINAL APPS, NATIVE LINUX DESKTOP SOFTWARE.",
        "decrypt",
        VIOLET,
        TOP,
    ),
    card(
        79.3,
        87.6,
        "MY AGENTS WORK ON A DESKTOP LIKE THIS ONE EVERY DAY,\nTHROUGH MCP SERVERS I WROTE AND PUBLISHED.",
        "print",
        VIOLET,
        TOP,
    ),
    // DEVREL
    card(
        88.6,
        97.2,
        "FOUR: DEVELOPER RELATIONS. IF YOU LIKE HOW I PRESENT MY OWN WORK,\nI CAN DO THE SAME FOR YOUR PROJECT.",
        "beams",
        CORAL,
        TOP,
    ),
    card(
        97.4,
        103.7,
        "DEMOS, DOCS AND QUICKSTARTS, RELEASE POSTS,\nAND VIDEOS LIKE THIS ONE.",
        "middleout",
        CORAL,
        TOP,
    ),
    // WORK
    card(
        104.6,
        112.0,
        "ALL OF THIS IS BACKED BY WORK YOU CAN READ:\nOPEN SOURCE ON GITHUB, AS BLACKOPSREPL AND SOLVERFORGE.",
        "decrypt",
        GREEN,
        TOP,
    ),
    card(
        112.3,
        119.6,
        "BEFORE SOLVERFORGE: DATA PIPELINES, RESEARCH PLATFORMS\nAT THE UNIVERSITY OF BERN, AND AI PRODUCTS.",
        "slide",
        COLD,
        TOP,
    ),
    // CONTACT
    card(
        120.8,
        128.6,
        "WRITE TO ME WITH A SHORT DESCRIPTION OF THE PROBLEM.\nTHE ADDRESS IS BELOW.",
        "print",
        LIME,
        TOP,
    ),
];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 70)
}
