// BOOT: the name decrypts in, the portrait prints dot by dot, and a shell
// answers who, where and what.
use demofx::Clip;

use super::{dot_grid, typed, LIME, PAPER};
use crate::gfx::{advance, Frame};
use crate::portrait::Portrait;
use crate::timeline::Now;

const SHELL: [(f32, &str, &str); 3] = [
    (3.2, "$ whoami", "vittorio distefano"),
    (5.4, "$ cat /etc/location", "bergamo, italy"),
    (7.6, "$ ls ~/offers", "planning advisory agents redona"),
];

pub fn draw(f: &mut Frame, portrait: &Portrait, name: &Clip, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.clear(0x020503);
    dot_grid(f, LIME, 0.05 * fade);

    // The name, two lines at 5x.
    let local = n.local - 1.6;
    if local > 0.0 {
        let speed = (name.seconds() / 3.0).max(1.0);
        f.clip(name, (local * 60.0 * speed) as usize, 90, 160, 5, fade);
    }
    let sub = "ENGINEER. FOUNDER OF SOLVERFORGE.";
    f.text(
        90,
        268,
        &typed(sub, 4.4, 30.0, n.local),
        2,
        PAPER,
        0.85 * fade,
    );

    // The shell.
    for (i, (at, cmd, out)) in SHELL.iter().enumerate() {
        let y = 340 + i as i32 * 64;
        let shown = typed(cmd, *at, 16.0, n.local);
        f.text(90, y, &shown, 2, LIME, fade);
        let done = at + cmd.len() as f32 / 16.0 + 0.25;
        if n.local > done {
            f.text(90, y + 26, out, 2, PAPER, 0.8 * fade);
        } else if n.local > *at - 0.6 {
            // Blinking block cursor while typing.
            if (t * 3.0).fract() < 0.6 {
                let x = 90 + shown.chars().count() as i32 * advance(2);
                f.rect(x + 2, y, x + 18, y + 18, LIME, fade);
            }
        }
    }

    // The portrait prints top to bottom.
    let reveal = ((n.local - 1.2) / 5.5).clamp(0.0, 1.0);
    portrait.draw(f, 810.0, 112.0, 4.0, LIME, reveal, fade, t);
    1.0
}
