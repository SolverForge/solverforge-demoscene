mod audio;
mod canvas;
mod data;
mod font;
mod logo;
mod noise;
mod post;
mod scenes;
mod timeline;
mod tui;
mod views;

use std::env;
use std::io::{self, Write};
use std::time::Instant;

use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use canvas::{COLS, ROWS};
use post::{Post, HEIGHT, WIDTH};
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
    eprintln!("  planner123-06-demo                         open the demo window with live audio");
    eprintln!("  planner123-06-demo --render [seconds]      stream BGR24 frames to stdout, write {AUDIO_WAV_OUT}");
    eprintln!(
        "  planner123-06-demo --stills t1,t2,.. [--out dir]   write PPM stills at those times"
    );
}

fn screen() -> Buffer {
    Buffer::empty(Rect::new(0, 0, COLS, ROWS))
}

/// Build the picture and the soundtrack in parallel.
fn prepare() -> (Demo, audio::Track) {
    let started = Instant::now();
    let song = std::thread::spawn(audio::Track::render);
    let demo = Demo::new();
    let track = song.join().expect("soundtrack render");
    eprintln!("[init] ready in {:.1}s", started.elapsed().as_secs_f32());
    (demo, track)
}

fn windowed() {
    eprintln!("\nPLANNER123_06 -- THE DEMO RUNS INSIDE PLANNER123\nESC quit, SPACE next section\n");
    let (demo, track) = prepare();
    let stream = track.play();
    let mut window = Window::new(
        "PLANNER123_06 - SolverForge",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X1,
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("failed to open window");
    window.set_target_fps(FPS);

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
        let mut buf = screen();
        let look = demo.frame(&mut buf, t);
        post.render(&buf, &look);
        window
            .update_with_buffer(&post.pixels, WIDTH, HEIGHT)
            .expect("window update failed");
    }
}

fn render(seconds: f32) -> io::Result<()> {
    let (demo, track) = prepare();
    track.write_wav(seconds, AUDIO_WAV_OUT);
    let frames = (seconds.max(0.0) * FPS as f32).ceil() as usize;
    eprintln!("[render] {WIDTH}x{HEIGHT} @ {FPS}fps, {seconds:.1}s -> stdout (BGR24)");
    let mut post = Post::new();
    let mut bgr = Vec::with_capacity(WIDTH * HEIGHT * 3);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for i in 0..frames {
        let t = i as f32 / FPS as f32;
        let mut buf = screen();
        let look = demo.frame(&mut buf, t);
        post.render(&buf, &look);
        post.to_bgr(&mut bgr);
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
    let demo = Demo::new();
    std::fs::create_dir_all(dir)?;
    let mut post = Post::new();
    for t in times
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
    {
        let mut buf = screen();
        let look = demo.frame(&mut buf, t);
        post.render(&buf, &look);
        let mut ppm = format!("P6\n{WIDTH} {HEIGHT}\n255\n").into_bytes();
        for px in &post.pixels {
            ppm.extend_from_slice(&[(px >> 16) as u8, (px >> 8) as u8, *px as u8]);
        }
        let path = format!("{dir}/t{t:07.2}.ppm");
        std::fs::write(&path, ppm)?;
        eprintln!("[still] {path}");
    }
    Ok(())
}
