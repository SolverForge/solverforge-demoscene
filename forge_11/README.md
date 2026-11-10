```
███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
      ── DESCRIBE, GENERATE, SOLVE, REPAIR // TEXTMODE ──
```

A 90-second, 1920x1080, 60 fps demo for SolverForge. It replays one real
agent session: a maintenance problem described in plain language, the model a
coding agent writes for it with the solverforge-modeling skill, the solve on
the laptop, and a disruption pushed into the running solve. Your model. Your
machine. Your solver version.

## The data rule

Every word, number and assignment on screen comes from `assets/capture.json`,
the capture bundle a real run produces (contract below). The demo never
invents data: the prompt is the prompt, the transcript lines are the agent's,
the code is the generated code, the scores are trajectory points, the plan is
the plan. Two display rules only: typography folds to the 8x8 font's ASCII,
home directories show as `~`, so no user name reaches a frame, and the
session's scratch directory (`~/.hermes/cache/scratch/<run>`) shows as
`$SCRATCH`, so the host's scratch layout stays off screen.

The solve command is the captured launch
(`PORT=7881 <scratch>/run-target/release/depot_maintenance`) shortened
through its directory: the capture's build command set
`CARGO_TARGET_DIR` to that run-target, so the screen types
`$ PORT=7881 $CARGO_TARGET_DIR/release/depot_maintenance` -- the same
binary, spelled the way the build named it. (`./target/release/...` would
be false here: the app dir has no target of its own; `cargo run --release`
is equivalent but is not what ran.) A binary in the app's own `target`
would show as `./target/release/<bin>`.

`assets/capture.json` is the real session of 2026-11-10: a city bus depot (12
buses, 20 jobs, 4 technicians, 10 days) modelled by an agent with the
solverforge-modeling skill, solved to 0hard/-100soft in 153 ms, then a bus
breakdown pushed into the running solve as a problem change and healed in
17 ms (5 jobs changed, 15 untouched, 1 added).

- `make render` loads `assets/capture.json` and refuses anything whose schema
  is not `forge11.capture/1`.
- `make preview` renders `assets/capture.placeholder.json`
  (`forge11.placeholder/1`, obviously synthetic `PH-` labels) and burns
  **PLACEHOLDER DATA** into every frame, so a preview can never pass for the
  real session.

## Scenes

| Bars | Time | Section | What you see |
|---:|---|---|---|
| 0-2 | 0:00 | Boot | Warp starfield, copper bars, the SOLVERFORGE banner beamed in by glyphfx, the four verbs landing on the beat |
| 2-9 | 0:04 | 01 Describe | The prompt typed to the coding agent word by word on the sixteenth-note grid, domain words lit; the plan's buses drive past on the river road |
| 9-19 | 0:18 | 02 Generate | The forge: the agent transcript scrolls by; each constraint excerpt is struck on the anvil on a bar line, enters white-hot through a glyphfx effect, then quenches from orange to its syntax colours in a puff of steam |
| 19-27 | 0:38 | 03 Solve | The run command types in; the score replays the captured trajectory on a log clock, the real solver milliseconds counting; the plan's blocks drop onto a technician x day board; feasibility lands on a beat |
| 27-39 | 0:54 | 04 Repair | The live plan; the captured disruption strikes on the second bar and enters the running solve through the captured mechanism (shown verbatim, wrapped over two lines); **Cordelia** cuts in over the score panel, defiant, her eyes charge and ignite, and on the downbeat the repair enters the solve her eye-lasers sear across to the board (see below); a new job drops in red-hot (or a technician's row is struck out); changed jobs travel to their new slots while every other block stays put; the score recovers; then each change is reviewed in turn, ghost to new slot; changed / untouched / added counts from the capture |
| 39-45 | 1:18 | 05 Close | YOUR MODEL. YOUR MACHINE. YOUR SOLVER VERSION., each with a line from the capture; the ouroboros draws itself; solverforge.org |

Every cut is on a downbeat: a glyph-rain wipe over the last half second, a
flash and a short scanline tear. The SolverForge mark sits in the top-left
corner throughout and steps aside for the large logo.

## Cordelia

The agent's hero beat. Times come from the script (`scenes/cordelia.rs`,
`Beat::new`), shared by picture and sound; on the real capture:

