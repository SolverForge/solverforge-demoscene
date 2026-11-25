# ROSTER_13

Who covers Thursday? The SolverForge Workbench plugin for Hermes Agent plans a
month of hospital shifts, then replans it live when a nurse calls in sick.
1920x1080, 60 fps, 63 bars at 120 BPM (126 s).

The film is the real Hermes Desktop: every take is the plugin running in it,
recorded headless with seshat and cut on bar lines (`src/cut.rs`, written by
the production's `build.py`). The demo draws its own scenes between the takes
and points at what the narration names over them. It never scales a take or
puts its finish on one, so the plugin's text reaches the screen as recorded.

- **Cold open (4 bars).** Thursday 4 March, 07:00: Cas South's shift on Ward
  A, its card and name ringed.
- **Intro (5 bars).** Over rolling copper, the month rises as a wave of
  glyphs, one per shift of the dataset (`src/scenes/intro.rs`, pinned by a
  test to its 536 shifts). glyphfx beams in the title; 50 employees, 3 wards
  and 536 shifts land one per beat pair. The stars streak into the first take.
- **Solve, explain (19 bars).** The solve starting, its search at four times
  speed, the verdict, and the rule that costs most and what is behind it.
- **The call (3 bars).** Cordelia, in three stills from the SolverForge art
  direction (`assets/`): she hears it as the still resolves out of glyphs and
  the ring pulses from the left, she decides, she raises her hand. "CAS
  SOUTH", "SICK ON THURSDAY", "WHO COVERS THURSDAY?", "REPLAN."
- **Replan (15 bars).** The replan form, Cas off for Thursday with everything
  before kept, the search a step faster, then the result.
- **Before, after, compare, every model (13 bars).** The same shift with Cas
  South, then with Oto North; the published shifts that moved; the two runs
  side by side; the catalogue.
- **Outro (4 bars).** The logo draws itself; the address and the install line
  stay to the last frame.

Over the takes the chapter is typed by glyphfx into the Desktop's empty
sidebar, under the corner logo, and a sped-up search shows its speed. The
takes come in and go out through a cell dissolve; a straight cut between two
takes carries a short slice glitch.

The soundtrack is written for this film, in D minor on the loop Dm - Bb - F -
C, with a 3-3-2 riff and a guitar motif that climbs to a bent note and steps
back. Its parts sit on the cut: a clock for the cold open, taiko under the
intro's numbers, a drop for the solve with the solo riding the search, frame
drums for the explanation, the kit stopping for the call, an E minor drop for
the replan's search, half-time for before and after, and a cadence into D
minor. The instruments are the series': Karplus-Strong santur and tanpura, a
duduk, frame drums and taiko, supersaws, a square bass and a tracker guitar.
The music keeps one level throughout; `src/audio/song.rs` has the details.

## Run and render

```sh
make run                                # window with live audio; SPACE jumps to the next segment
make render FOOTAGE=path/footage.mp4    # roster_13.mp4 with the score, via ffmpeg
make check
make clippy
```

Without `FOOTAGE` the takes' place shows a dark grid. `--wav` writes the
soundtrack alone, `--stills t1,t2,..` writes frames, and `--bench t1,t2,..`
times the scene and the post pass. ffmpeg decodes the footage.

## Post-production

1. The Desktop takes are recorded headless with seshat, one per chapter, on
   an isolated Hermes home with the plugin installed.
2. `build.py` cuts them into `footage.mp4` (full frame, 60 fps, sped up only
   where a search runs), places the narration (Edge, en-GB-LibbyNeural) into
   `voice.wav`, checks that no line overlaps another, and writes `src/cut.rs`.
3. The demo renders the picture over the footage and the score; the voice is
   mixed over the score at one level.
