mod audio;
mod board;
mod capture;
mod code;
mod fx;
mod gfx;
mod json;
mod post;
mod scenes;
mod script;
mod story;
mod timeline;

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

use audio::song::Cues;
use capture::Capture;
use gfx::{Frame, H, W};
use post::Post;
use scenes::Demo;
use timeline::DURATION;

const FPS: usize = 60;
const AUDIO_WAV_OUT: &str = "target/demo_audio.wav";
const DEFAULT_CAPTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let value_after = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args.get(i + 1).cloned())
    };
    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
        return ExitCode::SUCCESS;
    }
    let path = value_after("--capture")
        .flatten()
        .unwrap_or_else(|| DEFAULT_CAPTURE.to_string());
    let allow_placeholder = args.iter().any(|a| a == "--allow-placeholder");
    let cap = match Capture::load(&path).and_then(|c| c.guard(allow_placeholder).map(|_| c)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("forge-11-demo: {e}");
            return ExitCode::FAILURE;
        }
    };
    eprintln!(
        "[capture] {path}: schema {}, {} steps, {} excerpts, {} + {} assignments",
        cap.schema,
        cap.steps.len(),
        cap.code.len(),
        cap.solve.plan.len(),
        cap.repair.solve.plan.len()
    );

    let result = if args.iter().any(|a| a == "--cues") {
        print_cues(&cap);
        Ok(())
    } else if let Some(seconds) = value_after("--render") {
        let seconds = seconds.and_then(|s| s.parse().ok()).unwrap_or(DURATION);
        render(cap, seconds)
    } else if let Some(Some(times)) = value_after("--stills") {
        let dir = value_after("--out")
            .flatten()
            .unwrap_or_else(|| "target/stills".into());
        stills(cap, &times, &dir)
    } else {
        windowed(cap);
        Ok(())
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("forge-11-demo: {e}");
            ExitCode::FAILURE
        }
    }
}

fn usage() {
    eprintln!("usage:");
    eprintln!("  forge-11-demo [--capture FILE] [--allow-placeholder]   window with live audio");
    eprintln!("  forge-11-demo ... --render [seconds]   stream BGR24 frames to stdout, write {AUDIO_WAV_OUT}");
    eprintln!(
        "  forge-11-demo ... --stills t1,t2,.. [--out dir]   write PPM stills at those times"
    );
    eprintln!("  forge-11-demo ... --cues               print when every captured event plays");
    eprintln!();
    eprintln!("The capture defaults to assets/capture.json and must be schema forge11.capture/1;");
    eprintln!("--allow-placeholder accepts forge11.placeholder/1 and burns PLACEHOLDER DATA in.");
}

fn cues(s: &script::Script) -> Cues {
    Cues {
        typing: s.typing.words.iter().map(|w| w.0).collect(),
        steps: s.steps.clone(),
        anvils: s.code.iter().map(|c| c.0).collect(),
        drops: s.drops.clone(),
        improvements: s.improvements.clone(),
        solve_feasible: s.solve_feasible,
        event: s.event,
        moves: s.moves.clone(),
        repair_feasible: s.repair_feasible,
        reviews: s.reviews.clone(),
        cordelia: {
            let b = s.cordelia;
            [b.enter, b.ignite, b.fire, b.cut, b.exit]
        },
    }
}

/// Build the picture, then the soundtrack from the same schedule.
fn prepare(cap: Capture) -> (Demo, audio::Track) {
    let started = Instant::now();
    let demo = Demo::new(cap);
    let track = audio::Track::render(&cues(&demo.script));
    eprintln!("[init] ready in {:.1}s", started.elapsed().as_secs_f32());
    (demo, track)
}

