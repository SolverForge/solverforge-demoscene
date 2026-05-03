# Route Splice 04

A 116-second SolverForge demoscene intro about list variables, successor links, insertion, and local route repair inside a top-down route plan.

**A route list is ordered movement through constrained space.**

## What Is This

`route_splice_04` turns a SolverForge route/list variable into spectacle. A luminous route visits stations in order. When a required station appears, SolverForge evaluates insertion positions, splices the station into the ordered list, shifts downstream indices, and repairs only the local route constraints.

The central metaphor is a route-plan board where:

- stations are list entries
- numbered beads are route indices
- gold thread is the ordered route
- arrowheads and `prev > next` labels are successor links
- candidate arcs are insertion trials
- one splice breaks one link and forms two
- local badges show `TIME`, `CAP`, `ORDER`, and `DIST`
- a blocked link redraws only the affected detour segment

The default experience is a windowed Rust demo with a standalone arena-tracker synth song: low ostinato bass, gated chord stabs, punchy sampled-style drums, sparse lead hooks, echo returns, fills, pads, and a staged solo section. The visuals do not drive the music.

## Scenes

| # | Scene | Time | Effect |
|---|---:|---|---|
| 1 | Invocation | 0-10s | Route plan emerges from blackness |
| 2 | Stations | 10-22s | Route beads light in list order |
| 3 | Successors | 22-34s | Moving arrows and `prev > next` labels make successor structure explicit |
| 4 | Required Station | 34-46s | A blue required station appears off-route |
| 5 | Insertion Search | 46-60s | Candidate arcs score insertion positions and the best splice wins |
| 6 | Splice | 60-74s | One link breaks, two links form, downstream indices shift |
| 7 | Constraint Repair | 74-90s | Only local route constraints pulse and settle |
| 8 | Reroute | 90-100s | A blocked link causes a compact detour |
| 9 | Final Route | 100-108s | The completed ordered path glows |
| 10 | Scrolltext | 108-116s | Classic demoscene outro and loop |

## Running

```bash
make build
make run
```

`make run` opens the demo window and starts live audio.

Controls:

- `ESC`: quit
- `SPACE`: advance to the next scene boundary

Direct launch:

```bash
cargo run --release
```

## Rendering

```bash
make render       # render route_splice_04.mp4 with generated audio
make render-gif   # render route_splice_04.gif
```

Headless export is explicit:

```bash
cargo run --release -- --render 116 | \
  ffmpeg -f rawvideo -pixel_format bgr24 \
    -video_size 1280x720 -framerate 60 -i - \
    -i target/demo_audio.wav \
    -c:v libx264 -c:a aac -b:a 192k \
    -crf 18 -preset slow -pix_fmt yuv420p \
    route_splice_04.mp4
```

The `--render` flag writes raw BGR24 frames to stdout and generates `target/demo_audio.wav` for muxing.

## Commands

| Command | Effect |
|---|---|
| `make run` | Build and run the windowed demo |
| `make render` | Render MP4 via ffmpeg with generated audio |
| `make render-gif` | Render a shareable GIF |
| `make check` | Run `cargo check --release` |
| `make clippy` | Run `cargo clippy --release -- -W clippy::all` |

## Architecture

```text
src/
├── main.rs        CLI, minifb window loop, headless export
├── audio/         live cpal playback and deterministic WAV synthesis
├── config.rs      dimensions, FPS, duration, export names
├── framebuffer.rs raw BGR framebuffer and minifb conversion
├── scene.rs       scene timing and SPACE boundaries
├── route_plan.rs  station-backed route-plan geometry
├── route.rs       list-variable route state and insertion/reroute data
└── render.rs      software-rendered route plan, route effects, captions
```

Design principles:

- windowed demo first
- explicit headless export for ffmpeg
- one stable route-plan stage for all route mutations
- standalone song first, visual timing second
- ordered route mutation must be legible without reading docs

## License

MIT - see [LICENSE](../LICENSE).

**SOLVERFORGE // LIST VARIABLES ARE ORDERED SPACE // AMIGA FOREVER**
