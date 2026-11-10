// 02 GENERATE: the forge. On the left the agent's real transcript scrolls
// by; on the right every captured constraint excerpt is struck on the
// anvil: glyphfx brings it in white-hot, it lands on the beat in a burst of
// sparks, then quenches from orange to its syntax colours in a puff of steam.
use super::{mmss, typed, AMBER, DIM, EMERALD, LIME, MINT, PAPER};
use crate::capture::Capture;
use crate::code::{colours, Listing, CODE_BOX, GUTTER};
use crate::fx::{self, HOT};
use crate::gfx::{clip, mix, ramp, size, smooth, wrap, Frame, Ink, Size, S1, S2, W};
use crate::script::Script;
use crate::timeline::{beat_pulse, Now};

const LOG: (i32, i32, i32, i32) = (40, 140, 640, 970);
const EDITOR: (i32, i32, i32, i32) = (680, 140, 1860, 970);
/// Transcript text: a little larger than the 1x grid.
const TS: Size = size(12, 22);

pub fn draw(f: &mut Frame, c: &Capture, s: &Script, listings: &[Listing], t: f32, _n: Now) {
    f.gradient(&[0x020101, 0x0c0402, 0x1a0803, 0x0a0301]);
    fx::copper(
        f,
        t,
        160.0,
        1060.0,
        &[0xa32000, 0xff7a00, 0x7a1a00],
        4,
        0.18,
    );
    furnace(f, t);
    let open = smooth((t - s.intro_end(crate::timeline::Section::Generate)) / 0.4);
    if open <= 0.0 {
        return;
    }
    transcript(f, c, s, t, open);
    editor(f, c, s, listings, t, open);
}

/// Glowing coals along the bottom edge, as characters.
fn furnace(f: &mut Frame, t: f32) {
    let tq = (t * 15.0) as i32;
    let cols = W as i32 / S1.w;
    for r in 0..3 {
        let y = 1080 - (3 - r) * S1.h;
        for c in 0..cols {
            let v = fx::h2(c * 7 + r, tq);
            let lick = ((c as f32 * 0.3 + t * 3.0).sin() * 0.5 + 0.5) * 0.4;
            let heat = r as f32 / 3.0 * 0.7 + lick + v * 0.3 - 0.3;
            if heat > 0.05 {
                let ch = ['^', '*', '"', '\'', '.', '^'][(v * 6.0) as usize % 6];
                f.char_at(c * S1.w, y, ch, S1, ramp(&HOT, heat.min(1.0) * 0.8), 0.9);
            }
        }
    }
}

fn transcript(f: &mut Frame, c: &Capture, s: &Script, t: f32, open: f32) {
    let (x0, y0, x1, y1) = LOG;
    fx::panel(f, x0, y0, x1, y1, AMBER, open);
    if open < 0.5 {
        return;
    }
    f.text(x0 + 20, y0 + 22, "AGENT TRANSCRIPT", S1, AMBER, 1.0);
    let skill = "+ solverforge-modeling skill";
    f.text(
        x1 - 20 - skill.len() as i32 * S1.w,
        y0 + 22,
        skill,
        S1,
        LIME,
        0.9,
    );
    f.rect(x0 + 14, y0 + 48, x1 - 14, y0 + 49, AMBER, 0.35);

    let cols = ((x1 - x0 - 40) / TS.w) as usize;
    let pitch = TS.h + 4;
    let fit = ((y1 - 20 - (y0 + 62)) / pitch) as usize;
    // Every step shown so far, wrapped, newest last.
    let mut rows: Vec<(String, u32, f32)> = Vec::new();
    for (i, (step, &at)) in c.steps.iter().zip(&s.steps).enumerate() {
        if t < at {
            break;
        }
        let age = t - at;
        let (mark, col) = match step.kind.as_str() {
            "tool" => ("$", AMBER),
            "file" => ("+", LIME),
            _ => (">", PAPER),
        };
        let head = format!("{} {} ", mmss(step.t_ms), mark);
        let body = wrap(&step.text, cols - head.len());
        let newest = i + 1 == c.steps.len() || t < s.steps[i + 1];
        // The newest step types itself in.
        let total: usize = body.iter().map(|l| l.chars().count()).sum();
        let mut left = if newest {
            ((age / 0.35).min(1.0) * total as f32).ceil() as usize
        } else {
            total
        };
        let flash = if newest { (-age * 4.0).exp() } else { 0.0 };
        for (j, line) in body.iter().enumerate() {
            let k = left.min(line.chars().count());
            if j > 0 && k == 0 {
                break;
            }
            left -= k;
            let prefix = if j == 0 {
                head.clone()
            } else {
                " ".repeat(head.len())
            };
            rows.push((format!("{prefix}{}", typed(line, k)), col, flash));
        }
    }
    let skip = rows.len().saturating_sub(fit);
    let mut y = y0 + 62;
    for (txt, col, flash) in rows.iter().skip(skip) {
        if *flash > 0.02 {
            f.rect(x0 + 10, y - 2, x1 - 10, y + TS.h + 2, AMBER, 0.25 * flash);
        }
        // Timestamp dim, the rest in the step's colour.
        f.text(x0 + 20, y, &typed(txt, 5), TS, DIM, 1.0);
        let rest: String = txt.chars().skip(5).collect();
        f.text(
            x0 + 20 + 5 * TS.w,
            y,
            &rest,
            TS,
            mix(*col, 0xffffff, *flash * 0.6),
            1.0,
        );
        y += pitch;
    }
}

