# VALBONDIONE_14

Nightfall in the upper Val Seriana. Cordelia ends a day's work in Valbondione,
and Victor walks with her along the Serio, past the falls, the mine and the
lake, to a campfire and the dawn. The film has no words; only its last frame
carries the SolverForge mark and wordmark.
1920x1080, 60 fps, 56 bars at 72 BPM (3:07).

The landscape is textmode: a 192x60 grid of 10x18 cells over smooth pixel
skies and the moon, with Amiga effects (copper bars, plasma, fire, meteors,
searchlights) and glyphfx clips drawn in the same cells. Cordelia and Victor
are pixel sprites in the scenery, walking, standing guard and roasting
marshmallows. RPG portrait windows above them carry their expressions, and
pictograms answer each other in a speech box between them.

- **Owl (4 bars).** Night over the firs, the moon rising out of fog; an owl
  resolves out of noise on a bough, its eyes light amber and it blinks.
- **Village (5 bars).** Valbondione at nightfall under Pizzo Coca: slate
  roofs, lit windows, the bell tower. Cordelia walks out of the village, tired.
- **Crossing (2 bars).** One traffic light swings on its wire. Red; she waits.
  Green; she walks on.
- **Forest (6 bars).** A knight stands at the forest's edge, sword planted.
  Victor joins her and they walk on together.
- **Fireflies (5 bars).** Over the gorge the fireflies join in a tangled
  round that untangles one 2-opt move at a time until the shortest loop glows.
- **Falls (7 bars).** The Barbellino dam opens and the Serio pours down its
  three drops under floodlights, with a moonbow in the spray. The pair watch
  from a granite outcrop above the pool.
- **Mine (5 bars).** Bondione's iron gallery: light spreads through the ore
  veins as a Dijkstra wavefront until the shortest path burns gold to a
  turning geode.
- **Lake (4 bars).** Lake Barbellino under an aurora; the runes of a stone
  ring wake one by one and light closes the ring.
- **Campfire (12 bars).** Victor lays the kindling and the fire takes. Her
  marshmallow catches fire, she blows it out, they laugh, and the synth solo
  climbs over the fire.
- **Dawn (6 bars).** The sky over the Orobie turns rose and gold; she sleeps
  on his shoulder while he keeps watch. The fireflies settle into a ring and
  the SolverForge mark draws itself inside it.

The soundtrack, *Below the Weir*, is written for this film in D minor. Its
melody turns around A-Bb-A and falls through a fourth over a descending bass
(D, C#, C, B, Bb, A). The piano answers the melody with held dyads on beat
three; bells answer on three and one; the kit keeps a half-time pocket
through the falls and opens into a backbeat under the campfire solo, whose
three-note pickup lands on the bar. Every sound is synthesized in
`src/audio/song.rs` (felt piano, wire, bowed strings, glass, bass, bent lead
guitar, kick and snare) and placed in a coupled four-delay stereo room. Nothing
is sampled.

## Run and render

```sh
make run       # window with live audio; SPACE jumps to the next scene
make render    # valbondione_14.mp4 with the score, via ffmpeg
make check
make clippy
make test
```

`--wav SECS` writes the soundtrack alone (`target/demo_audio.wav`),
`--stills t1,t2,.. [--out dir]` writes PPM frames, and `--bench t1,t2,..`
times the scene and the post pass.

## Assets

Portraits and sprites in `assets/` are extracted from on-model generations
of the SolverForge characters; `assets/portraits/provenance.json` records the
sources. The social cover in `docs/social/` uses the demo's own frame at
148 s.
