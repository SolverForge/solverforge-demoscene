mod font;
mod logo;
mod palette;

use minifb::{Key, Scale, Window, WindowOptions};
use palette::Surface;
use std::io::Write;
use std::time::Instant;

const W: usize = 1280;
const H: usize = 720;
const FPS: f64 = 60.0;

const PHRASES: &[&str] = &[
    "ZERO ERASURE // LIVE DELTAS // RUST IN THE VEINS",
    "SERIO WATCHES THE DELTA. THE DELTA WATCHES BACK.",
    "CONSTRAINTS BEND. LATENCY DIES. THE SCORE REMEMBERS.",
    "SOLVERFORGE // BERGAMO MMXXVI // NO JAVASCRIPT WAS HARMED",
    "HARD FEASIBILITY. SOFT STYLE. SPIFFY OR NOTHING.",
];

const GLYPHS: &[u8] = b"[]{}()<>:=+-*/\\|0123456789SFDS#";

#[derive(Clone, Copy)]
struct Drop {
    x: f32,
    y: f32,
    speed: f32,
    len: usize,
    seed: u32,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--render");
    let render_duration: f64 = args
        .windows(2)
        .find(|w| w[0] == "--render")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(30.0);

    if headless {
        run_headless(render_duration);
        return;
    }

    let mut window = Window::new(
        "SOLVERFORGE // SCREENSAVER_03",
        W,
        H,
        WindowOptions {
            scale: Scale::X1,
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("window");
    window.set_target_fps(60);

    let mut buffer = vec![0u32; W * H];
    let mut state = State::new();
    let start = Instant::now();
    let mut last = start;
    let mut overlay = true;
    let mut space_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32().min(0.05);
        last = now;
        let t = now.duration_since(start).as_secs_f64();

        let space_down = window.is_key_down(Key::Space);
        if space_down && !space_was_down {
            overlay = !overlay;
        }
        space_was_down = space_down;

        state.update(dt, t as f32);
        render_frame(&mut buffer, &state, t, overlay);
        window.update_with_buffer(&buffer, W, H).expect("present");
    }
}

struct State {
    drops: Vec<Drop>,
    phrase_idx: usize,
}

impl State {
    fn new() -> Self {
        let mut drops = Vec::new();
        let step = 16usize;
        for (i, x) in (0..W).step_by(step).enumerate() {
            let fi = i as f32;
            drops.push(Drop {
                x: x as f32 + 4.0,
                y: -((i * 37 % H) as f32),
                speed: 70.0 + (fi * 11.0).sin().abs() * 180.0 + (i % 7) as f32 * 11.0,
                len: 8 + (i * 5 % 18),
                seed: 0x9E37_79B9u32.wrapping_mul(i as u32 + 1),
            });
        }
        Self {
            drops,
            phrase_idx: 0,
        }
    }

