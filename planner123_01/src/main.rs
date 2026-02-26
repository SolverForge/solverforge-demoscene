// ═══════════════════════════════════════════════════════════════
//  ██████╗ ██╗      █████╗ ███╗   ██╗███╗   ██╗███████╗██████╗
//  ██╔══██╗██║     ██╔══██╗████╗  ██║████╗  ██║██╔════╝██╔══██╗
//  ██████╔╝██║     ███████║██╔██╗ ██║██╔██╗ ██║█████╗  ██████╔╝
//  ██╔═══╝ ██║     ██╔══██║██║╚██╗██║██║╚██╗██║██╔══╝  ██╔══██╗
//  ██║     ███████╗██║  ██║██║ ╚████║██║ ╚████║███████╗██║  ██║
//  ╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝
//           1 2 3  ── GREETINGS FROM THE RUST DEMOSCENE ──
//
//  POWERED BY: SOLVERFORGE CONSTRAINT ENGINE (RUST)
//  CODED IN:   PURE RUST. NO ASSETS. NO COMPROMISES.
//
//  AMIGA 1992 FOREVER.
//
//  PRESS ESC TO EXIT. SPACE TO ADVANCE. OR DON'T.
// ═══════════════════════════════════════════════════════════════

mod audio;
mod effects;
mod font;
mod logo;
mod palette;

use effects::copper::{CopperBars, CopperMode};
use effects::logo_reveal::LogoReveal;
use effects::plasma::Plasma;
use effects::screenshots::Screenshots;
use effects::scroll::ScrollText;
use effects::starfield::Starfield;
use effects::wireframe::Wireframe;

use minifb::{Key, Scale, Window, WindowOptions};
use std::io::Write;
use std::time::Instant;

// ── Audio ──────────────────────────────────────────────────────
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// ── Resolution ────────────────────────────────────────────────
// Internal render resolution. minifb will scale to window size.
const RENDER_W: usize = 1280;
const RENDER_H: usize = 720;

// ── Demo timeline ─────────────────────────────────────────────
const SCENE_LOGO: f64 = 0.0; // 0s - 10s: Logo reveal
const SCENE_PLASMA: f64 = 10.0; // 10s - 20s: Plasma + copper bars
const SCENE_STARFIELD: f64 = 20.0; // 20s - 32s: Starfield warp
const SCENE_WIREFRAME: f64 = 32.0; // 32s - 63s: Screenshots (31s: shot7/scroll5/shot7/scroll5/shot7)
const SCENE_SCROLL: f64 = 63.0; // 63s - 73s: Greetings scroll
const SCENE_OUTRO: f64 = 73.0; // 73s - 84s: Composite outro
const DEMO_END: f64 = 84.0;

