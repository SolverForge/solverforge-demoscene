```
  ███████╗███████╗██████╗ ██╗ ██████╗      ██████╗ ██████╗
  ██╔════╝██╔════╝██╔══██╗██║██╔═══██╗    ██╔═══██╗╚════██╗
  ███████╗█████╗  ██████╔╝██║██║   ██║    ██║   ██║ █████╔╝
  ╚════██║██╔══╝  ██╔══██╗██║██║   ██║    ██║   ██║██╔═══╝
  ███████║███████╗██║  ██║██║╚██████╔╝    ╚██████╔╝███████╗
  ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝ ╚═════╝      ╚═════╝ ╚══════╝
          ── MUSICA UNIVERSALIS // GREETINGS FROM THE RUST DEMOSCENE ──
```

A 166-second demoscene intro for [SolverForge](https://solverforge.com) and SERIO (Scoring Engine for Real-time Incremental Optimization) — the incremental constraint-scoring engine that powers it.

**SERIO: Scoring Engine for Real-time Incremental Optimization.**

Pure Rust. Software-rendered. Every pixel plotted by hand into a `Vec<u32>` framebuffer. Every note synthesized in real time. Seven planets. Twenty-one harmonic intervals. One proof: *incremental beats classical every time.*

**BERGAMO MMXXVI. THIS IS NOT A SOLVER. THIS IS A WEAPON.**

---

## What Is This

SERIO_02 is a demoscene production and a live benchmark. The [SERIO scoring engine](https://crates.io/crates/solverforge-scoring) evaluates constraint satisfaction incrementally — when one planet's orbit changes, only the arcs connected to it are re-evaluated. Not the whole system. Just the delta.

Classical solvers re-evaluate everything on every change. SERIO re-evaluates only what changed. This demo makes that difference *visible* — as an orrery, as a split-screen race, as a score graph converging in real time.

**PYTHAGORAS APPROVED.**

---

## The story

The demo explains itself on cards: one or two plain sentences per moment, timed to what is on screen (`src/story.rs`). Every card stays up long enough to read at a slow pace (entrance, 0.8 s plus 0.25 s a word, exit); demofx refuses a shorter window, and the scenes are as long as their cards need. A glyphfx effect brings the words in, they hold still to be read, then burn out letter by letter over a soft scrim.

- Seven planets each sing one note, a pitch in Hz shown under the planet's name. The ratio of a pair is one pitch divided by the other: Venus at 165 Hz over Luna at 110 Hz is 1.5, a pure 3:2. A pair is in tune when its ratio is within 1.5% of a pure interval.
- The puzzle: give every planet a note so that every pair is in tune, with no note used twice. The orbits and the rotation are only the stage: a planet's position never changes the score. The one rule that uses the orbits is a weight: pairs on next-door orbits count three times, next-but-one twice.
- SolverForge changes one note at a time and scores the result. The simple way re-checks all 21 pairs after every change.
- Only the 6 pairs that touch the changed planet can change. SERIO, SolverForge's scoring engine, re-checks only those 6 and keeps the other 15 scores. Same score, 3.5 times less work here; with 100 planets it would be 50 times less.
- SolverForge solves it for real, first slowed down so each change can be followed, then at full speed: hundreds of thousands of changes a second, each scored by SERIO.
- The solution scene shows the best tuning found: hard score 0 (no shared notes), soft score 0 (every pair in tune).

## The panel

The right-side panel shows the numbers behind the picture, read from the orrery on screen with the same functions the SERIO constraints use, so it never disagrees with the arcs:

- **Notes:** each planet's pitch in Hz.
- **Pairs:** all 21 pairs as high / low pitch, halved until below 2 (octaves folded), the nearest pure interval, the orbit weight (x3 next-door, x2 one apart) and OK / OFF. The pairs touching the planet that just changed are marked; during the re-check sweep the pair being checked is lit and the unchecked ones are dimmed.
- **Score:** hard (pairs on the same note) and soft (weighted distance from pure), and how many pairs are in tune.
- **Work:** in the re-check scene, notes changed and pairs re-checked; in the solver scenes, changes tried and kept, the last change in slow motion (old and new Hz, kept or undone), changes per second at full speed, and pair checks done by SERIO against re-checking all 21.

## Scenes

| # | Scene | Time | What you see |
|---|---|---|---|
| 1 | Genesis | 0 – 26s | Sun ignition, seven planets materializing from the void; each planet's note in Hz under its name; what a pitch ratio is, on a real pair. |
| 2 | Re-check everything | 26 – 66s | The puzzle and its one orbit rule; one planet moves a quarter tone off its note (or back) every 4 s and all 21 pairs are re-checked one by one, with a counter; the panel lights each pair as it is checked. |
| 3 | Classical vs SERIO | 66 – 95s | Split screen: classical re-checks 21 pairs per change, SERIO 6. Work counters, same answer. |
| 4 | SolverForge solves it | 95 – 137s | The real search from seven clashing notes: 17 s in slow motion (one change at a time), then full speed. Score graph, live telemetry. |
| 5 | Solution | 137 – 152s | The best tuning found, every pair in tune. |
| 6 | Outro | 152 – 166s | Logo, the closing cards, greetings scroller. |

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
make render       # render the full demo to MP4 with audio (requires ffmpeg)
make render-gif   # render a shareable GIF (640px, 15fps, palette-quantized)
```

Or manually:

```bash
cargo run --release -- --render 167 | \
  ffmpeg -f rawvideo -pixel_format bgr24 \
    -video_size 1280x720 -framerate 60 -i - \
    -i demo_audio.wav \
    -c:v libx264 -c:a aac -b:a 192k \
    -crf 18 -preset slow -pix_fmt yuv420p -shortest \
    serio_02.mp4
```

The `--render` flag accepts an optional duration in seconds (defaults to 170).

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
│   ├── commentary.rs        The right-side panel: notes in Hz, all 21 pairs
│   │                        (ratio, pure interval, weight, in tune), score, work
│   ├── plasma.rs            Classic multi-sine plasma with precomputed LUT
│   ├── starfield.rs         3D starfield with 800 stars, warp streaks
│   │                        Deterministic PRNG (no rand crate)
│   ├── particles.rs         General-purpose particle system
│   │                        Used for planet genesis and explosion effects
│   └── scroll.rs            Horizontal scrolltext with glow effect
│                            Per-character gradient cycling
├── story.rs                 The story cards: text, timing, glyphfx effect, placement
├── orrery/
│   ├── model.rs             Orrery world model (Orrery struct)
│   │                        Seven planets, Pythagorean interval definitions
│   └── solver.rs            Hill climbing on SERIO's incremental score director
│                            Slow-motion steps, timed steps, best tuning kept
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
- **Deterministic where it can be.** The solver is seeded and its slow-motion phase is frame-locked; the full-speed phase runs on a 14 ms wall-clock budget per frame, so its move counters vary with the machine.

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| [minifb](https://crates.io/crates/minifb) | 0.28 | Pixel framebuffer windowing |
| [fundsp](https://crates.io/crates/fundsp) | 0.23 | Audio DSP graph (oscillators, filters, effects) |
| [cpal](https://crates.io/crates/cpal) | 0.17 | Cross-platform audio output (ALSA/PulseAudio/CoreAudio/WASAPI) |
| [solverforge-scoring](https://crates.io/crates/solverforge-scoring) | 0.5.8 | SERIO incremental constraint scoring engine |
| [solverforge-core](https://crates.io/crates/solverforge-core) | 0.5.8 | Core types, world model, arc definitions |
| `demofx` (`../vendor/demofx`) | path | The story cards, on the repository's one vendored copy of [glyphfx](https://github.com/blackopsrepl/glyphfx) (needs a C compiler) |

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
