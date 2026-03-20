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
const GLYPH_SPACING: f32 = 18.0;
const GRID_STEP: usize = 64;
const GLITCH_ROWS: usize = 5;

const PHRASES: &[&str] = &[
    "ZERO ERASURE // LIVE DELTAS // RUST IN THE VEINS",
    "SERIO WATCHES THE DELTA. THE DELTA WATCHES BACK.",
    "CONSTRAINTS BEND. LATENCY DIES. THE SCORE REMEMBERS.",
    "SOLVERFORGE // BERGAMO MMXXVI // NO JAVASCRIPT WAS HARMED",
    "HARD FEASIBILITY. SOFT STYLE. SPIFFY OR NOTHING.",
];

const GLYPHS: &[u8] = b"[]{}()<>:=+-*/\\|0123456789SFDS#";

const SOLVER_STATE_LINES: &[&str] = &[
    "hard feasibility..... LOCKED",
    "soft score........... RISING",
    "delta evals.......... 000128",
    "temperature.......... COLD",
];

const WATCH_LIST_LINES: &[&str] = &[
    "planner123",
    "serio",
    "solverforge-core",
    "latency < intuition",
    "amiga forever",
];

#[derive(Clone, Copy)]
struct Drop {
    x: f32,
    y: f32,
    speed: f32,
    len: usize,
    seed: u32,
}

struct State {
    drops: Vec<Drop>,
    phrase_idx: usize,
    glitch_buffer: Vec<u32>,
}

#[derive(Clone, Copy)]
struct Panel<'a> {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    t: f64,
    title: &'a str,
    lines: &'a [&'a str],
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--render");
    let render_duration = parse_render_duration(&args).unwrap_or(30.0);

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
    .expect("failed to create screensaver window");
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
        render_frame(&mut buffer, &mut state, t, overlay);
        window
            .update_with_buffer(&buffer, W, H)
            .expect("failed to present framebuffer");
    }
}

fn parse_render_duration(args: &[String]) -> Option<f64> {
    args.windows(2)
        .find(|window| window[0] == "--render")
        .and_then(|window| window[1].parse().ok())
}

impl State {
    fn new() -> Self {
        let mut drops = Vec::new();
        for (i, x) in (0..W).step_by(16).enumerate() {
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
            glitch_buffer: vec![palette::NEAR_BLACK; W * GLITCH_ROWS],
        }
    }

    fn update(&mut self, dt: f32, t: f32) {
        self.phrase_idx = ((t / 8.0) as usize) % PHRASES.len();
        for drop in &mut self.drops {
            drop.y += drop.speed * dt;
            if drop.y - (drop.len as f32 * GLYPH_SPACING) > H as f32 + 60.0 {
                respawn_drop(drop, t);
            }
            drop.seed = lcg(drop.seed);
        }
    }
}

fn respawn_drop(drop: &mut Drop, t: f32) {
    drop.y = -((drop.seed % H as u32) as f32) - 80.0;
    drop.seed = lcg(drop.seed ^ t.to_bits());
    drop.speed = 80.0 + (drop.seed % 220) as f32;
    drop.len = 8 + (drop.seed as usize % 20);
}

fn run_headless(duration: f64) {
    let total_frames = (duration * FPS) as usize;
    let mut buffer = vec![0u32; W * H];
    let mut bgr = vec![0u8; W * H * 3];
    let mut out = std::io::BufWriter::new(std::io::stdout());
    let mut state = State::new();

    for frame in 0..total_frames {
        let t = frame as f64 / FPS;
        state.update((1.0 / FPS) as f32, t as f32);
        render_frame(&mut buffer, &mut state, t, true);
        write_bgr_frame(&buffer, &mut bgr);
        out.write_all(&bgr).expect("failed to write raw frame");
    }
}

fn write_bgr_frame(buffer: &[u32], bgr: &mut [u8]) {
    for (i, &px) in buffer.iter().enumerate() {
        bgr[i * 3] = (px & 0xFF) as u8;
        bgr[i * 3 + 1] = ((px >> 8) & 0xFF) as u8;
        bgr[i * 3 + 2] = ((px >> 16) & 0xFF) as u8;
    }
}

fn render_frame(buffer: &mut [u32], state: &mut State, t: f64, overlay: bool) {
    let mut surface = Surface { buf: buffer, w: W, h: H };
    clear_gradient(&mut surface, t);
    draw_grid(&mut surface, t);
    draw_glyph_rain(&mut surface, state, t);
    draw_glow_orb(&mut surface, t);
    draw_logo_cluster(&mut surface, t);
    draw_panels(&mut surface, t);
    if overlay {
        draw_overlay_copy(&mut surface, t, state.phrase_idx);
    }
    draw_crt_pass(&mut surface, &mut state.glitch_buffer, t);
}

