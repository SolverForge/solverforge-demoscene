# WORKBENCH_12

The demoscene frame around the SolverForge Workbench video, a plugin for
Hermes Agent. 1920x1080, 60 fps, 101 bars at 125 BPM (193.9 s):

- **Intro (7 bars).** A flight into the stars while glyphfx beams in the
  title. The plugin's five models then land one per beat: the four stock
  models, each with the demo it ships and its rules counting up by level,
  and a dashed slot for a model you build. The counts are the README's
  (`src/models.rs`, pinned by a test). The stars warp into the first
  downbeat.
- **Live (88 bars).** Bezels open over rolling copper bars, with the statue
  from Hermes Desktop's backdrop barely visible underneath. They frame a
  1440x774 window for the narrated Desktop take, and beside it a portrait
  box and a dialog box for the Nous Girl, who narrates. Each chapter of the
  take is titled above the window as glyphfx types it in ("04  SOLVE",
  "08  REPLAN"). Under the window, the plugin's models are named, with the
  one on screen lit. The chapters come from the cut (`src/cut.rs`, written
  from the production's `cut/cues.json`).
- **Outro (6 bars).** The logo draws itself, the cards say what every plan
  gets and what this is, and the address and the install line stay up to
  the last frame.

The soundtrack remixes the latest Planner123 tune (planner123_06). It keeps
the tune's loop, its riff on the supersaw stab, its solo and its basses, and
rearranges them into a form with an arc:

1. a taiko intro
2. a verse of santur, frame drums and duduk
3. a first drop with the electric solo
4. a breakdown on its own progression
5. a second drop lifted to D minor
6. a half-time section where guitar and duduk trade calls
7. a final drop
8. an outro cadence

The sound is ancient instruments played by a machine. The strings are plucked
Karplus-Strong strings, and the lead is an electric guitar that bends, slides,
sings with vibrato and dives, in the manner of One Must Fall. The music stays
at one level throughout. Its sections follow the cut's chapters.
`src/audio/song.rs` lists what is kept and what is new.

## Run and render

```sh
make run       # window with live audio; SPACE jumps to the next section
make render    # workbench_12.mp4 with audio, via ffmpeg
make check     # cargo check
make clippy
```

`workbench-12-demo --live` prints when the live section runs, the cut's
length, and where its windows are (`take`, `portrait`, `dialog`, as
`x y w h`), for compositing. `--wav` writes the soundtrack alone, and
`--bench t1,t2,..` times the scene and the post pass at those times.

The window keeps 60 fps. Each frame's post pass runs on one frame while the
next frame's scene is drawn on another. The per-pixel passes are split
across cores in row bands, and their output is byte-identical to the
single-threaded passes.

## Post-production

The production folder holds the recorded chapters, the cut and the
composition scripts. The final video is assembled in this order:

1. The Desktop takes are recorded headless with seshat, one per chapter.
2. `build_cut.py` cuts them to the narrated take (1440x774, 60 fps). It keeps
   every interaction at normal speed or a little faster, speeds up the waits,
   and checks that every narration line ends inside its chapter.
3. `build_compose.py` mixes the sound once for the whole video: the music at
   one level throughout and the voice on top. It lays the take into the
   window and builds the EffectCraft project around this render. That
   project animates the Nous Girl from her pet spritesheet and types each
   line into the dialog box as she speaks it.

## Credits

The advisor is the Nous Girl pet for Hermes Desktop
(`~/.hermes/pets/nous-girl/spritesheet.webp`), after the Nous Research girl
logo.

The statue in `assets/hermes_statue.pgm` comes from Hermes Desktop's backdrop
(`apps/desktop/public/ds-assets/filler-bg0.jpg` in the
[Hermes Agent](https://github.com/NousResearch/hermes-agent) repository,
MIT License, © Nous Research). Here it is inverted and stored as a 960x540
greyscale map.
