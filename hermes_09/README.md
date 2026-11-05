# HERMES_09

The demoscene frame around the SolverForge Planner's entry video for the
Hermes Agent Plugin Hackathon. 84 seconds at 1920x1080, 60 fps:

- **Intro (0–12 s).** The install line is typed, then a week fills with
  meetings. Five task blocks land on top of them in red with a negative
  score, then slide one by one into free time until the score reads `0hard`.
  The score is recomputed every frame from the blocks on screen.
- **Live (12–72 s).** A bezel opens over rolling copper bars, with the statue
  from Hermes Desktop's backdrop barely visible underneath. The window is left
  black: the narrated Desktop take goes there in compositing.
- **Outro (72–84 s).** The logo draws itself, the guarantees run on the
  cards, and the repository address stays up to the last frame.

The soundtrack is synthwave in D harmonic minor (Dm – B♭ – Gm – A). It drops
to pads, a held bass and light hats under the narration, and brings the hook
back in the outro. Each task block landing in the intro is a blip in the mix.

## Run and render

```sh
make run       # window with live audio; SPACE jumps to the next section
make render    # hermes_09.mp4 with audio, via ffmpeg
make check     # cargo check
make clippy
```

`hermes-09-demo --live` prints where the live take goes, as
`<from s> <to s> <x> <y> <w> <h>`, for the compositing step.

## Post-production

The final video is assembled in the ArtCraft suite:

1. The Desktop takes are recorded with seshat.
2. They are cut to 60 s in FilmCraft (1600x900, 30 fps).
3. seshat narrates the cut and burns in its captions.
4. EffectCraft places the narrated cut over this render at 12 s, centred in
   the window. The take's scale follows the bezel's own opening curve:
   `smooth((t-12)/0.8) * smooth((72-t)/0.8)`.

## Credits

The statue in `assets/hermes_statue.pgm` comes from Hermes Desktop's backdrop
(`apps/desktop/public/ds-assets/filler-bg0.jpg` in the
[Hermes Agent](https://github.com/NousResearch/hermes-agent) repository,
MIT License, © Nous Research). Here it is inverted and stored as a 960x540
greyscale map.
