// ═══════════════════════════════════════════════════════════════
//   ██████╗ ███████╗██████╗ ██╗ ██████╗
//  ██╔════╝ ██╔════╝██╔══██╗██║██╔═══██╗
//  ╚█████╗  █████╗  ██████╔╝██║██║   ██║
//   ╚═══██╗ ██╔══╝  ██╔══██╗██║██║   ██║
//  ██████╔╝ ███████╗██║  ██║██║╚██████╔╝
//  ╚═════╝  ╚══════╝╚═╝  ╚═╝╚═╝ ╚═════╝
//
//  MUSICA UNIVERSALIS
//  THE PLANETS SING. THE SOLVER LISTENS.
//
//  SERIO: SCORING ENGINE FOR REAL-TIME INCREMENTAL OPTIMIZATION
//  7 CLASSICAL PLANETS. PYTHAGOREAN HARMONY. ZERO-ERASURE RUST.
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

use effects::dual_panel::DualPanel;
use effects::genesis;
use effects::orrery_render::{OrreryRenderOpts, TrailBuffer};
use effects::particles::ParticleSystem;
use effects::plasma::Plasma;
use effects::score_display::{self, ScoreGraph};
use effects::scroll::ScrollText;
use effects::starfield::Starfield;

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
const SCENE_GENESIS: f64 = 0.0; // 0s  – 12s : sun ignition, planets materialize
const SCENE_FULL_EVAL: f64 = 12.0; // 12s – 27s : "classical" full re-evaluation chaos
const SCENE_DUAL: f64 = 27.0; // 27s – 52s : split screen, SERIO vs classical
const SCENE_HARMONY: f64 = 52.0; // 52s – 68s : SERIO full speed, arcs converge
const SCENE_SOLUTION: f64 = 68.0; // 68s – 78s : Musica Universalis achieved
const SCENE_OUTRO: f64 = 78.0; // 78s – 84s : greetings, credits
const DEMO_END: f64 = 84.0;

const CROSSFADE: f64 = 2.0; // seconds crossfade between scenes

