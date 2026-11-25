// The characters in the scene, as Amiga sprites. Each is a full-figure
// illustration from the brand direction's dark artwork (see
// assets/sprites/provenance.json), stored as a small binary PPM on a flat
// black ground and drawn at 3x with nearest-neighbour pixels: keyed, levels
// lifted, reduced to six levels a channel through a 4x4 Bayer dither (the
// look of a 64-colour hardware sprite), with a one-pixel dark outline so
// the figure stands off any background. Scene light multiplies in; a rim
// light catches the edge pixels on its side.
//
// Walk cycles are four frames, each aligned on its own figure (alpha
// centroid x, lowest opaque row), so cycling never jitters in place.
use crate::fx::h2;
use crate::gfx::{mix, Frame, H, W};

pub const SCALE: i32 = 3;

const BAYER: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];
const OUTLINE: u32 = 0x040706;

pub struct Sprite {
    pub w: i32,
    pub h: i32,
    /// Colour per art pixel; None is transparent.
    pub px: Vec<Option<u32>>,
    /// Edge pixels: opaque with a transparent neighbour (left, right).
    edge: Vec<(bool, bool)>,
    /// Anchor: figure centre x and feet y, in art pixels.
    pub ax: i32,
    pub ay: i32,
}

fn ppm(data: &[u8]) -> (usize, usize, &[u8]) {
    let mut fields = Vec::new();
    let mut i = 0;
    while fields.len() < 4 {
        while data[i].is_ascii_whitespace() {
            i += 1;
        }
        let start = i;
        while !data[i].is_ascii_whitespace() {
            i += 1;
        }
        fields.push(String::from_utf8_lossy(&data[start..i]).to_string());
    }
    i += 1;
    assert_eq!(fields[0], "P6", "sprites are binary PPMs");
    let w: usize = fields[1].parse().expect("PPM width");
    let h: usize = fields[2].parse().expect("PPM height");
    (w, h, &data[i..i + w * h * 3])
}

