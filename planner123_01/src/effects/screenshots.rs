// ═══════════════════════════════════════════════════════════════
// SCREENSHOTS -- Planner123 app views, blit into pixel buffer.
// Embedded at compile time. Decoded once at startup.
// Displayed during scene 3 (wireframe scene), one per sub-scene.
// ═══════════════════════════════════════════════════════════════

/// The three embedded PNG files (compile-time bytes).
static PLAN_PNG: &[u8] = include_bytes!("screenshots/plan.png");
static GANTT_PNG: &[u8] = include_bytes!("screenshots/gantt.png");
static CALENDAR_PNG: &[u8] = include_bytes!("screenshots/calendar.png");

/// A decoded screenshot: RGBA pixels, original dimensions.
pub struct Screenshot {
    pub pixels: Vec<u32>, // 0x00RRGGBB
    pub width: usize,
    pub height: usize,
}

impl Screenshot {
    fn decode(data: &[u8]) -> Self {
        let decoder = png::Decoder::new(data);
        let mut reader = decoder.read_info().expect("PNG decode failed");
        let mut buf = vec![0u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).expect("PNG frame failed");

        let w = info.width as usize;
        let h = info.height as usize;
        let bytes = &buf[..info.buffer_size()];

        // Convert to 0x00RRGGBB depending on color type
        let pixels: Vec<u32> = match info.color_type {
            png::ColorType::Rgb => bytes
                .chunks_exact(3)
                .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
                .collect(),
            png::ColorType::Rgba => bytes
                .chunks_exact(4)
                .map(|c| ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32))
                .collect(),
            _ => vec![0u32; w * h],
        };

        Self {
            pixels,
            width: w,
            height: h,
        }
    }
}

/// All three screenshots, decoded once at startup.
pub struct Screenshots {
    pub plan: Screenshot,
    pub gantt: Screenshot,
    pub calendar: Screenshot,
}

impl Screenshots {
    pub fn load() -> Self {
        eprintln!("[screenshots] Decoding plan...");
        let plan = Screenshot::decode(PLAN_PNG);
        eprintln!("[screenshots] Decoding gantt...");
        let gantt = Screenshot::decode(GANTT_PNG);
        eprintln!("[screenshots] Decoding calendar...");
        let calendar = Screenshot::decode(CALENDAR_PNG);
        eprintln!("[screenshots] Done.");
        Self {
            plan,
            gantt,
            calendar,
        }
    }

    pub fn get(&self, idx: usize) -> &Screenshot {
        match idx {
            0 => &self.plan,
            1 => &self.gantt,
            _ => &self.calendar,
        }
    }
}

/// Blit a screenshot into the pixel buffer, scaled to fit a target rect,
/// with fade (0.0=transparent, 1.0=opaque), blended over whatever is in buffer.
/// The image is centered in the target rect, preserving aspect ratio (letterbox).
/// Uses bilinear interpolation for pixel-perfect downscaling.
pub fn blit(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    shot: &Screenshot,
    // target rect center and size
    cx: i32,
    cy: i32,
    dst_w: i32,
    dst_h: i32,
    fade: f32,
) {
    if fade <= 0.0 || dst_w <= 0 || dst_h <= 0 {
        return;
    }

    // Fit screenshot inside dst_w × dst_h preserving aspect ratio
    let src_ar = shot.width as f32 / shot.height as f32;
    let dst_ar = dst_w as f32 / dst_h as f32;

    let (fit_w, fit_h) = if src_ar > dst_ar {
        // wider than target: fit width
        (dst_w, (dst_w as f32 / src_ar).round() as i32)
    } else {
        // taller than target: fit height
        ((dst_h as f32 * src_ar).round() as i32, dst_h)
    };

    let x0 = cx - fit_w / 2;
    let y0 = cy - fit_h / 2;

    // Scale factors: how many source pixels per destination pixel
    let scale_x = (shot.width as f32 - 1.0) / (fit_w as f32 - 1.0).max(1.0);
    let scale_y = (shot.height as f32 - 1.0) / (fit_h as f32 - 1.0).max(1.0);

    let sw = shot.width;
    let sh = shot.height;

    for dy in 0..fit_h {
        let dst_y = y0 + dy;
        if dst_y < 0 || dst_y >= buf_h as i32 {
            continue;
        }

        // Bilinear Y coords
        let fy = (dy as f32 * scale_y).min((sh - 1) as f32);
        let y_lo = fy as usize;
        let y_hi = (y_lo + 1).min(sh - 1);
        let yt = fy - y_lo as f32;

        for dx in 0..fit_w {
            let dst_x = x0 + dx;
            if dst_x < 0 || dst_x >= buf_w as i32 {
                continue;
            }

            // Bilinear X coords
            let fx = (dx as f32 * scale_x).min((sw - 1) as f32);
            let x_lo = fx as usize;
            let x_hi = (x_lo + 1).min(sw - 1);
            let xt = fx - x_lo as f32;

            // Sample 4 source pixels
            let p00 = shot.pixels[y_lo * sw + x_lo];
            let p10 = shot.pixels[y_lo * sw + x_hi];
            let p01 = shot.pixels[y_hi * sw + x_lo];
            let p11 = shot.pixels[y_hi * sw + x_hi];

            // Bilinear interpolate each channel
            let src_px = bilerp(p00, p10, p01, p11, xt, yt);

            let dst_idx = dst_y as usize * buf_w + dst_x as usize;

            // Blend: lerp between background and screenshot
            if fade >= 1.0 {
                buffer[dst_idx] = src_px;
            } else {
                let bg = buffer[dst_idx];
                let r = lerp_ch((bg >> 16) & 0xFF, (src_px >> 16) & 0xFF, fade);
                let g = lerp_ch((bg >> 8) & 0xFF, (src_px >> 8) & 0xFF, fade);
                let b = lerp_ch(bg & 0xFF, src_px & 0xFF, fade);
                buffer[dst_idx] = (r << 16) | (g << 8) | b;
            }
        }
    }
}

