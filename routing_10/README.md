# ROUTING_10

The demoscene frame around the SolverForge Routing's entry video for the
Hermes Agent Plugin Hackathon. 56 seconds at 1920x1080, 60 fps:

- **Intro (0–16 s).** The install line is typed, then a road web fades in.
  Ten stops land on it one by one and join in the given order in rose while
  the kilometre counter ticks with the drawn polyline; the loop then redraws
  one hop at a time into the solved order in lime, and the counter settles
  from the crisscrossed total to the loop's. All of it is computed from the
  polylines on screen, through one pixels-to-kilometres factor, so the
  number can never disagree with the picture.
- **Live (16–44 s).** One 1488x800 bezel opens over rolling copper bars,
  with the statue from Hermes Desktop's backdrop barely visible underneath:
  the window the Desktop take plays in. Left black for compositing.
- **Outro (44–56 s).** The logo draws itself, the guarantees run on the
  cards, and the repository address stays up to the last frame.

The soundtrack is synthwave in F harmonic minor (Fm – D♭ – A♭ – C). It drops
to pads, a held bass and light hats under the narration, and brings the hook
back in the outro. Each stop landing in the intro is a blip in the mix.

## Run and render

```sh
make run       # window with live audio; SPACE jumps to the next section
make render    # routing_10.mp4 with audio, via ffmpeg
make check     # cargo check
make clippy
```

`routing-10-demo --live` prints when the live section runs and where the
take window is (`take`, as `x y w h`), for compositing.

## Post-production

The final video assembles from parts this repo's pipeline draws and records
separately:

1. The Desktop take is a scripted CDP drive of the real plugin in the real
   app: the route pane, the QR handoff (phone kind picked before the scan),
   the street runs, the pane scroll — captured as a frame sequence, every
   click asserted against the DOM so the take can never be a frozen lie.
2. The frame sequence becomes `take.mp4`, plays in the live window from
   16 s, and the edge-tts narration runs over it with captions burned in,
   the demo's own soundtrack ducked under it.
3. The demo renders headless (`make render`) with its soundtrack, the take
   composites into the live window with ffmpeg, and the result is the
   final video.

Covers stay in `docs/social/` under the repo's 9:16 convention
(`cover-9x16.html` and `routing_10-cover-1080x1920.png`).