| Time | Beat |
|---:|---|
| 56.0 | the bus breaks down: red strike, alarm, band drops to a heartbeat |
| 56.5 | Cordelia glitches in (row tear, chromatic split) over the score panel's chart, CRT scanlines, name tag `CORDELIA // AGENT`; eyes start charging |
| 57.5 | eyes ignite (white-hot core, star glint), one beat before the bar |
| 58.0 | downbeat, the repair enters the running solve: she fires. Beams from both eyes sear across to the board: the left holds the breakdown's own job (red-hot `job-21`), the right rides the changed job to its new slot; sparks at each strike every eighth note |
| 60.0 | the hook returns: beams cut, she tears out; the panel's chart and change list are back for the rest of the section |

The artwork is three generated keyframes (`assets/cordelia/charge|ignite|fire.png`),
image-to-image edits of the brand direction's approved dark DEFIANCE artwork
(`solverforge-brand-direction/assets/dark/cordelia-defiance.png`, sha256
`daa75552e9d2fc524640a6189a43dcad0c1f2d23f75dacb944a0e3a5e6c24a27`) with its
expression atlas as identity reference, prompts derived from the saved Cordelia
generation prompts. Exact prompts, model, request ids and sha256 of every frame
are in `assets/cordelia/provenance.json`. The demo embeds each frame as a
680x562 binary PPM (`magick <frame>.png -resize 680x562! -depth 8 PPM:<frame>.ppm`)
parsed by hand. The engine's beams are an effect layer on top: anchored to the
iris centres measured on each keyframe's pixels, carried through the same
push-in / breathing / kick transform as the picture, so the origin never leaves
her eyes; only the far end jitters.

## How the capture becomes the picture

- `src/capture.rs` parses the bundle (a hand-written JSON reader in
  `src/json.rs`), folds typography to the 8x8 font's ASCII and validates it.
- `src/script.rs` decides when everything plays: typing on the sixteenth
  grid; transcript steps in order, spaced by their real timestamps; one
  excerpt per whole bar slot; the solver's wall clock mapped onto the section
  on a log scale (solvers improve fastest early), stretched so that the first
  feasible score lands on a beat. The on-screen timer always shows the real
  milliseconds, never the demo clock.
- `src/board.rs` lays out one board for both plans: each cell keeps its
  solved jobs in place and appends the jobs that move in, so an untouched
  block never shifts. Tests check that exactly the `changed` jobs move and
  that `unchanged_count` matches the blocks that stay.
- `src/code.rs` renders each excerpt through glyphfx on a canvas exactly its
  own size; a test proves the settled clip matches the highlighted listing
  cell for cell, so the switch from glyphfx to syntax colours is seamless.

## Audio

Synthwave at 120 BPM in E harmonic minor (D sharp, never D natural): Em - C -
Am - B, one chord per bar, the B major dominant pulling home. A new
eight-bar hook: a question ending open on the dominant, an answer closing on E
through the D sharp. Detuned supersaw pads, gliding bass, gated snare, tom
fills and noise risers into every section, tracker-style bends on the leads.

The mix is driven by the same schedule as the picture (`src/audio/song.rs`):
every typed word clicks; every transcript step blips; every excerpt lands on
an anvil (an inharmonic FM ring over a thud); every block drop ticks; every
score improvement pings a note up the scale by how far the score has come;
feasibility hits the crash, a tonic stab and a sub drop; the disruption sounds
an alarm on the D sharp / A tritone and drops the band to a heartbeat for two
bars; Cordelia's charge whine glides B3 -> E5 -> B5 into the downbeat, the zap falls E6 -> E5 over a sub hit, the beam hums on E2 + B2 gated in sixteenths, and a B4 -> E4 power-down closes it as the hook returns (all in E harmonic minor, checked by the pitch test); each changed job whooshes to its slot; the healed plan brings the hook
back. Under Describe the band is light; Generate and Solve run the full
groove; the close holds the dominant, then lands on E minor.

## Running

