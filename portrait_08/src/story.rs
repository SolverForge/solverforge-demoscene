// The story, in Vittorio's own voice: the guitar, the games, the machine,
// books, home, then work. One or two plain sentences per card about
// what is on screen; demofx refuses any card too short to read.
use demofx::{card, CardSpec, Cards, Place};

/// The band under Instagram's top bar (lines up to 58 chars).
const TOP: Place = Place::new(0, 250, 1080, 18, 32);

const WARM: &[&str] = &["--final-gradient-stops", "f2d48a", "fff6e0"];
const FROST: &[&str] = &["--final-gradient-stops", "9ad7ff", "eef8ff"];
const EMBER: &[&str] = &["--final-gradient-stops", "ff9a4a", "ffe6cf"];
const BONE: &[&str] = &["--final-gradient-stops", "c9c2b0", "f4f0e6"];
const TERM: &[&str] = &["--final-gradient-stops", "8be28b", "eaffea"];
const SAND: &[&str] = &["--final-gradient-stops", "e8b86a", "fff1d6"];
const NIGHT: &[&str] = &["--final-gradient-stops", "c7b8ff", "f4f0ff"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];

pub const CARDS: [CardSpec; 17] = [
    // GUITAR
    card(
        1.0,
        7.8,
        "I'M VITTORIO DISTEFANO.\nBORN IN 1982, LIVING IN BERGAMO.",
        "print",
        WARM,
        TOP,
    ),
    card(
        8.2,
        15.4,
        "THIS IS MY GUITAR:\nAN EPIPHONE LES PAUL BLACK BEAUTY.",
        "slide",
        WARM,
        TOP,
    ),
    // WORLDS
    card(
        16.6,
        24.6,
        "I'VE SPENT YEARS IN OLD ONLINE WORLDS:\nVANILLA WOW, LINEAGE II, WARHAMMER ONLINE.",
        "decrypt",
        FROST,
        TOP,
    ),
    card(
        25.0,
        32.4,
        "LEAVE THE TOWN YOU KNOW. THE MAP IS BLANK\nUNTIL SOMEONE WALKS IT.",
        "rain",
        FROST,
        TOP,
    ),
    card(
        32.8,
        39.6,
        "COME BACK WITH SOMETHING NOBODY ELSE HAS,\nAND THE WORLD WILL TRADE FOR IT.",
        "slide",
        FROST,
        TOP,
    ),
    // CHALLENGE
    card(
        40.2,
        45.6,
        "SOMETHING HUGE IN THE WAY.\nHOW HARD CAN IT BE?",
        "burn",
        EMBER,
        TOP,
    ),
    card(
        45.8,
        51.2,
        "THE FIRST TRIES ARE RECON.\nYOU LOSE, AND YOU LEARN.",
        "burn",
        EMBER,
        TOP,
    ),
    card(
        51.4,
        56.8,
        "EVERY ATTACK HAS A TELL.\nREAD IT, WAIT, THEN MOVE.",
        "burn",
        EMBER,
        TOP,
    ),
    card(
        57.0,
        63.7,
        "NO SHORTCUTS: TRY, FAIL, ADJUST, SAVE,\nTRY AGAIN. IT GOES DOWN EVENTUALLY.",
        "print",
        BONE,
        TOP,
    ),
    // MACHINE
    card(
        64.6,
        73.0,
        "MY COMPUTER RUNS LINUX, SWAY AND TMUX,\nSET UP THE WAY I WANT IT.",
        "decrypt",
        TERM,
        TOP,
    ),
    card(
        73.4,
        83.2,
        "I'D RATHER UNDERSTAND A SYSTEM\nTHAN BE SOLD ONE.",
        "wipe",
        TERM,
        TOP,
    ),
    // READING
    card(
        84.6,
        91.4,
        "I READ DUNE FOR WHAT HERBERT WAS SAYING\nABOUT POWER, RELIGION AND MASS MOVEMENTS.",
        "decrypt",
        SAND,
        TOP,
    ),
    card(
        91.8,
        97.6,
        "REPUTATIONS DON'T IMPRESS ME.\nSHOW ME THE WORK.",
        "burn",
        SAND,
        TOP,
    ),
    // HOME
    card(
        98.6,
        105.6,
        "HOME IS BERGAMO, THERE IS A CAT NAMED BYRON,\nAND I READ ITS FOOD LABELS CLOSELY.",
        "middleout",
        NIGHT,
        TOP,
    ),
    card(
        105.9,
        113.4,
        "LIKE BYRON, I MOSTLY WEAR BLACK.\nPASTELS ARE NOT AN OPTION.",
        "print",
        NIGHT,
        TOP,
    ),
    // WORK
    card(
        114.6,
        121.0,
        "FOR WORK I BUILD SOLVERFORGE: SOFTWARE THAT PLANS\nSHIFTS, ROUTES AND TIMETABLES.",
        "decrypt",
        GREEN,
        TOP,
    ),
    card(
        121.4,
        127.6,
        "SAME HABIT AS EVERYTHING ELSE:\nFIND OUT HOW THINGS ACTUALLY WORK.",
        "middleout",
        GREEN,
        TOP,
    ),
];

pub fn cards() -> Cards {
    Cards::new(&CARDS, 80)
}
