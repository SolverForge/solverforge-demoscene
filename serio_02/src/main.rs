// ═══════════════════════════════════════════════════════════════
//  ████████╗██╗  ██╗███████╗    ██╗      ██████╗  ██████╗ ███╗   ███╗
//  ╚══╝██╔══╝██║  ██║██╔════╝    ██║     ██╔═══██╗██╔═══██╗████╗ ████║
//      ██║   ███████║█████╗      ██║     ██║   ██║██║   ██║██╔████╔██║
//      ██║   ██╔══██║██╔══╝      ██║     ██║   ██║██║   ██║██║╚██╔╝██║
//      ██║   ██║  ██║███████╗    ███████╗╚██████╔╝╚██████╔╝██║ ╚═╝ ██║
//      ╚═╝   ╚═╝  ╚═╝╚══════╝    ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝     ╚═╝
//
//  MUSICA UNIVERSALIS
//  SEVEN WORLDS. TUNABLE VOICES. PYTHAGOREAN HARMONY.
//
//  SERIO: SCORING ENGINE FOR REAL-TIME INCREMENTAL OPTIMIZATION
//  WHEN ONE WORLD'S VOICE CHANGES, ONLY ITS ARCS ARE RE-EVALUATED.
//
//  PRESS ESC TO EXIT. SPACE TO ADVANCE SCENE.
//  PYTHAGORAS APPROVED.
// ═══════════════════════════════════════════════════════════════

mod audio;
mod effects;
mod font;
mod logo;
mod orrery;
mod palette;

use effects::commentary;
use effects::dual_panel::DualPanel;
use effects::genesis;
use effects::orrery_render::{OrreryRenderOpts, TrailBuffer};
use effects::particles::ParticleSystem;
use effects::plasma::Plasma;
use effects::score_display::ScoreGraph;
use effects::scroll::{self, ScrollText};
use effects::starfield::Starfield;
use palette::Surface;

use orrery::model::Orrery;
use orrery::solver::SolverState;

use minifb::{Key, Scale, Window, WindowOptions};
use std::io::Write;
use std::time::Instant;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// ── Resolution ────────────────────────────────────────────────
const RENDER_W: usize = 1280;
const RENDER_H: usize = 720;

// ── Demo timeline ─────────────────────────────────────────────
// All scenes fixed duration.
const SCENE_GENESIS: f64 = 0.0; // 0s  – 14s: sun ignition, planets materialize
const SCENE_FULL_EVAL: f64 = 14.0; // 14s – 30s: "classical" full re-evaluation
const SCENE_DUAL: f64 = 30.0; // 30s – 55s: split screen, SERIO vs classical
const SCENE_HARMONY: f64 = 55.0; // 55s – 90s: SERIO full speed, 35 seconds
const SCENE_SOLUTION: f64 = 90.0; // 90s – 100s: admire the result
const SOLUTION_DURATION: f64 = 10.0;
const OUTRO_DURATION: f64 = 14.0;

