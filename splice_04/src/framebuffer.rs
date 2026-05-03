use crate::palette::Color;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height * 3],
        }
    }

    pub fn clear(&mut self, color: Color) {
        for chunk in self.pixels.chunks_exact_mut(3) {
            chunk[0] = color.b;
            chunk[1] = color.g;
            chunk[2] = color.r;
        }
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) * 3;
        self.pixels[idx] = color.b;
        self.pixels[idx + 1] = color.g;
        self.pixels[idx + 2] = color.r;
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color, alpha: f32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return;
        }

        let alpha = alpha.clamp(0.0, 1.0);
        if alpha <= 0.0 {
            return;
        }

        let idx = (y * self.width + x) * 3;
        let blend = |dst: u8, src: u8| -> u8 {
            (dst as f32 + (src as f32 - dst as f32) * alpha)
                .round()
                .clamp(0.0, 255.0) as u8
        };

        self.pixels[idx] = blend(self.pixels[idx], color.b);
        self.pixels[idx + 1] = blend(self.pixels[idx + 1], color.g);
        self.pixels[idx + 2] = blend(self.pixels[idx + 2], color.r);
    }

    pub fn add_pixel(&mut self, x: i32, y: i32, color: Color, amount: f32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return;
        }

        let amount = amount.max(0.0);
        if amount <= 0.0 {
            return;
        }

        let idx = (y * self.width + x) * 3;
        let add = |dst: u8, src: u8| -> u8 {
            (dst as f32 + src as f32 * amount).round().clamp(0.0, 255.0) as u8
        };

        self.pixels[idx] = add(self.pixels[idx], color.b);
        self.pixels[idx + 1] = add(self.pixels[idx + 1], color.g);
        self.pixels[idx + 2] = add(self.pixels[idx + 2], color.r);
    }

    pub fn to_minifb_buffer(&self, out: &mut [u32]) {
        for (idx, chunk) in self.pixels.chunks_exact(3).enumerate() {
            let b = chunk[0] as u32;
            let g = chunk[1] as u32;
            let r = chunk[2] as u32;
            out[idx] = (r << 16) | (g << 8) | b;
        }
    }
}
