mod audio;
mod config;
mod framebuffer;
mod math;
mod palette;
mod render;
mod route;
mod route_plan;
mod scene;

use std::env;
use std::io::{self, Write};
use std::time::Instant;

use minifb::{Key, Scale, Window, WindowOptions};

use config::{DemoConfig, AUDIO_WAV_OUT, DEFAULT_DURATION_SECONDS};
use framebuffer::Framebuffer;
use palette::VOID;
use render::render_frame;
use route::RouteState;
use route_plan::RoutePlan;
use scene::next_scene_boundary;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let config = DemoConfig::default();

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--version") {
        println!("route-splice-04-demo 0.1.0");
        return Ok(());
    }

    if let Some(seconds) = render_duration(&args) {
        run_headless(&config, seconds)
    } else if args.iter().any(|arg| arg.starts_with('-')) {
        eprintln!("unknown option");
        print_usage();
        Ok(())
    } else {
        run_windowed(&config);
        Ok(())
    }
}

fn render_duration(args: &[String]) -> Option<f32> {
    args.iter().position(|arg| arg == "--render").map(|idx| {
        args.get(idx + 1)
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(DEFAULT_DURATION_SECONDS)
    })
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  route-splice-04-demo                    # open windowed demo");
    eprintln!(
        "  route-splice-04-demo --render <seconds> # stream BGR24 frames and write audio WAV"
    );
    eprintln!("  route-splice-04-demo --help");
}

fn run_windowed(config: &DemoConfig) {
    eprintln!();
    eprintln!("ROUTE_SPLICE_04 -- LIST VARIABLE ROUTE");
    eprintln!("SolverForge demoscene intro");
    eprintln!("Controls: ESC = exit, SPACE = advance scene");
    eprintln!("Resolution: {}x{}", config.width, config.height);
    eprintln!();

    let plan = RoutePlan::new();
    let route = RouteState::new(&plan);
    let audio_clock = audio::AudioClock::new();
    let _audio_stream = audio::start_audio(audio_clock.clone());

    let mut window = Window::new(
        "ROUTE_SPLICE_04 - SolverForge List Variable Route",
        config.width,
        config.height,
        WindowOptions {
            scale: Scale::X1,
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("failed to create minifb window");
    window.set_target_fps(config.fps as usize);

    let mut fb = Framebuffer::new(config.width, config.height);
    let mut minifb_buffer = vec![0u32; config.width * config.height];
    let start = Instant::now();
    let mut time_offset = 0.0_f32;
    let mut space_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let elapsed = start.elapsed().as_secs_f32() + time_offset;
        let demo_time = elapsed % DEFAULT_DURATION_SECONDS;

        let space_down = window.is_key_down(Key::Space);
        if space_down && !space_was_down {
            if let Some(next) = next_scene_boundary(demo_time) {
                time_offset += next - demo_time;
            } else {
                time_offset += DEFAULT_DURATION_SECONDS - demo_time;
            }
        }
        space_was_down = space_down;

        let elapsed = start.elapsed().as_secs_f32() + time_offset;
        let demo_time = elapsed % DEFAULT_DURATION_SECONDS;
        audio_clock.set_time(demo_time);

        fb.clear(VOID);
        render_frame(&mut fb, &plan, &route, demo_time);
        fb.to_minifb_buffer(&mut minifb_buffer);
        window
            .update_with_buffer(&minifb_buffer, config.width, config.height)
            .expect("window update failed");
    }
}

fn run_headless(config: &DemoConfig, seconds: f32) -> io::Result<()> {
    let seconds = seconds.max(0.0);
    let plan = RoutePlan::new();
    let route = RouteState::new(&plan);
    audio::render_audio_wav(seconds, AUDIO_WAV_OUT);

    eprintln!(
        "[render] Headless video: {}x{} @ {}fps, {:.1}s -> stdout (BGR24)",
        config.width, config.height, config.fps, seconds
    );

    let frames = (seconds * config.fps as f32).ceil() as usize;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut fb = Framebuffer::new(config.width, config.height);

    for frame_idx in 0..frames {
        let t = frame_idx as f32 / config.fps as f32;
        fb.clear(VOID);
        render_frame(&mut fb, &plan, &route, t);
        out.write_all(&fb.pixels)?;
        if frame_idx > 0 && frame_idx % (config.fps as usize * 10) == 0 {
            eprintln!("[render] {:.0}s / {:.0}s", t, seconds);
        }
    }

    out.flush()?;
    eprintln!("[render] Done. {frames} frames written.");
    Ok(())
}
