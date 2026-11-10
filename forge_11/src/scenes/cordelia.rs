// CORDELIA: the agent's hero beat inside 04 REPAIR. The bus breaks down,
// the band drops to a heartbeat under the alarm, and Cordelia glitches in
// over the score panel, defiant. Her eyes charge and ignite on the beats
// before the bar line; on the downbeat where the repair enters the running
// solve she fires, and her eye-lasers sear across to the board: one beam
// holds the breakdown's own job, the other rides the changed job to its new
// slot. On the next bar the hook returns and she cuts out.
//
// The artwork is three generated keyframes (charge, ignite, fire) built on
// the brand direction's saved Cordelia prompts; prompts, model and hashes
// are in assets/cordelia/provenance.json. The beams here are an effect
// layer over the art, anchored to the iris centres measured on each frame,
// and carried through the same transform as the picture so they stay
// locked to her eyes while she breathes, pushes in and jitters.
use super::{EMERALD, MINT, PAPER};
use crate::fx;
use crate::gfx::{mix, smooth, Frame, S1, W};
use crate::script::Script;
use crate::timeline::{snap_beat, BAR, BEAT};

const CHARGE: &[u8] = include_bytes!("../../assets/cordelia/charge.ppm");
const IGNITE: &[u8] = include_bytes!("../../assets/cordelia/ignite.ppm");
const FIRE: &[u8] = include_bytes!("../../assets/cordelia/fire.ppm");

/// Iris centres [left, right] in each 680x562 keyframe (provenance.json).
const EYES: [[(f32, f32); 2]; 3] = [
    [(437.0, 343.0), (510.0, 371.0)],
    [(437.0, 343.0), (510.0, 371.0)],
    [(446.0, 348.0), (510.0, 371.0)],
];

/// The panel she occupies: over the score panel's chart and list, the
/// event line, the score and the board stay in view.
pub const PANEL: (i32, i32, i32, i32) = (40, 452, 720, 970);

/// When the beat plays, on the demo clock. Picture and sound share it.
#[derive(Clone, Copy, Debug)]
pub struct Beat {
    /// Glitch-in, eyes charging.
    pub enter: f32,
    /// Eyes ignite, a beat before the fire.
    pub ignite: f32,
    /// The beams fire: the downbeat the repair enters the running solve.
    pub fire: f32,
    /// The beams cut and she glitches out, as the hook returns.
    pub exit: f32,
}

pub const OUT: f32 = 0.4;

impl Beat {
    pub fn new(s: &Script) -> Self {
        let fire = snap_beat(s.repair.from.max(s.event + 2.0 * BEAT));
        Self {
            enter: snap_beat(s.event + BEAT),
            ignite: fire - BEAT,
            fire,
            exit: fire + BAR,
        }
    }
}

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
    frames: [Image; 3],
    pub beat: Beat,
}

/// Where the beams strike, in frame pixels.
pub struct Targets {
    pub left: Option<(f32, f32)>,
    pub right: Option<(f32, f32)>,
}

impl Cordelia {
    pub fn new(s: &Script) -> Self {
        Self {
            frames: [ppm(CHARGE), ppm(IGNITE), ppm(FIRE)],
            beat: Beat::new(s),
        }
    }

    pub fn active(&self, t: f32) -> bool {
        (self.beat.enter..self.beat.exit + OUT).contains(&t)
    }

    /// The picture transform at t: scale and the source point shown at the
    /// panel centre. Push-in over the beat, breathing, a kick on the fire.
    fn transform(&self, t: f32) -> (f32, f32, f32) {
        let b = self.beat;
        let p = smooth((t - b.enter) / (b.exit - b.enter));
        let kick = if t >= b.fire {
            0.035 * (-(t - b.fire) * 7.0).exp()
        } else {
            0.0
        };
        let scale = 1.0 + 0.10 * p + kick;
        let breathe = (t * std::f32::consts::TAU / (2.0 * BEAT)).sin();
        // Drift towards her eyes as she pushes in (parallax on the face).
        let cx = 340.0 + 60.0 * p + 2.0 * breathe;
        let cy = 281.0 + 40.0 * p - 3.0 * breathe;
        (scale, cx, cy)
    }

