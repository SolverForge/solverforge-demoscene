```
███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
      ── DESCRIBE, GENERATE, SOLVE, REPAIR // TEXTMODE ──
```

A 4:16, 1920x1080, 60 fps demo for SolverForge. It replays one real
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
that does not apply here; the second ran 2:42. This cut runs 4:16 because it
asks the viewer to read only one thing at a time (below): every text that
used to share the screen with another now waits for it, and each still gets
its full reading time. Comfortable reading wins over length; nothing is
rushed to save seconds, and nothing is held longer than it needs (a test
fails if a section ends more than two bars after its last text).

| Bars | Time | Was | Section | What you see |
|---:|---|---|---|---|
| 0-10 | 0:00 | 20 s -> 20 s | Boot | Flight into a warp starfield, always forward; copper bars roll in; the SOLVERFORGE banner is beamed in on bar three and shines once on bar five; then the line that says this is a real session; then the four verbs land on the beats |
| 10-22 | 0:20 | 20 s -> 24 s | 01 Describe | Title, then its line; the terminal's header (who, which agent, when) read alone; then the prompt types word by word, domain words lit, its last screen held 3 s. The buses on the river road no longer carry names, so nothing written moves while the prompt is read |
| 22-50 | 0:44 | 32 s -> 56 s | 02 Generate | Title, line; the transcript header; the tool calls scroll in as one glance; "Now the constraint bodies." is held; then the six excerpts land one at a time while the transcript stands dimmed; then the rest of the transcript, holding what the agent says ("Compiles clean.", "I built the app and ran a real solve.") |
| 50-72 | 1:40 | 30 s -> 44 s | 03 Solve | Title, line; the run command types alone; the board and its axes with their line; the score panel with what red means; the run replays with no text on screen; FEASIBLE is read while the numbers stand still, then what zero means; the rest of the run; what amber means |
| 72-107 | 2:24 | 34 s -> 70 s | 04 Repair | Title, line; LIVE PLAN for a bar; the breakdown's own sentence read alone while the board focuses on the broken bus; what the depot does, as Cordelia charges; the fire and the heal with no text; PLAN HEALED read while the numbers stand still; what moved; then each of the five changes replayed (below); the closing line |
| 107-128 | 3:34 | 26 s -> 42 s | 05 Close | Three sentences, each read alone, then its line from the capture under it; the ouroboros draws itself; solverforge.org holds over the final E minor |

### The repair recap

After the heal every changed job is replayed so the eye can follow it. The
card lifts off its new slot, reappears at its old one (outlined red, with a
dotted arc to the new slot outlined white), its label comes up --
`job-04: Giulia day 3 -> Marco day 3`, straight from the capture's `changed`
list -- and it travels along the arc and lands with sparks; the rest of the
board is dimmed. The first three travel for 1.4 s, the last two for 1.0 s;
each owns the screen for 4-4.5 s so its label is read in full (`Recap` in
`script.rs`; `every_recap_moves_and_is_read` pins it).

### One text at a time

`Script::reads` is the reading timeline: every text event in the video
(banner, verbs, section titles and lines, panel headers, the typing prompt,
transcript glances and sayings, excerpts, the run command, lower thirds,
stamps, LIVE PLAN, the breakdown, each recap label, the close sentences and
their lines, the address) with its start, its reading time and its kind. It
is built in order, each text placed after the previous one ends, needing at
least 15 characters a second plus a second (captions: their card reading
window). `one_text_is_read_at_a_time` fails if two reading windows overlap,
if any text is up for less than it needs, or if a caption on screen is
missing from the table. Scenes dim the panel that is not being read
(transcript versus editor; the board under the score line and during the
recap).

What changed to get there, from the old cut:

- Boot: the banner's second shine ran under the line; it is gone. The verbs
  waited for the line.
- Describe: the agent/date line, the header and "> describe the problem in
  plain language" appeared together with the start of typing; the last
  repeated the opener and is dropped, the other two are one header read
  before typing starts. Bus names drove past under the prompt; dropped.
- Generate: transcript lines typed in while excerpts landed; now they
  alternate. The agent's last message shows its first sentence (the rest
  repeats what Solve shows).
- Solve: the command typed while the axes lit and the board's line came up,
  and two chips appeared; serialised, chips dropped (the version is in the
  close). The run ticked while red was explained; it now starts after. The
  FEASIBLE stamp and "Zero: ..." overlapped; the run pauses on FEASIBLE
  (`Replay::hold_at`), then the line, then the rest of the run, then amber.
- Repair: the breakdown, the typed mechanism line and the event kind came up
  together; the mechanism (code) and the kind are no longer drawn (the
  mechanism is still required in the capture). The change list, the four
  counters (CHANGED / UNTOUCHED / ADDED / KEPT RUNNING) and "what moved" all
  ran during or right after the heal; the counters and list repeated the
  "what moved" line and the recap labels and are dropped. PLAN HEALED is
  read while the run pauses.
- Close: each sentence's capture line came up 1.2 s into the sentence; it
  now waits until the sentence has been read.
- Lower thirds and openers settle in 0.3 s instead of 0.5-0.9 s, so reading
  starts at once; the close keeps its slower effects (counted in its window).

Why each length: everything above, laid end to end, plus the song's bar
grid (every section starts on a downbeat, the strike on a bar line, the fire
on the bar after the depot's line).

The SolverForge mark sits in the top-left corner throughout and steps aside
for the large logo. Every cut is on a downbeat: a glyph-rain wipe, a flash,
a short tear and copper bars in the new section's colours.

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
| 2:34 | the bus breaks down on a bar line: red strike, alarm, band drops to a heartbeat; the board focuses on the broken bus while its sentence is read |
| 2:44 | Cordelia glitches in (row tear, chromatic split) over the score panel's chart, eyes **narrowed** on the problem, as the caption says what happens next |
| 2:45 | **charge**: the glint becomes a glow (a dissolve on the beat) |
| 2:46 | **build**: the glow builds |
| 2:47 | **ignite**: white-hot, star glint, two beats before the bar |
| 2:48 | the downbeat the repair enters the running solve: **fire** (no beams in the art). The engine's beams sear from her eyes to the board, surging on every beat with the hum: the left holds the breakdown's own job, the right rides each changed job to its new slot; sparks every eighth note |
| 2:51 | the plan has healed: PLAN HEALED lands |
| 2:51.5 | beams cut, power-down; she recoils into a **smirk** (glitch accent) |
| 2:53.5 | she tears out |

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
an alarm on the D sharp / A tritone and drops the band to a heartbeat on Am - C
from the strike until the fire, through Cordelia's charge on B; her charge whine glides
B3 -> E5 -> B5 into the downbeat, which resolves the dominant on Em; the zap
falls E6 -> E5 over a sub hit, the beam hums on E2 + B2 gated in sixteenths
with a small E6 -> E5 zap on every beat it surges, and a B4 -> E4 power-down
closes it as the beams cut (all in E harmonic minor, checked by the pitch
test); each changed job whooshes to its slot (and again as each recap
travels, under a bell); the healed plan brings the
hook's answer back on the next bar.

The arrangement follows the 128-bar structure. The intro is ten bars: two
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
├── timeline.rs     120 BPM, 128 bars, sections on bar lines
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
