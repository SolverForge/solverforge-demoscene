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

## Linux Install

```bash
make install-linux
```

This installs:

- `~/.local/bin/solverforge-screensaver`
- `~/.local/bin/solverforge-screensaver-launch`
- `~/.local/bin/solverforge-screensaverctl`
- `~/.local/share/applications/solverforge-screensaver.desktop`

Then you can run it from anywhere with:

```bash
solverforge-screensaverctl run
```

### Controls

| Key | Action |
|---|---|
| `ESC` | Quit |
| `SPACE` | Toggle overlay copy |

## Set as Your Linux Screensaver

Once installed, configure idle launch with:

```bash
solverforge-screensaverctl set --timeout 300
```

Current auto-configuration support:

- **Hyprland / hypridle** — writes a sourced `hypridle` listener config
- **Wayland / swayidle** — installs a user `systemd` service around `swayidle`
- **X11 / xautolock** — installs a user `systemd` service around `xautolock`

Useful commands:

```bash
solverforge-screensaverctl status
solverforge-screensaverctl unset
```

If no supported idle manager is detected, the screensaver still installs and runs fine manually.

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