    /// Source pixel -> frame pixel through the current transform.
    fn to_frame(&self, t: f32, (sx, sy): (f32, f32)) -> (f32, f32) {
        let (scale, cx, cy) = self.transform(t);
        let (x0, y0, x1, y1) = PANEL;
        let (pcx, pcy) = ((x0 + x1) as f32 * 0.5, (y0 + y1) as f32 * 0.5);
        (pcx + (sx - cx) * scale, pcy + (sy - cy) * scale)
    }

    /// The eyes on screen at t (the fire frame's anchors once firing).
    pub fn eyes(&self, t: f32) -> [(f32, f32); 2] {
        let k = self.key(t);
        let e = EYES[k];
        [self.to_frame(t, e[0]), self.to_frame(t, e[1])]
    }

    fn key(&self, t: f32) -> usize {
        if t >= self.beat.fire {
            2
        } else if t >= self.beat.ignite {
            1
        } else {
            0
        }
    }

    pub fn draw(&self, f: &mut Frame, t: f32, targets: &Targets) {
        if !self.active(t) {
            return;
        }
        let b = self.beat;
        let since = t - b.enter;
        let gone = t - b.exit;
        // Entrance: rows tear in over 0.3 s; exit: they tear out.
        let tear = if since < 0.3 {
            1.0 - since / 0.3
        } else if gone > 0.0 {
            gone / OUT
        } else {
            0.0
        };
        let alpha = smooth(since / 0.12) * (1.0 - smooth(gone / OUT));
        let (x0, y0, x1, y1) = PANEL;
        let (scale, cx, cy) = self.transform(t);
        let (pcx, pcy) = ((x0 + x1) as f32 * 0.5, (y0 + y1) as f32 * 0.5);
        let key = self.key(t);
        // Crossfade a few frames into each new key.
        let since_key = t - [b.enter, b.ignite, b.fire][key];
        let blend = if key > 0 {
            smooth(since_key / 0.08)
        } else {
            1.0
        };
        let (img, prev) = (&self.frames[key], &self.frames[key.saturating_sub(1)]);
        let jitter_t = (t * 60.0) as i32;
        let inv = 1.0 / scale;
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
                    // Chromatic split while she tears in.
                    let r = img.sample(sx + 8.0 * tear, sy) & 0xff0000;
                    c = (c & 0x00ffff) | r;
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

    /// The charge, then the ignition: light pooling in the irises.
    fn eyes_glow(&self, f: &mut Frame, t: f32, alpha: f32) {
        let b = self.beat;
        let charge = smooth((t - b.enter) / (b.fire - b.enter));
        let flick = 0.85 + 0.15 * (t * 53.0).sin();
        for (i, (ex, ey)) in self.eyes(t).into_iter().enumerate() {
            let r = 10.0 + 26.0 * charge + if t >= b.fire { 14.0 } else { 0.0 };
            let k = alpha * (0.25 + 0.75 * charge) * flick;
            f.glow(ex, ey, r * 2.2, EMERALD, 0.35 * k);
            f.glow(ex, ey, r, MINT, 0.6 * k);
            f.glow(ex, ey, r * 0.35, 0xffffff, k);
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
        if !(0.0..=b.exit - b.fire + 0.05).contains(&age) {
            return;
        }
        // The beams reach the board over a few frames, then hold.
        let reach = smooth(age / 0.1);
        let fade = alpha * (1.0 - smooth((t - b.exit + 0.1) / 0.15));
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
            let w = 1.0 + 0.6 * (-age * 8.0).exp();
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
        for data in [CHARGE, IGNITE, FIRE] {
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
