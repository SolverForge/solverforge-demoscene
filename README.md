```
 ███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
 ██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
 ███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
 ╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
 ███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
 ╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝

 ██████╗ ███████╗███╗   ███╗ ██████╗ ███████╗ ██████╗███████╗███╗   ██╗███████╗
 ██╔══██╗██╔════╝████╗ ████║██╔═══██╗██╔════╝██╔════╝██╔════╝████╗  ██║██╔════╝
 ██║  ██║█████╗  ██╔████╔██║██║   ██║███████╗██║     █████╗  ██╔██╗ ██║█████╗
 ██║  ██║██╔══╝  ██║╚██╔╝██║██║   ██║╚════██║██║     ██╔══╝  ██║╚██╗██║██╔══╝
 ██████╔╝███████╗██║ ╚═╝ ██║╚██████╔╝███████║╚██████╗███████╗██║ ╚████║███████╗
 ╚═════╝ ╚══════╝╚═╝     ╚═╝ ╚═════╝ ╚══════╝ ╚═════╝╚══════╝╚═╝  ╚═══╝╚══════╝
```

**Demoscene intros for SolverForge products. Coded in Rust. No JavaScript was harmed.**

---

## What Is This

Audiovisual demos in the tradition of the Amiga/PC demoscene -- built to showcase [SolverForge](https://solverforge.com) products. Pure Rust. Software-rendered. Procedural audio. No external assets. No GPU. One binary per demo.

## Demos

| Directory | Product | Duration | Status |
|---|---|---|---|
| [`planner123_01/`](planner123_01/) | [Planner123]() | 84s | Complete |
| [`serio_02/`](serio_02/) | [SERIO](https://crates.io/crates/solverforge-scoring) | 114s | Complete |
| [`screensaver_03/`](screensaver_03/) | SolverForge Screensaver | Endless / 30s render preset | Complete |

Each demo is a standalone Rust crate. See the README inside each directory for details on scenes, architecture, controls, and rendering.

## Quick Start

Each demo has its own Makefile. `cd` into a demo directory and run:

```bash
cd planner123_01
make run          # build and launch (opens a window, plays audio)
make render       # render to MP4 via ffmpeg
make help         # see all targets
```

For the Linux screensaver:

```bash
cd screensaver_03
make install-linux                # install into ~/.local/bin
~/.local/bin/solverforge-screensaverctl run
~/.local/bin/solverforge-screensaverctl set --timeout 300
```

## System Requirements

**Build:**
- Rust toolchain (stable, edition 2021)
- A C linker (for native dependencies)

**Runtime (windowed):**
- X11 or Wayland (minifb needs a display server)
- Audio output (ALSA, PulseAudio, or PipeWire)

**Runtime (headless render):**
- ffmpeg with libx264 support

On Debian/Ubuntu:
```bash
sudo apt install build-essential libasound2-dev libx11-dev libxkbcommon-dev ffmpeg
```

On Fedora:
```bash
sudo dnf install alsa-lib-devel libX11-devel libxkbcommon-devel ffmpeg
```

On Arch:
```bash
sudo pacman -S alsa-lib libx11 libxkbcommon ffmpeg
```

## About the Demoscene

The [demoscene](https://en.wikipedia.org/wiki/Demoscene) is a computer art subculture that produces audiovisual programs as real-time art. Originating on the Amiga and Commodore 64 in the late 1980s, it emphasizes pushing hardware to its limits through clever programming. In 2021, UNESCO recognized the demoscene as intangible cultural heritage.

This project carries that tradition forward in Rust: every pixel software-rendered, every sample synthesized, no external assets, no GPU abstraction layers. Just code and math.

## License

MIT -- see [LICENSE](LICENSE).

**SOLVERFORGE // BUILT IN ITALY // GREETS TO ALL OPTIMIZATION HACKERS**