impl Sprite {
    pub fn from_ppm(data: &[u8]) -> Self {
        let (w, h, raw) = ppm(data);
        let mut rgb = vec![[0.0f32; 3]; w * h];
        let mut on = vec![false; w * h];
        for i in 0..w * h {
            let p = [
                raw[i * 3] as f32,
                raw[i * 3 + 1] as f32,
                raw[i * 3 + 2] as f32,
            ];
            rgb[i] = p;
            on[i] = p[0].max(p[1]).max(p[2]) > 26.0;
        }
        // Drop isolated specks of the ground.
        let mut keep = on.clone();
        for y in 0..h {
            for x in 0..w {
                if !on[y * w + x] {
                    continue;
                }
                let n = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
                    .iter()
                    .filter(|(dx, dy)| {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        nx >= 0
                            && ny >= 0
                            && (nx as usize) < w
                            && (ny as usize) < h
                            && on[ny as usize * w + nx as usize]
                    })
                    .count();
                keep[y * w + x] = n >= 2;
            }
        }
        // Levels: lift so the bright tenth sits high.
        let mut lums: Vec<f32> = (0..w * h)
            .filter(|&i| keep[i])
            .map(|i| 0.3 * rgb[i][0] + 0.59 * rgb[i][1] + 0.11 * rgb[i][2])
            .collect();
        lums.sort_by(f32::total_cmp);
        let p90 = lums
            .get(lums.len() * 9 / 10)
            .copied()
            .unwrap_or(128.0)
            .max(10.0);
        let gain = (172.0 / p90).clamp(1.0, 2.6);
        let mut px = vec![None; w * h];
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if keep[i] {
                    let d = BAYER[y % 4][x % 4] / 16.0 - 0.5;
                    let q = |v: f32| {
                        let v = (v * gain / 255.0).clamp(0.0, 1.0) * 5.0 + d * 0.9;
                        (v.round().clamp(0.0, 5.0) * 51.0) as u32
                    };
                    px[i] = Some((q(rgb[i][0]) << 16) | (q(rgb[i][1]) << 8) | q(rgb[i][2]));
                }
            }
        }
        // The outline: transparent pixels touching the figure.
        let solid = px.clone();
        let at = |x: i32, y: i32| {
            x >= 0
                && y >= 0
                && x < w as i32
                && y < h as i32
                && solid[y as usize * w + x as usize].is_some()
        };
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                if !at(x, y) && (at(x - 1, y) || at(x + 1, y) || at(x, y - 1) || at(x, y + 1)) {
                    px[y as usize * w + x as usize] = Some(OUTLINE);
                }
            }
        }
        let edge = (0..h as i32)
            .flat_map(|y| (0..w as i32).map(move |x| (x, y)))
            .map(|(x, y)| (at(x, y) && !at(x - 2, y), at(x, y) && !at(x + 2, y)))
            .collect();
        let (mut sx, mut n, mut bottom) = (0.0f32, 0.0f32, 0);
        for y in 0..h {
            for x in 0..w {
                if solid[y * w + x].is_some() {
                    sx += x as f32;
                    n += 1.0;
                    bottom = bottom.max(y as i32);
                }
            }
        }
        Sprite {
            w: w as i32,
            h: h as i32,
            px,
            edge,
            ax: (sx / n.max(1.0)).round() as i32,
            ay: bottom,
        }
    }

    /// Draw with the anchor (centre, feet) at pixel (x, y). `light` tints
    /// the figure by `light_k`; `rim` lights the edge pixels on one side
    /// (-1 left, 1 right); `alpha` < 1 dissolves it pixel by pixel; `bob`
    /// lifts it a few art pixels (laughter).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        f: &mut Frame,
        x: i32,
        y: i32,
        light: u32,
        light_k: f32,
        rim: Option<(u32, i32, f32)>,
        alpha: f32,
        bob: i32,
    ) {
        if alpha <= 0.0 {
            return;
        }
        let x0 = x - self.ax * SCALE;
        let y0 = y - (self.ay + 1 + bob) * SCALE;
        for sy in 0..self.h {
            let py = y0 + sy * SCALE;
            if py + SCALE <= 0 || py >= H as i32 {
                continue;
            }
            for sx in 0..self.w {
                let i = (sy * self.w + sx) as usize;
                let Some(c) = self.px[i] else { continue };
                if alpha < 1.0 && h2(sx, sy) > alpha {
                    continue;
                }
                let mut c = if c == OUTLINE {
                    c
                } else {
                    mix(c, multiply(c, light), light_k)
                };
                if let Some((rgb, dir, k)) = rim {
                    let (l, r) = self.edge[i];
                    if (dir < 0 && l) || (dir > 0 && r) {
                        c = mix(c, rgb, k);
                    }
                }
                let px = x0 + sx * SCALE;
                if px + SCALE <= 0 || px >= W as i32 {
                    continue;
                }
                f.rect(px, py, px + SCALE, py + SCALE, c, 1.0);
            }
        }
    }

    /// Centres of the warm, bright pixels (lit eyes), left of the anchor
    /// and right of it, in art pixels relative to the anchor.
    pub fn warm_points(&self) -> Vec<(f32, f32)> {
        let mut sides = [(0.0f32, 0.0f32, 0.0f32); 2];
        for y in 0..self.h {
            for x in 0..self.w {
                let Some(p) = self.px[(y * self.w + x) as usize] else {
                    continue;
                };
                let (r, g, b) = ((p >> 16) & 255, (p >> 8) & 255, p & 255);
                if r >= 200 && (90..=210).contains(&g) && b <= 110 && r > g + 30 {
                    let s = &mut sides[(x >= self.ax) as usize];
                    s.0 += x as f32;
                    s.1 += y as f32;
                    s.2 += 1.0;
                }
            }
        }
        sides
            .iter()
            .filter(|s| s.2 > 0.0)
            .map(|s| (s.0 / s.2 - self.ax as f32, s.1 / s.2 - (self.ay + 1) as f32))
            .collect()
    }

    /// Top-left of the sprite in pixels when its anchor is at (x, y).
    pub fn origin(&self, x: i32, y: i32) -> (i32, i32) {
        (x - self.ax * SCALE, y - (self.ay + 1) * SCALE)
    }

    /// The sprite as text: one character per 10x18 screen cell, by the
    /// coverage and brightness it holds, for glyphfx to animate.
    pub fn as_text(&self) -> String {
        let (cw, ch) = (10.0 / SCALE as f32, 18.0 / SCALE as f32);
        let cols = (self.w as f32 / cw).ceil() as i32;
        let rows = (self.h as f32 / ch).ceil() as i32;
        let ramp = [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
        let mut out = String::new();
        for r in 0..rows {
            let mut line = String::new();
            for c in 0..cols {
                let (mut cov, mut lum, mut n) = (0.0, 0.0, 0.0);
                for y in (r as f32 * ch) as i32..((r + 1) as f32 * ch) as i32 {
                    for x in (c as f32 * cw) as i32..((c + 1) as f32 * cw) as i32 {
                        n += 1.0;
                        if x < self.w && y < self.h {
                            if let Some(p) = self.px[(y * self.w + x) as usize] {
                                if p != OUTLINE {
                                    cov += 1.0;
                                    lum += (((p >> 16) & 255) + ((p >> 8) & 255) + (p & 255))
                                        as f32
                                        / 765.0;
                                }
                            }
                        }
                    }
                }
                let ch = if cov / n < 0.3 {
                    ' '
                } else {
                    ramp[(1.0 + (lum / cov) * 8.99).min(9.0) as usize]
                };
                line.push(ch);
            }
            out.push_str(line.trim_end());
            out.push('\n');
        }
        out.trim_end_matches('\n').to_string()
    }
}

