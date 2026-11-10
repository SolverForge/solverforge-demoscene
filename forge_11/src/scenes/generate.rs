// 02 GENERATE: the forge, flat out. After the title and its one line the
// screen is texture, not reading: on the left the agent's real transcript
// races past; on the right every captured constraint excerpt is struck on
// the anvil two beats apart -- glyphfx brings it in white-hot, the camera
// punches in on the hit, sparks fly from three points, heat shimmers over
// the code while it quenches to its syntax colours, and the outgoing
// excerpt scrambles into the next one's entrance. Then the struck code
// rains down the panel and assembles into the repo's files, each path
// lighting as its letters land.
use super::{mmss, typed, AMBER, DIM, EMERALD, LIME, MINT, PAPER};
use crate::capture::Capture;
use crate::code::{colours, Listing, CODE_BOX, GUTTER};
use crate::fx::{self, HOT};
use crate::gfx::{clip, ease_out, mix, ramp, size, smooth, wrap, Frame, Ink, Size, S1, S2, W};
use crate::script::Script;
use crate::timeline::{beat_pulse, Now};

const LOG: (i32, i32, i32, i32) = (40, 140, 640, 970);
const EDITOR: (i32, i32, i32, i32) = (680, 140, 1860, 970);
/// Transcript text: a little larger than the 1x grid.
const TS: Size = size(12, 22);
/// The excerpt's glyphfx entrance, then the quench.
const ENTRANCE: f32 = 0.3;
const QUENCH: f32 = 0.5;
/// The outgoing excerpt scrambles over its last fraction of a second.
const SCRAMBLE: f32 = 0.18;

/// The latest strike at or before t: (index, age).
fn strike(s: &Script, t: f32) -> Option<(usize, f32)> {
    let k = s.code.iter().rposition(|&(a, _)| t >= a)?;
    Some((k, t - s.code[k].0))
}

pub fn draw(f: &mut Frame, c: &Capture, s: &Script, listings: &[Listing], t: f32, _n: Now) {
    let forge = t >= s.log_head && t < s.assemble.1;
    let hit = strike(s, t)
        .filter(|_| t < s.assemble.0)
        .map_or(0.0, |(_, a)| (-a * 6.0).exp());
    f.gradient(&[0x020101, 0x0c0402, mix(0x1a0803, 0x3a1004, hit), 0x0a0301]);
    fx::copper(
        f,
        t * if forge { 1.6 } else { 1.0 },
        160.0,
        1060.0,
        &[0xa32000, 0xff7a00, 0x7a1a00],
        4,
        0.18 + 0.3 * hit,
    );
    furnace(f, t, if forge { 1.0 + 0.6 * hit } else { 1.0 });
    let open = smooth((t - s.intro_end(crate::timeline::Section::Generate)) / 0.3);
    if open <= 0.0 {
        return;
    }
    transcript(f, c, s, t, open);
    if t < s.assemble.0 {
        editor(f, c, s, listings, t, open);
    } else {
        repo(f, c, s, listings, t);
    }
    // The hit: a copper accent across the editor, heat over the code, and
    // the camera punching in on the anvil.
    if let Some((_, age)) = strike(s, t).filter(|_| t < s.assemble.0) {
        let (x0, y0, x1, y1) = EDITOR;
        if age < 0.35 {
            let y = y0 as f32 + 100.0 + (y1 - y0 - 120) as f32 * ease_out(age / 0.35);
            fx::accent_bar(f, x0, x1, y, 0xff7a00, 1.0 - age / 0.35);
        }
        let (_, by0, _, by1) = CODE_BOX;
        fx::shimmer(
            f,
            x0 + 10,
            by0,
            x1 - 10,
            by1,
            t,
            7.0 * (1.0 - smooth(age / 0.8)),
        );
        let punch = (-age * 9.0).exp();
        fx::camera(
            f,
            (x0 + x1) as f32 * 0.5,
            420.0,
            1.0 + 0.035 * punch,
            0.0,
            7.0 * (-age * 12.0).exp() * (age * 80.0).cos(),
        );
    }
}

