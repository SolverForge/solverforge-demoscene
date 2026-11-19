// The statue from Hermes Desktop's backdrop (apps/desktop/public/ds-assets/
// filler-bg0.jpg in the Hermes repository, MIT, Nous Research), inverted to
// light lines on dark and stored as a 960x540 greyscale PGM. Drawn behind the
// intro, where it fades in, and behind the live take: barely there and
// drifting slowly, the way Desktop keeps it behind the conversation.
use crate::gfx::{pack_px, par_rows, parts, Frame, H, W};

const PGM: &[u8] = include_bytes!("../assets/hermes_statue.pgm");

pub struct Statue {
    w: usize,
    h: usize,
    px: &'static [u8],
}

impl Statue {
    /// Parse the binary PGM header: "P5", width, height, maxval, one byte of
    /// whitespace, then the pixels.
    pub fn load() -> Self {
        let mut fields = Vec::new();
        let mut i = 0;
        while fields.len() < 4 {
            while PGM[i].is_ascii_whitespace() {
                i += 1;
            }
            let start = i;
            while !PGM[i].is_ascii_whitespace() {
                i += 1;
            }
            fields.push(std::str::from_utf8(&PGM[start..i]).expect("PGM header"));
        }
        assert_eq!(fields[0], "P5", "the statue must be a binary PGM");
        let (w, h): (usize, usize) = (fields[1].parse().unwrap(), fields[2].parse().unwrap());
        let px = &PGM[i + 1..i + 1 + w * h];
        Self { w, h, px }
    }

    /// Add the statue as `rgb` light at strength `k`, drifted by `t` seconds.
    pub fn draw(&self, f: &mut Frame, rgb: u32, k: f32, t: f32) {
        if k <= 0.0 {
            return;
        }
        // Slightly larger than the frame so the slow drift never shows an edge.
        let zoom = 1.06;
        let (dx, dy) = ((t * 0.11).sin() * 0.02, (t * 0.07).cos() * 0.015);
        let (tr, tg, tb) = parts(rgb);
        // The source column of every frame column is the same on every row.
        let cols: Vec<usize> = (0..W)
            .map(|x| {
                let u = ((x as f32 / W as f32 - 0.5) / zoom + 0.5 + dx) * self.w as f32;
                u.clamp(0.0, (self.w - 1) as f32) as usize
            })
            .collect();
        par_rows(&mut f.px, |y, out| {
            let v = ((y as f32 / H as f32 - 0.5) / zoom + 0.5 + dy) * self.h as f32;
            let row = &self.px[(v.clamp(0.0, (self.h - 1) as f32)) as usize * self.w..];
            for (p, &col) in out.iter_mut().zip(&cols) {
                let s = row[col] as f32 / 255.0;
                if s <= 0.02 {
                    continue;
                }
                let (r, g, b) = parts(*p);
                let a = s * k;
                *p = pack_px(r + tr * a, g + tg * a, b + tb * a);
            }
        });
    }
}