const CROSSFADE: f64 = 2.0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--render");
    let render_duration: f64 = args
        .windows(2)
        .find(|w| w[0] == "--render")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(120.0); // generous upper bound; demo ends when solver solves

    if headless {
        eprintln!(
            "[render] Headless mode: {}x{} @ 60fps, {:.0}s -> stdout (BGR24)",
            RENDER_W, RENDER_H, render_duration
        );
        render_audio_wav(render_duration, "demo_audio.wav");
        run_headless(render_duration);
        return;
    }

    eprintln!();
    eprintln!("╔═══════════════════════════════════════════════════════════╗");
    eprintln!("║   SERIO_02 -- MUSICA UNIVERSALIS                         ║");
    eprintln!("║   SEVEN WORLDS. TWENTY-ONE INTERVALS. ONE SOLUTION.      ║");
    eprintln!("║   A MEDITATION ON INCREMENTAL EVALUATION                 ║");
    eprintln!("║   SOLVERFORGE -- BERGAMO MMXXVI                          ║");
    eprintln!("╚═══════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("  Controls: ESC = exit, SPACE = advance to next scene");
    eprintln!("  Resolution: {}x{}", RENDER_W, RENDER_H);
    eprintln!();

    let window_opts = WindowOptions {
        scale: Scale::X1,
        resize: true,
        ..WindowOptions::default()
    };

    let mut window = Window::new(
        "MUSICA UNIVERSALIS -- SolverForge SERIO",
        RENDER_W,
        RENDER_H,
        window_opts,
    )
    .expect("Failed to create window");
    window.set_target_fps(60);

    let mut buffer = vec![0u32; RENDER_W * RENDER_H];

    // ── Initialize solver ─────────────────────────────────────
    let orrery = Orrery::initial();
    let mut solver = SolverState::new(orrery);

    // Consonant display orrery for genesis (shows gold arcs before solver runs)
    let mut genesis_orrery = Orrery::consonant();

    // ── Audio setup ───────────────────────────────────────────
    let freq_state = audio::synth::initial_freq_state();
    let _audio_stream = start_audio(freq_state.clone());

    // ── Effects ───────────────────────────────────────────────
    let plasma = Plasma::new();
    let mut starfield = Starfield::new();
    let mut trail = TrailBuffer::new(RENDER_W, RENDER_H);
    let mut dual = DualPanel::new();
    let mut particles = ParticleSystem::new();
    let mut scroller = ScrollText::new(2, 280.0, RENDER_W);
    let mut score_graph = ScoreGraph::new();
    let mut worst_soft: i64 = 1;

    // Scene starts for SPACE-to-advance (fixed ones only; scene 3 can't be skipped)
    let fixed_boundaries: &[f64] = &[SCENE_GENESIS, SCENE_FULL_EVAL, SCENE_DUAL, SCENE_HARMONY];

    // ── Demo state ────────────────────────────────────────────
    let start_time = Instant::now();
    let mut last_time = start_time;
    let mut time_offset: f64 = 0.0;
    let mut space_was_down = false;
    let mut prev_history_len = 0usize;
    let mut slow_move_timer: f64 = 0.0;
    let mut last_move_time: f64 = -10.0;
    let mut last_moved_planet: Option<usize> = None;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f64().min(0.05);
        last_time = now;

        let elapsed = now.duration_since(start_time).as_secs_f64() + time_offset;
        let demo_time = elapsed;

        // ── SPACE: advance to next scene ──────────────────────
        // Can skip to scenes 0-3. Can't skip past scene 3 (must solve first).
        let space_down = window.is_key_down(Key::Space);
        if space_down && !space_was_down {
            for &boundary in fixed_boundaries {
                if boundary > demo_time + 0.5 {
                    time_offset += boundary - demo_time;
                    break;
                }
            }
        }
        space_was_down = space_down;

        // ── Run solver steps ──────────────────────────────────
        let (scene, scene_t, _, _) = get_scene(demo_time);

        // Scenes 1-2: animated sweep only (no real solver).
        // The sweep shows the MECHANISM. The solver runs in scene 3.
        if scene == 1 || scene == 2 {
            let interval = if scene == 1 { 4.0 } else { 3.5 };
            slow_move_timer += dt;
            if slow_move_timer >= interval {
                slow_move_timer -= interval;
                let fake_planet = ((demo_time / interval) as usize) % 7;
                last_move_time = demo_time;
                last_moved_planet = Some(fake_planet);
                if scene == 2 {
                    dual.on_move(fake_planet);
                }
            }
        } else if scene == 3 || scene == 4 {
            solver.step_timed(14.0);
            slow_move_timer = 0.0;
        } else {
            slow_move_timer = 0.0;
        }

        // ── Process new moves for visualization ───────────────
        let new_history_len = solver.history.len();
        if new_history_len > prev_history_len {
            let new_moves = &solver.history[prev_history_len..new_history_len];
            // For dual panel, just register the last move (the visuals can't keep up
            // with thousands per frame anyway — show the most recent)
            if let Some(last_mv) = new_moves.last() {
                dual.on_move(last_mv.planet_idx);
            }

            // For particles, spawn from a sample of moves (not all — too many)
            let sample_stride = (new_moves.len() / 4).max(1);
            for (idx, mv) in new_moves.iter().enumerate() {
                if idx % sample_stride == 0 {
                    particles.spawn_move(
                        solver.solution(),
                        mv.planet_idx,
                        640.0,
                        360.0,
                        mv.soft_delta(),
                        mv.accepted,
                    );
                }

                score_graph.push(mv.score_after);
                let soft = mv.score_after.soft();
                if soft < worst_soft {
                    worst_soft = soft;
                }
            }
            prev_history_len = new_history_len;
        }

        // ── Update effects ────────────────────────────────────
        let (_, _, fade_in, fade_out) = get_scene(demo_time);
        let fade = fade_in * fade_out;

        dual.update(dt);
        particles.update(dt);
        scroller.update(dt);
        starfield.update(dt, 0.1);

        // ── Animate planet orbits ─────────────────────────────
        {
            let sol = solver.director.working_solution_mut();
            sol.update_angles(dt, 1.0);
        }
        genesis_orrery.update_angles(dt, 1.0);

        // ── Update audio frequencies ──────────────────────────
        let planet_freqs: [f64; 7] = {
            let planets = &solver.solution().planets;
            [
                planets[0].frequency(),
                planets[1].frequency(),
                planets[2].frequency(),
                planets[3].frequency(),
                planets[4].frequency(),
                planets[5].frequency(),
                planets[6].frequency(),
            ]
        };
        audio::synth::update_frequencies(&freq_state, &planet_freqs);

        // ── Clear buffer ──────────────────────────────────────
        for px in buffer.iter_mut() {
            *px = palette::NEAR_BLACK;
        }

        // ── Render scene ──────────────────────────────────────
        let current_score = solver.current_score();

        // One score sample per frame during scene 3 → smooth real-time graph animation
        if scene == 3 {
            score_graph.push(current_score);
        }

        // Narration progress: ~3 seconds per line
        let narration_progress = |lines: usize, scene_duration: f64| -> f32 {
            let needed = lines as f64 * 3.5; // 3s per line
            (scene_t / needed.min(scene_duration)).clamp(0.0, 1.0) as f32
        };

        // Orrery center X for non-dual scenes: left of center to leave room for right panel
        let orrery_cx = 440.0;
        let orrery_cy = 280.0;
        // Narration center X: centered in the orrery area
        let narr_cx = orrery_cx as i32;

        let mut s = Surface { buf: &mut buffer, w: RENDER_W, h: RENDER_H };

        match scene {
            0 => {
                // GENESIS: space background + planet materialization
                starfield.render(&mut s, fade * 0.3, demo_time);
                genesis::render(
                    &mut s,
                    &genesis_orrery,
                    &mut trail,
                    scene_t,
                    fade,
                    demo_time,
                );

                // Big centered narration — placed above the bottom title area
                let np = narration_progress(scroll::NARRATION_GENESIS.len(), 14.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_GENESIS,
                    narr_cx,
                    RENDER_H as i32 / 2 + 100,
                    np,
                    fade,
                );

                // Commentary: right side
                commentary::render_commentary(
                    &mut s,
                    &solver,
                    scene,
                    fade,
                );
            }

            1 => {
                // FULL EVAL: single orrery, arcs sweep one-by-one after each move
                starfield.render(&mut s, fade * 0.15, demo_time);

                let age = (demo_time - last_move_time) as f32;
                let ring_flash = (1.0 - age / 2.5).clamp(0.0, 1.0);
                // Sweep: arcs evaluate one-by-one over 1.5s after each move
                let no_move_yet = last_move_time < 0.0;
                let sweep = if no_move_yet {
                    -1.0
                } else if age < 1.5 {
                    (age / 1.5) * 21.0
                } else {
                    -1.0 // sweep done, all arcs visible
                };
                let arcs_visible = if no_move_yet { 0.0 } else { 1.0 };
                let opts = OrreryRenderOpts {
                    cx: orrery_cx,
                    cy: orrery_cy,
                    fade,
                    show_labels: false,
                    show_arcs: arcs_visible,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: true,
                    moved_planet: if ring_flash > 0.01 { last_moved_planet } else { None },
                    move_ring_alpha: ring_flash,
                    sweep_frontier: sweep,
                };
                effects::orrery_render::render(
                    &mut s,
                    &genesis_orrery,
                    &mut trail,
                    &opts,
                    demo_time,
                );
                particles.render(&mut s);

                // Arc counter: shows "N / 21" during sweep
                if sweep >= 0.0 {
                    let count = (sweep as usize + 1).min(21);
                    let counter_text = format!("{} / 21 arcs checked", count);
                    font::draw_text_centered(
                        &mut s,
                        &counter_text,
                        orrery_cx as i32,
                        orrery_cy as i32 + 200,
                        2,
                        palette::dim(palette::CLASSICAL_RED, fade * 0.9),
                    );
                }

                // Title above orrery
                font::draw_text_centered_glow(
                    &mut s,
                    "THE WEIGHT OF KNOWING",
                    narr_cx,
                    20,
                    2,
                    palette::dim(palette::CLASSICAL_RED, fade),
                    palette::dim(palette::CLASSICAL_RED, fade * 0.4),
                );

                // Big centered narration below orrery
                let np = narration_progress(scroll::NARRATION_FULL_EVAL.len(), 16.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_FULL_EVAL,
                    narr_cx,
                    RENDER_H as i32 - 140,
                    np,
                    fade,
                );

                // Commentary: right side
                commentary::render_commentary(
                    &mut s,
                    &solver,
                    scene,
                    fade,
                );
            }

            2 => {
                // DUAL PANEL: split screen comparison
                starfield.render(&mut s, fade * 0.12, demo_time);
                dual.render(
                    &mut s,
                    &genesis_orrery,
                    demo_time,
                    fade,
                    scene_t,
                );
                particles.render(&mut s);

                // Narration: top center, between panel labels and orreries
                let np = narration_progress(scroll::NARRATION_DUAL.len(), 25.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_DUAL,
                    RENDER_W as i32 / 2,
                    85,
                    np,
                    fade * 0.95,
                );
            }

            3 => {
                // CONVERGENCE: SERIO at full speed, arcs turn gold
                starfield.render(&mut s, fade * 0.15, demo_time);

                let opts = OrreryRenderOpts {
                    cx: orrery_cx,
                    cy: orrery_cy,
                    fade,
                    show_labels: true,
                    show_arcs: 1.0,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: false,
                    moved_planet: None,
                    move_ring_alpha: 0.0,
                    sweep_frontier: -1.0,
                };
                effects::orrery_render::render(
                    &mut s,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );
                particles.render(&mut s);

                // Title
                font::draw_text_centered_glow(
                    &mut s,
                    "UNLEASHED",
                    narr_cx,
                    20,
                    2,
                    palette::dim(palette::SERIO_BLUE, fade * 0.9),
                    palette::dim(palette::SERIO_BLUE, fade * 0.3),
                );

                // Score graph: left side, below orrery center
                score_graph.render(
                    &mut s,
                    40,
                    RENDER_H as i32 - 210,
                    fade * 0.9,
                );

                // Narration below score graph
                let np = narration_progress(scroll::NARRATION_CONVERGENCE.len(), 24.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_CONVERGENCE,
                    narr_cx,
                    RENDER_H as i32 - 120,
                    np,
                    fade * 0.9,
                );

                // Commentary: right side
                commentary::render_commentary(
                    &mut s,
                    &solver,
                    scene,
                    fade,
                );
            }

            4 => {
                // SOLUTION: Musica Universalis achieved
                starfield.render(&mut s, fade * 0.2, demo_time);

                let opts = OrreryRenderOpts {
                    cx: orrery_cx,
                    cy: orrery_cy,
                    fade,
                    show_labels: true,
                    show_arcs: 1.0,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: false,
                    moved_planet: None,
                    move_ring_alpha: 0.0,
                    sweep_frontier: -1.0,
                };
                effects::orrery_render::render(
                    &mut s,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );

                // Big golden title
                let pulse = (demo_time as f32 * 1.5).sin() * 0.1 + 0.9;
                font::draw_text_centered_glow(
                    &mut s,
                    "MUSICA UNIVERSALIS",
                    narr_cx,
                    20,
                    3,
                    palette::dim(palette::GOLD, fade * pulse),
                    palette::dim(palette::AMBER_400, fade * 0.5),
                );

                // Feasibility status
                let hard = current_score.hard();
                let status = if hard >= 0 { "FEASIBLE" } else { "INFEASIBLE" };
                let status_col = if hard >= 0 {
                    palette::EMERALD_400
                } else {
                    palette::RUST
                };
                font::draw_text_centered(
                    &mut s,
                    status,
                    narr_cx,
                    RENDER_H as i32 - 160,
                    3,
                    palette::dim(status_col, fade),
                );

                // Big centered narration
                let np = narration_progress(scroll::NARRATION_SOLUTION.len(), 10.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_SOLUTION,
                    narr_cx,
                    RENDER_H as i32 - 120,
                    np,
                    fade * 0.9,
                );

                // Commentary: right side (final stats)
                commentary::render_commentary(
                    &mut s,
                    &solver,
                    scene,
                    fade,
                );
            }

            _ => {
                // OUTRO: greetings + credits
                starfield.render(&mut s, fade * 0.25, demo_time);
                plasma.render_overlay(&mut s, demo_time, fade * 0.12);

                let logo_r = 120.0f32;
                let pulse = (scene_t as f32 * 1.2).sin() * 0.1 + 0.9;
                logo::draw_logo(
                    &mut s,
                    RENDER_W as f32 / 2.0,
                    RENDER_H as f32 / 2.0 - 80.0,
                    logo_r,
                    1.0,
                    demo_time,
                    fade * pulse,
                );

                font::draw_text_centered_glow(
                    &mut s,
                    "SOLVERFORGE",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 60,
                    4,
                    palette::dim(palette::EMERALD_500, fade * pulse),
                    palette::dim(palette::EMERALD_400, fade * 0.5),
                );

                font::draw_text_centered(
                    &mut s,
                    "MUSICA UNIVERSALIS",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 95,
                    1,
                    palette::dim(palette::EMERALD_600, fade * 0.7),
                );

                font::draw_text_centered(
                    &mut s,
                    "github.com/solverforge",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 112,
                    1,
                    palette::dim(palette::EMERALD_600, fade * 0.6),
                );

                // Narration: left side
                let np = narration_progress(scroll::NARRATION_OUTRO.len(), 14.0);
                scroll::render_narration(
                    &mut s,
                    scroll::NARRATION_OUTRO,
                    60,
                    80,
                    np,
                    fade * 0.8,
                );

                // Scrolltext at bottom
                scroller.render(
                    &mut s,
                    demo_time,
                    RENDER_H as i32 - 50,
                    fade,
                );

                font::draw_text_centered(
                    &mut s,
                    "CODED IN RUST // BERGAMO MMXXVI // PYTHAGORAS APPROVED",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 - 20,
                    1,
                    palette::dim(palette::EMERALD_700, fade * 0.5),
                );
            }
        }

        // ── "PRESS SPACE" blink ───────────────────────────────
        let blink = ((demo_time * 2.0).sin() as f32 * 0.5 + 0.5).abs();
        font::draw_text_centered(
            &mut s,
            "PRESS SPACE TO ADVANCE",
            RENDER_W as i32 / 2,
            RENDER_H as i32 - 8,
            1,
            palette::dim(palette::EMERALD_800, fade * blink * 0.5),
        );


        window
            .update_with_buffer(&buffer, RENDER_W, RENDER_H)
            .expect("Window update failed");
    }
}

