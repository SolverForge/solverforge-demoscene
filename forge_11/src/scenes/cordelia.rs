// CORDELIA: the agent's hero beat inside 04 REPAIR. The bus breaks down,
// the band drops to a heartbeat under the alarm, and two bars later
// Cordelia glitches in over the score panel. Her performance runs through
// six generated keyframes on the beat: eyes narrowed on the problem, a
// glint that becomes a glow, the glow building, white-hot ignition, the
// fire on the downbeat where the repair enters the running solve (eye-
// lasers sear across to the board while the changed jobs travel), and a
// smirk as the plan heals and she tears out.
//
// Every keyframe is an image-to-image edit of the brand direction's
// approved dark DEFIANCE artwork; prompts, model, request ids and hashes are
// in assets/cordelia/provenance.json. No frame has beams painted in: the
// beams here are an effect layer, anchored to the iris centres measured on
// each frame and carried through the same transform as the picture, so
// they stay locked to her eyes while she breathes, pushes in and recoils.
use super::{EMERALD, MINT, PAPER};
use crate::fx;
use crate::gfx::{mix, smooth, Frame, S1, W};
use crate::script::{Performance, Script};
use crate::timeline::BEAT;

const NARROW: &[u8] = include_bytes!("../../assets/cordelia/narrow.ppm");
const CHARGE: &[u8] = include_bytes!("../../assets/cordelia/charge.ppm");
const BUILD: &[u8] = include_bytes!("../../assets/cordelia/build.ppm");
const IGNITE: &[u8] = include_bytes!("../../assets/cordelia/ignite.ppm");
const FIRE: &[u8] = include_bytes!("../../assets/cordelia/fire.ppm");
const SMIRK: &[u8] = include_bytes!("../../assets/cordelia/smirk.ppm");

/// Keyframes in performance order.
const KEYS: usize = 6;
const SMIRK_KEY: usize = 5;

/// Iris centres [left, right] in each 680x562 keyframe (provenance.json).
const EYES: [[(f32, f32); 2]; KEYS] = [
    [(437.0, 343.0), (509.0, 370.0)],
    [(437.0, 343.0), (510.0, 371.0)],
    [(434.0, 346.0), (513.0, 370.0)],
    [(437.0, 343.0), (510.0, 371.0)],
    [(440.0, 340.0), (514.0, 363.0)],
    [(435.0, 315.0), (504.0, 342.0)],
];

/// How lit her eyes are on each keyframe (the engine's glow follows it).
const HEAT: [f32; KEYS] = [0.12, 0.35, 0.6, 0.85, 1.0, 0.3];

/// The panel she occupies: over the score panel's chart and list, the
/// event line, the score and the board stay in view.
pub const PANEL: (i32, i32, i32, i32) = (40, 452, 720, 970);

pub const OUT: f32 = 0.4;

struct Image {
    w: usize,
    h: usize,
    px: Vec<u32>,
}

