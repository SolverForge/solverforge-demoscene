// 03 SOLVE: the generated app runs on the laptop. The command types in,
// the score replays from the captured trajectory on a log clock while the
// plan's blocks drop into the board; feasibility lands on a beat.
use super::score::{self, Chart, PANEL};
use super::{cursor, run_command, typed, EMERALD, LIME, MINT, PAPER};
use crate::board::Board;
use crate::capture::Capture;
use crate::fx;
use crate::gfx::{clip, ease_out, mix, smooth, Frame, S1, S2, W};
use crate::script::Script;
use crate::timeline::Now;

pub fn draw(f: &mut Frame, c: &Capture, s: &Script, board: &Board, t: f32, n: Now) {
    backdrop(f, t);
    command(f, c, t, n.local);
    let open = smooth(n.local / 0.5);
    score::panel(f, &c.solve.trajectory, &s.solve, None, t, open, EMERALD);
    let (x0, _, x1, y1) = PANEL;
    let chart = Chart {
        x0: x0 + 30,
        y0: 500,
        x1: x1 - 30,
        y1: y1 - 50,
    };
    score::chart(f, &c.solve.trajectory, &s.solve, t, &chart, open);

    let feasible_age = s.solve_feasible.map_or(-1.0, |at| t - at);
    board.draw_grid(f, smooth((n.local - 0.3) / 0.5), EMERALD, 0.0, t);
    for (i, b) in board.blocks.iter().take(board.solved_count).enumerate() {
        let Some(r) = b.solved else { continue };
        let age = t - s.drops[i];
        if age < 0.0 {
            continue;
        }
        let fall = ease_out(age / 0.18);
        let mut r = r;
        r.y0 -= 22.0 * (1.0 - fall);
        r.y1 -= 22.0 * (1.0 - fall);
        let shimmer = if feasible_age >= 0.0 {
            let d = (r.x0 - 760.0) / 1100.0;
            (-((feasible_age * 2.2 - d) * 6.0).powi(2)).exp()
        } else {
            0.0
        };
        let rgb = mix(
            mix(EMERALD, LIME, (-age * 5.0).exp()),
            0xffffff,
            shimmer * 0.7,
        );
        Board::draw_block(f, &r, b, rgb, smooth(age / 0.1));
    }
    if feasible_age >= 0.0 {
        let (cx, cy) = (1310.0, 580.0);
        fx::sparks(f, cx - 300.0, cy, feasible_age, 31, 60, 1.6);
        fx::sparks(f, cx + 300.0, cy, feasible_age - 0.03, 32, 60, 1.6);
    }
    chips(f, c, board.bottom() + 22, smooth((n.local - 1.0) / 0.5));
    super::repair::stamp(f, "FEASIBLE", LIME, feasible_age);
    if (0.0..0.4).contains(&feasible_age) {
        fx::flash(f, LIME, 0.18 * (-feasible_age * 8.0).exp());
    }
}

fn backdrop(f: &mut Frame, t: f32) {
    f.gradient(&[0x010403, 0x03120c, 0x05180f, 0x010302]);
    fx::copper(f, t, 140.0, 1060.0, &[EMERALD, LIME, 0x0ea5a4], 4, 0.16);
    fx::river(f, t, 1000, 4, &[0x02140c, 0x0b5a3a, EMERALD, 0xd1fae5], 0.5);
}

/// The command that ran the solve, typed at the prompt.
fn command(f: &mut Frame, c: &Capture, t: f32, local: f32) {
    let Some(cmd) = run_command(c) else { return };
    let line = shell_line(cmd);
    let size = if line.len() as i32 * S2.w <= W as i32 - 200 {
        S2
    } else {
        S1
    };
    let line = clip(&line, ((W as i32 - 200) / size.w) as usize);
    let n = ((local / 1.2).min(1.0) * line.len() as f32).ceil() as usize;
    f.text_shadow(150, 140, &typed(&line, n), size, PAPER, 1.0);
    if n < line.len() || local < 2.0 {
        cursor(f, 150 + n as i32 * size.w, 140, size, LIME, t);
    }
}

/// What the run did not need: a sign-in, a remote solver.
fn chips(f: &mut Frame, c: &Capture, y: i32, a: f32) {
    if a <= 0.0 {
        return;
    }
    let ver = c.version("solverforge").map(|v| format!("SOLVERFORGE {v}"));
    let mut items = vec!["RUNS ON THIS LAPTOP".to_string(), "NO SIGN-IN".to_string()];
    items.extend(ver);
    let mut x = 770;
    for it in items {
        let w = (it.len() as i32 + 2) * S1.w;
        f.rect(x, y, x + w, y + 30, 0x02140c, 0.85 * a);
        f.frame_rect(x, y, x + w, y + 30, MINT, a * 0.8);
        f.text(x + S1.w, y + 6, &it, S1, MINT, a);
        x += w + 16;
    }
}

/// The command, shortened truthfully. A binary launched by absolute path
/// from a cargo target dir is shown through that dir: `./target/...` when it
/// is the app's own `target`, else `$CARGO_TARGET_DIR/...` (the capture's
/// build command sets CARGO_TARGET_DIR to that directory). Anything else is
/// shown as captured.
pub fn shell_line(cmd: &str) -> String {
    let mut toks: Vec<&str> = cmd.split_whitespace().collect();
    let Some(i) = toks.iter().position(|tok| !tok.contains('=')) else {
        return format!("$ {cmd}");
    };
    let prog = toks[i];
    let parts: Vec<&str> = prog.split('/').collect();
    let Some(r) = parts.iter().rposition(|p| *p == "release" || *p == "debug") else {
        return format!("$ {cmd}");
    };
    if r < 1 {
        return format!("$ {cmd}");
    }
    let dir = if parts[r - 1] == "target" {
        "./target"
    } else {
        "$CARGO_TARGET_DIR"
    };
    let short = format!("{dir}/{}", parts[r..].join("/"));
    toks[i] = &short;
    format!("$ {}", toks.join(" "))
}

#[cfg(test)]
mod tests {
    use super::shell_line;

    #[test]
    fn a_target_dir_binary_is_shown_through_its_dir() {
        assert_eq!(
            shell_line(
                "PORT=7881 ~/.hermes/cache/scratch/forge11/run-target/release/depot_maintenance"
            ),
            "$ PORT=7881 $CARGO_TARGET_DIR/release/depot_maintenance"
        );
        assert_eq!(
            shell_line("/home/x/app/target/release/app"),
            "$ ./target/release/app"
        );
        assert_eq!(shell_line("cargo run --release"), "$ cargo run --release");
    }
}
