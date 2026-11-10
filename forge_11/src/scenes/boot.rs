// BOOT: the forge lights, over ten bars of intro. Two bars of flight into
// the starfield alone, always forward and gaining speed; copper bars roll in
// from the top; on bar three the SOLVERFORGE banner is beamed in by glyphfx
// and held; a highlight runs across it on bars five and nine, the bars
// swing behind it; on bar seven one line says what this is; on the last bar
// the four verbs of the story land one per beat while the stars warp into
// the first downbeat of DESCRIBE.
use super::{Timed, DIM, EMERALD, LIME, MINT};
use crate::fx;
use crate::gfx::{size, smooth, Frame, Size, S2, W};

/// The banner's cell: 91 columns at 18 px leave a margin either side.
const BANNER_CELL: Size = size(18, 33);
use crate::script::Script;
use crate::timeline::{bars, beat_pulse, Now, BEAT};

const BANNER: &str = "\
███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝";

const VERBS: [&str; 4] = ["DESCRIBE", "GENERATE", "SOLVE", "REPAIR"];

const STOPS: &[&str] = &["--final-gradient-stops", "10b981", "50fc00", "e8ffd6"];
/// The banner lands on bar five; the highlight passes on bar seven and
/// again on bar ten.
const BANNER_AT: f32 = bars(2.0);
const SHINE: [f32; 2] = [bars(4.5), bars(8.0)];
const SHINE_LEN: f32 = 1.8;

pub struct Boot {
    banner: Timed,
    shine: Vec<Timed>,
}

impl Boot {
    pub fn new() -> Self {
        Self {
            banner: Timed::new("beams", BANNER, 1101, STOPS, BANNER_AT, 4.0),
            shine: SHINE
                .iter()
                .enumerate()
                .map(|(i, &at)| {
                    Timed::new("highlight", BANNER, 1110 + i as u64, STOPS, at, SHINE_LEN)
                })
                .collect(),
        }
    }
}

pub fn draw(f: &mut Frame, b: &Boot, _s: &Script, t: f32, n: Now) {
    f.gradient(&[0x000000, 0x02080a, 0x041410, 0x000000]);
    // We fly into the stars: only forward, ever faster, warping into the
    // cut. The starfield is driven by distance travelled, the integral of
    // a speed that never falls, so no star ever moves backwards.
    fx::starfield(f, travelled(t, n.len), 1.0, smooth(t / 3.0), 0x6ee7b7);
    // Copper rolls in from the top on bar two and settles behind the banner.
    let enter = smooth((t - bars(1.0)) / bars(2.0));
    fx::copper(
        f,
        t,
        -300.0 + 440.0 * enter,
        1080.0,
        &[EMERALD, LIME, 0x0ea5a4],
        5,
        0.35 * enter * (0.85 + 0.15 * beat_pulse(t, 5.0)),
    );
    let cols = BANNER.lines().map(|l| l.chars().count()).max().unwrap_or(0) as i32;
    let c = BANNER_CELL;
    let x = (W as i32 - cols * c.w) / 2;
    let plate = smooth((t - BANNER_AT + 0.5) / 0.8);
    if plate > 0.0 {
        f.dim(
            x - 40,
            300,
            x + cols * c.w + 40,
            300 + 6 * c.h + 30,
            1.0 - 0.65 * plate,
        );
    }
    match b
        .shine
        .iter()
        .find(|s| (s.from..s.from + SHINE_LEN).contains(&t))
    {
        Some(shine) => shine.draw(f, t, x, 315, c, 1.0),
        None => b.banner.draw(f, t, x, 315, c, 1.0),
    }
    // Once landed, the banner pulses on the beat.
    if t > BANNER_AT + 4.0 {
        let k = 0.10 * beat_pulse(t, 6.0);
        f.glow(W as f32 * 0.5, 415.0, 700.0, LIME, k);
    }

    // The verbs land on the last bar's four beats, into the first downbeat.
    let y = 610;
    let verbs_at = n.len - bars(1.0);
    let gap = 3 * S2.w;
    let total: i32 = VERBS.iter().map(|v| v.len() as i32 * S2.w).sum::<i32>() + gap * 3;
    let mut vx = (W as i32 - total) / 2;
    for (i, v) in VERBS.iter().enumerate() {
        let at = verbs_at + i as f32 * BEAT - 0.02;
        let k = smooth((t - at) / 0.12);
        let col = if t - at < 0.15 { 0xffffff } else { MINT };
        f.text_shadow(vx, y, v, S2, col, k);
        if i < 3 {
            f.text(vx + v.len() as i32 * S2.w + S2.w, y, "/", S2, DIM, k);
        }
        vx += v.len() as i32 * S2.w + gap;
    }
}

/// Forward speed of the flight at t: a steady cruise that keeps gaining,
/// then the warp over the last bar. Never decreasing.
fn speed(t: f32, len: f32) -> f32 {
    let rush = smooth((t - (len - bars(1.0))) / bars(1.0));
    0.18 + 0.02 * t + 1.6 * rush * rush
}

/// Distance travelled by t: the integral of `speed`, so it only grows.
fn travelled(t: f32, len: f32) -> f32 {
    let steps = (t / 0.01).ceil().max(1.0) as usize;
    let dt = t / steps as f32;
    (0..steps)
        .map(|i| speed((i as f32 + 0.5) * dt, len) * dt)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flight_only_goes_forward() {
        let len = crate::timeline::span(crate::timeline::Section::Boot).1;
        let mut last = -1.0;
        let mut t = 1.0 / 60.0;
        while t < len {
            let d = travelled(t, len);
            assert!(d > last, "the stars move backwards at {t}");
            assert!(speed(t + 1.0 / 60.0, len) >= speed(t, len));
            last = d;
            t += 1.0 / 60.0;
        }
    }
}