    fn update(&mut self, dt: f32, t: f32) {
        self.phrase_idx = ((t / 8.0) as usize) % PHRASES.len();
        for drop in &mut self.drops {
            drop.y += drop.speed * dt;
            if drop.y - (drop.len as f32 * 18.0) > H as f32 + 60.0 {
                drop.y = -((drop.seed % H as u32) as f32) - 80.0;
                drop.seed = lcg(drop.seed ^ (t.to_bits()));
                drop.speed = 80.0 + (drop.seed % 220) as f32;
                drop.len = 8 + (drop.seed as usize % 20);
            }
            drop.seed = lcg(drop.seed);
        }
    }
}

fn run_headless(duration: f64) {
    let total = (duration * FPS) as usize;
    let mut buffer = vec![0u32; W * H];
    let mut bgr = vec![0u8; W * H * 3];
    let mut out = std::io::BufWriter::new(std::io::stdout());
    let mut state = State::new();

    for frame in 0..total {
        let t = frame as f64 / FPS;
        state.update((1.0 / FPS) as f32, t as f32);
        render_frame(&mut buffer, &state, t, true);
        for (i, &px) in buffer.iter().enumerate() {
            bgr[i * 3] = (px & 0xFF) as u8;
            bgr[i * 3 + 1] = ((px >> 8) & 0xFF) as u8;
            bgr[i * 3 + 2] = ((px >> 16) & 0xFF) as u8;
        }
        out.write_all(&bgr).expect("stdout write");
    }
}

fn render_frame(buffer: &mut [u32], state: &State, t: f64, overlay: bool) {
    let mut s = Surface { buf: buffer, w: W, h: H };
    clear_gradient(&mut s, t);
    draw_grid(&mut s, t);
    draw_glyph_rain(&mut s, state, t);
    draw_glow_orb(&mut s, t);
    draw_logo_cluster(&mut s, t);
    draw_panels(&mut s, t);
    if overlay {
        draw_overlay_copy(&mut s, t, state.phrase_idx);
    }
    draw_crt_pass(&mut s, t);
}

fn clear_gradient(s: &mut Surface, t: f64) {
    for y in 0..s.h {
        let fy = y as f32 / s.h as f32;
        let horizon = (1.0 - (fy - 0.58).abs() * 1.7).clamp(0.0, 1.0);
        let pulse = (t as f32 * 0.35).sin() * 0.5 + 0.5;
        let base = palette::lerp_color(palette::rgb(2, 4, 8), palette::MIDNIGHT, fy * 0.8);
        let glow = palette::lerp_color(
            palette::rgb(0, 0, 0),
            palette::dim(palette::EMERALD_800, horizon * (0.22 + 0.08 * pulse)),
            horizon,
        );
        let row = palette::add_color(base, glow);
        let off = y * s.w;
        for x in 0..s.w {
            s.buf[off + x] = row;
        }
    }
}

fn draw_grid(s: &mut Surface, t: f64) {
    let vcol = palette::dim(palette::EMERALD_700, 0.20);
    for x in (0..s.w as i32).step_by(64) {
        palette::bresenham(s, x, 0, x, s.h as i32 - 1, vcol);
    }
    for i in 0..18 {
        let depth = i as f32 / 17.0;
        let y = (s.h as f32 * (0.55 + depth * depth * 0.45)) as i32;
        let spread = (s.w as f32 * (0.12 + depth * 0.38)) as i32;
        let shimmer = ((t * 0.7 + i as f64).sin() * 12.0) as i32;
        palette::bresenham(
            s,
            s.w as i32 / 2 - spread,
            y,
            s.w as i32 / 2 + spread,
            y + shimmer / 6,
            palette::dim(palette::EMERALD_600, 0.10 + depth * 0.18),
        );
    }
}

fn draw_glyph_rain(s: &mut Surface, state: &State, t: f64) {
    for (i, drop) in state.drops.iter().enumerate() {
        for j in 0..drop.len {
            let y = drop.y - j as f32 * 18.0;
            if y < -20.0 || y > s.h as f32 + 20.0 {
                continue;
            }
            let idx = ((drop.seed as usize).wrapping_add(j * 13).wrapping_add((t * 30.0) as usize)) % GLYPHS.len();
            let ch = GLYPHS[idx] as char;
            let head = j == 0;
            let fade = 1.0 - j as f32 / drop.len as f32;
            let color = if head {
                palette::lerp_color(palette::CHROME, palette::EMERALD_300, 0.65)
            } else {
                palette::dim(palette::EMERALD_500, 0.10 + fade * 0.55)
            };
            font::draw_char(s, ch, drop.x as i32, y as i32, 2, color);
            if head && i % 5 == 0 {
                font::draw_char(
                    s,
                    '.',
                    drop.x as i32 + 10,
                    y as i32,
                    2,
                    palette::dim(palette::WHITE, 0.4),
                );
            }
        }
    }
}

fn draw_glow_orb(s: &mut Surface, t: f64) {
    let cx = s.w as i32 / 2;
    let cy = s.h as i32 / 2 - 10;
    for r in [220, 170, 130, 90] {
        let pulse = ((t * 0.8 + r as f64 * 0.03).sin() * 0.5 + 0.5) as f32;
        ring(s, cx, cy, r, palette::dim(palette::EMERALD_600, 0.04 + pulse * 0.06));
    }
}

fn draw_logo_cluster(s: &mut Surface, t: f64) {
    let cx = s.w as f32 / 2.0 + (t * 0.43).sin() as f32 * 18.0;
    let cy = s.h as f32 / 2.0 - 20.0 + (t * 0.61).cos() as f32 * 12.0;
    let pulse = ((t * 1.4).sin() * 0.5 + 0.5) as f32;
    let radius = 112.0 + pulse * 10.0;

    for &(dx, dy, alpha) in &[(-10.0, 0.0, 0.08), (10.0, 0.0, 0.08), (0.0, 8.0, 0.06)] {
        logo::draw_logo(
            s,
            cx + dx,
            cy + dy,
            radius,
            1.0,
            t,
            alpha,
        );
    }
    logo::draw_logo(s, cx, cy, radius, 1.0, t, 1.0);

    font::draw_text_centered_glow(
        s,
        "SOLVERFORGE",
        cx as i32,
        cy as i32 + 124,
        4,
        palette::EMERALD_400,
        palette::dim(palette::EMERALD_500, 0.32 + pulse * 0.15),
    );
    font::draw_text_centered(
        s,
        "zero-erasure optimization engine",
        cx as i32,
        cy as i32 + 168,
        1,
        palette::dim(palette::EMERALD_300, 0.9),
    );
}

fn draw_panels(s: &mut Surface, t: f64) {
    let drift = (t * 0.6).sin() as i32 * 10;
    draw_panel(s, 72, 72 + drift, 320, 122, t, "solver state", &[
        "hard feasibility..... LOCKED",
        "soft score........... RISING",
        "delta evals.......... 000128",
        "temperature.......... COLD",
    ]);
    draw_panel(s, s.w as i32 - 390, 110 - drift, 308, 138, t + 2.1, "watch list", &[
        "planner123",
        "serio",
        "solverforge-core",
        "latency < intuition",
        "amiga forever",
    ]);
}

fn draw_panel(s: &mut Surface, x: i32, y: i32, w: i32, h: i32, t: f64, title: &str, lines: &[&str]) {
    rect_fill(s, x, y, w, h, palette::dim(palette::MIDNIGHT, 0.72));
    rect_outline(s, x, y, w, h, palette::dim(palette::EMERALD_600, 0.55));
    rect_outline(s, x + 2, y + 2, w - 4, h - 4, palette::dim(palette::EMERALD_800, 0.45));
    font::draw_text(s, title, x + 12, y + 10, 1, palette::EMERALD_300);
    let bar_w = (((t.sin() * 0.5 + 0.5) * (w - 24) as f64) as i32).max(20);
    rect_fill(s, x + 12, y + 24, bar_w, 4, palette::dim(palette::EMERALD_500, 0.8));
    for (i, line) in lines.iter().enumerate() {
        let pulse = ((t * 1.2 + i as f64).sin() * 0.5 + 0.5) as f32;
        font::draw_text(
            s,
            line,
            x + 14,
            y + 42 + i as i32 * 16,
            1,
            palette::dim(palette::CHROME, 0.65 + pulse * 0.25),
        );
    }
}

fn draw_overlay_copy(s: &mut Surface, t: f64, phrase_idx: usize) {
    let phrase = PHRASES[phrase_idx];
    font::draw_text_centered(
        s,
        phrase,
        s.w as i32 / 2,
        s.h as i32 - 34,
        1,
        palette::dim(palette::EMERALD_400, 0.85),
    );
    let blink = ((t * 2.0).sin() * 0.5 + 0.5) as f32;
    font::draw_text(
        s,
        "ESC TO EXIT // SPACE TO TOGGLE OVERLAY",
        28,
        s.h as i32 - 28,
        1,
        palette::dim(palette::EMERALD_700, 0.35 + blink * 0.2),
    );
    font::draw_text(
        s,
        "CRT PHOSPHOR // SILENT MODE // SOLVERFORGE AFTER DARK",
        28,
        24,
        1,
        palette::dim(palette::EMERALD_600, 0.45),
    );
}

fn draw_crt_pass(s: &mut Surface, t: f64) {
    for y in 0..s.h {
        let scan = if y % 2 == 0 { 0.96 } else { 0.86 };
        let vignette_y = 1.0 - ((y as f32 / s.h as f32) - 0.5).abs() * 0.5;
        let off = y * s.w;
        for x in 0..s.w {
            let vignette_x = 1.0 - ((x as f32 / s.w as f32) - 0.5).abs() * 0.9;
            let v = (scan * vignette_x * vignette_y).clamp(0.0, 1.0);
            s.buf[off + x] = palette::dim(s.buf[off + x], v);
        }
    }

    let glitch_center = ((t * 0.37).sin() * 0.5 + 0.5) * s.h as f64;
    let gy = glitch_center as i32;
    for row in -2..=2 {
        let y = gy + row;
        if !(0..s.h as i32).contains(&y) {
            continue;
        }
        let shift = ((t * 8.0 + row as f64).sin() * 18.0) as i32;
        let start = y as usize * s.w;
        let mut temp = vec![palette::NEAR_BLACK; s.w];
        for x in 0..s.w as i32 {
            let sx = (x - shift).clamp(0, s.w as i32 - 1) as usize;
            temp[x as usize] = palette::add_color(
                palette::dim(s.buf[start + sx], 0.8),
                palette::dim(palette::EMERALD_500, 0.08),
            );
        }
        s.buf[start..start + s.w].copy_from_slice(&temp);
    }
}

fn rect_fill(s: &mut Surface, x: i32, y: i32, w: i32, h: i32, col: u32) {
    for yy in y.max(0)..(y + h).min(s.h as i32) {
        let off = yy as usize * s.w;
        for xx in x.max(0)..(x + w).min(s.w as i32) {
            s.buf[off + xx as usize] = palette::add_color(s.buf[off + xx as usize], col);
        }
    }
}

fn rect_outline(s: &mut Surface, x: i32, y: i32, w: i32, h: i32, col: u32) {
    palette::bresenham(s, x, y, x + w, y, col);
    palette::bresenham(s, x, y + h, x + w, y + h, col);
    palette::bresenham(s, x, y, x, y + h, col);
    palette::bresenham(s, x + w, y, x + w, y + h, col);
}

fn ring(s: &mut Surface, cx: i32, cy: i32, r: i32, col: u32) {
    let steps = ((r as f32 * 6.0) as i32).max(32);
    let mut prev = None;
    for i in 0..=steps {
        let a = i as f32 / steps as f32 * std::f32::consts::TAU;
        let x = cx + (a.cos() * r as f32) as i32;
        let y = cy + (a.sin() * r as f32 * 0.65) as i32;
        if let Some((px, py)) = prev {
            palette::bresenham(s, px, py, x, y, col);
        }
        prev = Some((x, y));
    }
}

fn lcg(x: u32) -> u32 {
    x.wrapping_mul(1664525).wrapping_add(1013904223)
}