/// A binary PPM (P6, maxval 255), parsed by hand.
fn ppm(data: &[u8]) -> Image {
    let mut fields = Vec::new();
    let mut i = 0;
    while fields.len() < 4 {
        while data[i].is_ascii_whitespace() {
            i += 1;
        }
        if data[i] == b'#' {
            while data[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let start = i;
        while !data[i].is_ascii_whitespace() {
            i += 1;
        }
        fields.push(
            std::str::from_utf8(&data[start..i])
                .unwrap_or("0")
                .to_string(),
        );
    }
    i += 1;
    assert_eq!(fields[0], "P6", "cordelia keyframe is not a binary PPM");
    let w: usize = fields[1].parse().unwrap_or(0);
    let h: usize = fields[2].parse().unwrap_or(0);
    let px = data[i..i + w * h * 3]
        .as_chunks::<3>()
        .0
        .iter()
        .map(|c| (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32)
        .collect();
    Image { w, h, px }
}

impl Image {
    fn sample(&self, x: f32, y: f32) -> u32 {
        let x = x.clamp(0.0, self.w as f32 - 1.001);
        let y = y.clamp(0.0, self.h as f32 - 1.001);
        let (x0, y0) = (x as usize, y as usize);
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let p = |xx: usize, yy: usize| self.px[yy * self.w + xx];
        let top = mix(p(x0, y0), p(x0 + 1, y0), fx);
        let bot = mix(p(x0, y0 + 1), p(x0 + 1, y0 + 1), fx);
        mix(top, bot, fy)
    }
}

pub struct Cordelia {
    frames: Vec<Image>,
    pub beat: Performance,
}

/// Where the beams strike, in frame pixels.
pub struct Targets {
    pub left: Option<(f32, f32)>,
    pub right: Option<(f32, f32)>,
}

impl Cordelia {
    pub fn new(s: &Script) -> Self {
        Self {
            frames: [NARROW, CHARGE, BUILD, IGNITE, FIRE, SMIRK]
                .iter()
                .map(|d| ppm(d))
                .collect(),
            beat: s.cordelia,
        }
    }

    pub fn active(&self, t: f32) -> bool {
        (self.beat.enter..self.beat.exit + OUT).contains(&t)
    }

    fn starts(&self) -> [f32; KEYS] {
        let b = self.beat;
        [b.enter, b.charge, b.build, b.ignite, b.fire, b.cut]
    }

    /// The keyframe at t, the one before it, and the crossfade between.
    fn key(&self, t: f32) -> (usize, usize, f32) {
        let st = self.starts();
        let k = st.iter().rposition(|&a| t >= a).unwrap_or(0);
        if k == 0 {
            return (0, 0, 1.0);
        }
        // Dissolves on the beat; the smirk cuts in fast under a glitch.
        let len = if k == SMIRK_KEY { 0.12 } else { 0.18 };
        (k, k - 1, smooth((t - st[k]) / len))
    }

    /// The picture transform at t: scale and the source point shown at the
    /// panel centre. Push-in over the beat, breathing, a kick on the fire,
    /// a small recoil on the smirk.
    fn transform(&self, t: f32) -> (f32, f32, f32) {
        let b = self.beat;
        let p = smooth((t - b.enter) / (b.cut - b.enter));
        let kick = if t >= b.fire {
            0.035 * (-(t - b.fire) * 7.0).exp()
        } else {
            0.0
        };
        let recoil = if t >= b.cut {
            smooth((t - b.cut) / 0.25)
        } else {
            0.0
        };
        let scale = 1.0 + 0.10 * p + kick - 0.04 * recoil;
        let breathe = (t * std::f32::consts::TAU / (2.0 * BEAT)).sin();
        // Drift towards her eyes as she pushes in (parallax on the face).
        let cx = 340.0 + 60.0 * p + 2.0 * breathe;
        let cy = 281.0 + 40.0 * p - 3.0 * breathe - 14.0 * recoil;
        (scale, cx, cy)
    }

    /// Source pixel -> frame pixel through the current transform.
    fn to_frame(&self, t: f32, (sx, sy): (f32, f32)) -> (f32, f32) {
        let (scale, cx, cy) = self.transform(t);
        let (x0, y0, x1, y1) = PANEL;
        let (pcx, pcy) = ((x0 + x1) as f32 * 0.5, (y0 + y1) as f32 * 0.5);
        (pcx + (sx - cx) * scale, pcy + (sy - cy) * scale)
    }

    /// The eyes on screen at t, following the anchors across a dissolve.
    pub fn eyes(&self, t: f32) -> [(f32, f32); 2] {
        let (k, prev, blend) = self.key(t);
        let lerp =
            |a: (f32, f32), b: (f32, f32)| (a.0 + (b.0 - a.0) * blend, a.1 + (b.1 - a.1) * blend);
        let e = [
            lerp(EYES[prev][0], EYES[k][0]),
            lerp(EYES[prev][1], EYES[k][1]),
        ];
        [self.to_frame(t, e[0]), self.to_frame(t, e[1])]
    }

    fn heat(&self, t: f32) -> f32 {
        let (k, prev, blend) = self.key(t);
        let mut h = HEAT[prev] + (HEAT[k] - HEAT[prev]) * blend;
        if k == SMIRK_KEY {
            // The afterglow dies away over the bar.
            h *= (-(t - self.beat.cut) * 1.5).exp();
        }
        h
    }

    pub fn draw(&self, f: &mut Frame, t: f32, targets: &Targets) {
        if !self.active(t) {
            return;
        }
        let b = self.beat;
        let since = t - b.enter;
        let gone = t - b.exit;
        // Entrance: rows tear in over 0.3 s; exit: they tear out; a short
        // glitch accent on the fire and on the smirk.
        let accent = |at: f32, len: f32| {
            let a = t - at;
            if (0.0..len).contains(&a) {
                0.45 * (1.0 - a / len)
            } else {
                0.0
            }
        };
        let tear = if since < 0.3 {
            1.0 - since / 0.3
        } else if gone > 0.0 {
            gone / OUT
        } else {
            accent(b.fire, 0.12).max(accent(b.cut, 0.2))
        };
        let alpha = smooth(since / 0.12) * (1.0 - smooth(gone / OUT));
        let (x0, y0, x1, y1) = PANEL;
        let (scale, cx, cy) = self.transform(t);
        let (pcx, pcy) = ((x0 + x1) as f32 * 0.5, (y0 + y1) as f32 * 0.5);
        let (key, prev, blend) = self.key(t);
        let (img, prev) = (&self.frames[key], &self.frames[prev]);
        let jitter_t = (t * 60.0) as i32;
        let inv = 1.0 / scale;
        // Light from her eyes flickers over hair and face while she fires.
        let flicker = if (b.fire..b.cut).contains(&t) {
            0.06 * (0.5 + 0.5 * (t * 37.0).sin() * (t * 23.0).cos())
        } else {
            0.0
        };
        for y in y0..y1 {
            let band = (y - y0) / 6;
            let shift = if tear > 0.0 {
                let h = fx::h2(band, jitter_t);
                if h < tear {
                    (fx::h2(band, jitter_t + 7) - 0.5) * 240.0 * tear
                } else {
                    0.0
                }
            } else {
                0.0
            };
            // A dropped band during the tear.
            if tear > 0.0 && fx::h2(band, jitter_t + 3) < 0.4 * tear {
                continue;
            }
            let sy = cy + (y as f32 - pcy) * inv;
            let scan = if y % 3 == 0 { 0.82 } else { 1.0 };
            for x in x0..x1 {
                let sx = cx + (x as f32 + shift - pcx) * inv;
                let mut c = img.sample(sx, sy);
                if blend < 1.0 {
                    c = mix(prev.sample(sx, sy), c, blend);
                }
                if tear > 0.0 {
                    // Chromatic split while she tears.
                    let r = img.sample(sx + 8.0 * tear, sy) & 0xff0000;
                    c = (c & 0x00ffff) | r;
                }
                if flicker > 0.0 {
                    c = crate::gfx::add_rgb(c, EMERALD, flicker);
                }
                let c = crate::gfx::scale(c, scan);
                f.blend(x, y, c, alpha);
            }
        }
        // The frame around her: a CRT plate with a name tag.
        let edge = mix(
            EMERALD,
            0xffffff,
            0.4 * (-(t - b.fire).max(0.0) * 6.0).exp(),
        );
        f.frame_rect(x0, y0, x1, y1, edge, 0.9 * alpha);
        f.frame_rect(x0 + 3, y0 + 3, x1 - 3, y1 - 3, EMERALD, 0.35 * alpha);
        let tag = " CORDELIA // AGENT ";
        let tw = tag.len() as i32 * S1.w;
        let ty = y1 - 34;
        f.rect(x0 + 14, ty, x0 + 14 + tw, ty + 24, 0x020806, 0.85 * alpha);
        f.frame_rect(x0 + 14, ty, x0 + 14 + tw, ty + 24, MINT, alpha);
        f.text(x0 + 14, ty + 3, tag, S1, PAPER, alpha);

        self.eyes_glow(f, t, alpha);
        self.beams(f, t, targets, alpha);
    }

    /// Light pooling in the irises, as hot as the keyframe.
    fn eyes_glow(&self, f: &mut Frame, t: f32, alpha: f32) {
        let b = self.beat;
        let heat = self.heat(t);
        let flick = 0.85 + 0.15 * (t * 53.0).sin();
        let firing = (b.fire..b.cut).contains(&t);
        for (i, (ex, ey)) in self.eyes(t).into_iter().enumerate() {
            let r = 8.0 + 30.0 * heat + if firing { 14.0 } else { 0.0 };
            let k = alpha * (0.15 + 0.85 * heat) * flick;
            f.glow(ex, ey, r * 2.2, EMERALD, 0.35 * k);
            f.glow(ex, ey, r, MINT, 0.6 * k);
            f.glow(ex, ey, r * 0.35, 0xffffff, k * heat);
            // A star glint on the ignite beat.
            let since = t - b.ignite;
            if (0.0..0.5).contains(&since) {
                let g = alpha * (1.0 - since / 0.5);
                let l = 40.0 + 60.0 * smooth(since / 0.15) + 10.0 * i as f32;
                f.line((ex - l, ey), (ex + l, ey), 2.0, 0xffffff, 0.8 * g);
                f.line(
                    (ex, ey - l * 0.6),
                    (ex, ey + l * 0.6),
                    2.0,
                    0xffffff,
                    0.6 * g,
                );
            }
        }
    }

    fn beams(&self, f: &mut Frame, t: f32, targets: &Targets, alpha: f32) {
        let b = self.beat;
        let age = t - b.fire;
        if !(0.0..=b.cut - b.fire + 0.12).contains(&age) {
            return;
        }
        // The beams reach the board over a few frames, hold, and cut.
        let reach = smooth(age / 0.1);
        let fade = alpha * (1.0 - smooth((t - b.cut) / 0.12));
        let eyes = self.eyes(t);
        let fallback = (W as f32 * 0.66, 560.0);
        let hits = [
            targets.left.or(targets.right).unwrap_or(fallback),
            targets.right.or(targets.left).unwrap_or(fallback),
        ];
        for (i, (&(ex, ey), &(hx, hy))) in eyes.iter().zip(hits.iter()).enumerate() {
            let (tx, ty) = (ex + (hx - ex) * reach, ey + (hy - ey) * reach);
            // Jitter the far end a little; the origin never leaves the eye.
            let j = (fx::h2(i as i32, (t * 60.0) as i32) - 0.5) * 6.0;
            let (tx, ty) = (tx + j * 0.4, ty + j);
            let pulse = 0.85 + 0.15 * (t * 40.0 + i as f32 * 2.0).sin();
            // Each beat the beams surge with the hum.
            let surge = 1.0 + 0.25 * crate::timeline::beat_pulse(t, 7.0);
            let w = (1.0 + 0.6 * (-age * 8.0).exp()) * surge;
            f.line((ex, ey), (tx, ty), 34.0 * w, EMERALD, 0.10 * fade);
            f.line((ex, ey), (tx, ty), 16.0 * w, EMERALD, 0.30 * fade * pulse);
            f.line((ex, ey), (tx, ty), 8.0 * w, MINT, 0.6 * fade * pulse);
            f.line((ex, ey), (tx, ty), 3.0, 0xffffff, fade);
            f.glow(ex, ey, 40.0, MINT, 0.7 * fade);
            if reach > 0.95 {
                let flick = 0.7 + 0.3 * (t * 71.0 + i as f32).sin();
                f.glow(tx, ty, 70.0, EMERALD, 0.6 * fade * flick);
                f.glow(tx, ty, 26.0, 0xffffff, 0.8 * fade * flick);
                // A fresh burst of sparks every eighth note.
                let burst = ((age / (BEAT / 2.0)) as i32).max(0);
                let sage = age - burst as f32 * BEAT / 2.0;
                fx::sparks(f, tx, ty, sage, 900 + 10 * burst + i as i32, 18, 0.9);
            }
        }
        // The strike's flash on the fire downbeat.
        if age < 0.3 {
            fx::flash(f, MINT, 0.18 * (1.0 - age / 0.3) * alpha);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyframes_decode_at_the_anchor_size() {
        for data in [NARROW, CHARGE, BUILD, IGNITE, FIRE, SMIRK] {
            let img = ppm(data);
            assert_eq!((img.w, img.h), (680, 562));
            assert_eq!(img.px.len(), 680 * 562);
        }
        for eyes in EYES {
            for (x, y) in eyes {
                assert!((0.0..680.0).contains(&x) && (0.0..562.0).contains(&y));
            }
        }
    }
}

#[cfg(test)]
mod performance {
    use super::*;
    use crate::capture::Capture;

    /// The performance runs in order, on the beat, inside REPAIR, and the
    /// beams are held for at least a bar.
    #[test]
    fn the_performance_is_on_the_beat() {
        let c = Capture::parse(include_str!("../../assets/capture.placeholder.json")).unwrap();
        let mut caps = vec![c];
        let real = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");
        if std::path::Path::new(real).exists() {
            caps.push(Capture::load(real).unwrap());
        }
        for c in caps {
            let s = Script::new(&c);
            let p = s.cordelia;
            let seq = [p.enter, p.charge, p.build, p.ignite, p.fire, p.cut, p.exit];
            for w in seq.windows(2) {
                assert!(w[0] < w[1], "{seq:?}");
            }
            for t in seq {
                let off = (t / BEAT - (t / BEAT).round()).abs();
                assert!(off < 1e-3, "{t} off the beat");
            }
            assert!(p.cut - p.fire >= crate::timeline::BAR - 1e-3);
            assert!(p.fire - p.enter >= 2.0 * crate::timeline::BAR - 1e-3);
            let (r0, r1) = crate::timeline::span(crate::timeline::Section::Repair);
            assert!(p.enter > r0 && p.exit + OUT < r1);
        }
    }
}