// ── Headless render ───────────────────────────────────────────

fn run_headless(duration_secs: f64) {
    let fps = 60.0f64;
    let total_frames = (duration_secs * fps) as usize;

    let mut buffer = vec![0u32; RENDER_W * RENDER_H];
    let mut bgr_frame = vec![0u8; RENDER_W * RENDER_H * 3];
    let mut out = std::io::BufWriter::new(std::io::stdout());

    let orrery = Orrery::initial();
    let mut solver = SolverState::new(orrery);
    let mut genesis_orrery = Orrery::consonant();
    let mut trail = TrailBuffer::new(RENDER_W, RENDER_H);
    let mut dual = DualPanel::new();
    let mut particles = ParticleSystem::new();
    let mut scroller = ScrollText::new(2, 280.0, RENDER_W);
    let mut score_graph = ScoreGraph::new();
    let plasma = Plasma::new();
    let mut starfield = Starfield::new();
    let mut worst_soft: i64 = 1;
    let mut prev_history_len = 0usize;
    let mut slow_move_timer: f64 = 0.0;
    let mut last_move_time: f64 = -10.0;
    let mut last_moved_planet: Option<usize> = None;
    let dt = 1.0 / fps;

    for frame in 0..total_frames {
        let demo_time = frame as f64 * dt;
        let (scene, scene_t, fade_in, fade_out) = get_scene(demo_time);

        // Stop after outro finishes
        let demo_end = SCENE_SOLUTION + SOLUTION_DURATION + OUTRO_DURATION;
        if demo_time > demo_end + 2.0 {
            eprintln!("[render] Demo finished at {:.1}s", demo_time);
            break;
        }
        let fade = fade_in * fade_out;

        if scene == 1 || scene == 2 {
            let interval = if scene == 1 { 4.0 } else { 3.5 };
            slow_move_timer += dt;
            if slow_move_timer >= interval {
                slow_move_timer -= interval;
                let fake_planet = ((demo_time / interval) as usize) % 7;
                last_move_time = demo_time;
                last_moved_planet = Some(fake_planet);
                if scene == 2 {
                    dual.on_move(fake_planet);
                }
            }
        } else if scene == 3 || scene == 4 {
            solver.step_timed(14.0);
            slow_move_timer = 0.0;
        } else {
            slow_move_timer = 0.0;
        }

        let new_len = solver.history.len();
        if new_len > prev_history_len {
            let new_moves = &solver.history[prev_history_len..new_len];
            if let Some(last_mv) = new_moves.last() {
                dual.on_move(last_mv.planet_idx);
            }
            let sample_stride = (new_moves.len() / 4).max(1);
            for (idx, mv) in new_moves.iter().enumerate() {
                if idx % sample_stride == 0 {
                    particles.spawn_move(
                        solver.solution(),
                        mv.planet_idx,
                        640.0,
                        360.0,
                        mv.soft_delta(),
                        mv.accepted,
                    );
                }
                score_graph.push(mv.score_after);
                let sc = mv.score_after.soft();
                if sc < worst_soft {
                    worst_soft = sc;
                }
            }
            prev_history_len = new_len;
        }

        dual.update(dt);
        particles.update(dt);
        scroller.update(dt);
        starfield.update(dt, 0.1);

        // Animate orbits
        {
            let sol = solver.director.working_solution_mut();
            sol.update_angles(dt, 1.0);
        }
        genesis_orrery.update_angles(dt, 1.0);

        for px in buffer.iter_mut() {
            *px = palette::NEAR_BLACK;
        }

        let current_score = solver.current_score();

        if scene == 3 {
            score_graph.push(current_score);
        }

        let narration_progress = |lines: usize, scene_duration: f64| -> f32 {
            let needed = lines as f64 * 3.5;
            (scene_t / needed.min(scene_duration)).clamp(0.0, 1.0) as f32
        };

        let orrery_cx = 440.0;
        let orrery_cy = 280.0;
        let narr_cx = orrery_cx as i32;

        let mut s = Surface { buf: &mut buffer, w: RENDER_W, h: RENDER_H };

        match scene {
            0 => {
                starfield.render(&mut s, fade * 0.3, demo_time);
                genesis::render(
                    &mut s,
                    &genesis_orrery,
                    &mut trail,
                    scene_t,
                    fade,
                    demo_time,
                );
                let np = narration_progress(scroll::NARRATION_GENESIS.len(), 14.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_GENESIS,
                    narr_cx,
                    RENDER_H as i32 / 2 + 100,
                    np,
                    fade,
                );
                commentary::render_commentary(
                    &mut s,
                    &solver,
                    scene,
                    fade,
                );
            }
            1 => {
                starfield.render(&mut s, fade * 0.15, demo_time);
                let age = (demo_time - last_move_time) as f32;
                let ring_flash = (1.0 - age / 2.5).clamp(0.0, 1.0);
                let no_move_yet = last_move_time < 0.0;
                let sweep = if no_move_yet {
                    -1.0
                } else if age < 1.5 {
                    (age / 1.5) * 21.0
                } else {
                    -1.0
                };
                let arcs_visible = if no_move_yet { 0.0 } else { 1.0 };
                let opts = OrreryRenderOpts {
                    cx: orrery_cx,
                    cy: orrery_cy,
                    fade,
                    show_labels: false,
                    show_arcs: arcs_visible,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: true,
                    moved_planet: if ring_flash > 0.01 { last_moved_planet } else { None },
                    move_ring_alpha: ring_flash,
                    sweep_frontier: sweep,
                };
                effects::orrery_render::render(
                    &mut s,
                    &genesis_orrery,
                    &mut trail,
                    &opts,
                    demo_time,
                );
                particles.render(&mut s);
                if sweep >= 0.0 {
                    let count = (sweep as usize + 1).min(21);
                    let counter_text = format!("{} / 21 arcs checked", count);
                    font::draw_text_centered(
                        &mut s,
                        &counter_text,
                        orrery_cx as i32,
                        orrery_cy as i32 + 200,
                        2,
                        palette::dim(palette::CLASSICAL_RED, fade * 0.9),
                    );
                }
                font::draw_text_centered_glow(
                    &mut s,
                    "THE WEIGHT OF KNOWING",
                    narr_cx,
                    20,
                    2,
                    palette::dim(palette::CLASSICAL_RED, fade),
                    palette::dim(palette::CLASSICAL_RED, fade * 0.4),
                );
                let np = narration_progress(scroll::NARRATION_FULL_EVAL.len(), 16.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_FULL_EVAL,
                    narr_cx,
                    RENDER_H as i32 - 140,
                    np,
                    fade,
                );
                commentary::render_commentary(
                    &mut s,
                    &solver,
                    scene,
                    fade,
                );
            }
            2 => {
                starfield.render(&mut s, fade * 0.12, demo_time);
                dual.render(
                    &mut s,
                    &genesis_orrery,
                    demo_time,
                    fade,
                    scene_t,
                );
                particles.render(&mut s);
                let np = narration_progress(scroll::NARRATION_DUAL.len(), 25.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_DUAL,
                    RENDER_W as i32 / 2,
                    85,
                    np,
                    fade * 0.95,
                );
            }
            3 => {
                starfield.render(&mut s, fade * 0.15, demo_time);
                let opts = OrreryRenderOpts {
                    cx: orrery_cx,
                    cy: orrery_cy,
                    fade,
                    show_labels: true,
                    show_arcs: 1.0,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: false,
                    moved_planet: None,
                    move_ring_alpha: 0.0,
                    sweep_frontier: -1.0,
                };
                effects::orrery_render::render(
                    &mut s,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );
                particles.render(&mut s);
                font::draw_text_centered_glow(
                    &mut s,
                    "UNLEASHED",
                    narr_cx,
                    20,
                    2,
                    palette::dim(palette::SERIO_BLUE, fade * 0.9),
                    palette::dim(palette::SERIO_BLUE, fade * 0.3),
                );
                score_graph.render(
                    &mut s,
                    40,
                    RENDER_H as i32 - 210,
                    fade * 0.9,
                );
                let np = narration_progress(scroll::NARRATION_CONVERGENCE.len(), 24.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_CONVERGENCE,
                    narr_cx,
                    RENDER_H as i32 - 120,
                    np,
                    fade * 0.9,
                );
                commentary::render_commentary(
                    &mut s,
                    &solver,
                    scene,
                    fade,
                );
            }
            4 => {
                starfield.render(&mut s, fade * 0.2, demo_time);
                let opts = OrreryRenderOpts {
                    cx: orrery_cx,
                    cy: orrery_cy,
                    fade,
                    show_labels: true,
                    show_arcs: 1.0,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: false,
                    moved_planet: None,
                    move_ring_alpha: 0.0,
                    sweep_frontier: -1.0,
                };
                effects::orrery_render::render(
                    &mut s,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );
                let pulse = (demo_time as f32 * 1.5).sin() * 0.1 + 0.9;
                font::draw_text_centered_glow(
                    &mut s,
                    "MUSICA UNIVERSALIS",
                    narr_cx,
                    20,
                    3,
                    palette::dim(palette::GOLD, fade * pulse),
                    palette::dim(palette::AMBER_400, fade * 0.5),
                );
                let hard = current_score.hard();
                let status = if hard >= 0 { "FEASIBLE" } else { "INFEASIBLE" };
                let status_col = if hard >= 0 {
                    palette::EMERALD_400
                } else {
                    palette::RUST
                };
                font::draw_text_centered(
                    &mut s,
                    status,
                    narr_cx,
                    RENDER_H as i32 - 160,
                    3,
                    palette::dim(status_col, fade),
                );
                let np = narration_progress(scroll::NARRATION_SOLUTION.len(), 10.0);
                scroll::render_narration_centered(
                    &mut s,
                    scroll::NARRATION_SOLUTION,
                    narr_cx,
                    RENDER_H as i32 - 120,
                    np,
                    fade * 0.9,
                );
                commentary::render_commentary(
                    &mut s,
                    &solver,
                    scene,
                    fade,
                );
            }
            _ => {
                starfield.render(&mut s, fade * 0.25, demo_time);
                plasma.render_overlay(&mut s, demo_time, fade * 0.12);
                let logo_r = 120.0f32;
                let pulse = (scene_t as f32 * 1.2).sin() * 0.1 + 0.9;
                logo::draw_logo(
                    &mut s,
                    RENDER_W as f32 / 2.0,
                    RENDER_H as f32 / 2.0 - 80.0,
                    logo_r,
                    1.0,
                    demo_time,
                    fade * pulse,
                );
                font::draw_text_centered_glow(
                    &mut s,
                    "SOLVERFORGE",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 60,
                    4,
                    palette::dim(palette::EMERALD_500, fade * pulse),
                    palette::dim(palette::EMERALD_400, fade * 0.5),
                );
                font::draw_text_centered(
                    &mut s,
                    "MUSICA UNIVERSALIS",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 95,
                    1,
                    palette::dim(palette::EMERALD_600, fade * 0.7),
                );
                font::draw_text_centered(
                    &mut s,
                    "github.com/solverforge",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 112,
                    1,
                    palette::dim(palette::EMERALD_600, fade * 0.6),
                );
                let np = narration_progress(scroll::NARRATION_OUTRO.len(), 14.0);
                scroll::render_narration(
                    &mut s,
                    scroll::NARRATION_OUTRO,
                    60,
                    80,
                    np,
                    fade * 0.8,
                );
                scroller.render(
                    &mut s,
                    demo_time,
                    RENDER_H as i32 - 50,
                    fade,
                );
                font::draw_text_centered(
                    &mut s,
                    "CODED IN RUST // BERGAMO MMXXVI // PYTHAGORAS APPROVED",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 - 20,
                    1,
                    palette::dim(palette::EMERALD_700, fade * 0.5),
                );
            }
        }



        // BGR24 output
        for (i, &px) in buffer.iter().enumerate() {
            let r = ((px >> 16) & 0xFF) as u8;
            let g = ((px >> 8) & 0xFF) as u8;
            let b = (px & 0xFF) as u8;
            bgr_frame[i * 3] = b;
            bgr_frame[i * 3 + 1] = g;
            bgr_frame[i * 3 + 2] = r;
        }
        out.write_all(&bgr_frame).expect("stdout write failed");

        if frame % (fps as usize * 5) == 0 {
            eprintln!("[render] {:.0}s / {:.0}s", demo_time, duration_secs);
        }
    }

    eprintln!("[render] Done. {} frames written.", total_frames);
}

