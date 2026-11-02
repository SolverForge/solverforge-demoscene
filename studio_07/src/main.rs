mod audio;
mod gfx;
mod portrait;
mod post;
mod routes;
mod scenes;
mod story;
mod timeline;

use std::env;
use std::io::{self, Write};
use std::time::Instant;

use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};

use gfx::{Frame, H, W};
use post::Post;
use scenes::Demo;
use timeline::DURATION;

const FPS: usize = 60;
const AUDIO_WAV_OUT: &str = "target/demo_audio.wav";

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let value_after = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args.get(i + 1).cloned())
    };

    if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
        return Ok(());
    }
    if let Some(seconds) = value_after("--render") {
        let seconds = seconds.and_then(|s| s.parse().ok()).unwrap_or(DURATION);
        return render(seconds);
    }
    if let Some(Some(times)) = value_after("--stills") {
        let dir = value_after("--out")
            .flatten()
            .unwrap_or_else(|| "target/stills".into());
        return stills(&times, &dir);
    }
    if args.iter().any(|a| a.starts_with('-')) {
        usage();
        return Ok(());
    }
    windowed();
    Ok(())
}

fn usage() {
    eprintln!("usage:");
    eprintln!("  studio-07-demo                         open the demo window with live audio");
    eprintln!("  studio-07-demo --render [seconds]      stream BGR24 frames to stdout, write {AUDIO_WAV_OUT}");
    eprintln!("  studio-07-demo --stills t1,t2,.. [--out dir]   write PPM stills at those times");
}

/// Build the picture (the route search runs here) and then the soundtrack,
/// which plays a blip on every kept move of that search.
fn prepare() -> (Demo, audio::Track) {
    let started = Instant::now();
    let demo = Demo::new();
    let track = audio::Track::render(&demo.move_times);
    eprintln!(
        "[init] {} kept moves, ready in {:.1}s",
        demo.move_times.len(),
        started.elapsed().as_secs_f32()
    );
    (demo, track)
}

fn windowed() {
    eprintln!("\nSTUDIO_07 -- VITTORIO DISTEFANO\nESC quit, SPACE next section\n");
    let (mut demo, track) = prepare();
    let stream = track.play();
    let mut window = Window::new(
        "STUDIO_07 - Vittorio Distefano",
        W,
        H,
        WindowOptions {
            scale: Scale::X1,
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
        demo.frame(&mut frame, t);
        post.apply(&mut frame);
        window
            .update_with_buffer(&frame.px, W, H)
            .expect("window update failed");
    }
}

fn render(seconds: f32) -> io::Result<()> {
    let (mut demo, track) = prepare();
    track.write_wav(seconds, AUDIO_WAV_OUT);
    let frames = (seconds.max(0.0) * FPS as f32).ceil() as usize;
    eprintln!("[render] {W}x{H} @ {FPS}fps, {seconds:.1}s -> stdout (BGR24)");
    let mut frame = Frame::new();
    let mut post = Post::new();
    let mut bgr = Vec::with_capacity(W * H * 3);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for i in 0..frames {
        let t = i as f32 / FPS as f32;
        demo.frame(&mut frame, t);
        post.apply(&mut frame);
        post::to_bgr(&frame, &mut bgr);
        out.write_all(&bgr)?;
        if i > 0 && i % (FPS * 10) == 0 {
            eprintln!("[render] {:.0}s / {seconds:.0}s", t);
        }
    }
    out.flush()?;
    eprintln!("[render] done, {frames} frames");
    Ok(())
}

fn stills(times: &str, dir: &str) -> io::Result<()> {
    let mut demo = Demo::new();
    std::fs::create_dir_all(dir)?;
    let mut frame = Frame::new();
    let mut post = Post::new();
    for t in times
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
    {
        demo.frame(&mut frame, t);
        post.apply(&mut frame);
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
