// The valley's scenery, painted into the text grid in the engraved manner
// of the character art: silhouettes are drawn with their edges as slope
// characters over the sky (never as stepped blocks), faces are hatched in
// the direction of their slope, moonlit edges carry a thin light line, and
// distance fades into the haze colour. Painters take a camera position in
// cells; parallax comes from passing each layer a fraction of it.
use crate::ascii::{Grid, CLEAR, COLS, ROWS};
use crate::fx::{fbm, h2, noise, smooth};
use crate::gfx::{mix, ramp, scale};

/// The sky above `to_row`: clear cells over the pixel gradient (drawn by
/// the stage), with each row's colour kept for passes that blend into it.
pub fn sky(g: &mut Grid, stops: &[u32], to_row: i32) {
    for r in 0..ROWS {
        let c = ramp(stops, (r as f32 / to_row.max(1) as f32).min(1.0));
        g.sky[r as usize] = c;
        if r < to_row {
            for col in 0..COLS {
                g.set(col, r, ' ', 0, CLEAR);
            }
        }
    }
}

/// Twinkling stars in clear sky above `to_row`, drifting with `cam`.
pub fn stars(g: &mut Grid, t: f32, cam: f32, to_row: i32, density: f32, k: f32) {
    if k <= 0.0 {
        return;
    }
    let shift = cam as i32;
    for r in 0..to_row.min(ROWS) {
        for c in 0..COLS {
            if g.at(c, r).bg != CLEAR {
                continue;
            }
            let wc = c + shift;
            let h = h2(wc, r * 7 + 3);
            if h > density {
                continue;
            }
            let tw = 0.6 + 0.4 * (t * (0.8 + 2.5 * h2(wc, r)) + 40.0 * h).sin();
            let mag = h2(r, wc);
            let b = (tw * k * (0.25 + 0.75 * mag * mag)).clamp(0.0, 1.0);
            let ch = if mag > 0.93 {
                '+'
            } else if mag > 0.6 {
                '\''
            } else {
                '.'
            };
            let col = mix(0x9fb4c8, 0xfff6e0, h2(wc * 3, r));
            g.put(c, r, ch, scale(col, b));
        }
    }
}

/// Height of a mountain range at world column `x`, in rows above its base.
fn range_height(x: f32, freq: f32, seed: f32, amp: f32) -> f32 {
    let u = x * freq;
    let n = 1.0 - (fbm(u + seed, seed * 0.37, 5) * 2.0 - 1.0).abs();
    let broad = fbm(u * 0.35 + seed * 3.1, 1.7, 3);
    (n.powf(2.2) * 0.85 + broad * 0.3) * amp
}

/// One mountain range's colours.
#[derive(Clone, Copy)]
pub struct Range {
    pub rock: u32,
    pub lit: u32,
    pub snow: Option<u32>,
    pub haze: u32,
}

/// A mountain range: the crest drawn as slope characters, the faces
/// hatched along their slope (lit side brighter), snow streaks on the high
/// lit faces, the base fading into the haze.
pub fn ridge(g: &mut Grid, cam: f32, base: i32, amp: f32, freq: f32, seed: f32, p: Range) {
    let hf = |c: i32| range_height(c as f32 + cam, freq, seed, amp);
    for c in 0..COLS {
        let (hl, h, hr) = (hf(c - 1), hf(c), hf(c + 1));
        let top = base as f32 - h;
        let row = top.floor() as i32;
        let frac = top - top.floor();
        let slope = (hl - hr) * 0.5; // > 0: falls to the right
                                     // The moon is up and to the left: faces falling to the right are lit.
        let light = (0.5 + slope * 0.9).clamp(0.0, 1.0);
        let peak = h >= hl && h >= hr && h > amp * 0.55;
        let crest = if peak {
            '^'
        } else if slope > 0.55 {
            '\\'
        } else if slope < -0.55 {
            '/'
        } else if frac < 0.4 {
            '-'
        } else {
            '_'
        };
        if let Some(cell) = g.cell(c, row) {
            cell.ch = crest;
            cell.fg = mix(p.lit, 0xffffff, 0.25 * light);
        }
        let high = h / amp;
        for r in row + 1..ROWS {
            let depth = (r - row) as f32;
            let fade = smooth(depth / (base as f32 + 6.0 - top).max(4.0));
            let face = mix(
                mix(p.rock, p.lit, light * 0.3 * (1.0 - fade)),
                p.haze,
                fade * 0.6,
            );
            let n = noise((c as f32 + cam) * 0.5, r as f32 * 0.8 + seed);
            let strata = h2(c + cam as i32, r) < 0.04 + 0.12 * light * (1.0 - fade);
            let (mut ch, mut fg) = if strata {
                (
                    if slope >= 0.0 { '\\' } else { '/' },
                    mix(face, p.lit, 0.35 + 0.3 * light),
                )
            } else if n > 0.74 {
                ('.', mix(face, p.lit, 0.25))
            } else {
                (' ', face)
            };
            if let Some(white) = p.snow {
                let line =
                    ((high - 0.55) * 8.0).min(3.5) * (0.5 + noise((c as f32 + cam) * 0.6, seed));
                if depth < line && light > 0.35 {
                    let fleck = noise((c as f32 + cam) * 1.7, r as f32 * 2.3 + seed);
                    if fleck > 0.45 {
                        ch = if fleck > 0.7 { '\'' } else { '.' };
                        fg = mix(face, white, 0.45 + 0.4 * light);
                    }
                }
            }
            g.set(c, r, ch, fg, face);
        }
    }
}