// ── Scene routing ─────────────────────────────────────────────

fn get_scene(demo_time: f64) -> (usize, f64, f32, f32) {
    let scene_outro = SCENE_SOLUTION + SOLUTION_DURATION;
    let demo_end = scene_outro + OUTRO_DURATION;

    let scenes: &[(f64, f64)] = &[
        (SCENE_GENESIS, SCENE_FULL_EVAL), // 0: genesis
        (SCENE_FULL_EVAL, SCENE_DUAL),    // 1: full eval
        (SCENE_DUAL, SCENE_HARMONY),      // 2: dual panel
        (SCENE_HARMONY, SCENE_SOLUTION),  // 3: convergence, 35 seconds
        (SCENE_SOLUTION, scene_outro),    // 4: solution
        (scene_outro, demo_end),          // 5: outro
    ];

    for (i, &(start, end)) in scenes.iter().enumerate() {
        if demo_time >= start && demo_time < end {
            let scene_t = demo_time - start;
            let duration = end - start;
            let fade_in = ease_in_out(((scene_t / CROSSFADE) as f32).clamp(0.0, 1.0));
            let fade_out = ease_in_out((((duration - scene_t) / CROSSFADE) as f32).clamp(0.0, 1.0));
            return (i, scene_t, fade_in, fade_out);
        }
    }
    // Past the end — stay on outro
    (5, demo_time - scene_outro, 1.0, 1.0)
}