fn main() {
    // ── Headless render mode: --render [duration_secs] ────────
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--render");
    let render_duration: f64 = args
        .windows(2)
        .find(|w| w[0] == "--render")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(DEMO_END);

    if headless {
        eprintln!(
            "[render] Headless mode: {}x{} @ 60fps, {:.0}s → stdout (BGR24)",
            RENDER_W, RENDER_H, render_duration
        );
        eprintln!("[render] Pipe to ffmpeg, e.g.:");
        eprintln!("[render]   cargo run -r -p planner123-demo -- --render | ffmpeg -f rawvideo -pixel_format bgr24 -video_size {}x{} -framerate 60 -i - -c:v libx264 -pix_fmt yuv420p demo.mp4", RENDER_W, RENDER_H);
        run_headless(render_duration);
        return;
    }

    eprintln!();
    eprintln!("╔═══════════════════════════════════════════╗");
    eprintln!("║   SOLVERFORGE DEMOSCENE -- PLANNER123     ║");
    eprintln!("║   AMIGA FOREVER. CODED IN RUST. 2026.    ║");
    eprintln!("╚═══════════════════════════════════════════╝");
    eprintln!();
    eprintln!("  Controls: ESC = exit, SPACE = advance to next scene");
    eprintln!("  Resolution: {}x{}", RENDER_W, RENDER_H);
    eprintln!();

    // ── Window setup ──────────────────────────────────────────
    let window_opts = WindowOptions {
        scale: Scale::X1,
        resize: true,
        ..WindowOptions::default()
    };

    let mut window = Window::new(
        "PLANNER123 - SOLVERFORGE DEMOSCENE INTRO",
        RENDER_W,
        RENDER_H,
        window_opts,
    )
    .expect("Failed to create window");

    window.set_target_fps(60);

    // ── Pixel buffer ──────────────────────────────────────────
    let mut buffer = vec![0u32; RENDER_W * RENDER_H];

    // ── Audio setup ───────────────────────────────────────────
    let audio_stream = start_audio();

    // ── Effect instances ──────────────────────────────────────
    let plasma = Plasma::new();
    let mut starfield = Starfield::new();
    let mut wireframe = Wireframe::new(RENDER_W, RENDER_H);
    let copper = CopperBars::new();
    let mut scroller = ScrollText::new(3, 400.0, RENDER_W);
    // Scene 3 interlude scrolltext — smaller, used between screenshots
    let mut interlude_scroller = ScrollText::new(2, 180.0, RENDER_W);
    let mut logo_reveal = LogoReveal::new();
    let screenshots = Screenshots::load();

    // ── Demo state ────────────────────────────────────────────
    let start_time = Instant::now();
    let mut last_time = start_time;
    let mut time_offset: f64 = 0.0; // accumulated skip offset from SPACE presses
    let mut space_was_down = false; // edge detection for SPACE key

    // Scene start times for SPACE-to-advance
    let scene_starts: &[f64] = &[
        SCENE_LOGO,
        SCENE_PLASMA,
        SCENE_STARFIELD,
        SCENE_WIREFRAME,
        SCENE_SCROLL,
        SCENE_OUTRO,
        DEMO_END,
    ];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f64();
        last_time = now;

        let elapsed = now.duration_since(start_time).as_secs_f64() + time_offset;
        let demo_time = elapsed % (DEMO_END + 5.0); // loop

        // ── SPACE to advance to next scene ─────────────────────
        let space_down = window.is_key_down(Key::Space);
        if space_down && !space_was_down {
            // Find the next scene boundary after current time
            for &boundary in scene_starts {
                if boundary > demo_time + 0.5 {
                    // Jump forward
                    time_offset += boundary - demo_time;
                    break;
                }
            }
        }
        space_was_down = space_down;

        // ── Clear buffer to deep black ─────────────────────────
        let bg = palette::NEAR_BLACK;
        for px in buffer.iter_mut() {
            *px = bg;
        }

        // ── Scene selection + crossfade ────────────────────────
        let (scene, scene_time, fade_in, fade_out) = get_scene(demo_time);

        let fade = fade_in * fade_out;

        // ── Render current scene ───────────────────────────────
        match scene {
            0 => {
                // Scene 0: Logo Reveal
                logo_reveal.update(dt, RENDER_W, RENDER_H);
                logo_reveal.render(&mut buffer, RENDER_W, RENDER_H, scene_time, fade);
            }

            1 => {
                // Scene 1: Plasma + Copper Bars
                plasma.render(&mut buffer, RENDER_W, RENDER_H, scene_time, fade, false);
                copper.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    fade * 0.7,
                    CopperMode::Classic,
                );
                copper.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    fade * 0.3,
                    CopperMode::Scanlines,
                );

                // Title text
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "PLANNER123",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 - 80,
                    5,
                    palette::dim(palette::WHITE, fade * 0.9),
                    palette::dim(palette::EMERALD_400, fade * 0.5),
                );
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "CONSTRAINT SCHEDULING",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 20,
                    2,
                    palette::dim(palette::EMERALD_400, fade * 0.7),
                    palette::dim(palette::EMERALD_600, fade * 0.4),
                );
            }

            2 => {
                // Scene 2: Starfield Warp
                // Fade in warp factor mid-scene
                let warp = ((scene_time - 6.0) / 6.0).clamp(0.0, 1.0) as f32;
                starfield.update(dt, warp);
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade, scene_time);

                // Scrolltext at bottom
                scroller.update(dt);
                scroller.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    RENDER_H as i32 - 80,
                    fade,
                );

                // "WARP SPEED" text fades in with warp
                if warp > 0.5 {
                    let wt = ((warp - 0.5) * 2.0) * fade;
                    font::draw_text_centered_glow(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        "WARP SPEED",
                        RENDER_W as i32 / 2,
                        RENDER_H as i32 / 2 - 30,
                        4,
                        palette::dim(palette::WHITE, wt * 0.8),
                        palette::dim(palette::CYAN_400, wt * 0.6),
                    );
                }
            }

            3 => {
                // Scene 3: Screenshot showcase — Plan, Gantt, Calendar
                // 31s split into variable-length phases:
                //   Phase 0 (0–7s):   Screenshot — Plan
                //   Phase 1 (7–12s):  Scrolltext interlude
                //   Phase 2 (12–19s): Screenshot — Gantt
                //   Phase 3 (19–24s): Scrolltext interlude
                //   Phase 4 (24–31s): Screenshot — Calendar
                const PHASE_STARTS: [f64; 6] = [0.0, 7.0, 12.0, 19.0, 24.0, 31.0];
                const XFADE: f64 = 0.8;

                // Find which phase we're in
                let phase = PHASE_STARTS
                    .windows(2)
                    .position(|w| scene_time >= w[0] && scene_time < w[1])
                    .unwrap_or(4);
                let phase_start = PHASE_STARTS[phase];
                let phase_dur = PHASE_STARTS[phase + 1] - phase_start;
                let phase_t = scene_time - phase_start;

                let phase_fade_in = ((phase_t / XFADE) as f32).clamp(0.0, 1.0);
                let phase_fade_out = (((phase_dur - phase_t) / XFADE) as f32).clamp(0.0, 1.0);
                let phase_fade = phase_fade_in * phase_fade_out * fade;

                // Always advance interlude scroller so it never restarts
                interlude_scroller.update(dt);

                // Wireframe spinning behind — dimmed so screenshots pop
                wireframe.update(scene_time);
                wireframe.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    fade * 0.2,
                    scene_time,
                    false,
                );

                if phase % 2 == 0 {
                    // ── Even phases: screenshot ─────────────────────────
                    let shot_idx = phase / 2; // 0=Plan, 1=Gantt, 2=Calendar

                    // Full-screen screenshot with proper 16:9 aspect ratio
                    // Fill as much of the 1280×720 canvas as possible
                    let dst_w = RENDER_W as i32;
                    let dst_h = RENDER_H as i32;
                    let shot_cx = RENDER_W as i32 / 2;
                    let shot_cy = RENDER_H as i32 / 2;

                    let shot = screenshots.get(shot_idx);

                    // Dark vignette behind the screenshot
                    {
                        let x0 = 0usize;
                        let x1 = RENDER_W;
                        let y0 = 0usize;
                        let y1 = RENDER_H;
                        for y in y0..y1 {
                            for x in x0..x1 {
                                let idx = y * RENDER_W + x;
                                buffer[idx] = palette::dim(buffer[idx], 1.0 - phase_fade * 0.85);
                            }
                        }
                    }

                    // Blit screenshot (bilinear, fits inside canvas preserving 16:9)
                    effects::screenshots::blit(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        shot,
                        shot_cx,
                        shot_cy,
                        dst_w,
                        dst_h,
                        phase_fade,
                    );

                    // Thin border around screenshot
                    effects::screenshots::draw_border(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        shot_cx,
                        shot_cy,
                        dst_w,
                        dst_h,
                        palette::dim(palette::EMERALD_600, phase_fade * 0.6),
                    );

                    // Text overlay: left-aligned to left edge of screen + margin
                    let left_x = 32i32;
                    let numbers = ["01 -", "02 -", "03 -"];
                    let headings = ["THE PLAN.", "THE GANTT.", "THE CALENDAR."];
                    let captions = [
                        "MULTI-PROJECT MANAGEMENT. DEPENDENCIES. PRIORITIES.",
                        "CONSTRAINT-OPTIMIZED. DEADLINES RESPECTED. MATH.",
                        "YOUR WEEK, SOLVED. TASKS FIT AROUND YOUR EVENTS.",
                    ];

                    // Slide in from top with phase_t
                    let slide = ((phase_t * 3.0) as f32).clamp(0.0, 1.0);
                    let text_fade = phase_fade * slide;

                    // Dark backing strip behind the text labels at top
                    {
                        let strip_h = 68usize;
                        for y in 0..strip_h {
                            for x in 0..RENDER_W {
                                let idx = y * RENDER_W + x;
                                buffer[idx] = palette::dim(buffer[idx], 1.0 - text_fade * 0.7);
                            }
                        }
                    }

                    // Number line: small, muted
                    font::draw_text(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        numbers[shot_idx],
                        left_x,
                        10,
                        1,
                        palette::dim(palette::EMERALD_700, text_fade * 0.9),
                    );

                    // Heading: large, white glow
                    font::draw_text_glow(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        headings[shot_idx],
                        left_x,
                        22,
                        3,
                        palette::dim(palette::WHITE, text_fade),
                        palette::dim(palette::EMERALD_500, text_fade * 0.5),
                    );

                    // Caption at bottom
                    font::draw_text_glow(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        captions[shot_idx],
                        left_x,
                        RENDER_H as i32 - 26,
                        1,
                        palette::dim(palette::EMERALD_400, text_fade * 0.9),
                        palette::dim(palette::EMERALD_700, text_fade * 0.3),
                    );

                    // Dot indicators (bottom right)
                    {
                        let dot_y = RENDER_H as i32 - 20;
                        let dot_gap = 14i32;
                        let right_x = RENDER_W as i32 - 32;
                        for i in 0..3usize {
                            let dx = right_x - (2 - i as i32) * dot_gap;
                            let col = if i == shot_idx {
                                palette::dim(palette::EMERALD_400, phase_fade)
                            } else {
                                palette::dim(palette::EMERALD_800, phase_fade * 0.4)
                            };
                            for dy in -2i32..=2 {
                                for ddx in -2i32..=2 {
                                    if ddx * ddx + dy * dy <= 4 {
                                        let px = dx + ddx;
                                        let py = dot_y + dy;
                                        if px >= 0
                                            && py >= 0
                                            && px < RENDER_W as i32
                                            && py < RENDER_H as i32
                                        {
                                            buffer[py as usize * RENDER_W + px as usize] = col;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // ── Odd phases: scrolltext interlude ────────────────
                    // Plasma backdrop (subdued)
                    plasma.render_overlay(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        scene_time,
                        0.25 * phase_fade,
                    );

                    interlude_scroller.render(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        scene_time,
                        RENDER_H as i32 / 2,
                        phase_fade,
                    );
                }
            }

            4 => {
                // Scene 4: Full Greetings Scroll
                // Plasma background (subdued)
                plasma.render_overlay(&mut buffer, RENDER_W, RENDER_H, scene_time, 0.35 * fade);

                // Copper bars at top and bottom
                copper.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    fade * 0.5,
                    CopperMode::Bands,
                );

                // Main scrolltext -- large and proud
                scroller.update(dt);
                scroller.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    RENDER_H as i32 / 2,
                    fade,
                );

                // "GREETINGS" header
                ScrollText::render_title(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "*** GREETINGS FROM THE RUST DEMOSCENE ***",
                    60,
                    1,
                    palette::dim(palette::EMERALD_500, fade * 0.7),
                    palette::dim(palette::EMERALD_700, fade * 0.3),
                    fade,
                    scene_time,
                );
            }

            5 | _ => {
                // Scene 5: Outro -- composite everything
                plasma.render_overlay(&mut buffer, RENDER_W, RENDER_H, scene_time, 0.2 * fade);
                starfield.update(dt, 0.3);
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.4, scene_time);

                // Logo centered, large
                let logo_r = RENDER_H.min(RENDER_W) as f32 * 0.25;
                logo::draw_logo(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    RENDER_W as f32 / 2.0,
                    RENDER_H as f32 / 2.0 - 40.0,
                    logo_r,
                    1.0,
                    scene_time,
                    fade,
                );

                // Pulsing title
                let pulse = (scene_time as f32 * 2.0).sin() * 0.15 + 0.85;
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "SOLVERFORGE",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + (logo_r as i32) + 20,
                    4,
                    palette::dim(palette::EMERALD_500, fade * pulse),
                    palette::dim(palette::EMERALD_400, fade * pulse * 0.7),
                );

                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "github.com/solverforge",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + (logo_r as i32) + 70,
                    2,
                    palette::dim(palette::EMERALD_600, fade * 0.6),
                );

                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "CODED IN RUST // NO JAVASCRIPT WAS HARMED",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 - 40,
                    1,
                    palette::dim(palette::EMERALD_700, fade * 0.5),
                );
            }
        }

        // ── Blinking "PRESS SPACE TO ADVANCE" ──────────────────
        {
            let blink = ((demo_time * 2.5).sin() * 0.5 + 0.5) as f32; // 0..1 pulsing
            if blink > 0.2 {
                let alpha = (blink - 0.2) / 0.8; // fade in/out smoothly
                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "PRESS SPACE TO ADVANCE",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 - 18,
                    1,
                    palette::dim(palette::EMERALD_600, alpha * 0.6),
                );
            }
        }

        // ── HUD: scene name + time ─────────────────────────────
        let scene_names = [
            "LOGO REVEAL",
            "PLASMA",
            "STARFIELD WARP",
            "VECTORS",
            "GREETINGS",
            "OUTRO",
        ];
        if scene < scene_names.len() {
            font::draw_text(
                &mut buffer,
                RENDER_W,
                RENDER_H,
                scene_names[scene],
                8,
                8,
                1,
                palette::dim(palette::EMERALD_800, 0.5),
            );
        }

        // ── Present frame ──────────────────────────────────────
        window
            .update_with_buffer(&buffer, RENDER_W, RENDER_H)
            .expect("Failed to update window");
    }

    // Cleanup audio
    drop(audio_stream);
}