fn main() {
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
        run_headless(render_duration);
        return;
    }

    eprintln!();
    eprintln!("╔═══════════════════════════════════════════════════════════╗");
    eprintln!("║   SERIO_01 -- MUSICA UNIVERSALIS                         ║");
    eprintln!("║   SCORING ENGINE FOR REAL-TIME INCREMENTAL OPTIMIZATION  ║");
    eprintln!("║   7 PLANETS. PYTHAGOREAN HARMONY. ZERO-ERASURE RUST.    ║");
    eprintln!("║   CODED IN RUST. PYTHAGORAS APPROVED. 2026.             ║");
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
        "MUSICA UNIVERSALIS - SERIO INCREMENTAL SCORING ENGINE",
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

    // ── Scene starts for SPACE-to-advance ─────────────────────
    let scene_starts: &[f64] = &[
        SCENE_GENESIS,
        SCENE_FULL_EVAL,
        SCENE_DUAL,
        SCENE_HARMONY,
        SCENE_SOLUTION,
        SCENE_OUTRO,
        DEMO_END,
    ];

    // ── Demo state ────────────────────────────────────────────
    let start_time = Instant::now();
    let mut last_time = start_time;
    let mut time_offset: f64 = 0.0;
    let mut space_was_down = false;
    let mut prev_history_len = 0usize;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f64().min(0.05);
        last_time = now;

        let elapsed = now.duration_since(start_time).as_secs_f64() + time_offset;
        let demo_time = elapsed % (DEMO_END + 5.0);

        // ── SPACE: advance to next scene ──────────────────────
        let space_down = window.is_key_down(Key::Space);
        if space_down && !space_was_down {
            for &boundary in scene_starts {
                if boundary > demo_time + 0.5 {
                    time_offset += boundary - demo_time;
                    break;
                }
            }
        }
        space_was_down = space_down;

        // ── Run solver steps ──────────────────────────────────
        // More moves per frame in later scenes for satisfying resolution
        let (scene, scene_t, _, _) = get_scene(demo_time);
        let moves_per_frame = match scene {
            0 => 0, // genesis: no moves yet
            1 => 1, // full eval demo: slow deliberate moves
            2 => 2, // dual panel: 2 moves/frame
            3 => 8, // harmony: accelerating
            4 => 0, // solution: stopped
            _ => 0,
        };

        if moves_per_frame > 0 {
            solver.step(moves_per_frame);
        }

        // ── Process new moves for visualization ───────────────
        let new_history_len = solver.history.len();
        if new_history_len > prev_history_len {
            let new_moves = &solver.history[prev_history_len..new_history_len];
            for mv in new_moves {
                // Dual panel: notify of moves
                dual.on_move(mv.planet_idx);

                // Particles
                let _planet = &solver.solution().planets[mv.planet_idx];
                particles.spawn_move(
                    solver.solution(),
                    mv.planet_idx,
                    640.0, // single-panel center (for non-dual scenes)
                    360.0,
                    mv.soft_delta(),
                    mv.accepted,
                );

                // Score graph
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

        // ── Update audio frequencies ──────────────────────────
        let radii: [f64; 7] = {
            let planets = &solver.solution().planets;
            [
                planets[0].orbital_radius(),
                planets[1].orbital_radius(),
                planets[2].orbital_radius(),
                planets[3].orbital_radius(),
                planets[4].orbital_radius(),
                planets[5].orbital_radius(),
                planets[6].orbital_radius(),
            ]
        };
        audio::synth::update_frequencies(&freq_state, &radii);

        // ── Clear buffer ──────────────────────────────────────
        let bg = palette::NEAR_BLACK;
        for px in buffer.iter_mut() {
            *px = bg;
        }

        // ── Render scene ──────────────────────────────────────
        let elapsed_for_mps = now.duration_since(start_time).as_secs_f64();
        let mps = solver.moves_per_sec(elapsed_for_mps);
        let current_score = solver.current_score();

        match scene {
            0 => {
                // GENESIS: space background + planet materialization
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.3, demo_time);
                genesis::render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    &mut trail,
                    scene_t,
                    fade,
                    demo_time,
                );
            }

            1 => {
                // FULL EVAL: single panel, classical all-flash
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.15, demo_time);

                // Force full flash on all arcs
                let flash = if solver.history.len() > prev_history_len.saturating_sub(3) {
                    0.8
                } else {
                    0.0
                };
                let opts = OrreryRenderOpts {
                    cx: 640.0,
                    cy: 360.0,
                    fade,
                    show_labels: false,
                    show_arcs: true,
                    highlighted_pairs: None,
                    flash_all: true,
                    flash_alpha: flash,
                    show_rings: true,
                };
                effects::orrery_render::render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );

                particles.render(&mut buffer, RENDER_W, RENDER_H);

                // Labels
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "CLASSICAL FULL EVALUATION",
                    640,
                    30,
                    2,
                    palette::dim(palette::CLASSICAL_RED, fade),
                    palette::dim(palette::CLASSICAL_RED, fade * 0.4),
                );
                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "O(n^2) PER MOVE -- ALL 21 PAIR ARCS FLASH",
                    640,
                    54,
                    1,
                    palette::dim(palette::CHROME, fade * 0.6),
                );

                score_display::render_score_hud(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    solver.total_moves,
                    mps,
                    fade,
                );
            }

            2 => {
                // DUAL PANEL: split screen comparison
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.12, demo_time);
                dual.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    demo_time,
                    fade,
                );
                particles.render(&mut buffer, RENDER_W, RENDER_H);
                score_display::render_score_hud(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    solver.total_moves,
                    mps,
                    fade * 0.7,
                );
            }

            3 => {
                // HARMONY: full speed SERIO, arcs converge
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.15, demo_time);

                let opts = OrreryRenderOpts {
                    cx: 580.0,
                    cy: 360.0,
                    fade,
                    show_labels: true,
                    show_arcs: true,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: false,
                };
                effects::orrery_render::render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );

                particles.render(&mut buffer, RENDER_W, RENDER_H);

                // Score graph in bottom-right
                score_graph.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    (RENDER_W - 420) as i32,
                    (RENDER_H - 100) as i32,
                    fade * 0.9,
                );

                score_display::render_score_hud(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    solver.total_moves,
                    mps,
                    fade,
                );

                score_display::render_constraint_bars(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    worst_soft.abs(),
                    fade,
                    RENDER_W as i32 - 350,
                    RENDER_H as i32 - 160,
                );

                // "HARMONY EMERGES" label fades in at scene midpoint
                if scene_t > 8.0 {
                    let h_alpha = ((scene_t - 8.0) / 4.0).clamp(0.0, 1.0) as f32 * fade;
                    font::draw_text_centered_glow(
                        &mut buffer,
                        RENDER_W,
                        RENDER_H,
                        "HARMONY EMERGES",
                        640,
                        RENDER_H as i32 / 2 - 200,
                        3,
                        palette::dim(palette::GOLD, h_alpha * 0.8),
                        palette::dim(palette::AMBER_400, h_alpha * 0.3),
                    );
                }
            }

            4 => {
                // SOLUTION: Musica Universalis achieved
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.2, demo_time);

                let opts = OrreryRenderOpts {
                    cx: 580.0,
                    cy: 370.0,
                    fade,
                    show_labels: true,
                    show_arcs: true,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: false,
                };
                effects::orrery_render::render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );

                // "MUSICA UNIVERSALIS" title
                let pulse = (demo_time as f32 * 1.5).sin() * 0.1 + 0.9;
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "MUSICA UNIVERSALIS",
                    640,
                    40,
                    3,
                    palette::dim(palette::GOLD, fade * pulse),
                    palette::dim(palette::AMBER_400, fade * 0.5),
                );

                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "Harmony of the Cosmos -- Pythagoras, 530 BCE",
                    640,
                    72,
                    1,
                    palette::dim(palette::CHROME, fade * 0.5),
                );

                // Final score
                score_display::render_score_hud(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    solver.total_moves,
                    mps,
                    fade,
                );

                // Constraint satisfaction summary
                let hard = current_score.hard();
                let _soft = current_score.soft();
                let status = if hard >= 0 { "FEASIBLE" } else { "INFEASIBLE" };
                let status_col = if hard >= 0 {
                    palette::EMERALD_400
                } else {
                    palette::RUST
                };
                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    status,
                    640,
                    RENDER_H as i32 - 60,
                    3,
                    palette::dim(status_col, fade),
                );
            }

            _ => {
                // OUTRO: greetings + credits
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.25, demo_time);
                plasma.render_overlay(&mut buffer, RENDER_W, RENDER_H, demo_time, fade * 0.12);

                // Large logo centered
                let logo_r = 120.0f32;
                let pulse = (scene_t as f32 * 1.2).sin() * 0.1 + 0.9;
                logo::draw_logo(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    RENDER_W as f32 / 2.0,
                    RENDER_H as f32 / 2.0 - 60.0,
                    logo_r,
                    1.0,
                    demo_time,
                    fade * pulse,
                );

                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "SOLVERFORGE",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 80,
                    4,
                    palette::dim(palette::EMERALD_500, fade * pulse),
                    palette::dim(palette::EMERALD_400, fade * 0.5),
                );

                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "SERIO -- SCORING ENGINE FOR REAL-TIME INCREMENTAL OPTIMIZATION",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 115,
                    1,
                    palette::dim(palette::EMERALD_600, fade * 0.7),
                );

                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "github.com/solverforge",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 132,
                    1,
                    palette::dim(palette::EMERALD_600, fade * 0.6),
                );

                // Scrolltext at bottom
                scroller.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    demo_time,
                    RENDER_H as i32 - 50,
                    fade,
                );

                font::draw_text_centered(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "CODED IN RUST // PYTHAGORAS APPROVED // NO JAVASCRIPT WAS HARMED",
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
            &mut buffer,
            RENDER_W,
            RENDER_H,
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
    let mut trail = TrailBuffer::new(RENDER_W, RENDER_H);
    let mut dual = DualPanel::new();
    let mut particles = ParticleSystem::new();
    let mut scroller = ScrollText::new(2, 280.0, RENDER_W);
    let mut score_graph = ScoreGraph::new();
    let plasma = Plasma::new();
    let mut starfield = Starfield::new();
    let mut worst_soft: i64 = 1;
    let mut prev_history_len = 0usize;

    let dt = 1.0 / fps;

    for frame in 0..total_frames {
        let demo_time = frame as f64 * dt;
        let (scene, scene_t, fade_in, fade_out) = get_scene(demo_time);
        let fade = fade_in * fade_out;

        // Solver steps per frame
        let moves_per_frame = match scene {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 8,
            _ => 0,
        };
        if moves_per_frame > 0 {
            solver.step(moves_per_frame);
        }

        // Process new moves
        let new_len = solver.history.len();
        if new_len > prev_history_len {
            for mv in &solver.history[prev_history_len..new_len] {
                dual.on_move(mv.planet_idx);
                particles.spawn_move(
                    solver.solution(),
                    mv.planet_idx,
                    640.0,
                    360.0,
                    mv.soft_delta(),
                    mv.accepted,
                );
                score_graph.push(mv.score_after);
                let s = mv.score_after.soft();
                if s < worst_soft {
                    worst_soft = s;
                }
            }
            prev_history_len = new_len;
        }

        dual.update(dt);
        particles.update(dt);
        scroller.update(dt);
        starfield.update(dt, 0.1);

        // Clear
        for px in buffer.iter_mut() {
            *px = palette::NEAR_BLACK;
        }

        let current_score = solver.current_score();
        let mps = solver.moves_per_sec(demo_time.max(0.001));

        match scene {
            0 => {
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.3, demo_time);
                genesis::render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    &mut trail,
                    scene_t,
                    fade,
                    demo_time,
                );
            }
            1 => {
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.15, demo_time);
                let opts = OrreryRenderOpts {
                    cx: 640.0,
                    cy: 360.0,
                    fade,
                    show_labels: false,
                    show_arcs: true,
                    highlighted_pairs: None,
                    flash_all: true,
                    flash_alpha: 0.6,
                    show_rings: true,
                };
                effects::orrery_render::render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );
                particles.render(&mut buffer, RENDER_W, RENDER_H);
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "CLASSICAL FULL EVALUATION",
                    640,
                    30,
                    2,
                    palette::dim(palette::CLASSICAL_RED, fade),
                    palette::dim(palette::CLASSICAL_RED, fade * 0.4),
                );
                score_display::render_score_hud(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    solver.total_moves,
                    mps,
                    fade,
                );
            }
            2 => {
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.12, demo_time);
                dual.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    demo_time,
                    fade,
                );
                particles.render(&mut buffer, RENDER_W, RENDER_H);
                score_display::render_score_hud(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    solver.total_moves,
                    mps,
                    fade * 0.7,
                );
            }
            3 => {
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.15, demo_time);
                let opts = OrreryRenderOpts {
                    cx: 580.0,
                    cy: 360.0,
                    fade,
                    show_labels: true,
                    show_arcs: true,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: false,
                };
                effects::orrery_render::render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );
                particles.render(&mut buffer, RENDER_W, RENDER_H);
                score_graph.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    (RENDER_W - 420) as i32,
                    (RENDER_H - 100) as i32,
                    fade * 0.9,
                );
                score_display::render_score_hud(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    solver.total_moves,
                    mps,
                    fade,
                );
                score_display::render_constraint_bars(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    worst_soft.abs(),
                    fade,
                    RENDER_W as i32 - 350,
                    RENDER_H as i32 - 160,
                );
            }
            4 => {
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.2, demo_time);
                let opts = OrreryRenderOpts {
                    cx: 580.0,
                    cy: 370.0,
                    fade,
                    show_labels: true,
                    show_arcs: true,
                    highlighted_pairs: None,
                    flash_all: false,
                    flash_alpha: 0.0,
                    show_rings: false,
                };
                effects::orrery_render::render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    solver.solution(),
                    &mut trail,
                    &opts,
                    demo_time,
                );
                let pulse = (demo_time as f32 * 1.5).sin() * 0.1 + 0.9;
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "MUSICA UNIVERSALIS",
                    640,
                    40,
                    3,
                    palette::dim(palette::GOLD, fade * pulse),
                    palette::dim(palette::AMBER_400, fade * 0.5),
                );
                score_display::render_score_hud(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    current_score,
                    solver.total_moves,
                    mps,
                    fade,
                );
            }
            _ => {
                starfield.render(&mut buffer, RENDER_W, RENDER_H, fade * 0.25, demo_time);
                plasma.render_overlay(&mut buffer, RENDER_W, RENDER_H, demo_time, fade * 0.12);
                let logo_r = 120.0f32;
                let pulse = (scene_t as f32 * 1.2).sin() * 0.1 + 0.9;
                logo::draw_logo(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    RENDER_W as f32 / 2.0,
                    RENDER_H as f32 / 2.0 - 60.0,
                    logo_r,
                    1.0,
                    demo_time,
                    fade * pulse,
                );
                font::draw_text_centered_glow(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    "SOLVERFORGE",
                    RENDER_W as i32 / 2,
                    RENDER_H as i32 / 2 + 80,
                    4,
                    palette::dim(palette::EMERALD_500, fade * pulse),
                    palette::dim(palette::EMERALD_400, fade * 0.5),
                );
                scroller.render(
                    &mut buffer,
                    RENDER_W,
                    RENDER_H,
                    demo_time,
                    RENDER_H as i32 - 50,
                    fade,
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
    let scenes: &[(f64, f64)] = &[
        (SCENE_GENESIS, SCENE_FULL_EVAL),
        (SCENE_FULL_EVAL, SCENE_DUAL),
        (SCENE_DUAL, SCENE_HARMONY),
        (SCENE_HARMONY, SCENE_SOLUTION),
        (SCENE_SOLUTION, SCENE_OUTRO),
        (SCENE_OUTRO, DEMO_END),
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
    (5, demo_time - SCENE_OUTRO, 1.0, 1.0)
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
