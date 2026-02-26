```
 ██████╗ ██╗      █████╗ ███╗   ██╗███╗   ██╗███████╗██████╗
 ██╔══██╗██║     ██╔══██╗████╗  ██║████╗  ██║██╔════╝██╔══██╗
 ██████╔╝██║     ███████║██╔██╗ ██║██╔██╗ ██║█████╗  ██████╔╝
 ██╔═══╝ ██║     ██╔══██║██║╚██╗██║██║╚██╗██║██╔══╝  ██╔══██╗
 ██║     ███████╗██║  ██║██║ ╚████║██║ ╚████║███████╗██║  ██║
 ╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝
          1 2 3  ── GREETINGS FROM THE RUST DEMOSCENE ──
```

An 84-second demoscene intro for [Planner123](https://planner123.com) -- a constraint-based personal task scheduler built on the SolverForge optimization engine.

Pure Rust. Software-rendered. Every pixel plotted by hand into a `Vec<u32>` framebuffer. Every note synthesized in real time. No GPU. No shaders. No external assets. One binary.

**AMIGA 1992 FOREVER.**

---

## Scenes

| # | Scene | Time | Effect |
|---|---|---|---|
| 1 | Logo Reveal | 0 - 10s | Particle system + scanline sweep revealing the SolverForge ouroboros |
| 2 | Plasma + Copper | 10 - 20s | Classic multi-sine plasma with Amiga copper bar raster effects |
| 3 | Starfield Warp | 20 - 32s | 800-star 3D starfield with depth projection and tunnel rings |
| 4 | Screenshots + Wireframe | 32 - 63s | Embedded Planner123 screenshots interleaved with rotating 3D wireframe objects (cube, icosahedron, torus) |
| 5 | Greetings Scroll | 63 - 73s | Horizontal scrolltext with per-character emerald-amber-rust gradient cycling |
| 6 | Outro | 73 - 84s | Composite finale, then loop |

All scenes crossfade with 2.5-second transitions. The demo loops continuously.

## Audio

A full procedural synthwave track at 125 BPM, generated in real time by [fundsp](https://crates.io/crates/fundsp):

- **Drums:** Kick, snare, hi-hat, open-hat with industrial patterns and rock overbeat
- **Bass:** Square-wave with sidechain pumping
- **Lead:** Saw-wave with multi-tap echo delay
- **Arp:** Arpeggiated chord sequences
- **Pads:** Detuned-saw chord washes
- **Solo:** Multi-phrase melody with vibrato, tremolo, and psychedelic plate reverb
- **Mix:** Sidechain compression, multi-stage bus saturation, brick-wall limiting

Four arrangement phases: drums-only intro, groove (full band), bridge, and solo with hard rock overbeat.

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

The demo has a headless mode (`--render`) that writes raw BGR24 frames to stdout. Pipe to ffmpeg for encoding.

```bash
make render       # render full 84s demo to MP4 (requires ffmpeg)
make render-gif   # render a shareable GIF (640px, 15fps, palette-quantized)
```

Or manually:

```bash
cargo run --release -- --render 84 | \
  ffmpeg -f rawvideo -pixel_format bgr24 \
    -video_size 1280x720 -framerate 60 -i - \
    -c:v libx264 -crf 18 -preset slow -pix_fmt yuv420p \
    demo.mp4
```

The `--render` flag accepts an optional duration in seconds (defaults to 84).

## Architecture

```
src/
├── main.rs              1083 lines  Entry point, demo timeline, scene routing, audio init
│                                    Parses --render flag, manages minifb window, crossfades
├── audio/
│   └── synth.rs          937 lines  Procedural synthwave engine
│                                    125 BPM, 4 phases, drums/bass/lead/arp/pads/solo
│                                    Sidechain compression, plate reverb, bus saturation
├── effects/
│   ├── logo_reveal.rs    333 lines  Logo entrance: particle system, scanline sweep,
│   │                                progressive draw, text reveal, Swiss-design brackets
│   ├── plasma.rs         132 lines  Classic multi-sine plasma with precomputed LUT
│   │                                Full-screen and overlay modes
│   ├── copper.rs         150 lines  Amiga copper bar emulation
│   │                                Modes: Classic (flying), Bands (scrolling), Scanlines (CRT)
│   ├── starfield.rs      176 lines  3D starfield with 800 stars, warp streaks, tunnel rings
│   │                                Deterministic PRNG (no rand crate)
│   ├── wireframe.rs      464 lines  Software 3D wireframe renderer using glam
│   │                                Cube, icosahedron, torus. Perspective projection,
│   │                                phosphor trails, Swiss-design grid overlay
│   ├── screenshots.rs    236 lines  Compile-time PNG embeds via include_bytes!()
│   │                                Bilinear interpolation blit with fade/blend
│   ├── scroll.rs         180 lines  Horizontal scrolltext with glow effect
│   │                                Per-character gradient cycling (emerald/amber/rust)
│   └── screenshots/
│       ├── plan.png               Planner123 Plan view
│       ├── gantt.png              Planner123 Gantt view
│       └── calendar.png          Planner123 Calendar view
├── font.rs               323 lines  8x8 bitmap font covering ASCII 32-127
│                                    draw_char, draw_text, draw_text_centered, glow variants
├── logo.rs               497 lines  Procedural SolverForge ouroboros logo
│                                    8-point polygon, forked snake head, center crosshair,
│                                    corner brackets, midpoint squares (from SVG coordinates)
└── palette.rs            183 lines  Color system: SolverForge brand emerald (300-800),
                                     neutrals, synthwave accents (cyan/purple/magenta/amber),
                                     copper/plasma/scroll palettes, RGB pack/lerp/dim/add/fade
                         ─────
                         4702 lines  TOTAL
```

### Design Principles

- **Zero runtime I/O.** Screenshots are `include_bytes!()`. The font is a hardcoded byte array. The logo is procedural geometry. Audio is real-time synthesis. Nothing is loaded from disk at runtime.
- **Pure software rendering.** Every pixel is written to a `Vec<u32>` framebuffer. Bresenham lines, scanline triangle fill, manual perspective projection, manual bilinear interpolation. No OpenGL, no Vulkan, no wgpu.
- **Deterministic.** The same binary produces the same visual output every run (audio timing may vary slightly due to OS scheduling). The headless render mode produces bit-identical frames.

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| [minifb](https://crates.io/crates/minifb) | 0.28 | Pixel framebuffer windowing |
| [fundsp](https://crates.io/crates/fundsp) | 0.23 | Audio DSP graph (oscillators, filters, effects) |
| [cpal](https://crates.io/crates/cpal) | 0.17 | Cross-platform audio output (ALSA/PulseAudio/CoreAudio/WASAPI) |
| [noise](https://crates.io/crates/noise) | 0.9 | Procedural noise generation |
| [glam](https://crates.io/crates/glam) | 0.29 | SIMD vector/matrix math for 3D transforms |
| [png](https://crates.io/crates/png) | 0.17 | PNG decoding for embedded screenshots |

## Makefile Targets

```
make help         Show available targets (default)
make build        Build release binary
make run          Build and run the demo
make render       Render to MP4 via ffmpeg
make render-gif   Render to GIF (shareable)
make clean        Remove build artifacts + output
make check        Run cargo check
make clippy       Run clippy lints
make loc          Count lines of Rust source
```

## Resolution

Internal render resolution is 1280x720 at 60fps. The minifb window scales to fit. Headless render outputs at native resolution.

## License

MIT -- see [LICENSE](../LICENSE).

**CODED IN RUST // NO JAVASCRIPT WAS HARMED // AMIGA 1992 FOREVER**