/// Headless render: fixed 60fps timestep, write BGR24 frames to stdout.
/// ffmpeg reads from stdin and encodes to MP4.
fn run_headless(duration: f64) {
    const FPS: f64 = 60.0;
    let dt = 1.0 / FPS;
    let total_frames = (duration * FPS).ceil() as u64;

    let mut buffer = vec![0u32; RENDER_W * RENDER_H];
    // BGR24: 3 bytes per pixel
    let mut bgr_frame = vec![0u8; RENDER_W * RENDER_H * 3];

    let plasma = Plasma::new();
    let mut starfield = Starfield::new();
    let mut wireframe = Wireframe::new(RENDER_W, RENDER_H);
    let copper = CopperBars::new();
    let mut scroller = ScrollText::new(3, 400.0, RENDER_W);
    let mut interlude_scroller = ScrollText::new(2, 180.0, RENDER_W);
    let mut logo_reveal = LogoReveal::new();
    let screenshots = Screenshots::load();

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::with_capacity(RENDER_W * RENDER_H * 3 * 4, stdout.lock());

    for frame in 0..total_frames {
        let demo_time = frame as f64 * dt;

        if frame % (FPS as u64 * 5) == 0 {
            eprintln!(
                "[render] frame {}/{} ({:.0}s / {:.0}s)",
                frame, total_frames, demo_time, duration
            );
        }

        // Clear buffer
        let bg = palette::NEAR_BLACK;
        for px in buffer.iter_mut() {
            *px = bg;
        }

        let (scene, scene_time, fade_in, fade_out) = get_scene(demo_time);
        let fade = fade_in * fade_out;

        match scene {
            0 => {
                logo_reveal.update(dt, RENDER_W, RENDER_H);
                logo_reveal.render(&mut buffer, RENDER_W, RENDER_H, scene_time, fade);
            }
            1 => {
                plasma.render(&mut buffer, RENDER_W, RENDER_H, scene_time, fade, false);
                copper.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    fade * 0.7,
                    CopperMode::Classic,
                );
                copper.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    fade * 0.3,
                    CopperMode::Scanlines,
                );
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "PLANNER123",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 - 80,
                    5,
                    palette::dim(palette::WHITE, fade * 0.9),
                    palette::dim(palette::EMERALD_400, fade * 0.5),
                );
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "CONSTRAINT SCHEDULING",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 20,
                    2,
                    palette::dim(palette::EMERALD_400, fade * 0.7),
                    palette::dim(palette::EMERALD_600, fade * 0.4),
                );
            }
            2 => {
                let warp = ((scene_time - 6.0) / 6.0).clamp(0.0, 1.0) as f32;
                starfield.update(dt, warp);
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade, scene_time);
                scroller.update(dt);
                scroller.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    RENDER_H as i32 - 80,
                    fade,
                );
                if warp > 0.5 {
                    let wt = ((warp - 0.5) * 2.0) * fade;
                    font::draw_text_centered_glow(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        "WARP SPEED",
                        RENDER_W as i32 / 2,
                        RENDER_H as i32 / 2 - 30,
                        4,
                        palette::dim(palette::WHITE, wt * 0.8),
                        palette::dim(palette::CYAN_400, wt * 0.6),
                    );
                }
            }
            3 => {
                // Scene 3: 31s — variable phases: shot(7s)/scroll(5s)/shot(7s)/scroll(5s)/shot(7s)
                const PHASE_STARTS: [f64; 6] = [0.0, 7.0, 12.0, 19.0, 24.0, 31.0];
                const XFADE: f64 = 0.8;
                let phase = PHASE_STARTS
                    .windows(2)
                    .position(|w| scene_time >= w[0] && scene_time < w[1])
                    .unwrap_or(4);
                let phase_start = PHASE_STARTS[phase];
                let phase_dur = PHASE_STARTS[phase + 1] - phase_start;
                let phase_t = scene_time - phase_start;
                let phase_fade_in = ((phase_t / XFADE) as f32).clamp(0.0, 1.0);
                let phase_fade_out = (((phase_dur - phase_t) / XFADE) as f32).clamp(0.0, 1.0);
                let phase_fade = phase_fade_in * phase_fade_out * fade;

                // Always advance so scroller never restarts between phases
                interlude_scroller.update(dt);

                wireframe.update(scene_time);
                wireframe.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    fade * 0.2,
                    scene_time,
                    false,
                );

                if phase % 2 == 0 {
                    let shot_idx = phase / 2;
                    let dst_w = RENDER_W as i32;
                    let dst_h = RENDER_H as i32;
                    let shot_cx = RENDER_W as i32 / 2;
                    let shot_cy = RENDER_H as i32 / 2;
                    let shot = screenshots.get(shot_idx);
                    for y in 0..RENDER_H {
                        for x in 0..RENDER_W {
                            let idx = y * RENDER_W + x;
                            buffer[idx] = palette::dim(buffer[idx], 1.0 - phase_fade * 0.85);
                        }
                    }
                    effects::screenshots::blit(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        shot,
                        shot_cx,
                        shot_cy,
                        dst_w,
                        dst_h,
                        phase_fade,
                    );
                    effects::screenshots::draw_border(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        shot_cx,
                        shot_cy,
                        dst_w,
                        dst_h,
                        palette::dim(palette::EMERALD_600, phase_fade * 0.6),
                    );
                    let left_x = 32i32;
                    let numbers = ["01 -", "02 -", "03 -"];
                    let headings = ["THE PLAN.", "THE GANTT.", "THE CALENDAR."];
                    let captions = [
                        "MULTI-PROJECT MANAGEMENT. DEPENDENCIES. PRIORITIES.",
                        "CONSTRAINT-OPTIMIZED. DEADLINES RESPECTED. MATH.",
                        "YOUR WEEK, SOLVED. TASKS FIT AROUND YOUR EVENTS.",
                    ];
                    let slide = ((phase_t * 3.0) as f32).clamp(0.0, 1.0);
                    let text_fade = phase_fade * slide;
                    for y in 0..68usize {
                        for x in 0..RENDER_W {
                            let idx = y * RENDER_W + x;
                            buffer[idx] = palette::dim(buffer[idx], 1.0 - text_fade * 0.7);
                        }
                    }
                    font::draw_text(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        numbers[shot_idx],
                        left_x,
                        10,
                        1,
                        palette::dim(palette::EMERALD_700, text_fade * 0.9),
                    );
                    font::draw_text_glow(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        headings[shot_idx],
                        left_x,
                        22,
                        3,
                        palette::dim(palette::WHITE, text_fade),
                        palette::dim(palette::EMERALD_500, text_fade * 0.5),
                    );
                    font::draw_text_glow(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        captions[shot_idx],
                        left_x,
                        RENDER_H as i32 - 26,
                        1,
                        palette::dim(palette::EMERALD_400, text_fade * 0.9),
                        palette::dim(palette::EMERALD_700, text_fade * 0.3),
                    );
                    {
                        let dot_y = RENDER_H as i32 - 20;
                        let dot_gap = 14i32;
                        let right_x = RENDER_W as i32 - 32;
                        for i in 0..3usize {
                            let dx = right_x - (2 - i as i32) * dot_gap;
                            let col = if i == shot_idx {
                                palette::dim(palette::EMERALD_400, phase_fade)
                            } else {
                                palette::dim(palette::EMERALD_800, phase_fade * 0.4)
                            };
                            for dy in -2i32..=2 {
                                for ddx in -2i32..=2 {
                                    if ddx * ddx + dy * dy <= 4 {
                                        let px = dx + ddx;
                                        let py = dot_y + dy;
                                        if px >= 0
                                            && py >= 0
                                            && px < RENDER_W as i32
                                            && py < RENDER_H as i32
                                        {
                                            buffer[py as usize * RENDER_W + px as usize] = col;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    plasma.render_overlay(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        scene_time,
                        0.25 * phase_fade,
                    );
                    interlude_scroller.render(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        scene_time,
                        RENDER_H as i32 / 2,
                        phase_fade,
                    );
                }
            }
            4 => {
                plasma.render_overlay(&mut buffer, RENDER_W, RENDER_H, scene_time, 0.35 * fade);
                copper.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    fade * 0.5,
                    CopperMode::Bands,
                );
                scroller.update(dt);
                scroller.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    scene_time,
                    RENDER_H as i32 / 2,
                    fade,
                );
                ScrollText::render_title(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "*** GREETINGS FROM THE RUST DEMOSCENE ***",
                    60,
                    1,
                    palette::dim(palette::EMERALD_500, fade * 0.7),
                    palette::dim(palette::EMERALD_700, fade * 0.3),
                    fade,
                    scene_time,
                );
            }
            _ => {
                plasma.render_overlay(&mut buffer, RENDER_W, RENDER_H, scene_time, 0.2 * fade);
                starfield.update(dt, 0.3);
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.4, scene_time);
                let logo_r = RENDER_H.min(RENDER_W) as f32 * 0.25;
                logo::draw_logo(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    RENDER_W as f32 / 2.0,
                    RENDER_H as f32 / 2.0 - 40.0,
                    logo_r,
                    1.0,
                    scene_time,
                    fade,
                );
                let pulse = (scene_time as f32 * 2.0).sin() * 0.15 + 0.85;
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "SOLVERFORGE",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + (logo_r as i32) + 20,
                    4,
                    palette::dim(palette::EMERALD_500, fade * pulse),
                    palette::dim(palette::EMERALD_400, fade * pulse * 0.7),
                );
                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "github.com/solverforge",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + (logo_r as i32) + 70,
                    2,
                    palette::dim(palette::EMERALD_600, fade * 0.6),
                );
                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "CODED IN RUST // NO JAVASCRIPT WAS HARMED",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 - 40,
                    1,
                    palette::dim(palette::EMERALD_700, fade * 0.5),
                );
            }
        }

        // Convert minifb 0x00RRGGBB → BGR24
        for (i, &px) in buffer.iter().enumerate() {
            let r = ((px >> 16) & 0xFF) as u8;
            let g = ((px >> 8) & 0xFF) as u8;
            let b = (px & 0xFF) as u8;
            bgr_frame[i * 3] = b;
            bgr_frame[i * 3 + 1] = g;
            bgr_frame[i * 3 + 2] = r;
        }

        out.write_all(&bgr_frame).expect("stdout write failed");
    }

    eprintln!("[render] Done. {} frames written.", total_frames);
}