fn print_cues(cap: &Capture) {
    let s = script::Script::new(cap);
    let round = |v: &[f32]| {
        v.iter()
            .map(|t| (t * 100.0).round() / 100.0)
            .collect::<Vec<_>>()
    };
    println!(
        "prompt {}x{} cells, {} lines, {} words typed {:.2}..{:.2}",
        s.prompt_size.w,
        s.prompt_size.h,
        s.prompt_lines.len(),
        s.typing.words.len(),
        s.typing.words.first().map_or(0.0, |w| w.0),
        s.typing.words.last().map_or(0.0, |w| w.0)
    );
    println!("steps {:?}", round(&s.steps));
    println!("code {:?}", s.code);
    println!(
        "solve from {:.2} len {:.2} feasible {:?}",
        s.solve.from, s.solve.len, s.solve_feasible
    );
    println!(
        "event {:.2}; repair from {:.2} len {:.2} feasible {:?}",
        s.event, s.repair.from, s.repair.len, s.repair_feasible
    );
    println!("moves {:?}", round(&s.moves));
    println!("cordelia {:?}", s.cordelia);
    println!("reviews {:?} each {:.2}", round(&s.reviews), s.review_len);
    println!("close {:?} logo {:.2}", s.close, s.logo);
    for c in &s.captions {
        println!(
            "caption {:7.2}..{:7.2} {:?} {:?}: {}",
            c.from, c.to, c.place, c.cap, c.text
        );
    }
}

fn windowed(cap: Capture) {
    eprintln!("\nSOLVERFORGE -- DESCRIBE, GENERATE, SOLVE, REPAIR\nESC quit, SPACE next section\n");
    let (mut demo, track) = prepare(cap);
    let stream = track.play();
    let mut window = Window::new(
        "SolverForge -- describe, generate, solve, repair",
        W,
        H,
        WindowOptions {
            scale: Scale::FitScreen,
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("failed to open window");
    window.set_target_fps(FPS);
    let mut frame = Frame::new();
    let mut post = Post::new();
    let wall = Instant::now();
    let mut offset = 0.0f32;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // The audio playhead is the master clock; without audio, the wall clock.
        let mut t = if stream.is_some() {
            track.time()
        } else {
            (wall.elapsed().as_secs_f32() + offset) % DURATION
        };
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            let next = timeline::next_boundary(t);
            track.seek(next);
            offset += (next - t).rem_euclid(DURATION);
            t = next;
        }
        let bloom = demo.frame(&mut frame, t);
        post.apply(&mut frame, bloom);
        demo.watermark(&mut frame, t);
        window
            .update_with_buffer(&frame.px, W, H)
            .expect("window update failed");
    }
}

fn render(cap: Capture, seconds: f32) -> io::Result<()> {
    let (mut demo, track) = prepare(cap);
    track.write_wav(seconds, AUDIO_WAV_OUT);
    let frames = (seconds.max(0.0) * FPS as f32).ceil() as usize;
    eprintln!("[render] {W}x{H} @ {FPS}fps, {seconds:.1}s -> stdout (BGR24)");
    let mut frame = Frame::new();
    let mut post = Post::new();
    let mut bgr = Vec::with_capacity(W * H * 3);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let started = Instant::now();
    for i in 0..frames {
        let t = i as f32 / FPS as f32;
        let bloom = demo.frame(&mut frame, t);
        post.apply(&mut frame, bloom);
        demo.watermark(&mut frame, t);
        post::to_bgr(&frame, &mut bgr);
        out.write_all(&bgr)?;
        if i > 0 && i % (FPS * 10) == 0 {
            eprintln!(
                "[render] {:.0}s / {seconds:.0}s ({:.1} fps)",
                t,
                i as f32 / started.elapsed().as_secs_f32()
            );
        }
    }
    out.flush()?;
    eprintln!("[render] done, {frames} frames");
    Ok(())
}

fn stills(cap: Capture, times: &str, dir: &str) -> io::Result<()> {
    let mut demo = Demo::new(cap);
    std::fs::create_dir_all(dir)?;
    let mut frame = Frame::new();
    let mut post = Post::new();
    for t in times
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
    {
        let bloom = demo.frame(&mut frame, t);
        post.apply(&mut frame, bloom);
        demo.watermark(&mut frame, t);
        let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
        for px in &frame.px {
            ppm.extend_from_slice(&[(px >> 16) as u8, (px >> 8) as u8, *px as u8]);
        }
        let path = format!("{dir}/t{t:07.2}.ppm");
        std::fs::write(&path, ppm)?;
        eprintln!("[still] {path}");
    }
    Ok(())
}