/// Bilinear interpolation across 4 packed 0x00RRGGBB pixels.
#[inline(always)]
fn bilerp(p00: u32, p10: u32, p01: u32, p11: u32, tx: f32, ty: f32) -> u32 {
    let lerp_row = |a: u32, b: u32, t: f32| -> (f32, f32, f32) {
        let ar = ((a >> 16) & 0xFF) as f32;
        let ag = ((a >> 8) & 0xFF) as f32;
        let ab = (a & 0xFF) as f32;
        let br = ((b >> 16) & 0xFF) as f32;
        let bg = ((b >> 8) & 0xFF) as f32;
        let bb = (b & 0xFF) as f32;
        (ar + (br - ar) * t, ag + (bg - ag) * t, ab + (bb - ab) * t)
    };
    let (r0, g0, b0) = lerp_row(p00, p10, tx);
    let (r1, g1, b1) = lerp_row(p01, p11, tx);
    let r = (r0 + (r1 - r0) * ty) as u32;
    let g = (g0 + (g1 - g0) * ty) as u32;
    let b = (b0 + (b1 - b0) * ty) as u32;
    (r.min(255) << 16) | (g.min(255) << 8) | b.min(255)
}

/// Draw a 1px border around the blit rect.
pub fn draw_border(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    cx: i32,
    cy: i32,
    dst_w: i32,
    dst_h: i32,
    color: u32,
) {
    let x0 = cx - dst_w / 2 - 1;
    let y0 = cy - dst_h / 2 - 1;
    let x1 = cx + dst_w / 2 + 1;
    let y1 = cy + dst_h / 2 + 1;
    hline(buffer, buf_w, buf_h, x0, x1, y0, color);
    hline(buffer, buf_w, buf_h, x0, x1, y1, color);
    vline(buffer, buf_w, buf_h, y0, y1, x0, color);
    vline(buffer, buf_w, buf_h, y0, y1, x1, color);
}

#[inline(always)]
fn lerp_ch(a: u32, b: u32, t: f32) -> u32 {
    (a as f32 + (b as f32 - a as f32) * t) as u32
}

fn hline(buf: &mut [u32], w: usize, h: usize, x0: i32, x1: i32, y: i32, col: u32) {
    if y < 0 || y >= h as i32 {
        return;
    }
    for x in x0.max(0)..=x1.min(w as i32 - 1) {
        buf[y as usize * w + x as usize] = col;
    }
}

fn vline(buf: &mut [u32], w: usize, h: usize, y0: i32, y1: i32, x: i32, col: u32) {
    if x < 0 || x >= w as i32 {
        return;
    }
    for y in y0.max(0)..=y1.min(h as i32 - 1) {
        buf[y as usize * w + x as usize] = col;
    }
}
