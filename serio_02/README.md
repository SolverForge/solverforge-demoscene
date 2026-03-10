```
  ███████╗███████╗██████╗ ██╗ ██████╗      ██████╗ ██████╗
  ██╔════╝██╔════╝██╔══██╗██║██╔═══██╗    ██╔═══██╗╚════██╗
  ███████╗█████╗  ██████╔╝██║██║   ██║    ██║   ██║ █████╔╝
  ╚════██║██╔══╝  ██╔══██╗██║██║   ██║    ██║   ██║██╔═══╝
  ███████║███████╗██║  ██║██║╚██████╔╝    ╚██████╔╝███████╗
  ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝ ╚═════╝      ╚═════╝ ╚══════╝
          ── MUSICA UNIVERSALIS // GREETINGS FROM THE RUST DEMOSCENE ──
```

A 96-second demoscene intro for [SolverForge](https://solverforge.com) and SERIO (Scoring Engine for Real-time Incremental Optimization) — the incremental constraint-scoring engine that powers it.

**SERIO: Scoring Engine for Real-time Incremental Optimization.**

Pure Rust. Software-rendered. Every pixel plotted by hand into a `Vec<u32>` framebuffer. Every note synthesized in real time. Seven planets. Twenty-one harmonic intervals. One proof: *incremental beats classical every time.*

**BERGAMO MMXXVI. THIS IS NOT A SOLVER. THIS IS A WEAPON.**

---

## What Is This

SERIO_02 is a demoscene production and a live benchmark. The [SERIO scoring engine](https://crates.io/crates/solverforge-scoring) evaluates constraint satisfaction incrementally — when one planet's orbit changes, only the arcs connected to it are re-evaluated. Not the whole system. Just the delta.

Classical solvers re-evaluate everything on every change. SERIO re-evaluates only what changed. This demo makes that difference *visible* — as an orrery, as a split-screen race, as a score graph converging in real time.

**PYTHAGORAS APPROVED.**

---

## Scenes

| # | Scene | Time | Effect |
|---|---|---|---|
| 1 | Genesis | 0 – 14s | Sun ignition, seven planets materializing from the void. Particle system + starfield. |
| 2 | Full Evaluation | 14 – 30s | Classical solver: every arc re-evaluated on every tick. Commentary overlay. Orrery rotates. |
| 3 | SERIO vs Classical | 30 – 55s | Split-screen dual panel. SERIO left, classical right. Score graph diverges. The gap widens. |
| 4 | Harmony | 55 – 90s | SERIO at full speed. 35 seconds of pure incremental convergence. Pythagorean harmony achieved. |
| 5 | Solution | 90 – 100s | Admire the solved orrery. Every interval locked. Every voice in tune. |
| 6 | Outro | 100 – 114s | Composite finale with scroll text. Greetings roll. Then loop. |

All scenes crossfade over 2.0 seconds. The demo loops continuously in windowed mode.

## Audio

A full procedural synthwave track generated in real time by [fundsp](https://crates.io/crates/fundsp):

- **Drums:** Kick, snare, hi-hat, open-hat with industrial patterns
- **Bass:** Square-wave with sidechain pumping synchronized to the orrery
- **Lead:** Saw-wave with multi-tap echo delay
- **Arp:** Arpeggiated Pythagorean chord sequences (tuned to just intonation)
- **Pads:** Detuned-saw chord washes tracking harmonic state
- **Mix:** Sidechain compression, bus saturation, brick-wall limiting

The audio follows the demo's arc: sparse at genesis, dense at harmony, triumphant at solution.

## Running

```bash
make run          # build and launch (opens a window, plays audio)

# or directly:
cargo run --release
```

### Controls

| Key | Action |
|---|---|
| `ESC` | Quit |
| `SPACE` | Advance to next scene |

## Rendering to Video

The demo has a headless mode (`--render`) that writes raw BGR24 frames to stdout and a WAV file to disk. Pipe to ffmpeg for encoding.

```bash
make render       # render full 96s demo to MP4 with audio (requires ffmpeg)
make render-gif   # render a shareable GIF (640px, 15fps, palette-quantized)
```

Or manually:

```bash
cargo run --release -- --render 96 | \
  ffmpeg -f rawvideo -pixel_format bgr24 \
    -video_size 1280x720 -framerate 60 -i - \
    -i demo_audio.wav \
    -c:v libx264 -c:a aac -b:a 192k \
    -crf 18 -preset slow -pix_fmt yuv420p -shortest \
    serio_02.mp4
```

The `--render` flag accepts an optional duration in seconds (defaults to 120).

## Architecture

```
src/
├── main.rs                  Entry point, demo timeline, scene routing, audio init
│                            Parses --render flag, manages minifb window, crossfades
├── audio/
│   └── mod.rs               Procedural synthwave engine
│                            Sidechain compression, plate reverb, bus saturation
├── effects/
│   ├── genesis.rs           Sun ignition + planet materialization
│   │                        Particle burst, radial sweep, emergence from void
│   ├── orrery_render.rs     Software 3D orrery renderer
│   │                        Orbital mechanics, phosphor trails, perspective projection
│   │                        TrailBuffer, OrreryRenderOpts, planet color palettes
│   ├── dual_panel.rs        Split-screen SERIO vs Classical comparison
│   │                        Two independent orrery instances, score divergence graph
│   ├── score_display.rs     Live score graph (ScoreGraph)
│   │                        Plots cumulative constraint satisfaction over time
│   ├── commentary.rs        Overlay text commentary system
│   │                        Timed captions with fade-in/fade-out
│   ├── plasma.rs            Classic multi-sine plasma with precomputed LUT
│   ├── starfield.rs         3D starfield with 800 stars, warp streaks
│   │                        Deterministic PRNG (no rand crate)
│   ├── particles.rs         General-purpose particle system
│   │                        Used for planet genesis and explosion effects
│   └── scroll.rs            Horizontal scrolltext with glow effect
│                            Per-character gradient cycling
├── orrery/
│   ├── model.rs             Orrery world model (Orrery struct)
│   │                        Seven planets, Pythagorean interval definitions
│   └── solver.rs            SolverState wrapping SERIO incremental engine
│                            Drives solverforge-scoring, tracks delta evaluations
├── font.rs                  8x8 bitmap font covering ASCII 32-127
│                            draw_char, draw_text, draw_text_centered, glow variants
├── logo.rs                  Procedural SolverForge ouroboros logo
│                            8-point polygon, forked snake head, center crosshair
└── palette.rs               Color system: SolverForge brand emerald (300-800),
                             neutrals, synthwave accents, orrery planet palettes,
                             RGB pack/lerp/dim/add/fade
```

### Design Principles

- **Zero runtime I/O.** The font is a hardcoded byte array. The logo is procedural geometry. Audio is real-time synthesis. Nothing is loaded from disk at runtime.
- **Pure software rendering.** Every pixel is written to a `Vec<u32>` framebuffer. Bresenham lines, manual perspective projection, manual orbital mechanics. No OpenGL, no Vulkan, no wgpu.
- **The demo IS the benchmark.** The orrery solver running on screen is the actual SERIO engine — `solverforge-scoring` and `solverforge-core` crates — not a simulation of it.
- **Deterministic.** The same binary produces the same visual output every run. The headless render mode produces bit-identical frames.

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| [minifb](https://crates.io/crates/minifb) | 0.28 | Pixel framebuffer windowing |
| [fundsp](https://crates.io/crates/fundsp) | 0.23 | Audio DSP graph (oscillators, filters, effects) |
| [cpal](https://crates.io/crates/cpal) | 0.17 | Cross-platform audio output (ALSA/PulseAudio/CoreAudio/WASAPI) |
| [solverforge-scoring](https://crates.io/crates/solverforge-scoring) | 0.5.8 | SERIO incremental constraint scoring engine |
| [solverforge-core](https://crates.io/crates/solverforge-core) | 0.5.8 | Core types, world model, arc definitions |

## Makefile Targets

```
make help         Show available targets (default)
make build        Build release binary
make run          Build and run the demo
make render       Render to MP4 with audio via ffmpeg
make render-gif   Render to GIF (shareable)
make clean        Remove build artifacts + output
make check        Run cargo check
make clippy       Run clippy lints
make loc          Count lines of Rust source
```

## Resolution

Internal render resolution is 1280x720 at 60fps. The minifb window scales to fit. Headless render outputs at native resolution.

## License

MIT — see [LICENSE](../LICENSE).

**CODED IN RUST // SEVEN WORLDS // BERGAMO MMXXVI // AMIGA FOREVER**