/// One stand of firs' colours.
#[derive(Clone, Copy)]
pub struct Stand {
    pub dark: u32,
    pub edge: u32,
    pub lit_k: f32,
}

/// A row of Douglas firs standing on `base`, up to `height` rows tall, one
/// every `spacing` columns in world space. Tiers of drooping branches; the
/// outline is slope characters over what is behind; the moon side (left)
/// carries a light edge; the trunk shows at the foot.
pub fn firs(g: &mut Grid, cam: f32, base: i32, height: i32, spacing: f32, seed: i32, s: Stand) {
    let first = ((cam - height as f32) / spacing).floor() as i32;
    let last = ((cam + COLS as f32 + height as f32) / spacing).ceil() as i32;
    for i in first..=last {
        let wx = i as f32 * spacing + h2(i, seed) * spacing * 0.7;
        let cx = (wx - cam).round() as i32;
        let hh = (height as f32 * (0.62 + 0.38 * h2(i, seed + 1))) as i32;
        let foot = base + (h2(i, seed + 2) * 2.0) as i32;
        let top = foot - hh;
        for r in top..=foot - 2 {
            let k = (r - top) as f32 / hh.max(1) as f32;
            // Tiers: each flares out and droops, then tucks in.
            let tier = ((r - top) as f32 / (2.0 + hh as f32 * 0.06)).fract();
            let half = (k * hh as f32 * 0.36 * (0.72 + 0.28 * tier)).round() as i32;
            for dc in -half..=half {
                let c = cx + dc;
                if dc > -half && dc < half {
                    let needle = h2(c * 3 + i, r);
                    let lit = (-dc as f32 / half.max(1) as f32 * 0.5 + 0.5) * s.lit_k;
                    let ch = if needle < 0.16 {
                        if dc < 0 {
                            '/'
                        } else {
                            '\\'
                        }
                    } else if needle < 0.26 {
                        '\''
                    } else {
                        ' '
                    };
                    let fg = mix(scale(s.dark, 2.2), s.edge, lit * 0.7);
                    g.set(c, r, ch, fg, mix(s.dark, scale(s.dark, 1.5), lit));
                } else if let Some(cell) = g.cell(c, r) {
                    // The outline over whatever is behind.
                    let left = dc < 0;
                    cell.ch = if r == top {
                        '^'
                    } else if tier > 0.72 {
                        '_'
                    } else if left {
                        '/'
                    } else {
                        '\\'
                    };
                    cell.fg = if left || r == top {
                        mix(scale(s.dark, 2.6), s.edge, s.lit_k)
                    } else {
                        scale(s.dark, 2.2)
                    };
                }
            }
        }
        for r in foot - 1..=foot {
            g.set(cx, r, '|', scale(s.dark, 3.0), scale(s.dark, 0.8));
        }
    }
}

/// Drifting fog over rows r0..r1: cell colours lean toward `col`, densest
/// in the middle of the band.
pub fn fog(g: &mut Grid, t: f32, cam: f32, r0: i32, r1: i32, col: u32, k: f32) {
    if k <= 0.0 {
        return;
    }
    for r in r0.max(0)..r1.min(ROWS) {
        let band = smooth(1.0 - ((r - r0) as f32 / (r1 - r0).max(1) as f32 - 0.5).abs() * 2.0);
        for c in 0..COLS {
            let n = fbm(
                (c as f32 + cam) * 0.03 + t * 0.05,
                r as f32 * 0.11 - t * 0.015,
                4,
            );
            let a = ((n - 0.36) * 2.0).clamp(0.0, 1.0) * band * k;
            if a <= 0.02 {
                continue;
            }
            if let Some(cell) = g.solid(c, r) {
                cell.bg = mix(cell.bg, col, a * 0.75);
                cell.fg = mix(cell.fg, col, a * 0.55);
            }
        }
    }
}

/// The river: rows of flowing water, lit from `palette`, with moonlight
/// glints. `flow` is cells per second.
pub fn river(g: &mut Grid, t: f32, cam: f32, r0: i32, rows: i32, palette: &[u32], flow: f32) {
    for r in r0..r0 + rows {
        let depth = (r - r0) as f32 / rows.max(1) as f32;
        let speed = flow * (0.6 + 0.8 * depth);
        let shift = cam + t * speed;
        for c in 0..COLS {
            let u = c as f32 + shift;
            let wave = 0.5
                + 0.3 * (u * 0.23 + t * 1.7 + r as f32 * 1.3).sin()
                + 0.4 * (noise(u * 0.11, r as f32 * 0.7 + t * 0.3) - 0.5);
            let ch = [' ', '-', '~', '~', '='][((wave * 4.99).max(0.0) as usize).min(4)];
            let col = ramp(palette, wave * (0.45 + 0.55 * depth));
            let bg = mix(palette[0], palette[1], 0.25 * depth);
            let glint = h2(u as i32 / 2, r + (t * 3.0) as i32) > 0.988;
            let fg = if glint { 0xeef6f2 } else { col };
            g.set(c, r, if glint { '*' } else { ch }, fg, bg);
        }
    }
}

