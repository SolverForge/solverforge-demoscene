// The story, told in the sky of each scene.
//
// Every scene has a short card: one or two full sentences (double size) set
// in the empty space above the action. A
// glyphfx effect that belongs to the scene brings the words in, they hold
// still to be read, then dissolve. Cards play while the picture is calm; the
// big moments -- the ingot thrown apart, the laser etching, the logo -- have
// the screen to themselves.
use crate::canvas::{Cell, TextCanvas, BIG_COLS, COLS};
use crate::gfx::{Clip, ClipSpec};
use crate::noise::h2;
use crate::timeline::{bars, last_strike, smooth};

struct CardSpec {
    from: f32,
    to: f32,
    body: &'static str,
    effect: &'static str,
    opts: &'static [&'static str],
    /// The words flash with each hammer blow.
    strikes: bool,
}

const fn card(
    from: f32,
    to: f32,
    body: &'static str,
    effect: &'static str,
    opts: &'static [&'static str],
) -> CardSpec {
    CardSpec {
        from,
        to,
        body,
        effect,
        opts,
        strikes: false,
    }
}

const COLD: &[&str] = &["--final-gradient-stops", "a9c8ee", "ffffff"];
const WATER: &[&str] = &["--final-gradient-stops", "8fd0ff", "ffffff"];
const HOT: &[&str] = &["--final-gradient-stops", "ffb050", "ffe8c0"];
const GREEN: &[&str] = &["--final-gradient-stops", "34d399", "ecfdf5"];

/// Bars are on the song's grid (timeline.rs). Lines are at most 60 wide.
const CARDS: [CardSpec; 12] = [
    card(0.4, 3.9, "AT VALBONDIONE, HIGH IN THE VAL SERIANA ABOVE BERGAMO,\nTHE RIVER SERIO IS BORN.", "print", COLD),
    card(
        4.2,
        7.9,
        "SINCE 1931 THE BARBELLINO DAM HOLDS THE SERIO BACK\nFOR POWER. ONLY FIVE DAYS A YEAR IS THE WATER RELEASED.",
        "middleout",
        COLD,
    ),
    card(
        8.3,
        11.9,
        "THEN THE FALLS COME BACK: 315 METRES IN THREE DROPS,\nAMONG THE HIGHEST IN ITALY.",
        "rain",
        WATER,
    ),
    card(
        12.1,
        15.8,
        "SERIO IS ALSO THE NAME OF THE SCORING ENGINE\nAT THE HEART OF SOLVERFORGE.",
        "decrypt",
        GREEN,
    ),
    card(16.2, 19.8, "FOR CENTURIES THE VALLEY BELOW\nPUT THIS WATER TO WORK.", "slide", COLD),
    card(
        20.2,
        23.8,
        "IRON FROM THE MINES OF BONDIONE AND THE VAL DI SCALVE\nWAS SMELTED AT GAVAZZO AND REFINED INTO STEEL.",
        "burn",
        HOT,
    ),
    CardSpec {
        strikes: true,
        ..card(
            24.3,
            28.8,
            "AT PONTE NOSSA, WHERE THE NOSSA STREAM JOINS THE SERIO,\nWATER WHEELS LIFTED HEAVY TRIP HAMMERS CALLED MAGLI.",
            "beams",
            HOT,
        )
    },
    CardSpec {
        strikes: true,
        ..card(
            29.0,
            33.3,
            "BLOW AFTER BLOW THEY BEAT RED-HOT STEEL INTO TOOLS.\nTHE MAGLIO BELTRAMI KEPT BEATING UNTIL 1964;\nTODAY IT IS A MUSEUM.",
            "unstable",
            HOT,
        )
    },
    card(
        36.3,
        39.8,
        "THE TOOLS WERE HARDENED IN THE SAME WATER\nTHAT HAD DRIVEN THE HAMMER.",
        "waves",
        COLD,
    ),
    card(
        46.2,
        49.0,
        "SOLVERFORGE IS AT THAT STAGE NOW:\nNO NEW FEATURES FOR THEIR OWN SAKE.",
        "decrypt",
        GREEN,
    ),
    card(
        49.1,
        51.9,
        "WE ARE CONSOLIDATING AND HARDENING EVERY PART,\nBLOW BY BLOW, ON THE WAY TO SOLVERFORGE 1.0.0.",
        "sweep",
        GREEN,
    ),
    card(
        52.3,
        55.7,
        "THEN THE DAM CLOSES, AND THE FALLS GO QUIET\nUNTIL NEXT TIME.",
        "wipe",
        COLD,
    ),
];

const CREDITS: &str =
    "HAMMER_05 BY SOLVERFORGE  -  TEXT EFFECTS: GLYPHFX  -  MUSIC: THE PLANNER123 THEME, REMIXED  -  BERGAMO MMXXVI";