fn clear_gradient(surface: &mut Surface, t: f64) {
    for y in 0..surface.h {
        let fy = y as f32 / surface.h as f32;
        let horizon = (1.0 - (fy - 0.58).abs() * 1.7).clamp(0.0, 1.0);
        let pulse = (t as f32 * 0.35).sin() * 0.5 + 0.5;
        let base = palette::lerp_color(palette::rgb(2, 4, 8), palette::MIDNIGHT, fy * 0.8);
        let glow = palette::lerp_color(
            palette::rgb(0, 0, 0),
            palette::dim(palette::EMERALD_800, horizon * (0.22 + 0.08 * pulse)),
            horizon,
        );
        let row = palette::add_color(base, glow);
        let offset = y * surface.w;
        for x in 0..surface.w {
            surface.buf[offset + x] = row;
        }
    }
}

fn draw_grid(surface: &mut Surface, t: f64) {
    let vertical_color = palette::dim(palette::EMERALD_700, 0.20);
    for x in (0..surface.w as i32).step_by(GRID_STEP) {
        palette::bresenham(surface, x, 0, x, surface.h as i32 - 1, vertical_color);
    }

    for i in 0..18 {
        let depth = i as f32 / 17.0;
        let y = (surface.h as f32 * (0.55 + depth * depth * 0.45)) as i32;
        let spread = (surface.w as f32 * (0.12 + depth * 0.38)) as i32;
        let shimmer = ((t * 0.7 + i as f64).sin() * 12.0) as i32;
        palette::bresenham(
            surface,
            surface.w as i32 / 2 - spread,
            y,
            surface.w as i32 / 2 + spread,
            y + shimmer / 6,
            palette::dim(palette::EMERALD_600, 0.10 + depth * 0.18),
        );
    }
}

