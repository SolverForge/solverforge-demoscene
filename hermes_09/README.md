# HERMES_09

The demoscene frame around the SolverForge Planner's entry video for the
Hermes Agent Plugin Hackathon. 84 seconds at 1920x1080, 60 fps:

- **Intro (0–12 s).** The install line is typed, then a week fills with
  meetings. Five task blocks land on top of them in red with a negative
  score, then slide one by one into free time until the score reads `0hard`.
  The score is recomputed every frame from the blocks on screen.
- **Live (12–72 s).** Bezels open over rolling copper bars, with the statue
  from Hermes Desktop's backdrop barely visible underneath: a 1440x810 window
  for the Desktop take, and beside it a portrait box and a dialog box for the
  advisor, the Nous Girl pet, who speaks the narration the way SimCity's
  advisor did on the SNES. All three are left black for compositing.
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

`hermes-09-demo --live` prints when the live section runs and where its
windows are (`take`, `portrait`, `dialog`, as `x y w h`), for compositing.

## Post-production

The final video is assembled in the ArtCraft suite:

1. The Desktop takes are recorded with seshat.
2. They are cut to 60 s in FilmCraft (1600x900, 30 fps).
3. seshat narrates the cut in the advisor's voice, without burned-in
   captions: the dialog box carries the words.
4. EffectCraft places the narrated cut over this render at 12 s, centred in
   the take's window, scaled with the bezel's own opening curve
   `smooth((t-12)/0.8) * smooth((72-t)/0.8)`. It animates the Nous Girl from
   her pet spritesheet (idle, a wave as she starts, gestures while she talks,
   a jump on "zero hard"), and types each narrated line into the dialog box
   over the time it is spoken.

## Credits

The advisor is the Nous Girl pet for Hermes Desktop
(`~/.hermes/pets/nous-girl/spritesheet.webp`), after the Nous Research girl
logo.

The statue in `assets/hermes_statue.pgm` comes from Hermes Desktop's backdrop
(`apps/desktop/public/ds-assets/filler-bg0.jpg` in the
[Hermes Agent](https://github.com/NousResearch/hermes-agent) repository,
MIT License, © Nous Research). Here it is inverted and stored as a 960x540
greyscale map.