```bash
make run          # window + live audio, real capture
make run-preview  # window + live audio, placeholder capture
make render       # forge_11.mp4 from assets/capture.json (refuses the placeholder)
make preview      # forge_11_preview.mp4 from the placeholder, marked PLACEHOLDER DATA
make check        # cargo check --release
make clippy       # cargo clippy --release -- -W clippy::all
make test         # cargo test --release
```

Controls: `ESC` quits, `SPACE` jumps to the next section.

Flags: `--capture FILE`, `--allow-placeholder`, `--render [seconds]` (BGR24
frames to stdout, `target/demo_audio.wav`), `--stills t1,t2,... [--out dir]`
(PPM stills), `--cues` (when every captured event plays).

## Capture contract (v1)

```json
{
  "schema": "forge11.capture/1",
  "captured_at": "ISO-8601",
  "versions": {"solverforge_cli": "x.y.z", "solverforge": "x.y.z", "agent_model": "..."},
  "prompt": "the exact plain-language problem text given to the agent",
  "agent_steps": [{"t_ms": 0, "kind": "say|tool|file", "text": "verbatim, <=160 chars"}],
  "code": [{"path": "src/constraints/....rs", "title": "...", "excerpt": "verbatim, <=14 lines"}],
  "solve": {
    "wall_ms": 0,
    "trajectory": [{"t_ms": 0, "hard": 0, "medium": 0, "soft": 0}],
    "plan": [{"job": "id/label", "bus": "label", "technician": "label", "day": 0}]
  },
  "repair": {
    "event": {"kind": "technician_unavailable|bus_aog|...", "label": "human sentence", "detail": {}},
    "mechanism": "how the change entered the running solve, verbatim",
    "wall_ms": 0,
    "trajectory": [],
    "plan": [],
    "changed": [{"job": "...", "from": {"technician": "...", "day": 0}, "to": {"technician": "...", "day": 0}}],
    "unchanged_count": 0
  },
  "provenance": {"workdir": "...", "session_id": "...", "commands": ["..."]}
}
```

Score levels absent from the app's score type are omitted, not zero-filled;
the demo shows only the levels present. `event.detail.technician` (or a
technician with solved work and none after the repair) is the row struck out;
`event.detail.bus` (or, for a bus event, the bus of the job the repair adds)
flashes that bus's blocks. Jobs only in `repair.plan` are the disruption's
own and drop in with the strike. Repair trajectory times are shown relative
to the repair's first point. The session id is never drawn,
and only the last component of the workdir is.

## Architecture

```text
../vendor/glyphfx/  pinned glyphfx sources (shared by every demo)
../vendor/demofx/   glyphfx clips as cells, the 8x8 font, the logo
assets/             capture.placeholder.json (committed), capture.json (the real run),
                    cordelia/ (keyframes, embedded PPMs, provenance.json)
src/
├── main.rs         CLI, capture guard, window loop, headless export, stills
├── json.rs         a small JSON reader
├── capture.rs      the bundle: types, validation, the release guard
├── script.rs       the schedule shared by picture and sound
├── timeline.rs     120 BPM, 45 bars, sections on bar lines
├── board.rs        the technician x day board for both plans
├── code.rs         excerpt layout, syntax colours, glyphfx entrance
├── gfx.rs          frame, anti-aliased text at any cell size, primitives
├── fx.rs           copper bars, river, sparks, steam, starfield, glitch, wipe
├── post.rs         bloom, scanlines, vignette
├── scenes/         boot, describe, generate, solve, repair, cordelia, close, score panel
└── audio/          song.rs (the score), mod.rs (playback, WAV)
```

## Credits

Text effects: [glyphfx](https://github.com/blackopsrepl/glyphfx), a C port of
[ttfx](https://github.com/omacom/ttfx), itself a port of
[TerminalTextEffects](https://github.com/ChrisBuilds/terminaltexteffects) by
ChrisBuilds. MIT; see `../vendor/glyphfx/LICENSE` and `../vendor/glyphfx/NOTICE`.

## License

MIT -- see [LICENSE](../LICENSE).

**SOLVERFORGE // YOUR MODEL. YOUR MACHINE. YOUR SOLVER VERSION. // AMIGA FOREVER**