fn draw_glyph_rain(surface: &mut Surface, state: &State, t: f64) {
    for (i, drop) in state.drops.iter().enumerate() {
        for j in 0..drop.len {
            let y = drop.y - j as f32 * GLYPH_SPACING;
            if y < -20.0 || y > surface.h as f32 + 20.0 {
                continue;
            }

            let idx = ((drop.seed as usize)
                .wrapping_add(j * 13)
                .wrapping_add((t * 30.0) as usize))
                % GLYPHS.len();
            let ch = GLYPHS[idx] as char;
            let fade = 1.0 - j as f32 / drop.len as f32;
            let color = if j == 0 {
                palette::lerp_color(palette::CHROME, palette::EMERALD_300, 0.65)
            } else {
                palette::dim(palette::EMERALD_500, 0.10 + fade * 0.55)
            };

            font::draw_char(surface, ch, drop.x as i32, y as i32, 2, color);
            if j == 0 && i % 5 == 0 {
                font::draw_char(
                    surface,
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

fn draw_glow_orb(surface: &mut Surface, t: f64) {
    let cx = surface.w as i32 / 2;
    let cy = surface.h as i32 / 2 - 10;
    for radius in [220, 170, 130, 90] {
        let pulse = ((t * 0.8 + radius as f64 * 0.03).sin() * 0.5 + 0.5) as f32;
        ring(
            surface,
            cx,
            cy,
            radius,
            palette::dim(palette::EMERALD_600, 0.04 + pulse * 0.06),
        );
    }
}

fn draw_logo_cluster(surface: &mut Surface, t: f64) {
    let cx = surface.w as f32 / 2.0 + (t * 0.43).sin() as f32 * 18.0;
    let cy = surface.h as f32 / 2.0 - 20.0 + (t * 0.61).cos() as f32 * 12.0;
    let pulse = ((t * 1.4).sin() * 0.5 + 0.5) as f32;
    let radius = 112.0 + pulse * 10.0;

    for &(dx, dy, brightness) in &[(-10.0, 0.0, 0.08), (10.0, 0.0, 0.08), (0.0, 8.0, 0.06)] {
        logo::draw_logo(surface, cx + dx, cy + dy, radius, 1.0, t, brightness);
    }
    logo::draw_logo(surface, cx, cy, radius, 1.0, t, 1.0);

    font::draw_text_centered_glow(
        surface,
        "SOLVERFORGE",
        cx as i32,
        cy as i32 + 124,
        4,
        palette::EMERALD_400,
        palette::dim(palette::EMERALD_500, 0.32 + pulse * 0.15),
    );
    font::draw_text_centered(
        surface,
        "zero-erasure optimization engine",
        cx as i32,
        cy as i32 + 168,
        1,
        palette::dim(palette::EMERALD_300, 0.9),
    );
}

fn draw_panels(surface: &mut Surface, t: f64) {
    let drift = (t * 0.6).sin() as i32 * 10;
    let panels = [
        Panel {
            x: 72,
            y: 72 + drift,
            w: 320,
            h: 122,
            t,
            title: "solver state",
            lines: SOLVER_STATE_LINES,
        },
        Panel {
            x: surface.w as i32 - 390,
            y: 110 - drift,
            w: 308,
            h: 138,
            t: t + 2.1,
            title: "watch list",
            lines: WATCH_LIST_LINES,
        },
    ];

    for panel in panels {
        draw_panel(surface, panel);
    }
}

fn draw_panel(surface: &mut Surface, panel: Panel<'_>) {
    rect_fill(
        surface,
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        palette::dim(palette::MIDNIGHT, 0.72),
    );
    rect_outline(
        surface,
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        palette::dim(palette::EMERALD_600, 0.55),
    );
    rect_outline(
        surface,
        panel.x + 2,
        panel.y + 2,
        panel.w - 4,
        panel.h - 4,
        palette::dim(palette::EMERALD_800, 0.45),
    );
    font::draw_text(
        surface,
        panel.title,
        panel.x + 12,
        panel.y + 10,
        1,
        palette::EMERALD_300,
    );

    let bar_width = (((panel.t.sin() * 0.5 + 0.5) * (panel.w - 24) as f64) as i32).max(20);
    rect_fill(
        surface,
        panel.x + 12,
        panel.y + 24,
        bar_width,
        4,
        palette::dim(palette::EMERALD_500, 0.8),
    );

    for (i, line) in panel.lines.iter().enumerate() {
        let pulse = ((panel.t * 1.2 + i as f64).sin() * 0.5 + 0.5) as f32;
        font::draw_text(
            surface,
            line,
            panel.x + 14,
            panel.y + 42 + i as i32 * 16,
            1,
            palette::dim(palette::CHROME, 0.65 + pulse * 0.25),
        );
    }
}

fn draw_overlay_copy(surface: &mut Surface, t: f64, phrase_idx: usize) {
    font::draw_text_centered(
        surface,
        PHRASES[phrase_idx],
        surface.w as i32 / 2,
        surface.h as i32 - 34,
        1,
        palette::dim(palette::EMERALD_400, 0.85),
    );

    let blink = ((t * 2.0).sin() * 0.5 + 0.5) as f32;
    font::draw_text(
        surface,
        "ESC TO EXIT // SPACE TO TOGGLE OVERLAY",
        28,
        surface.h as i32 - 28,
        1,
        palette::dim(palette::EMERALD_700, 0.35 + blink * 0.2),
    );
    font::draw_text(
        surface,
        "CRT PHOSPHOR // SILENT MODE // SOLVERFORGE AFTER DARK",
        28,
        24,
        1,
        palette::dim(palette::EMERALD_600, 0.45),
    );
}

fn draw_crt_pass(surface: &mut Surface, glitch_buffer: &mut [u32], t: f64) {
    for y in 0..surface.h {
        let scan = if y % 2 == 0 { 0.96 } else { 0.86 };
        let vignette_y = 1.0 - ((y as f32 / surface.h as f32) - 0.5).abs() * 0.5;
        let offset = y * surface.w;
        for x in 0..surface.w {
            let vignette_x = 1.0 - ((x as f32 / surface.w as f32) - 0.5).abs() * 0.9;
            let brightness = (scan * vignette_x * vignette_y).clamp(0.0, 1.0);
            surface.buf[offset + x] = palette::dim(surface.buf[offset + x], brightness);
        }
    }

    let glitch_center = ((t * 0.37).sin() * 0.5 + 0.5) * surface.h as f64;
    let start_y = glitch_center as i32 - (GLITCH_ROWS as i32 / 2);

    for row in 0..GLITCH_ROWS {
        let y = start_y + row as i32;
        if !(0..surface.h as i32).contains(&y) {
            continue;
        }

        let shift = ((t * 8.0 + row as f64).sin() * 18.0) as i32;
        let src_start = y as usize * surface.w;
        let row_slice = &mut glitch_buffer[row * surface.w..(row + 1) * surface.w];
        for x in 0..surface.w as i32 {
            let src_x = (x - shift).clamp(0, surface.w as i32 - 1) as usize;
            row_slice[x as usize] = palette::add_color(
                palette::dim(surface.buf[src_start + src_x], 0.8),
                palette::dim(palette::EMERALD_500, 0.08),
            );
        }
        surface.buf[src_start..src_start + surface.w].copy_from_slice(row_slice);
    }
}

fn rect_fill(surface: &mut Surface, x: i32, y: i32, w: i32, h: i32, color: u32) {
    for yy in y.max(0)..(y + h).min(surface.h as i32) {
        let offset = yy as usize * surface.w;
        for xx in x.max(0)..(x + w).min(surface.w as i32) {
            let idx = offset + xx as usize;
            surface.buf[idx] = palette::add_color(surface.buf[idx], color);
        }
    }
}

fn rect_outline(surface: &mut Surface, x: i32, y: i32, w: i32, h: i32, color: u32) {
    palette::bresenham(surface, x, y, x + w, y, color);
    palette::bresenham(surface, x, y + h, x + w, y + h, color);
    palette::bresenham(surface, x, y, x, y + h, color);
    palette::bresenham(surface, x + w, y, x + w, y + h, color);
}

fn ring(surface: &mut Surface, cx: i32, cy: i32, radius: i32, color: u32) {
    let steps = ((radius as f32 * 6.0) as i32).max(32);
    let mut prev = None;
    for i in 0..=steps {
        let angle = i as f32 / steps as f32 * std::f32::consts::TAU;
        let x = cx + (angle.cos() * radius as f32) as i32;
        let y = cy + (angle.sin() * radius as f32 * 0.65) as i32;
        if let Some((px, py)) = prev {
            palette::bresenham(surface, px, py, x, y, color);
        }
        prev = Some((x, y));
    }
}

fn lcg(x: u32) -> u32 {
    x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223)
}
