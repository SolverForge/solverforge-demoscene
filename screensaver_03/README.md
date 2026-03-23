```
 ███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗  ██████╗ ███████╗
 ██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝
 ███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝█████╗
 ╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██╔══╝
 ███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║███████╗
 ╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝╚══════╝
            ── SCREENSAVER_03 // SWAY PHOSPHOR RITUAL ──
```

GREETINGS FROM THE RUST DEMOCENE!

This is a silent SolverForge screensaver for Sway.

Pure Rust rendering, presented as a real Wayland client. The art is still software-generated, but the windowing path is now native to Sway: fullscreen surfaces, stable app id, and direct `swayidle` launch.

---

## What Is This

`screensaver_03` is a more utility oriented branch of the SolverForge demoscene series: less intro, more terminal-lit threat display. It is now explicitly scoped as a `sway` screensaver, backed by a simple Rust binary plus `swayidle` integration. It also supports a headless render mode for MP4 or GIF capture - which is what we generally use to render the videos we post on socials.

The mood is deliberate: black phosphor, emerald telemetry, drifting watch panels, low-frequency glitch bands and just enough UI fiction to imply the solver is watching the room when nobody is touching the keyboard.

## Features

- **Glyph rain.** Constraint-flavored symbols and SolverForge fragments cascade across the frame.
- **Procedural logo cluster.** The SolverForge ouroboros drifts, pulses, and blooms at center screen.
- **Floating diagnostics.** Fake solver metrics, watch lists, and status bars animate continuously.
- **CRT treatment.** Scanlines, vignette falloff, glow passes, and glitch-row displacement sell the tube.
- **Silent by design.** No soundtrack, no assets, no runtime media pipeline.
- **Headless renderer.** `--render` streams raw BGR24 frames to stdout for ffmpeg capture.
- **Wayland-native presentation.** Fullscreen Sway client with app id `solverforge-screensaver`.
- **Multi-output aware.** Creates one fullscreen window per active output so the whole desktop is covered.

## Running

```bash
make run          # build and launch the saver directly

# or directly:
cargo run --release
```

### Controls

| Key | Action |
|---|---|
| `ESC` | Quit |
| `SPACE` | Toggle overlay copy |

## Rendering to Video

The screensaver has a headless mode (`--render`) that writes raw BGR24 frames to stdout. Pipe that stream to ffmpeg for encoding.

```bash
make render       # render a 30s MP4 preset (requires ffmpeg)
make render-gif   # render a shareable GIF (640px, 15fps, palette-quantized)
```

Or manually:

```bash
cargo run --release -- --render 30 | \
  ffmpeg -f rawvideo -pixel_format bgr24 \
    -video_size 1920x1080 -framerate 60 -i - \
    -c:v libx264 -crf 18 -preset slow -pix_fmt yuv420p \
    screensaver_03.mp4
```

The `--render` flag accepts an optional duration in seconds and defaults to 30.

## Supported Environment

Supported target right now:

- `sway`
- `swayidle`
- a Linux desktop session where launching a normal Wayland client from `swayidle` is acceptable

Everything else is out of scope for this entry today.

If you want support for another compositor, desktop, or operating system, open an issue or send a PR.

## Installing on Your Machine

Install the binary with Cargo:

```bash
cargo install --path .
```

Or build it locally:

```bash
cargo build --release
```

- Binary path after `cargo install`: `~/.cargo/bin/solverforge-screensaver`
- Binary path after local build: `target/release/solverforge-screensaver`

Put that binary somewhere on your `PATH`, or launch it directly from the build output directory.

## Using It with Sway

This project does not ship a helper daemon or installer anymore. The intended integration path is `swayidle`.

A minimal example:

```bash
swayidle -w \
  timeout 300 '~/.cargo/bin/solverforge-screensaver' \
  resume 'pkill -x solverforge-screensaver || true' \
  before-sleep 'pkill -x solverforge-screensaver || true'
```

Or inside your `swayidle` config:

```ini
timeout 300 '~/.cargo/bin/solverforge-screensaver'
resume 'pkill -x solverforge-screensaver || true'
before-sleep 'pkill -x solverforge-screensaver || true'
```

Practical notes:

- Launch the binary manually once before wiring it into `swayidle`, so you know the path and Wayland session are correct.
- `ESC` exits the saver. `SPACE` toggles the overlay text.
- The intended Sway rule should match `app_id="solverforge-screensaver"`, not the title.
- SolverForge Linux should launch it directly from `swayidle`, not via `swaymsg exec` and not through a terminal wrapper.
- If you want support beyond `sway`, open an issue or send a PR.

## Architecture

```
src/
├── main.rs              505 lines  Entry point, window loop, headless render mode,
│                                   glyph rain, overlay text, CRT post-pass
├── logo.rs              439 lines  Procedural SolverForge ouroboros renderer
│                                   Thick-line geometry, head/tail details, center node
├── font.rs              309 lines  Embedded 8x8 bitmap font and text/glow helpers
└── palette.rs            98 lines  SolverForge phosphor palette and raster helpers
                        ─────
                        1351 lines  TOTAL
```

### Design Principles

- **Zero runtime assets.** The font is embedded, the logo is procedural, and every frame is synthesized from code.
- **Software scene generation.** Every frame is synthesized into a `Vec<u32>` framebuffer and then copied into the presentation surface.
- **Deliberate restraint.** This entry is silent on purpose. The atmosphere comes from motion, pacing, contrast, and phosphor treatment.
- **Series continuity.** The same SolverForge visual vocabulary carries through: emerald diagnostics, hard-edged geometry, and terminal menace.
- **Sway-only support.** Other environments should be treated as unsupported until someone adds and verifies them.

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| [pixels](https://crates.io/crates/pixels) | 0.15 | Pixel framebuffer presentation |
| [winit](https://crates.io/crates/winit) | 0.29 | Wayland window/event loop integration |

## Makefile Targets

```
make help          Show available targets (default)
make build         Build release binary
make run           Build and run the screensaver
make render        Render to MP4 via ffmpeg
make render-gif    Render to GIF (shareable)
make clean         Remove build artifacts + output
make check         Run cargo check
make clippy        Run clippy lints
make loc           Count lines of Rust source
```

## Resolution

Internal render resolution is `1920x1080` at 60fps. Each Wayland window scales that framebuffer onto its output surface. Headless render output uses the same internal resolution.

## License

MIT -- see [LICENSE](../LICENSE).

**SOLVERFORGE // PHOSPHOR AFTER HOURS // SILENT MODE // AMIGA FOREVER**