/// Valbondione: stone houses with slate roofs and warm windows, San
/// Lorenzo's bell tower. Positions are world columns.
pub fn village(g: &mut Grid, t: f32, cam: f32, base: i32, light: f32) {
    // (world col, width, height)
    const HOUSES: [(i32, i32, i32); 9] = [
        (12, 16, 8),
        (32, 12, 7),
        (50, 18, 9),
        (74, 10, 6),
        (96, 20, 10),
        (124, 14, 7),
        (146, 16, 8),
        (170, 12, 6),
        (190, 18, 9),
    ];
    let wall = 0x1b2220;
    let roof = 0x0c1211;
    for (i, &(wx, w, h)) in HOUSES.iter().enumerate() {
        let x0 = wx - cam as i32;
        let top = base - h;
        let rh = w / 3;
        let mid = x0 + w / 2;
        for k in 0..rh {
            let r = top - rh + k;
            let half = ((k + 1) as f32 / rh as f32 * (w / 2 + 1) as f32) as i32;
            for c in mid - half..=mid + half {
                if c == mid - half {
                    g.put(c, r, '/', 0x5a6864);
                } else if c == mid + half {
                    g.put(c, r, '\\', 0x2a3432);
                } else {
                    g.set(
                        c,
                        r,
                        if (c + k) % 3 == 0 { '=' } else { ' ' },
                        0x26302e,
                        roof,
                    );
                }
            }
        }
        for r in top..base {
            for c in x0..x0 + w {
                let stone = h2(c + i as i32, r) < 0.12;
                g.set(c, r, if stone { '.' } else { ' ' }, 0x3a4441, wall);
            }
        }
        for wr in (top + 2..base - 1).step_by(3) {
            for wc in (x0 + 2..x0 + w - 2).step_by(4) {
                if h2(wc + i as i32 * 17, wr) > 0.7 {
                    continue;
                }
                let flick = if h2(wc, wr * 3) > 0.85 {
                    0.6 + 0.4 * (t * 7.0 + wc as f32).sin().abs()
                } else {
                    1.0
                };
                let warm = mix(0xffb347, 0xffe2a0, h2(wc, wr));
                let k = light * flick;
                g.set(wc, wr, '#', scale(warm, k), scale(warm, k * 0.6));
                g.set(wc + 1, wr, '#', scale(warm, k), scale(warm, k * 0.6));
            }
        }
    }
    // The bell tower, its belfry lit, its spire.
    let tx = 108 - cam as i32;
    let ttop = base - 26;
    for r in ttop..base {
        for c in tx..tx + 6 {
            let edge = c == tx || c == tx + 5;
            g.set(c, r, if edge { '|' } else { ' ' }, 0x4a5652, 0x192020);
        }
    }
    for c in tx + 1..tx + 5 {
        let ch = if c % 2 == 0 { '(' } else { ')' };
        g.set(
            c,
            ttop + 3,
            ch,
            scale(0xffd27a, light),
            scale(0xffb347, light * 0.4),
        );
    }
    for k in 0..9 {
        let half = (k as f32 * 0.4) as i32;
        for c in tx + 2 - half..=tx + 3 + half {
            let r = ttop - 9 + k;
            if k == 0 {
                g.put(c, r, '+', 0x8a9894);
            } else if c == tx + 2 - half {
                g.put(c, r, '/', 0x6a7874);
            } else if c == tx + 3 + half {
                g.put(c, r, '\\', 0x3a4844);
            } else {
                g.set(c, r, ' ', 0, 0x111716);
            }
        }
    }
}

/// Ground from `row` down: grass and stones, scrolling with `cam`, its top
/// edge a line of grass blades.
pub fn ground(g: &mut Grid, cam: f32, row: i32, base: u32, tuft: u32) {
    for r in row.max(0)..ROWS {
        let depth = (r - row) as f32;
        for c in 0..COLS {
            let wc = c + cam as i32;
            let h = h2(wc, r * 11);
            let ch = if r == row {
                if h < 0.5 {
                    '\''
                } else {
                    ','
                }
            } else if h < 0.1 {
                '"'
            } else if h < 0.16 {
                ','
            } else if h < 0.18 {
                '.'
            } else {
                ' '
            };
            let bg = scale(base, 1.0 - depth * 0.02);
            g.set(c, r, ch, mix(tuft, 0xffffff, 0.08 * h2(wc, 3)), bg);
        }
    }
}
