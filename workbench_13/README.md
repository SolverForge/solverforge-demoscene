# WORKBENCH_13

A factory week planned, broken by a late delivery and repaired, with the
SolverForge Workbench for Hermes Agent. 1920x1080, 60 fps, 121 bars at
120 BPM (242 s).

The film mixes three sources, each doing its own job:

- **Recorded takes** of the real Hermes Desktop (seshat), shown at 1:1 in the
  top 1920x920 of the frame: opening the model and its demo, solving, moving
  the powder deliveries, checking, replanning, comparing the runs.
- **Native scenes** drawn from the recorded runs: the week, the powder and the
  repair. Every card, time and score comes from `data/`, which the production
  extracts from the same runs the takes show.
- **glyphfx** for the words: the intro's banner (rotozoom), the scene titles,
  the band's chapter titles, the lane names, the call and the close.

The band, the bottom 160 rows, stays the same from the first take to the last
scene: the Nous Girl's place (her sprite is laid in by the compose step), her
words typed as she says them, the chapter's title, the mark, and along its top
a rail with one segment per chapter.

## The film

1. **Intro.** The flight into the stars; the title turns in as a rotozoom
   banner and drops away; the plugin's five models land one per beat with
   their rules by level.
2. **The orders** (take). The model catalog, the factory dataset, its 409
   operations and the powder stock with four deliveries due Tuesday 06:00;
   Solve.
3. **The week.** The solve redrawn from its improving plans: two minutes of
   search in twenty seconds. The first pass pours the operations in, ten wait
   in the tray; every card glides from one plan to the next, blue while it
   moves; the last plan lands under a jade sweep.
4. **The issued week** (take). The result: feasible, 409 placed.
5. **The delay** (take). The four deliveries moved to Thursday 18:00, saved as
   a new dataset, checked as is: −300 hard, parts out of stock.
6. **The powder.** The four powders' stock on the week's time axis; the
   deliveries slide to Thursday; red, blue and grey run below zero; the five
   coating jobs without powder light up on the powder lines and are listed.
7. **The call.** Cordelia, drawn in by a scanline in concern; a red line cuts
   in and she dissolves to defiance through a raster tear on the downbeat, an
   emerald glint in her eyes, the schedule bars drawing in red; "keep what has
   started, replan the rest", she dissolves to resolve and the bars fill jade.
8. **Replan** (take). Keep everything that starts before Thursday 06:00;
   replan and solve.
9. **The repair.** The kept side of the line dims; the search's hard score
   draws over its three minutes; then what changed: the five coating jobs one
   at a time, named as they move, then the other 91 together. The score turns
   to the repaired plan's: 0 hard.
10. **The repaired week** (take). 409 placed, no rule broken; the three runs
    compared; back to the models.
11. **Close.** The mark, Cordelia at ease, the address and the install line.

Colours follow the SolverForge character language's dark mode: emerald night,
silver parchment, jade; blue explores, red flags a conflict, emerald resolves.

The soundtrack is a new score in G minor with the raised seventh (D Hijaz over
the dominant), one part per section: santur and frame drums for the orders, the
machine starting for the week, a held bend home for the issued week, a ticking
clock for the delay, an alarm (a D pedal, a heartbeat, Eb grinding on D) for
the powder, Cordelia's concern, defiance and decision on the beats of the call,
a taiko build for the replan, tremolo guitar with a twin for the repair, calls
traded with the duduk for the repaired week, and Eb - D - Gm to close.
`src/audio/song.rs` describes each part. The music keeps one level throughout;
the narration is mixed over it.

## Run and render

```sh
make run       # window with live audio; SPACE jumps to the next section
make render    # workbench_13.mp4: the frame and the score, the takes left dark
make check
make clippy
cargo test --release
```

`workbench-13-demo --layout` prints the footage spans and the portrait box;
`--matte` streams the footage matte (gray8, 255 where the take shows, with a
raster transition of its own at each take's start and end); `--wav` writes the
score; `--stills t1,t2,..` writes frames.

## Production

1. Record the takes on the isolated Desktop with seshat, the window 1920x920.
2. Extract `data/` from the recorded runs; synthesize the narration (Edge,
   en-GB-LibbyNeural); cut the takes onto the film's clock, which writes
   `src/cut.rs` (sections on bar lines and every spoken line's time).
3. Compose in one ffmpeg pass: the demo's frame, the takes through the matte,
   the Nous Girl's sprite in the band, the score at one gain and the voice.

Cordelia's keyframes (`assets/cordelia/`: concern, defiance, resolve, ease) are
new image-to-image edits of the character language's approved dark-mode artwork,
all on one framing so they dissolve with the face in place; prompts, request ids
and hashes are in `assets/cordelia/provenance.json`.
