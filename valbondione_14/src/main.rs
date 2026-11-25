mod amiga;
mod ascii;
mod audio;
mod fx;
mod gfx;
mod glyphs;
mod hud;
mod post;
mod scenes;
mod sprites;
mod timeline;
mod tour;
mod veins;
mod world;

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
    if let Some(seconds) = value_after("--wav") {
        let seconds = seconds.and_then(|s| s.parse().ok()).unwrap_or(DURATION);
        audio::Track::render().write_wav(seconds, AUDIO_WAV_OUT);
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
    if let Some(Some(times)) = value_after("--bench") {
        bench(&times);
        return Ok(());
    }
    windowed();
    Ok(())
}

fn usage() {
    eprintln!("usage:");
    eprintln!("  valbondione-14-demo                       open the demo window with live audio");
    eprintln!(
        "  valbondione-14-demo --wav [seconds]       write the soundtrack to {AUDIO_WAV_OUT}"
    );
    eprintln!("  valbondione-14-demo --render [seconds]    stream BGR24 frames to stdout");
    eprintln!("  valbondione-14-demo --stills t1,t2,.. [--out dir]   write PPM stills");
    eprintln!("  valbondione-14-demo --bench t1,t2,..      time the scene and the post pass");
}

fn windowed() {
    eprintln!("\nVALBONDIONE_14 -- NIGHTFALL\nESC quit, SPACE next scene\n");
    let started = Instant::now();
    let mut demo = Demo::new();
    let track = audio::Track::render();
    eprintln!("[init] ready in {:.1}s", started.elapsed().as_secs_f32());
    let stream = track.play();
    let mut window = Window::new(
        "VALBONDIONE_14",
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
        let look = demo.frame(&mut frame, t % DURATION);
        post.apply(&mut frame, look, t);
        window
            .update_with_buffer(&frame.px, W, H)
            .expect("window update failed");
    }
}

fn render(seconds: f32) -> io::Result<()> {
    let mut demo = Demo::new();
    let frames = (seconds.max(0.0) * FPS as f32).round() as usize;
    eprintln!("[render] {W}x{H} @ {FPS}fps, {seconds:.1}s -> stdout (BGR24)");
    let mut frame = Frame::new();
    let mut post = Post::new();
    let mut bgr = Vec::with_capacity(W * H * 3);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let started = Instant::now();
    for i in 0..frames {
        let t = i as f32 / FPS as f32;
        let look = demo.frame(&mut frame, t);
        post.apply(&mut frame, look, t);
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

fn stills(times: &str, dir: &str) -> io::Result<()> {
    let mut demo = Demo::new();
    std::fs::create_dir_all(dir)?;
    let mut frame = Frame::new();
    let mut post = Post::new();
    for t in times
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
    {
        let look = demo.frame(&mut frame, t);
        post.apply(&mut frame, look, t);
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

/// Time the scene and the post pass separately, 60 frames from each start.
fn bench(times: &str) {
    let mut demo = Demo::new();
    let mut frame = Frame::new();
    let mut post = Post::new();
    for t0 in times
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
    {
        let (mut scene, mut finish) = (0.0f64, 0.0f64);
        for i in 0..60 {
            let t = t0 + i as f32 / FPS as f32;
            let a = Instant::now();
            let look = demo.frame(&mut frame, t);
            let b = Instant::now();
            post.apply(&mut frame, look, t);
            scene += (b - a).as_secs_f64();
            finish += b.elapsed().as_secs_f64();
        }
        eprintln!(
            "[bench] t={t0:6.1}  scene {:5.1} ms  post {:5.1} ms",
            scene / 60.0 * 1e3,
            finish / 60.0 * 1e3
        );
    }
}