fn ease_in_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

// ── Audio ─────────────────────────────────────────────────────

fn start_audio(freq_state: audio::synth::FreqState) -> Option<cpal::Stream> {
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
    eprintln!("[audio] Starting planetary tones at {}Hz", sample_rate);

    let mut synth = audio::synth::build_audio(freq_state);
    synth.reset();
    synth.set_sample_rate(sample_rate as f64);

    let channels = config.channels() as usize;

    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
            let frames = data.len() / channels;
            for frame in 0..frames {
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
                eprintln!("[audio] Could not play: {e}");
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

fn render_audio_wav(duration: f64, path: &str) {
    const SAMPLE_RATE: u32 = 44100;
    let total_samples = (duration * SAMPLE_RATE as f64).ceil() as u64;

    eprintln!("[render] Rendering audio: {total_samples} samples @ {SAMPLE_RATE}Hz → {path}");

    let freq_state = audio::synth::initial_freq_state();
    let mut synth = audio::synth::build_audio(freq_state);
    synth.reset();
    synth.set_sample_rate(SAMPLE_RATE as f64);

    let mut pcm = Vec::with_capacity(total_samples as usize * 2);
    for _ in 0..total_samples {
        let (l, r) = synth.get_stereo();
        let l = (l.clamp(-1.0, 1.0) * 32767.0) as i16;
        let r = (r.clamp(-1.0, 1.0) * 32767.0) as i16;
        pcm.push(l);
        pcm.push(r);
    }

    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let byte_rate = SAMPLE_RATE * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = pcm.len() as u32 * 2;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for sample in &pcm {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    std::fs::write(path, &wav).expect("Failed to write audio WAV");
    eprintln!("[render] Audio written: {path} ({:.1} MB)", wav.len() as f64 / 1_048_576.0);
}
