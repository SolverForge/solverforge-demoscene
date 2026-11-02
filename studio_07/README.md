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
- **Four: Redona Broadcasting.** Video for shops in Bergamo: phone clips sent on WhatsApp come back as videos ready to post, each checked first. In Redona's own colours, Bergamo's civic red and yellow.
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
| 44-52 | 1:28 | Redona | Phones film, clips fly to the Redona TV, the video plays and is checked |
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

Synthesized sample by sample (`src/audio/song.rs`) into one buffer at startup; live playback and the WAV export read the same samples, and the audio playhead is the demo clock in the window.

- 120 BPM, A minor, Am - F - C - G, one chord per bar; a two-octave pad that ducks under the kick, a pulse arpeggio through a ping-pong delay, saw/square bass, synthesized kick, snare, claps and hats.
- The arrangement follows the sections bar for bar: pad and arpeggio for the name, a half-time kick over the city, the full groove for the route search, a bell for the advisory blueprint, a saw lead for the agents, claps, octave bass and a square lead for Redona, everything for the scroller, the bell over the pad for the address.
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
