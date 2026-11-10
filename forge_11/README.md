```
███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
      ── DESCRIBE, GENERATE, SOLVE, REPAIR // TEXTMODE ──
```

A 3:32, 1920x1080, 60 fps demo for SolverForge. It replays one real
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
that does not apply here; the second ran 2:42, the third 4:16. This cut runs
3:32. It still asks the viewer to read only one thing at a time (below), and
every text that is read gets its full reading time; but generated code and
the agent's transcript are no longer treated as reading. Nobody reads code at
demo speed, so GENERATE is now an 18 s burst of texture (56 s before), and
the breakdown's long sentence became two short lines timed with what the
board shows. Nothing is held longer than it needs (a test fails if a
section ends more than two bars after its last text), and nothing sits
static with nothing new to read (`no_dead_air`, below).

| Bars | Time | Was | Section | What you see |
|---:|---|---|---|---|
| 0-10 | 0:00 | 20 s -> 20 s | Boot | Flight into a warp starfield, always forward; copper bars roll in; the SOLVERFORGE banner is beamed in on bar three and shines once on bar five; then the line that says this is a real session; then the four verbs land on the beats |
| 10-22 | 0:20 | 20 s -> 24 s | 01 Describe | Title, then its line; the terminal's header (who, which agent, when) read alone; then the prompt types word by word, domain words lit, its last screen held 3 s. The buses on the river road no longer carry names, so nothing written moves while the prompt is read |
| 22-31 | 0:44 | 56 s -> 18 s | 02 Generate | Title, line (the only two reads). Then the forge flat out: the transcript races up to "Now the constraint bodies." at ten steps a second; the six excerpts are struck on the anvil two beats apart (glyphfx entrance white-hot, camera punch-in, sparks from three points, a shockwave ring, a copper accent, heat shimmer while the code quenches, the outgoing excerpt scrambling into the next one's entrance) while the rest of the transcript races past; then the code rains down and assembles into the repo's files, each path lighting as its letters land |
| 31-53 | 1:02 | 44 s -> 44 s | 03 Solve | Title, line; the run command types alone; the board and its axes with their line; the score panel with what red means (a red light runs along the hard score); the run replays with no text on screen; FEASIBLE is read while the numbers stand still, then what zero means; the rest of the run; what amber means (an amber light runs along the soft score) |
| 53-86 | 1:46 | 70 s -> 66 s | 04 Repair | Title, line; LIVE PLAN for a bar; the breakdown (below), strike to fire in 12 s; the fire and the heal with no text; PLAN HEALED read while the numbers stand still; what moved, on the bar line the music leaves the climax (the moved jobs ping in turn, the untouched ones are swept); each of the five changes replayed (below); the closing line while the week runs across the board a day at a time |
| 86-106 | 2:52 | 42 s -> 40 s | 05 Close | Three sentences, each read alone, then its line from the capture under it; the ouroboros draws itself; solverforge.org over the final E minor |

### The breakdown, strike to fire

| t | Read | Picture |
|---:|---|---|
| +0 | "Bus 05 breaks down in service." | red strike, the camera punches in on Bus 05's job and shakes; the alarm sweeps along its mechanic's row, the card flips red-hot and goes dark, hatched, its edge beating red |
| +3.0 | "It needs urgent electrical repair by day 2." | the urgent job materialises white-hot over the board with a shockwave; a red bracket draws under days 0-2 (its deadline), later days step back; it drops at day 1 and bounces off (the whole day lights red, its jobs jolt: the day is full), then at day 2, the same |
| +5.9 | | it hangs over the board, hotter on every beat; the five jobs that will have to move start to tremble, outlined amber |
| +6.9 | "The depot sends it to the running app." | the change flies off the board as a comet into the score panel |
| +8 | | it arrives: the panel rings, Cordelia tears in; heat streams off the urgent job into her eyes, faster each beat (charge, build, ignite) |
| +12 | | the fire downbeat: beams, the urgent job slams into its slot (shockwave, camera punch), the changed jobs travel |

Both lines come from the capture's event label (`story::event_lines`;
`story::EVENT_NEED` holds the wording of the second). `Script::breakdown`
holds every time; `the_breakdown_never_stands_still` fails if two new things
are more than two seconds apart, if the stretch is longer than twelve
seconds, or if an event line is held past its reading time.

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
(banner, verbs, section titles and lines, the prompt's header, the typing
prompt, the run command, lower thirds, stamps, LIVE PLAN, the breakdown's
lines, each recap label, the close sentences and their lines, the address)
with its start, its reading time and its kind. It
is built in order, each text placed after the previous one ends, needing at
least 15 characters a second plus a second (captions: their card reading
window). `one_text_is_read_at_a_time` fails if two reading windows overlap,
if any text is up for less than it needs, or if a caption on screen is
missing from the table. Scenes dim what is not being talked about (the
board under the score line, during the breakdown and during the recap).