const CREDITS_FROM: f32 = 53.6;
const CREDITS_ROW: i32 = 38;

/// Rows of the 1x grid covered by a card, for the scrim.
const CARD_ROWS: i32 = 9;
/// Body rows on the double-size grid.
const BODY_ROWS: usize = 3;
/// Share of the card's window the entrance may take before the words hold.
const ENTRANCE: f32 = 0.35;
/// Seconds the words take to dissolve.
const EXIT: f32 = 0.7;

struct Card {
    spec: &'static CardSpec,
    body: Clip,
    body_speed: f32,
    /// Horizontal extent of the settled words on the 1x grid.
    span: (i32, i32),
}

pub struct Story {
    cards: Vec<Card>,
    credits: Clip,
}

fn speed_for(clip: &Clip, window: f32) -> f32 {
    (clip.seconds() / (window * ENTRANCE)).max(1.0)
}

impl Story {
    pub fn new() -> Self {
        let cards = CARDS
            .iter()
            .enumerate()
            .map(|(i, spec)| {
                for line in spec.body.lines() {
                    assert!(line.len() <= BIG_COLS - 4, "card line too long: {line}");
                }
                let body = Clip::render(&ClipSpec {
                    effect: spec.effect,
                    text: spec.body,
                    cols: BIG_COLS,
                    rows: BODY_ROWS,
                    seed: 50 + i as u64,
                    opts: spec.opts,
                });
                let window = bars(spec.to - spec.from);
                let cells = body.final_cells();
                let lo = cells.iter().map(|c| c.0).min().unwrap_or(0) as i32 * 2;
                let hi = cells.iter().map(|c| c.0).max().unwrap_or(0) as i32 * 2 + 2;
                Card {
                    body_speed: speed_for(&body, window),
                    body,
                    spec,
                    span: (lo, hi),
                }
            })
            .collect();
        let credits = Clip::render(&ClipSpec {
            effect: "decrypt",
            text: CREDITS,
            cols: COLS,
            rows: 1,
            seed: 99,
            opts: &["--final-gradient-stops", "10b981", "6ee7b7"],
        });
        Self { cards, credits }
    }

    /// Draw the card for time t: scrim into `cv`, the words on `big` rows
    /// 1..=3.
    pub fn draw(&self, cv: &mut TextCanvas, big: &mut TextCanvas, t: f32) {
        for card in &self.cards {
            let (from, to) = (bars(card.spec.from), bars(card.spec.to));
            if t < from || t >= to {
                continue;
            }
            let local = t - from;
            let gone = ((t - (to - EXIT)) / EXIT).clamp(0.0, 1.0);
            let shown = smooth(local / 0.4);

            // Scrim: the scene darkens softly behind the words.
            let (lo, hi) = card.span;
            for row in 0..CARD_ROWS {
                let fall = smooth((CARD_ROWS as f32 - row as f32) / 3.0);
                for col in 0..COLS as i32 {
                    let out = (lo - col).max(col - hi).max(0) as f32;
                    let k = 0.8 * fall * smooth(1.0 - out / 10.0) * shown * (1.0 - smooth(gone));
                    let cell = cv.cells[row as usize * COLS + col as usize];
                    cv.set(col, row, cell.scaled(1.0 - k));
                }
            }

            let mut gain = 1.0;
            if card.spec.strikes {
                if let Some((ts, _)) = last_strike(t) {
                    gain += 0.45 * (-(t - ts) / 0.12).exp();
                }
            }
            let body_idx = (local * 60.0 * card.body_speed) as usize;
            dissolve(big, &card.body, body_idx, 0, 1, gain, gone);
        }

        if t >= bars(CREDITS_FROM) {
            let local = t - bars(CREDITS_FROM);
            self.credits.draw_at(
                cv,
                local,
                (self.credits.seconds() / 2.5).max(1.0),
                0,
                CREDITS_ROW,
                0.85,
            );
        }
    }
}

/// Draw a clip frame; while `gone` runs 0..1 the letters burn out one by one,
/// flaring to a dot just before they vanish.
fn dissolve(
    canvas: &mut TextCanvas,
    clip: &Clip,
    idx: usize,
    col0: i32,
    row0: i32,
    gain: f32,
    gone: f32,
) {
    if gone <= 0.0 {
        clip.draw(canvas, idx, col0, row0, gain);
        return;
    }
    clip.for_each_cell(idx, |col, row, cell| {
        let threshold = h2(col as i32 * 7 + 3, row as i32 * 13 + 1);
        if gone > threshold {
            return;
        }
        let cell = if gone > threshold - 0.18 {
            Cell::new('.', cell.scaled(1.6).rgb)
        } else {
            cell.scaled(gain)
        };
        canvas.set(col0 + col as i32, row0 + row as i32, cell);
    });
}