fn editor(f: &mut Frame, c: &Capture, s: &Script, listings: &[Listing], t: f32, open: f32) {
    let (x0, y0, x1, y1) = EDITOR;
    let Some(k) = s.code.iter().rposition(|&(a, _)| t >= a) else {
        fx::panel(f, x0, y0, x1, y1, EMERALD, open);
        return;
    };
    let (start, end) = s.code[k];
    let age = t - start;
    let l = &listings[k];
    let code = &c.code[k];

    // The panel border rings white on the strike, then settles.
    let ring = (-age * 5.0).exp();
    fx::heat_glow(
        f,
        (x0 + x1) as f32 * 0.5,
        (y0 + y1) as f32 * 0.5,
        620.0,
        (1.0 - smooth(age / 2.0)) * 0.9,
    );
    fx::panel(f, x0, y0, x1, y1, mix(EMERALD, 0xffffff, ring), open);

    let title_n = ((age / 0.4).min(1.0) * code.title.len() as f32).ceil() as usize;
    let title = clip(&code.title, ((x1 - x0 - 40) / S2.w) as usize);
    f.text_shadow(x0 + 24, y0 + 18, &typed(&title, title_n), S2, LIME, 1.0);
    let path = clip(&code.path, ((x1 - x0 - 200) / S1.w) as usize);
    f.text(x0 + 24, y0 + 64, &path, S1, MINT, 0.9);
    let count = format!("{}/{}", k + 1, c.code.len());
    f.text(
        x1 - 24 - count.len() as i32 * S1.w,
        y0 + 64,
        &count,
        S1,
        DIM,
        0.9,
    );
    f.rect(x0 + 14, y0 + 92, x1 - 14, y0 + 93, EMERALD, 0.4);

    let (cx, by0, _, by1) = CODE_BOX;
    let cy = by0 + ((by1 - by0) - l.lines.len() as i32 * l.pitch).max(0) / 3;
    let entrance = ((end - start) * 0.3).clamp(0.5, 1.1);
    let quench = smooth((age - entrance) / 1.2);
    // Leaving: letters burn out over the slot's last 0.3 s.
    let gone = if k + 1 < s.code.len() {
        ((t - (end - 0.3)) / 0.3).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let gx = cx + GUTTER as i32 * l.size.w;
    for (i, _) in l.lines.iter().enumerate() {
        let ln = format!("{:>3}", i + 1);
        f.text(
            cx,
            cy + i as i32 * l.pitch,
            &ln,
            l.size,
            DIM,
            0.8 * (1.0 - gone),
        );
    }
    if age < entrance {
        let speed = (l.clip.seconds() / entrance).max(1.0);
        let idx = (age * 60.0 * speed) as usize;
        // glyphfx rows are one cell high; spread them to the line pitch.
        l.clip.for_each_cell(idx, |col, row, cell| {
            let mut ink = Ink {
                f,
                x: gx,
                y: cy + row as i32 * (l.pitch - l.size.h),
                size: l.size,
                gain: 1.0,
                tint: None,
            };
            demofx::CellSink::set(&mut ink, col as i32, row as i32, cell);
        });
    } else {
        for (i, line) in l.lines.iter().enumerate() {
            let cols = colours(line);
            for (j, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                if gone > 0.0 && gone > fx::h2(j as i32 * 7 + 3, i as i32 * 13 + 1) {
                    continue;
                }
                let hot = ramp(&HOT, 0.75 - 0.2 * (j as f32 / 80.0));
                let rgb = mix(hot, cols[j], quench);
                f.char_at(
                    gx + j as i32 * l.size.w,
                    cy + i as i32 * l.pitch,
                    ch,
                    l.size,
                    rgb,
                    1.0,
                );
            }
        }
        let steam = (1.0 - quench) * (1.0 - smooth((age - entrance - 1.4) / 0.6));
        let bottom = cy + l.lines.len() as i32 * l.pitch;
        fx::steam(
            f,
            gx as f32,
            (x1 - 60) as f32,
            bottom as f32,
            t,
            steam.max(0.0) * 0.9 + 0.0,
            k as i32,
        );
    }

    // The strike: sparks from both top corners of the listing.
    fx::sparks(
        f,
        gx as f32 + 40.0,
        cy as f32,
        age,
        k as i32 * 2 + 1,
        70,
        1.0,
    );
    fx::sparks(
        f,
        (x1 - 80) as f32,
        cy as f32,
        age - 0.02,
        k as i32 * 2 + 2,
        70,
        1.0,
    );
    let pulse = beat_pulse(t, 6.0) * 0.08;
    f.rect(x0 + 2, y1 - 6, x1 - 2, y1 - 2, AMBER, pulse);
}
