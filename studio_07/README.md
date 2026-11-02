```
 ███████╗████████╗██╗   ██╗██████╗ ██╗ ██████╗
 ██╔════╝╚══██╔══╝██║   ██║██╔══██╗██║██╔═══██╗
 ███████╗   ██║   ██║   ██║██║  ██║██║██║   ██║
 ╚════██║   ██║   ██║   ██║██║  ██║██║██║   ██║
 ███████║   ██║   ╚██████╔╝██████╔╝██║╚██████╔╝
 ╚══════╝   ╚═╝    ╚═════╝ ╚═════╝ ╚═╝ ╚═════╝  _07
        ── VITTORIO DISTEFANO // WHAT HE BUILDS FOR YOU ──
```

A 136-second pixel demo about Vittorio Distefano, founder of SolverForge, and the four ways to work with him.

## The story

Told in his own voice, one card of plain sentences per moment:

- **Who.** Vittorio Distefano, a software engineer in Bergamo, Italy, who builds software for hard problems. His portrait prints dot by dot while a shell answers `whoami`.
- **Where.** Bergamo's upper town at night -- the Venetian walls, the Campanone, Santa Maria Maggiore, the Duomo, the Rocca, San Vigilio, the funicular -- and SolverForge's mark over it: an open-source constraint solver with a native Rust core and a Python surface.
- **One: planning systems.** Scheduling, routing and allocation, modeled and built on SolverForge. A route plan (36 stops, 4 vans of 10) starts careless and is improved by a local search, one kept move per step.
- **Two: advisory, for any software project.** Discovery, architecture, code reviews and performance, directly with him. A system sketch sharpens as the steps light up.
- **Three: agents and the tools under them.** MCP servers, terminal apps, native Linux desktop software. A tiling desktop where an agent works through the windows its log describes.
- **Four: developer relations.** If you like how he presents his own work, he can do the same for your project: demos, docs and quickstarts, release posts, videos like this one. A launch kit assembles tile by tile.
- **The proof.** A sine scroller of published projects, and the background before SolverForge.
- **The address.** hello@solverforge.ai, vdistefano.studio, solverforge.org, github.com/blackopsrepl.

## Scenes

| Bars | Time | Scene | What you see |
|---:|---|---|---|
| 0-6 | 0:00 | Boot | The name decrypts, the portrait prints, a shell answers who, where and what |
| 6-14 | 0:12 | City | Bergamo's upper town at night; the SolverForge mark rises |
| 14-27 | 0:28 | Plan | Route plan: careless start, kept moves one by one, length and van loads on the panel |
| 27-35 | 0:54 | Advice | Blueprint: discovery, design, review, tuning, next step; the sketch sharpens |
| 35-44 | 1:10 | Agents | Tiling desktop: planner, mail client, agent log and pointer |
| 44-52 | 1:28 | DevRel | A launch kit: docs, a quickstart that runs, the release post, a demo video |
| 52-60 | 1:44 | Work | Starfield, turning octagon, copper bars, sine scroller of projects |
| 60-68 | 2:00 | Contact | Portrait, name, address, the four offers |

## How it is made

- **Pixels.** A 1280x720 software framebuffer; anti-aliased discs, lines and rings, the series' 8x8 font at integer scales, and a phosphor pass (bloom, scanlines, vignette).
- **The route search is real.** `src/routes.rs` builds the plan at startup and runs a plain local search (move a stop, swap two stops, reverse a stretch); a move is kept only if the total length falls and no van carries more than 10 stops. The picture steps through the kept moves, and every number on the panel -- length, start, saved, moves, stops per van -- is computed from the routes on screen with the same `length()` the search uses.
- **Story cards** come from `../vendor/demofx`: a glyphfx effect brings the words in, they hold for a reading window demofx enforces, then burn out. The name in the boot and contact scenes is a glyphfx `decrypt` clip drawn at 5x and 3x.
- **Portrait.** `assets/portrait.pgm` is Vittorio's public LinkedIn photo, greyscale, 104x122, printed as a halftone (brighter, bigger dot).
- **Logo.** The SolverForge mark from `demofx::logo`: in the top-left corner throughout, stepping aside while the large mark stands over Bergamo.
- **Deterministic.** All randomness is hashed or seeded; `--render` output is identical run to run.

## Audio

Synthwave, synthesized sample by sample (`src/audio/song.rs`) into one buffer at startup; live playback and the WAV export read the same samples, and the audio playhead is the demo clock in the window.

- 120 BPM, A harmonic minor, Am - F - Dm - E, one chord per bar: detuned supersaw pads that duck under the kick, a pulse arpeggio through a ping-pong delay, a gated 80s snare, open hats, tom fills and noise risers into each new section.
- The bass is one gliding voice per bar: it slides in from the previous root and, in the big sections, slides up and down an octave every eighth.
- The leads bend all the time, in the tracker style of the One Must Fall soundtrack: legato notes glide into each other, notes after a rest and held notes scoop up from two semitones below, longer notes grow a vibrato, and every second phrase ends with an octave dive. A supersaw for the route search and developer relations, FM brass for the city, the advisory blueprint and the address, a width-modulated pulse for the agents, supersaw over pulse for the scroller.
- Every kept move of the route search is also a blip in the mix, at the moment it appears on screen (`Demo::move_times`).

## Running

```bash
make run          # window + live audio (ESC quit, SPACE next section)
make render       # studio_07.mp4 with audio (ffmpeg)
make render-gif   # studio_07.gif
make check        # cargo check --release
make clippy       # cargo clippy --release -- -W clippy::all
./target/release/studio-07-demo --stills 30,50 --out target/stills   # PPM stills
```

## Social cover

`docs/social/cover-9x16.html` and `docs/social/studio_07-cover-1080x1920.png`: the portrait halftone, the name, the four offers and the address, inside Instagram's 3:4 grid crop.
