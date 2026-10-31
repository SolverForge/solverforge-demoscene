```
   ███████╗███████╗██████╗ ██╗ ██████╗      ██████╗  ██╗
   ██╔════╝██╔════╝██╔══██╗██║██╔═══██╗    ██╔═████╗███║
   ███████╗█████╗  ██████╔╝██║██║   ██║    ██║██╔██║╚██║
   ╚════██║██╔══╝  ██╔══██╗██║██║   ██║    ████╔╝██║ ██║
   ███████║███████╗██║  ██║██║╚██████╔╝    ╚██████╔╝ ██║
   ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝ ╚═════╝      ╚═════╝  ╚═╝
          ── MUSICA UNIVERSALIS // PLANETS ON ORBITS ──
```

A 112-second demoscene intro for [SolverForge](https://solverforge.com) and SERIO, the incremental scoring engine inside it. Seven planets, 21 orbits, and a real solver scored by `solverforge-scoring`.

## The story

The demo explains itself on cards: one or two plain sentences per moment, timed to what is on screen (`src/story.rs`). Every card stays up long enough to read at a slow pace (entrance, 0.8 s plus 0.25 s a word, exit); demofx refuses a shorter window, and the scenes are as long as their cards need. A glyphfx effect brings the words in, they hold still to be read, then burn out letter by letter over a soft scrim.

- Seven planets circle the sun; each can take any of 21 orbits. The puzzle: give each planet its own orbit so that every pair of orbits is close to a pure ratio (2:1, 3:2, 4:3, ...).
- SolverForge moves one planet at a time and scores the new layout. The simple way re-scores all 21 pairs after every move.
- A move only affects the 6 pairs that include the planet that moved. SERIO re-scores only those 6; the other 15 keep their scores, so both sides reach the same total with 3.5 times less work.
- The search speeds up to 480 moves a second. Gold lines are pairs at a pure ratio, green are close, red are far from one; the score climbs toward zero.
- The solution scene shows the best layout found. With only 21 orbits not every pair can be pure; the score says how close it got.

## Scenes

| # | Scene | Time | What you see |
|---|---|---|---|
| 1 | Genesis | 0 – 15.5s | Sun ignition, seven planets on their orbits. |
| 2 | Classical full evaluation | 15.5 – 37s | One move per frame; every pair line flashes on every move. |
| 3 | Classical vs SERIO | 37 – 65s | Split screen: 21 pair evaluations per move against SERIO's 6, same score. |
| 4 | Harmony | 65 – 88.5s | 8 moves per frame, score graph and constraint bars. |
| 5 | Solution | 88.5 – 104s | The best layout found, FEASIBLE. |
| 6 | Outro | 104 – 112s | Logo, closing card, greetings scroller. |

## Running

```bash
make run          # window + live audio
make render       # serio_01.mp4 (ffmpeg)
make render-gif   # serio_01.gif
make check        # cargo check --release
make clippy       # cargo clippy --release -- -W clippy::all
```

Controls: `ESC` quits, `SPACE` jumps to the next scene. Building needs a C compiler (for glyphfx) besides the Rust toolchain.

## Architecture

```text
src/
├── main.rs        timeline, scene routing, window loop, headless export, audio
├── story.rs       the story cards: text, timing, glyphfx effect, placement
├── orrery/        model (planets, 21 orbit slots), constraints (SERIO), harmony
│                  (Pythagorean intervals), solver (late acceptance, best kept)
├── effects/       genesis, orrery renderer, dual panel, score display,
│                  particles, starfield, plasma, scroller
├── audio/         planetary tones driven by the orbit radii
├── font.rs        8x8 bitmap font
├── logo.rs        the SolverForge ouroboros
└── palette.rs     colours and helpers
```

The cards come from `../vendor/demofx`, on the repository's one vendored copy of [glyphfx](https://github.com/blackopsrepl/glyphfx).

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| [solverforge-scoring](https://crates.io/crates/solverforge-scoring) | =0.5.2 | SERIO incremental constraint scoring |
| [solverforge-core](https://crates.io/crates/solverforge-core) | =0.5.2 | Score types, planning solution trait |
| [minifb](https://crates.io/crates/minifb) | 0.28 | Pixel framebuffer window |
| [fundsp](https://crates.io/crates/fundsp) | 0.23 | Audio synthesis |
| [cpal](https://crates.io/crates/cpal) | 0.17 | Audio output |
| `demofx` (`../vendor/demofx`) | path | Story cards on glyphfx |

## License

MIT — see [LICENSE](../LICENSE).

**SOLVERFORGE // MUSICA UNIVERSALIS // BERGAMO MMXXVI // AMIGA FOREVER**
