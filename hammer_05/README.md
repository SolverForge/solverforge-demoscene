```
 ██╗  ██╗ █████╗ ███╗   ███╗███╗   ███╗███████╗██████╗
 ██║  ██║██╔══██╗████╗ ████║████╗ ████║██╔════╝██╔══██╗
 ███████║███████║██╔████╔██║██╔████╔██║█████╗  ██████╔╝
 ██╔══██║██╔══██║██║╚██╔╝██║██║╚██╔╝██║██╔══╝  ██╔══██╗
 ██║  ██║██║  ██║██║ ╚═╝ ██║██║ ╚═╝ ██║███████╗██║  ██║
 ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝  _05
        ── THE SERIO AND THE FORGE // TEXTMODE ──
```

A 107-second textmode demo for SolverForge. Every pixel on screen is a character.

## The story

The demo tells a true story from the valley above Bergamo, one card of plain sentences per scene, set in the sky above the action.

- The river Serio rises at Valbondione, in the upper Val Seriana. Since 1931 the Barbellino dam holds its water for hydroelectric power; five days a year it is released, and the Cascate del Serio return: 315 metres in three drops. SERIO, SolverForge's scoring engine, shares its name with the river.
- Down the valley, iron from the mines of Bondione and the Val di Scalve was smelted at Gavazzo and refined into steel. At Ponte Nossa, where the Nossa stream joins the Serio, water wheels lifted trip hammers (*magli*) that beat the red-hot steel into nails, spades and hoes. The Maglio Beltrami worked until 1964 and has been a museum since 1984 (via dei Magli 7, Ponte Nossa). The tools were tempered in the same water that drove the hammer.
- SolverForge is in that stage now: consolidating and hardening every part on the way to 1.0.0.

Sources: [Turismo Valbondione](https://turismovalbondione.it/le-cascate-del-serio/), [Associazione Magli Nossa](https://www.maglinossa.it/).

## Scenes

| Bars | Time | Scene | What you see |
|---:|---|---|---|
| 0-4 | 0:00 | Night | The Orobie wall in ASCII relief, stars, a trickle |
| 4-8 | 0:08 | Dam | The water is released; the wall lights up |
| 8-16 | 0:15 | Falls | Three falls, rock shelves, splash and mist |
| 16-20 | 0:31 | River | Flying low down the Val Seriana |
| 20-24 | 0:38 | Iron | Ore gathers over the furnace into a white-hot ingot |
| 24-36 | 0:46 | Ponte Nossa | Water wheel, trip hammer, sparks on every beat; the ingot is thrown apart and reassembled |
| 36-40 | 1:09 | Quench | Steam; the steel cools from orange to emerald |
| 40-52 | 1:17 | On the way to 1.0.0 | The name is laser-etched, then polished |
| 52-56 | 1:40 | Close | The dam closes; the ouroboros; credits |

## How it is made

- **One text grid.** The world is a 128x40 grid of 10x18 cells plus a 64x20 double-size grid for the story text. Mountains, water, the wheel and the hammer are written as characters.
- **Story cards.** Each scene gets one or two sentences in the empty space above the action, over a soft local shadow. A glyphfx effect that belongs to the scene brings the words in (the teletype `print` at night, `rain` over the falls, `burn` at the smelter, `beams` at the forge, `waves` for the quench, `decrypt` for SolverForge); they hold still to be read, then burn out letter by letter. At Ponte Nossa the words flash with each hammer blow. The big moments -- the ingot thrown apart, the laser etching, the logo -- have the screen to themselves.
- **glyphfx as the text engine.** The card entrances and the credits are glyphfx effects, and so is the ingot (`scattered`, `unstable`, `smoke`, `laseretch`, `highlight`). They are real glyphfx effects, compiled from the repository's one vendored copy, `../vendor/glyphfx` (pinned v0.3.4), and run on a virtual 60 fps clock at startup through `../vendor/demofx`.
- **Glyphs.** ASCII comes from the series' 8x8 font at double height; block and box-drawing characters are procedural so the banner joins seamlessly.
- **Phosphor.** Bloom, scanlines and vignette in software.
- **Deterministic.** All randomness is hashed from cell, seed and time; glyphfx is seeded. `--render` output is bit-identical run to run.

## Audio

A total remix of the Planner123 theme, synthesized sample by sample (`src/audio/song.rs`) into one buffer at startup; live playback and the WAV export read the same samples, and the audio playhead is the demo clock.

- **Kept:** 125 BPM, the Cm - Ab - Bb - G loop, the C5/Bb4 lead riff (re-cut to a 3-3-2 stab), the 16th square bass, the arp shape, and the first phrase of the solo.
- **New:** the hook, a two-part melody over the loop (bell in the falls, lead in the hammer and the mark), reharmonised pads (Cm9, Abmaj7, Bb add9, G5 add b7), the water roar driven by the same `flow()` curve as the picture, the anvil on every hammer strike, the steam, and an arrangement that follows the scenes bar for bar.

## Running

```bash
make run          # window + live audio
make render       # hammer_05.mp4 with audio (ffmpeg)
make render-gif   # hammer_05.gif
make check        # cargo check --release
make clippy       # cargo clippy --release -- -W clippy::all
```

Controls: `ESC` quits, `SPACE` jumps to the next section.

Other flags: `--render [seconds]` streams raw BGR24 frames to stdout and writes `target/demo_audio.wav`; `--stills t1,t2,... [--out dir]` writes PPM stills.

Building needs a C compiler (for glyphfx) besides the usual Rust toolchain.

## Architecture

```text
../vendor/glyphfx/  pinned glyphfx sources, LICENSE, NOTICE (shared by every demo)
../vendor/demofx/   glyphfx build + shim, clips parsed into sparse cells, the
                    8x8 font with procedural block/box glyphs
src/
├── main.rs       CLI, window loop (audio playhead = clock), headless export, stills
├── timeline.rs   125 BPM, 56 bars, sections, water flow, hammer strikes
├── scenes.rs     running order: worlds, glyphfx clips, copy, cuts and fades
├── canvas.rs     text grids and colour helpers
├── post.rs       cells -> pixels, logo, bloom, scanlines, vignette
├── noise.rs      hashes, value noise, ASCII shade ramp
├── valley.rs     the wall, the falls, mist, stars
├── river.rs      the river flight
├── forge.rs      furnace, ingot heat, water wheel, trip hammer, sparks, steam
├── story.rs      the story cards and the credits
├── logo.rs       the ouroboros, from the brand SVG, anti-aliased
└── audio/        song.rs (the remix), mod.rs (playback, WAV)
```

## Credits

Text effects: [glyphfx](https://github.com/blackopsrepl/glyphfx), a C port of [ttfx](https://github.com/omacom/ttfx), itself a port of [TerminalTextEffects](https://github.com/ChrisBuilds/terminaltexteffects) by ChrisBuilds. MIT; see `../vendor/glyphfx/LICENSE` and `../vendor/glyphfx/NOTICE`.

## License

MIT -- see [LICENSE](../LICENSE).

**SOLVERFORGE // THE RIVER LIFTS THE HAMMER // BERGAMO MMXXVI // AMIGA FOREVER**
