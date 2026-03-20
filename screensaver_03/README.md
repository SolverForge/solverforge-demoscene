```
 ███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗  ██████╗ ███████╗
 ██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝
 ███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝█████╗
 ╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██╔══╝
 ███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║███████╗
 ╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝╚══════╝
```

**A silent SolverForge screensaver in Rust.**

Think "Screensaver" through a SolverForge lens: black phosphor, emerald glyph rain, CRT bloom, drifting diagnostic panes and the Ouroboros logo stalking the center like a threat model with taste.

## Features

- Pure Rust, software-rendered into a `Vec<u32>` framebuffer
- Silent by design -- proper screensaver mood
- Glyph rain using constraint-flavored symbols and SolverForge vocabulary
- Floating HUD panels with fake live metrics and rotating slogans
- CRT scanlines, vignette, ghost trails, and glitch slices
- Headless `--render` mode for MP4 capture

## Running

```bash
make run
```

### Controls

| Key | Action |
|---|---|
| `ESC` | Quit |
| `SPACE` | Toggle overlay copy |

## Rendering

```bash
make render
```

Or manually:

```bash
cargo run --release -- --render 30 | \
  ffmpeg -f rawvideo -pixel_format bgr24 \
    -video_size 1280x720 -framerate 60 -i - \
    -c:v libx264 -crf 18 -preset slow -pix_fmt yuv420p \
    screensaver_03.mp4
```

## Design Notes

This screensaver is pure SolverForge mood: black phosphor, emerald diagnostics, and terminal-lit menace translated into software-rendered Rust.