Glances (`Script::glances`) are the exemption: GENERATE's transcript, its
code excerpts and the repo assembling are texture, on screen to be seen, not
read. They have no reading time; in exchange the test fails if a glance
shares the screen with any read. The agent's first-person lines ("I built
the app and ran a real solve.") are part of that texture, never a read.

`Script::beats` lists every stretch the scenes animate on purpose (the
flight, typing, cuts, the forge, the run, the breakdown, the heal, Cordelia,
the recaps, the light along the score while red and amber are explained, the
moved jobs pinging, the week running, the logo). `no_dead_air` fails if any
moment of the video is more than two seconds from a read, a glance or an
animated beat. The audit that added it found, besides the breakdown: the
score explained over a still panel (now a light runs along the level being
explained), "5 jobs move..." and "Nobody ... by hand" over a still board
(now the moved jobs ping and the week runs), GENERATE's three seconds of
transcript after the last excerpt (gone with the new GENERATE), and a
closing bar after the address (cut).

What changed to get there, from the old cut:

- Boot: the banner's second shine ran under the line; it is gone. The verbs
  waited for the line.
- Describe: the agent/date line, the header and "> describe the problem in
  plain language" appeared together with the start of typing; the last
  repeated the opener and is dropped, the other two are one header read
  before typing starts. Bus names drove past under the prompt; dropped.
- Generate: no longer read at all past its title and line (see Glances).
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
on the bar line the depot's line ends on).

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
| 1:56 | the bus breaks down on a bar line (the breakdown table above) |
| 2:04 | the change arrives in the app's panel; Cordelia glitches in (row tear, chromatic split) over the score panel's chart, eyes **narrowed** on the problem |
| 2:05 | **charge**: the glint becomes a glow (a dissolve on the beat); heat streams off the urgent job into her eyes |
| 2:06 | **build**: the glow builds, the stream thickens |
| 2:07 | **ignite**: white-hot, star glint, two beats before the bar |
| 2:08 | the downbeat the repair enters the running solve: **fire** (no beams in the art). The urgent job slams into its slot, the camera punches in; the engine's beams sear from her eyes to the board, surging on every beat with the hum (a camera kick and a mint flash each beat, a raster sweep each bar): the left holds the breakdown's own job, the right rides each changed job to its new slot; sparks every eighth note. The music's climax: the Planner123 riff |
| 2:11 | the plan has healed: PLAN HEALED lands |
| 2:11.5 | beams cut, power-down; she recoils into a **smirk** (glitch accent) |
| 2:13.5 | she tears out |

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
  grid; transcript steps in order, a tenth of a second apart; one excerpt
  struck every two beats; the solver's wall clock mapped onto the section
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
Am - B, one chord per bar, the B major dominant pulling home. One exception:
Cordelia's climax, from her entrance to the healed plan, is defiant rather
than sinister (and not triumphant either: it stays in minor). There the D
sharp leaves and the scale is E natural minor (E F# G A B C D), Aeolian
i - VI - VII - i (table below);
`every_pitch_is_in_its_scale` checks every chord bar against its own
section's scale (harmonic minor everywhere else, natural minor in the
climax), plus the alarm figures, the riff and the lasers. A new
eight-bar hook: a question ending open on the dominant, an answer closing on E
through the D sharp. Detuned supersaw pads, gliding bass, gated snare, tom
fills and noise risers into every section, tracker-style bends on the leads.

The mix is driven by the same schedule as the picture (`src/audio/song.rs`):
every typed word clicks; every transcript step blips; every excerpt lands on
an anvil (an inharmonic FM ring over a thud); every block drop ticks; every
score improvement pings a note up the scale by how far the score has come;
feasibility hits the crash, a tonic stab and a sub drop; the disruption sounds
an alarm on the D sharp / A tritone over a heartbeat, and from there the music
builds a layer a bar into the fire (below); her charge whine glides
B3 -> E5 -> B5 into the downbeat, where D (VII) lifts to Em; the zap
falls E6 -> E5 over a sub hit, the beam hums on the open fifth E2 + B2 gated
in sixteenths with a small E6 -> E5 zap on every beat it surges, and a
B4 -> E4 power-down
closes it as the beams cut (all in the climax's scale, checked by the pitch
test); each changed job whooshes to its slot (and again as each recap
travels, under a bell); the hook's answer comes back after the riff, as the
plan has healed. In the climax the score pings fold D sharp to D and the
healed plan is hit with a bare E power chord (root, fifth, octave), not a
major stab.

The build, strike to fire, one bar each, a new chord every bar (the test
`the_repair_lands_on_its_chords` pins the chords and that no alarm figure
repeats):

