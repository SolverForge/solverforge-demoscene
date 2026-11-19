mod audio;
mod cut;
mod gfx;
mod models;
mod post;
mod scenes;
mod statue;
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
    if args.iter().any(|a| a == "--live") {
        // For the compose step: when the live section runs, and where the
        // take, the advisor's portrait and her dialog box go.
        println!("live {} {}", timeline::LIVE_FROM, timeline::LIVE_TO);
        println!("cut {}", cut::TAKE_SECONDS);
        let windows = [
            ("take", scenes::HOLE),
            ("portrait", scenes::PORTRAIT),
            ("dialog", scenes::DIALOG),
        ];
        for (name, (x0, y0, x1, y1)) in windows {
            println!("{name} {x0} {y0} {} {}", x1 - x0, y1 - y0);
        }
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
    if let Some(Some(times)) = value_after("--bench") {
        bench(&times);
        return Ok(());
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
    eprintln!(
        "  workbench-12-demo                            open the demo window with live audio"
    );
    eprintln!(
        "  workbench-12-demo --wav [seconds]         write the soundtrack to {AUDIO_WAV_OUT}"
    );
    eprintln!("  workbench-12-demo --render [seconds]      stream BGR24 frames to stdout");
    eprintln!(
        "  workbench-12-demo --stills t1,t2,.. [--out dir]   write PPM stills at those times"
    );
    eprintln!(
        "  workbench-12-demo --live                    print the live section and the take window: x y w h"
    );
}

/// Build the picture and then the soundtrack.
fn prepare() -> (Demo, audio::Track) {
    let started = Instant::now();
    let demo = Demo::new();
    let track = audio::Track::render();
    eprintln!("[init] ready in {:.1}s", started.elapsed().as_secs_f32());
    (demo, track)
}

fn windowed() {
    eprintln!("\nWORKBENCH_12 -- SOLVERFORGE WORKBENCH FOR HERMES\nESC quit, SPACE next section\n");
    let (mut demo, track) = prepare();
    let stream = track.play();
    let mut window = Window::new(
        "WORKBENCH_12 - SolverForge Workbench for Hermes",
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

    // Two frames in flight: while one gets its post pass, the scene of the
    // next is drawn on the other, so a frame costs the slower of the two
    // halves instead of their sum. The next frame is drawn for one frame
    // later, so the picture shown stays on the audio clock.
    let mut frames = [Frame::new(), Frame::new()];
    let mut post = Post::new();
    let wall = Instant::now();
    let mut offset = 0.0f32;
    let mut shown = 0usize;
    demo.frame(&mut frames[0], 0.0);
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
        let [a, b] = &mut frames;
        let (done, next) = if shown == 0 { (a, b) } else { (b, a) };
        std::thread::scope(|s| {
            s.spawn(|| post.apply(done));
            demo.frame(next, (t + 1.0 / FPS as f32) % DURATION);
        });
        window
            .update_with_buffer(&done.px, W, H)
            .expect("window update failed");
        shown ^= 1;
    }
}

fn render(seconds: f32) -> io::Result<()> {
    let mut demo = Demo::new();
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
            demo.frame(&mut frame, t);
            let b = Instant::now();
            post.apply(&mut frame);
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
