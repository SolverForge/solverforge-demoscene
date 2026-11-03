```
 ██████╗  ██████╗ ██████╗ ████████╗██████╗  █████╗ ██╗████████╗
 ██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝██╔══██╗██╔══██╗██║╚══██╔══╝
 ██████╔╝██║   ██║██████╔╝   ██║   ██████╔╝███████║██║   ██║
 ██╔═══╝ ██║   ██║██╔══██╗   ██║   ██╔══██╗██╔══██║██║   ██║
 ██║     ╚██████╔╝██║  ██║   ██║   ██║  ██║██║  ██║██║   ██║
 ╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝   ╚═╝  _08
              ── VITTORIO DISTEFANO // THE PERSON ──
```

A 140-second vertical pixel demo (1080x1920, a native Instagram reel) about Vittorio Distefano as a person: his guitar, the worlds he has explored, how he takes on hard problems, his machine, what he reads, his home, and then his work.

## The story

Told in his own voice, one card of plain sentences per moment:

- **Who.** Born in 1982, living in Bergamo.
- **The guitar.** His Epiphone Les Paul Black Beauty, close up: pickups, the neck, the headstock, then the whole instrument. Its strings ring with the notes the soundtrack plays.
- **Worlds.** Years in old online worlds -- Vanilla WoW, Lineage II, Warhammer Online. A hooded mage walks toward us down a night road, gathering herbs into his bags; the trade channel scrolls and they are sold. Leave the town you know, map what is blank, come back with something nobody else has.
- **Hard problems.** Something huge in the way: how hard can it be? A small knight against a giant. The first tries are recon (ATTEMPT 1, 2 ... 17); every attack has a tell; read it, wait, move, and it goes down. Then a quiet hall and a typewriter: save, try again.
- **The machine.** Linux, Sway and tmux, set up his way: an editor on this demo's own source, trex, a build, and a mechanical keyboard whose keys go down with every letter typed.
- **Reading.** Dune, read for what Herbert was saying about power, religion and mass movements; the page turns into a desert under two moons. Reputations don't impress him: show me the work.
- **Home.** Bergamo at night through a window; Byron the cat on the sill beside his food tin. Like Byron, he mostly wears black. His reflection comes up in the glass.
- **Work.** The six strings come loose and become six routes out of a depot; a local search untangles them. For work he builds SolverForge -- same habit as everything else: find out how things actually work.
- **The name.** Portrait, name, and four words: engineer, creator, gamer, guitarist.

## Scenes

| Bars | Time | Scene | What you see |
|---:|---|---|---|
| 0-8 | 0:00 | Guitar | The Les Paul from the pickups up the neck to the headstock, then whole |
| 8-20 | 0:16 | Worlds | Night road away from a ruined keep, herbs, bags, professions, trade chat |
| 20-32 | 0:40 | Challenge | Knight against a giant, deaths and an attempt counter, then the typewriter hall |
| 32-42 | 1:04 | Machine | Waybar, tmux panes, trex, a build, the keyboard |
| 42-49 | 1:24 | Reading | Three books under a lamp; Dune opens into the desert |
| 49-57 | 1:38 | Home | Bergamo roofs at night, Byron, the reflection |
| 57-64 | 1:54 | Work | Strings become routes; the routes untangle |
| 64-70 | 2:08 | Name | Portrait, decrypting name, the guitar behind |

## How it is made

- **Pixels.** A 1080x1920 software framebuffer; anti-aliased discs, lines, rings and polygons, the series' 8x8 font, and a phosphor pass (bloom, scanlines, vignette). Story cards sit under Instagram's top bar.
- **The guitar is drawn to the real instrument.** Fret positions follow the 24.75" scale; the body outline and pickguard are traced from a front photograph of a black Les Paul Custom ([Wikimedia Commons, CC0](https://commons.wikimedia.org/wiki/File:Gibson_Custom_50th_Anniversary_1960_Les_Paul_Custom_(clip).jpg)), registered on its nut and bridge and scaled to the 13" x 17.25" body. The three-pickup layout is the Black Beauty's. The painted guitar is rasterised once at startup into a mip chain, so the camera moves cost only a sampled blit; the strings are drawn live on top.
- **One score.** `src/score.rs` holds every guitar note; the audio plays it and the strings on screen ring from it (`strings_at`), so picture and sound cannot drift.
- **The route search is real.** `src/scenes/work.rs` runs a plain local search (move a stop, swap two, reverse a stretch), keeping a move only if the routes get shorter; the picture steps through the kept moves.
- **Story cards** come from `../vendor/demofx`, with the reading window demofx enforces. The name is a glyphfx `decrypt` clip.
- **Portrait.** `assets/portrait.pgm` is Vittorio's public LinkedIn photo, greyscale, 104x122, printed as a halftone.
- **Fast enough to watch live.** Large fills, glows and the post pass run in parallel row bands; the corner logo is composited from a cached copy once it has drawn in. `--bench` times every sixth frame per section against the 16.7 ms budget.
- **Deterministic.** All randomness is hashed or seeded; `--render` output is identical run to run.

## Audio

A guitar instrumental, synthesized sample by sample (`src/audio/song.rs`) into one buffer at startup; live playback and the WAV export read the same samples.

- 120 BPM, E harmonic minor, Em - C - Am - B7, one chord per bar.
- The guitar is a Karplus-Strong plucked string; bends shorten the delay line while the note rings, so the pitch really slides. Clean arpeggios go straight to the mix; power chords, palm mutes and the lead go through drive, soft clipping and a speaker-cabinet filter.
- Under it: a picked bass, a kit (kick, gated snare, hats, toms, crash) with fills into the big sections, and a soft pad.

## Running

```bash
make run          # window + live audio (ESC quit, SPACE next section)
make render       # portrait_08.mp4 with audio (ffmpeg)
make render-gif   # portrait_08.gif
make check        # cargo check --release
make clippy       # cargo clippy --release -- -W clippy::all
./target/release/portrait-08-demo --stills 15.5,104 --out target/stills   # PPM stills
./target/release/portrait-08-demo --bench                                  # frame times per section
```

## Social cover

`docs/social/cover-9x16.html` and `docs/social/portrait_08-cover-1080x1920.png`: the guitar as the demo draws it, the name, where he is from and the closing four words, inside Instagram's 3:4 grid crop.
