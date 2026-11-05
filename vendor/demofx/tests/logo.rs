use demofx::{logo, Pixels};

struct Frame(Vec<u32>);

impl Pixels for Frame {
    fn size(&self) -> (usize, usize) {
        (321, 239)
    }
    fn get(&self, x: usize, y: usize) -> u32 {
        self.0[y * 321 + x]
    }
    fn put(&mut self, x: usize, y: usize, rgb: u32) {
        self.0[y * 321 + x] = rgb;
    }
}

#[test]
fn logo_coverage_matches_full_frame_rasterizer() {
    // Recorded from the full-frame rasterizer: subpixel centers, clipping,
    // small and large scales, partial reveals, translucent layers and no gain.
    let mut hash = 0xcbf29ce484222325u64;
    for scale in [0.08, 0.25, 0.576, 1.0, 1.63, 3.2] {
        for (cx, cy) in [
            (160.0, 119.0),
            (160.37, 119.81),
            (-5.25, 10.5),
            (319.9, 237.9),
        ] {
            for (progress, detail, gain) in [
                (0.0, 0.0, 1.0),
                (0.4, 0.0, 0.85),
                (0.9, 0.333, 0.08),
                (0.97, 0.8, 0.6),
                (1.0, 1.0, 1.0),
                (1.0, 1.0, 0.0),
            ] {
                let mut frame = Frame(
                    (0..321 * 239)
                        .map(|i| (i as u32).wrapping_mul(2654435761) & 0xffffff)
                        .collect(),
                );
                logo::draw(
                    &mut frame,
                    &logo::LogoState {
                        cx,
                        cy,
                        scale,
                        progress,
                        detail,
                        gain,
                    },
                );
                for pixel in frame.0 {
                    hash = (hash ^ pixel as u64).wrapping_mul(0x100000001b3);
                }
            }
        }
    }
    assert_eq!(hash, 0xb5ae9dca14a8f721);
}