/// Returns (scene_index, scene_local_time, fade_in, fade_out)
fn get_scene(demo_time: f64) -> (usize, f64, f32, f32) {
    const CROSSFADE: f64 = 2.5; // seconds of crossfade between scenes

    let scenes: &[(f64, f64)] = &[
        (SCENE_LOGO, SCENE_PLASMA),
        (SCENE_PLASMA, SCENE_STARFIELD),
        (SCENE_STARFIELD, SCENE_WIREFRAME),
        (SCENE_WIREFRAME, SCENE_SCROLL),
        (SCENE_SCROLL, SCENE_OUTRO),
        (SCENE_OUTRO, DEMO_END),
    ];

    for (i, &(start, end)) in scenes.iter().enumerate() {
        if demo_time >= start && demo_time < end {
            let scene_t = demo_time - start;
            let duration = end - start;

            // Fade in
            let fade_in = ((scene_t / CROSSFADE) as f32).clamp(0.0, 1.0);

            // Fade out
            let time_left = duration - scene_t;
            let fade_out = ((time_left / CROSSFADE) as f32).clamp(0.0, 1.0);

            return (i, scene_t, ease_in_out(fade_in), ease_in_out(fade_out));
        }
    }

    // Default: outro
    (5, demo_time - SCENE_OUTRO, 1.0, 1.0)
}

