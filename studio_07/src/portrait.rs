// Vittorio's portrait (his public LinkedIn photo, greyscale, 104x122) as a
// halftone of glowing dots on a little phosphor screen: the brighter the
// photo, the bigger the dot.
use crate::gfx::{hash, Frame};

const PGM: &[u8] = include_bytes!("../assets/portrait.pgm");

pub struct Portrait {
    pub w: usize,
    pub h: usize,
    /// Dot size per pixel, 0..1.
    ink: Vec<f32>,
}

impl Portrait {
    pub fn load() -> Self {
        let (w, h, data) = parse_pgm(PGM);
        let mut ink = vec![0.0; w * h];
        for y in 0..h {
            for x in 0..w {
                let lum = data[y * w + x] as f32 / 255.0;
                // Soften the photo's edges into the screen.
                let edge = x.min(y).min(w - 1 - x).min(h - 1 - y) as f32;
                let keep = (edge / 5.0).min(1.0);
                ink[y * w + x] = lum.powf(1.2) * keep;
            }
        }
        Self { w, h, ink }
    }

    /// Draw with its top-left at (x, y), `pitch` pixels per photo pixel.
    /// `reveal` 0..1 prints the dots top to bottom with a bright write head.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        f: &mut Frame,
        x: f32,
        y: f32,
        pitch: f32,
        rgb: u32,
        reveal: f32,
        gain: f32,
        t: f32,
    ) {
        let head = reveal * (self.h as f32 + 8.0);
        // The screen's bezel.
        let (x1, y1) = (x + self.w as f32 * pitch, y + self.h as f32 * pitch);
        let shown = (reveal * 4.0).min(1.0) * gain;
        f.rect(
            x as i32 - 10,
            y as i32 - 10,
            x1 as i32 + 10,
            y1 as i32 + 10,
            0x000000,
            0.6 * shown,
        );
        f.frame_rect(
            x as i32 - 10,
            y as i32 - 10,
            x1 as i32 + 10,
            y1 as i32 + 10,
            rgb,
            0.55 * shown,
        );
        f.frame_rect(
            x as i32 - 6,
            y as i32 - 6,
            x1 as i32 + 6,
            y1 as i32 + 6,
            rgb,
            0.2 * shown,
        );
        for py in 0..self.h {
            let row = py as f32;
            if row > head {
                break;
            }
            let fresh = (1.0 - (head - row) / 8.0).clamp(0.0, 1.0);
            for px in 0..self.w {
                let ink = self.ink[py * self.w + px];
                if ink < 0.04 {
                    continue;
                }
                // A faint shimmer, as if the dots were a live display.
                let flicker = 0.92
                    + 0.08 * (t * 3.0 + hash(px as i32, py as i32) * std::f32::consts::TAU).sin();
                let r = 0.5 * pitch * ink.sqrt() * 1.05;
                let cx = x + (px as f32 + 0.5) * pitch;
                let cy = y + (py as f32 + 0.5) * pitch;
                let c = crate::gfx::mix(rgb, 0xffffff, fresh * 0.8);
                f.disc(cx, cy, r, c, gain * flicker * (0.55 + 0.45 * ink));
            }
        }
    }
}

fn parse_pgm(bytes: &[u8]) -> (usize, usize, &[u8]) {
    let mut fields = Vec::new();
    let mut i = 0;
    while fields.len() < 4 {
        while bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let start = i;
        while !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        fields.push(std::str::from_utf8(&bytes[start..i]).expect("pgm header"));
    }
    assert_eq!(fields[0], "P5", "portrait must be a binary PGM");
    let w: usize = fields[1].parse().expect("pgm width");
    let h: usize = fields[2].parse().expect("pgm height");
    let data = &bytes[i + 1..i + 1 + w * h];
    (w, h, data)
}