/// Glowing coals along the bottom edge, as characters.
fn furnace(f: &mut Frame, t: f32, heat_k: f32) {
    let tq = (t * 15.0) as i32;
    let cols = W as i32 / S1.w;
    for r in 0..3 {
        let y = 1080 - (3 - r) * S1.h;
        for c in 0..cols {
            let v = fx::h2(c * 7 + r, tq);
            let lick = ((c as f32 * 0.3 + t * 3.0).sin() * 0.5 + 0.5) * 0.4;
            let heat = (r as f32 / 3.0 * 0.7 + lick + v * 0.3 - 0.3) * heat_k;
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
    let k = smooth((t - s.log_head) / 0.3);
    f.text(x0 + 20, y0 + 22, crate::script::LOG_HEADER, S1, AMBER, k);
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
        let text = if step.kind == "say" {
            crate::script::say_line(&step.text)
        } else {
            step.text.clone()
        };
        let body = wrap(&text, cols - head.len());
        let newest = i + 1 == c.steps.len() || t < s.steps[i + 1];
        // The newest step types itself in, fast.
        let total: usize = body.iter().map(|l| l.chars().count()).sum();
        let mut left = if newest {
            ((age / 0.12).min(1.0) * total as f32).ceil() as usize
        } else {
            total
        };
        let flash = (-age * 6.0).exp();
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
    // The log scrolls smoothly: new rows push the old ones up.
    let skip = rows.len().saturating_sub(fit);
    let mut y = y0 + 62;
    for (txt, col, flash) in rows.iter().skip(skip) {
        if *flash > 0.02 {
            f.rect(x0 + 10, y - 2, x1 - 10, y + TS.h + 2, AMBER, 0.3 * flash);
        }
        // Timestamp dim, the rest in the step's colour.
        f.text(x0 + 20, y, &typed(txt, 5), TS, DIM, 1.0);
        let rest: String = txt.chars().skip(5).collect();
        f.text(
            x0 + 20 + 5 * TS.w,
            y,
            &rest,
            TS,
            mix(*col, 0xffffff, *flash * 0.7),
            1.0,
        );
        y += pitch;
    }
}

fn editor(f: &mut Frame, c: &Capture, s: &Script, listings: &[Listing], t: f32, open: f32) {
    let (x0, y0, x1, y1) = EDITOR;
    let Some((k, age)) = strike(s, t) else {
        // The anvil heats up for the first strike.
        let warm = smooth((t - s.log_head) / 2.0);
        fx::heat_glow(
            f,
            (x0 + x1) as f32 * 0.5,
            (y0 + y1) as f32 * 0.5,
            620.0,
            0.5 * warm,
        );
        fx::panel(f, x0, y0, x1, y1, EMERALD, open);
        return;
    };
    let end = s.code[k].1;
    let l = &listings[k];
    let code = &c.code[k];

    // The panel border rings white on the strike, then settles.
    let ring = (-age * 5.0).exp();
    fx::heat_glow(
        f,
        (x0 + x1) as f32 * 0.5,
        (y0 + y1) as f32 * 0.5,
        620.0,
        0.5 + 0.5 * ring,
    );
    fx::panel(f, x0, y0, x1, y1, mix(EMERALD, 0xffffff, ring), open);

    let title_n = ((age / 0.15).min(1.0) * code.title.len() as f32).ceil() as usize;
    let title = clip(&code.title, ((x1 - x0 - 40) / S2.w) as usize);
    f.text_shadow(
        x0 + 24,
        y0 + 18,
        &typed(&title, title_n),
        S2,
        mix(LIME, 0xffffff, ring),
        1.0,
    );
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
    let quench = smooth((age - ENTRANCE) / QUENCH);
    // Leaving: the letters scramble into the next excerpt's entrance.
    let scramble = if k + 1 < s.code.len() {
        smooth((t - (end - SCRAMBLE)) / SCRAMBLE)
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
            0.8 * (1.0 - scramble),
        );
    }
    if age < ENTRANCE {
        let speed = (l.clip.seconds() / ENTRANCE).max(1.0);
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
        let tq = (t * 30.0) as i32;
        for (i, line) in l.lines.iter().enumerate() {
            let cols = colours(line);
            for (j, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let hot = ramp(&HOT, 0.85 - 0.25 * (j as f32 / 80.0));
                let mut rgb = mix(hot, cols[j], quench);
                let mut ch = ch;
                if scramble > 0.0 && scramble > fx::h2(j as i32 * 7 + 3, i as i32 * 13 + 1) {
                    ch = (b'!' + (fx::h2(j as i32 + tq, i as i32) * 90.0) as u8) as char;
                    rgb = ramp(&HOT, 0.6 + 0.4 * fx::h2(i as i32, j as i32 + tq));
                }
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
        let steam = (1.0 - quench) * (1.0 - smooth((age - ENTRANCE - QUENCH) / 0.3));
        let bottom = cy + l.lines.len() as i32 * l.pitch;
        fx::steam(
            f,
            gx as f32,
            (x1 - 60) as f32,
            bottom as f32,
            t,
            steam.max(0.0),
            k as i32,
        );
    }

    // The strike: sparks from three points along the anvil's edge, a
    // white ring from its middle.
    let mid = (x0 + x1) as f32 * 0.5;
    for (j, x) in [gx as f32 + 40.0, mid, (x1 - 80) as f32]
        .into_iter()
        .enumerate()
    {
        fx::sparks(
            f,
            x,
            cy as f32,
            age - 0.015 * j as f32,
            k as i32 * 5 + j as i32 + 1,
            if j == 1 { 120 } else { 80 },
            1.4,
        );
    }
    fx::ring(
        f,
        mid,
        cy as f32,
        30.0 + 520.0 * ease_out(age / 0.45),
        3.0,
        0xffd88a,
        0.8 * (1.0 - smooth(age / 0.45)),
    );
    if age < 0.12 {
        f.glow(mid, cy as f32, 260.0, 0xffd88a, 0.5 * (1.0 - age / 0.12));
    }
    let pulse = beat_pulse(t, 6.0) * 0.12;
    f.rect(x0 + 2, y1 - 6, x1 - 2, y1 - 2, AMBER, pulse);
}

/// The struck code rains down the panel and lands as the repo's files:
/// the last excerpt's letters fall away, then each path assembles letter
/// by letter from glyphs dropping in, white-hot, cooling to lime, with a
/// flash as each file is complete.
fn repo(f: &mut Frame, c: &Capture, s: &Script, listings: &[Listing], t: f32) {
    let (x0, y0, x1, y1) = EDITOR;
    let (a0, a1) = s.assemble;
    let age = t - a0;
    let len = a1 - a0;
    fx::heat_glow(
        f,
        (x0 + x1) as f32 * 0.5,
        (y0 + y1) as f32 * 0.5,
        620.0,
        0.4,
    );
    fx::panel(
        f,
        x0,
        y0,
        x1,
        y1,
        mix(LIME, 0xffffff, (-age * 4.0).exp()),
        1.0,
    );
    // The last excerpt falls away under gravity.
    if let Some(l) = listings.last() {
        let (cx, by0, _, by1) = CODE_BOX;
        let cy = by0 + ((by1 - by0) - l.lines.len() as i32 * l.pitch).max(0) / 3;
        let gx = cx + GUTTER as i32 * l.size.w;
        for (i, line) in l.lines.iter().enumerate() {
            for (j, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let d = age - 0.4 * fx::h2(j as i32, i as i32 + 50);
                if d > 0.9 {
                    continue;
                }
                let fall = 1400.0 * d.max(0.0).powi(2);
                let y = cy + i as i32 * l.pitch + fall as i32;
                if y > y1 - 20 {
                    continue;
                }
                let rgb = ramp(&HOT, 0.9 - 0.5 * d.clamp(0.0, 1.0));
                f.char_at(
                    gx + j as i32 * l.size.w,
                    y,
                    ch,
                    l.size,
                    rgb,
                    1.0 - d.max(0.0),
                );
            }
        }
    }
    // The repo: its name, then each file the code landed in.
    let base = c
        .workdir
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string();
    let head = format!("{base}/");
    let hk = smooth((age - 0.2) / 0.2);
    f.text_shadow(
        x0 + 24,
        y0 + 18,
        &head,
        S2,
        mix(LIME, 0xffffff, 1.0 - hk),
        hk,
    );
    f.rect(x0 + 14, y0 + 64, x1 - 14, y0 + 65, EMERALD, 0.4 * hk);
    let paths = crate::script::repo_paths(c);
    let sz = S2;
    for (i, p) in paths.iter().enumerate() {
        let line = format!("+ {p}");
        let y = y0 + 110 + i as i32 * (sz.h + 18);
        let start = 0.4 + i as f32 * (len - 1.2) / paths.len().max(1) as f32;
        for (j, ch) in line.chars().enumerate() {
            let land = start + 0.3 * fx::h2(j as i32 * 3, i as i32 + 7);
            let d = age - land;
            if d < -0.35 || ch == ' ' {
                continue;
            }
            let p = smooth((d + 0.35) / 0.35);
            let yy = y - (260.0 * (1.0 - p)) as i32;
            let rgb = mix(0xffffff, if j == 0 { LIME } else { MINT }, smooth(d / 0.4));
            f.char_at(x0 + 40 + j as i32 * sz.w, yy, ch, sz, rgb, p);
        }
        // The file complete: a flash along its line.
        let fin = age - (start + 0.35);
        if (0.0..0.5).contains(&fin) {
            let w = line.len() as i32 * sz.w;
            f.glow(
                (x0 + 40) as f32 + w as f32 * ease_out(fin / 0.5),
                (y + sz.h / 2) as f32,
                60.0,
                LIME,
                0.6 * (1.0 - fin / 0.5),
            );
        }
    }
    // Rain through the panel while the files assemble.
    let cols = (x1 - x0 - 40) / S1.w;
    for col in 0..cols {
        let sp = 300.0 + 500.0 * fx::h1(col);
        let off = fx::h1(col + 99) * (y1 - y0) as f32;
        let y = y0 as f32 + (off + age * sp).rem_euclid((y1 - y0) as f32);
        if fx::h1(col * 3) < 0.35 {
            let ch = (b'!' + (fx::h2(col, (t * 20.0) as i32) * 90.0) as u8) as char;
            let k = 0.5 * (1.0 - smooth((age - len + 0.6) / 0.6));
            f.char_at(x0 + 20 + col * S1.w, y as i32, ch, S1, ramp(&HOT, 0.7), k);
        }
    }
}