| Bar | Chord | Alarm figure (square siren, 16th grid) | Band |
|---|---|---|---|
| strike | Am | D#6 / A5 tritone in eighths | heartbeat kick, held bass, pad low |
| +1 | C | moved down: C6 / F#5 in eighths | four on the floor, off-beat hats, bass in eighths |
| +2 | Am | D#6 / A5, the second half in sixteenths | gated snare on 2 and 4, the arp enters |
| +3 | F#m7b5 | a rising line A5 C6 D#6 F#6, twice | sixteenth hats, octave bass, the pad opens |
| +4 | C | Cordelia enters: C5 E5 G5 B5 climbing, turning down, then up to E6 and down D6 C6 | off-beat kicks, sixteenth bass; a two-bar riser starts; her charge whine glides B3 -> E5 |
| +5 | D | D5 F#5 A5 D6 twice, A5 C6 B5 A5, then D6 held | snare roll, riser peaks; the charge whine reaches B5; the held D6 lifts to E on the fire |

The way in (bars 61-64 of the song, 2:02-2:08): bar +3 is F#m7b5 (F# A C
E), which sits in E harmonic and E natural minor alike, so it is the pivot;
the D sharp is gone from bar +4. C - D (VI - VII) climbs and D lifts to Em
on the fire (bar 64, 2:08), the Aeolian landing on the tonic with weight
rather than a dominant pulling to it.

The way out (bars 67-68, 2:14-2:16): the riff's fourth bar is Em, in both
scales, and is the pivot back. That bar drops the climax's off-beat kicks,
brings the pad to its post-climax level and the arp back in, and runs a tom
fill into the downbeat; the riff's last phrase (D5 D5 E5) runs right up to
the seam. On the downbeat (bar 68, 2:16) the hook's answer picks up on Em in
a saw voice like the riff's, the octave bass carries on for four more bars,
and the picture moves on with it ("5 jobs move ..." lands on that bar
line). The D sharp only comes back three bars later, on the answer's B
(bar 71). Short-term loudness (EBU R128 S, from the soundtrack WAV) around
the seam: -17.4 LUFS over the two bars before it, -18.1 over the two after.

Cordelia's fire is the climax, and its lead is Planner123's riff, copied
(not shared) from `planner123_06/src/audio/song.rs` (`PLANNER_RIFF`, 16th
steps over four bars, originally C minor over Cm - Ab - Bb - G at 125 BPM).
Here it plays on the same 16th grid at 120 BPM over Em - C - D - Em, the
original's i - VI - VII functions in E, same steps, same rests, same
contour, transposed up a major third into E natural minor:

| Planner123 | forge_11 |
|---|---|
| C5 (72) | E5 (76) |
| Bb4 (70) | D5 (74) |
| Ab4 (68) | C5 (72) |
| G4 (67) | B4 (71) |
| F4 (65) | A4 (69) |

It is voiced firm and dark rather than bright: a driven lead (a saw plus a
square an octave down through tanh drive and a low cutoff: grit and body,
little sparkle) with a pulse an octave below, over a driving band that is
not louder than the song around it: four on the floor plus off-beat kicks,
gated snare, sixteenth hats, octave bass, the riff taking the arp's place
and the pad kept dark. No major-chord stabs. Short-term loudness (EBU R128
S, soundtrack WAV): the climax (2:08-2:16) averages -16.9 LUFS, against
-18.7 for the build before it, -18.6 for the eight seconds after and -18.7
for SOLVE: about +1.8 dB, by density and gain, not by the limiter.
`the_riff_keeps_its_rhythm_and_contour` pins the transposition.

The arrangement follows the 106-bar structure. The intro is ten bars: two
of pad alone swelling under the stars (a filtered arp creeping in on bar
two), the banner landing on bar three with a crash, a sub drop and the
bass, kick building from bar five, the hook's question stated on bar seven
over the full groove, the snare on bar nine and a snare roll, toms and a
riser on bar ten into the first downbeat of Describe. Under the prompt the band
stays light and the answer comes back softly an octave down; Generate runs
flat out (sixteenth bass, off-beat kicks, the filter open) with the question
on supersaw and brass over the strikes and a snare roll on D#dim7 into Solve;
Solve starts lighter, brings the answer softly, the brass question as the
plan comes together and the answer again under amber, changing bass gear
every four bars; after the riff the recap carries the question on pulse,
then brass; the close states the question softly, the answer doubled, the
question on brass, the answer diving, and lands on E minor under the logo.
No figure loops unchanged for more than four bars over a still picture: the
arrangement changes gear at least every four bars.

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
own: they materialise with the breakdown's second line and slam into their
slot on the fire. Repair trajectory times are shown relative
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
├── timeline.rs     120 BPM, 106 bars, sections on bar lines
├── story.rs        every caption, in one table
├── board.rs        the technician x day board for both plans
├── code.rs         excerpt layout, syntax colours, glyphfx entrance
├── gfx.rs          frame, anti-aliased text at any cell size, primitives
├── fx.rs           copper bars, river, sparks, steam, starfield, glitch, wipe,
│                   camera punch/shake, heat shimmer, shockwave rings
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
