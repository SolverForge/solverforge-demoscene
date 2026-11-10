```
███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
      ── DESCRIBE, GENERATE, SOLVE, REPAIR // TEXTMODE ──
```

A 2:42, 1920x1080, 60 fps demo for SolverForge. It replays one real
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

The first cut ran 90 seconds, a limit carried over from a hackathon video
that does not apply here. This cut spends the extra time on comprehension,
not on more on screen: one idea per moment, each section opened by a line
that leads on from the last, each caption held for its reading window and no
longer, so the whole stays brisk.

| Bars | Time | Was | Section | What you see |
|---:|---|---|---|---|
| 0-10 | 0:00 | 4 s -> 20 s | Boot | Flight into a warp starfield, always forward and gaining speed; copper bars roll in; the SOLVERFORGE banner is beamed in by glyphfx on bar three and held, a glyphfx highlight runs across it on bars five and nine; one line says this is a real session; the four verbs land on the last bar's beats as the stars warp into the cut |
| 10-20 | 0:20 | 14 s -> 20 s | 01 Describe | Opener; the prompt typed to the coding agent word by word over 11 s (was 10 s, now after the opener explains what it is), domain words lit, its last screen held for 3 s; the plan's buses drive past on the river road |
| 20-36 | 0:40 | 20 s -> 32 s | 02 Generate | Opener; the agent transcript scrolls; each constraint excerpt is struck on the anvil and held 3-6 s by its length (was 2-5 s), white-hot glyphfx entrance, quench to syntax colours, steam |
| 36-51 | 1:12 | 16 s -> 30 s | 03 Solve | Opener; the run command; the board's axes introduced (each mechanic's row lit on a beat, then the days); what red means, with the score panel lit and the board dimmed while the run replays; FEASIBLE lands only after that line is read and holds 3.5 s; what the amber number means |
| 51-68 | 1:42 | 24 s -> 34 s | 04 Repair | Opener; a bar of the live week; the breakdown strikes on a bar line and everything but the broken bus's work steps back; Cordelia's performance (below); PLAN HEALED holds 3.5 s; the change list she covered is lit again as she leaves; what moved, then each change in turn with the rest of the board dimmed; one closing line |
| 68-81 | 2:16 | 12 s -> 26 s | 05 Close | Three sentences, one at a time, each with its line from the capture; the ouroboros draws itself in the middle of the frame; solverforge.org holds over the final E minor |

Why each length: every caption is up for `max(demofx::reading_window, 15
characters a second + 2 s)` and the next thing starts on the beat after it.
Boot is ten bars because the song's intro needs two bars of pad, four of
banner and four of hook to build into the first downbeat. Describe types the
prompt at a pace where the lit domain words can be followed, then moves on.
Generate gives each excerpt half-bar slots weighted by its line count. Solve
introduces the board and the score before they move, so the feasible moment
means something. Repair gives the breakdown a bar alone, Cordelia four bars,
each reviewed change a bar and a half.

Every cut is on a downbeat: a glyph-rain wipe over the last half second, a
flash, a short scanline tear and copper bars in the new section's colours
rolling down the frame over the first beat; when an opener hands over to its
scene a raster line sweeps the frame. The SolverForge mark sits in the
top-left corner throughout and steps aside for the large logo.

## Captions

Every line of copy is in one table, `src/story.rs`: a narrator telling what
happens at the depot and to its people, in the third person, each line
leading into the next. Numbers in captions are filled from the capture
(`{changed}`, `{unchanged}`, the broken bus, how fast the plans came back).
`src/script.rs` times them (never two in one place at once, each up for its
reading window); `src/scenes/captions.rs` draws them through a glyphfx
entrance, one clip per line so every line is centred. Tests pin the house
rules (no first person, no em dashes, no marketing words, no rhetorical
questions).

## Cordelia

The agent's hero beat. Times come from `Script::cordelia` (`script.rs`),
shared by picture and sound; on the real capture:

| Time | Beat |
|---:|---|
| 1:50 | the bus breaks down on a bar line: red strike, alarm, band drops to a heartbeat; the board focuses on the broken bus |
| 1:52 | Cordelia glitches in (row tear, chromatic split) over the score panel's chart, eyes **narrowed** on the problem, as the caption says what happens next |
| 1:53 | **charge**: the glint becomes a glow (a dissolve on the beat) |
| 1:54 | **build**: the glow builds |
| 1:55 | **ignite**: white-hot, star glint, two beats before the bar |
| 1:56 | the downbeat the repair enters the running solve: **fire** (no beams in the art). The engine's beams sear from her eyes to the board, surging on every beat with the hum: the left holds the breakdown's own job, the right rides each changed job to its new slot; sparks every eighth note |
| 1:59 | the plan has healed: PLAN HEALED lands |
| 1:59.5 | beams cut, power-down; she recoils into a **smirk** (glitch accent) |
| 2:01.5 | she tears out; the change list under her is lit again |

The artwork is six generated keyframes (`assets/cordelia/narrow|charge|build|ignite|fire|smirk.png`),
image-to-image edits of the brand direction's approved dark DEFIANCE artwork
(`solverforge-brand-direction/assets/dark/cordelia-defiance.png`, sha256
`daa75552e9d2fc524640a6189a43dcad0c1f2d23f75dacb944a0e3a5e6c24a27`) with its
expression atlas as identity reference, prompts derived from the saved Cordelia
generation prompts. Exact prompts, model, request ids and sha256 of every frame
are in `assets/cordelia/provenance.json`, including the first fire frame
the current one replaces (that one had beams painted into the art, which
disagreed with the engine's beams angling to their targets; the new one has
glowing eyes only). The demo embeds each frame as a
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
an alarm on the D sharp / A tritone and drops the band to a heartbeat for three
bars on Am - C - B, through Cordelia's charge; her charge whine glides
B3 -> E5 -> B5 into the downbeat, which resolves the dominant on Em; the zap
falls E6 -> E5 over a sub hit, the beam hums on E2 + B2 gated in sixteenths
with a small E6 -> E5 zap on every beat it surges, and a B4 -> E4 power-down
closes it as the beams cut (all in E harmonic minor, checked by the pitch
test); each changed job whooshes to its slot; the healed plan brings the
hook's answer back on the next bar.

The arrangement follows the 81-bar structure. The intro is ten bars: two
of pad alone swelling under the stars (a filtered arp creeping in on bar
two), the banner landing on bar three with a crash, a sub drop and the
bass, kick building from bar five, the hook's question stated on bar seven
over the full groove, the snare on bar nine and a snare roll, toms and a
riser on bar ten into the first downbeat of Describe. Under the prompt the band
stays light and the answer comes back softly an octave down; Generate states
question and answer in full and a brass dive; Solve brings the brass
question back as the plan comes together; the close states the question softly, then the answer
doubled, and lands on E minor under the logo.

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
├── timeline.rs     120 BPM, 81 bars, sections on bar lines
├── story.rs        every caption, in one table
├── board.rs        the technician x day board for both plans
├── code.rs         excerpt layout, syntax colours, glyphfx entrance
├── gfx.rs          frame, anti-aliased text at any cell size, primitives
├── fx.rs           copper bars, river, sparks, steam, starfield, glitch, wipe
├── post.rs         bloom, scanlines, vignette
├── scenes/         boot, describe, generate, solve, repair, cordelia, close, score panel, captions
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