/// Smooth ease in/out curve
fn ease_in_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Start the audio stream with the synthwave synthesizer.
/// Returns the stream (must be kept alive).
fn start_audio() -> Option<cpal::Stream> {
    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(d) => d,
        None => {
            eprintln!("[audio] No output device found -- running silent");
            return None;
        }
    };

    let config = match device.default_output_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[audio] Could not get output config: {e} -- running silent");
            return None;
        }
    };

    let sample_rate = config.sample_rate() as f32;
    eprintln!(
        "[audio] Starting synth at {}Hz, {:?}",
        sample_rate,
        config.sample_format()
    );

    let mut synth = audio::synth::build_synth(sample_rate);
    synth.reset();
    synth.set_sample_rate(sample_rate as f64);

    let channels = config.channels() as usize;

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
            let frames = data.len() / channels;
            for frame in 0..frames {
                // Get stereo sample from synth
                let out = synth.get_stereo();
                let l = out.0.clamp(-1.0, 1.0);
                let r = out.1.clamp(-1.0, 1.0);

                for ch in 0..channels {
                    data[frame * channels + ch] = if ch == 0 { l } else { r };
                }
            }
        },
        |err| eprintln!("[audio] Stream error: {err}"),
        None,
    );

    match stream {
        Ok(s) => {
            if let Err(e) = s.play() {
                eprintln!("[audio] Could not play stream: {e}");
                return None;
            }
            Some(s)
        }
        Err(e) => {
            eprintln!("[audio] Could not build stream: {e} -- running silent");
            None
        }
    }
}