fn multiply(a: u32, b: u32) -> u32 {
    let ch = |s: u32| (((a >> s) & 255) * ((b >> s) & 255) / 255) << s;
    ch(16) | ch(8) | ch(0)
}

macro_rules! sprite_bytes {
    ($name:literal) => {
        include_bytes!(concat!("../assets/sprites/", $name, ".ppm"))
    };
}

/// Every sprite the film draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Id {
    CordeliaWalk(usize),
    VictorWalk(usize),
    VictorGuard,
    CordeliaRoast,
    VictorRoast,
    Pair,
    Owl,
}

pub struct Cast {
    cordelia_walk: [Sprite; 4],
    victor_walk: [Sprite; 4],
    victor_guard: Sprite,
    cordelia_roast: Sprite,
    victor_roast: Sprite,
    pair: Sprite,
    owl: Sprite,
}

impl Cast {
    pub fn load() -> Self {
        let cw = [
            sprite_bytes!("cordelia-walk-0"),
            sprite_bytes!("cordelia-walk-1"),
            sprite_bytes!("cordelia-walk-2"),
            sprite_bytes!("cordelia-walk-3"),
        ];
        let vw = [
            sprite_bytes!("victor-walk-0"),
            sprite_bytes!("victor-walk-1"),
            sprite_bytes!("victor-walk-2"),
            sprite_bytes!("victor-walk-3"),
        ];
        Self {
            cordelia_walk: cw.map(|b| Sprite::from_ppm(b)),
            victor_walk: vw.map(|b| Sprite::from_ppm(b)),
            victor_guard: Sprite::from_ppm(sprite_bytes!("victor-guard")),
            cordelia_roast: Sprite::from_ppm(sprite_bytes!("cordelia-roast")),
            victor_roast: Sprite::from_ppm(sprite_bytes!("victor-roast")),
            pair: Sprite::from_ppm(sprite_bytes!("pair-dawn")),
            owl: Sprite::from_ppm(sprite_bytes!("owl")),
        }
    }

    pub fn get(&self, id: Id) -> &Sprite {
        match id {
            Id::CordeliaWalk(i) => &self.cordelia_walk[i % 4],
            Id::VictorWalk(i) => &self.victor_walk[i % 4],
            Id::VictorGuard => &self.victor_guard,
            Id::CordeliaRoast => &self.cordelia_roast,
            Id::VictorRoast => &self.victor_roast,
            Id::Pair => &self.pair,
            Id::Owl => &self.owl,
        }
    }
}

/// Which walk frame at time `t` for a stride of `period` seconds.
pub fn walk_frame(t: f32, period: f32, offset: f32) -> usize {
    (((t / period + offset) * 4.0).floor() as i64).rem_euclid(4) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sprite_keys_out_its_ground_and_keeps_a_figure() {
        let cast = Cast::load();
        let mut all: Vec<&Sprite> = cast
            .cordelia_walk
            .iter()
            .chain(cast.victor_walk.iter())
            .collect();
        all.extend([
            &cast.victor_guard,
            &cast.cordelia_roast,
            &cast.victor_roast,
            &cast.pair,
            &cast.owl,
        ]);
        for s in all {
            let opaque = s.px.iter().filter(|c| c.is_some()).count();
            let total = s.px.len();
            assert!(opaque > total / 10, "{opaque}/{total} pixels: no figure");
            assert!(
                opaque < total * 3 / 4,
                "{opaque}/{total} pixels: ground not keyed"
            );
        }
    }

    #[test]
    fn walk_frames_share_a_baseline() {
        let cast = Cast::load();
        for cycle in [&cast.cordelia_walk, &cast.victor_walk] {
            let feet: Vec<i32> = cycle.iter().map(|s| s.ay).collect();
            let (lo, hi) = (*feet.iter().min().unwrap(), *feet.iter().max().unwrap());
            assert!(hi - lo <= 2, "feet rows {feet:?}");
        }
    }
}
