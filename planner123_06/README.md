```
██████╗ ██╗      █████╗ ███╗   ██╗███╗   ██╗███████╗██████╗  ██╗██████╗ ██████╗
██╔══██╗██║     ██╔══██╗████╗  ██║████╗  ██║██╔════╝██╔══██╗███║╚════██╗╚════██╗
██████╔╝██║     ███████║██╔██╗ ██║██╔██╗ ██║█████╗  ██████╔╝╚██║ █████╔╝ █████╔╝
██╔═══╝ ██║     ██╔══██║██║╚██╗██║██║╚██╗██║██╔══╝  ██╔══██╗ ██║██╔═══╝  ╚═══██╗
██║     ███████╗██║  ██║██║ ╚████║██║ ╚████║███████╗██║  ██║ ██║███████╗██████╔╝
╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═╝╚══════╝╚═════╝  _06
            ── THE DEMO RUNS INSIDE PLANNER123 // TEXTMODE ──
```

A 107-second demo for [Planner123](https://github.com/blackopsrepl/Planner123), the keyboard-driven terminal calendar with a SolverForge planner built in. Planner123 is a ratatui app now, so the whole demo happens inside it: a shell launches the app, and every real key press sets off a demo effect inside the app's own window. The key's chip lights up in the status bar as it happens.

It replaces nothing: [`planner123_01`](../planner123_01/) is the intro for the earlier Tauri version of the app and stays as it was.

## What happens

| Bars | Time | Key | On screen |
|---|---|---|---|
| 0-4 | 0:00 | `planner123` | A CRT powers on; the command is typed at the shell; the app redraws the terminal row by row while its database loads (the status bar's braille spinner) |
| 4-12 | 0:08 | | The week of Sep 14, 2026; the events load in one per eighth note; copper bars run behind the hour grid while the now line swings through the day |
| 12-20 | 0:23 | `1` | Month view: every day cell is a window onto one plasma, pulsing on the kick; the PLANNER123 banner wobbles across |
| 20-24 | 0:38 | `p` | The Planner Inbox fills with five tasks, one per beat; `j` walks the selection down them |
| 24-34 | 0:46 | `o` | The solver runs (spinner), the proposal prints, the panels wipe away to the week, and each task flies from the sidebar queue to its slot, landing on a downbeat with a ripple and a bell |
| 34-38 | 1:05 | `A` | The proposal is written as events in one sweep; copper bars |
| 38-48 | 1:13 | `4` | The agenda scrolls over a starfield; the greetings ride a sine |
| 48-54 | 1:32 | `?` | The help overlay opens; `/credits` is typed into its filter and the rows become the credits; the SolverForge ouroboros draws itself |
| 54-56 | 1:44 | `q` | The app erases itself back to the shell prompt |

## Faithful to the app

Everything on screen is the real Planner123, redrawn: the header row, the 22-column sidebar, the week/month/agenda layouts, the three planner panels and their split (62/38, then 55/45), the help overlay's size, the status-bar chips in keymap order (dropped from the tail on overflow, `? help` pinned), the colours of its theme (`colors.toml`, also the app's fallback), and the app's own strings ("Planner proposal is ready for review; press A to apply."). Every key the demo presses is a binding in the app's current keymap (`src/keys/registry`).

The week and the inbox are the ones from the app's README screenshots. The proposal obeys the planner's real rules: inside the declared hours (Mon-Fri 09:00-17:00), never on top of an existing event, high-load tasks in the 09:00-12:00 focus window, at most one high-load block per day. The flights are choreographed; the demo does not run the solver.

## How it is built

- **ratatui is the canvas.** Each frame is a 128x40 `ratatui::buffer::Buffer`; the views draw into it with ratatui's own `Block`, `Clear` and string widgets, exactly as a terminal app would, and the demo then rasterises the buffer itself: no terminal backend. Each cell is 10x18 pixels (1280x720), and its background fill is drawn as well as its glyph.
- **Glyphs.** ASCII from an 8x8 bitmap font drawn at double height; box-drawing, block, braille (the spinner), `●`, `▸`, `…` and `┊` are generated in code.
- **Post.** Pixel-positioned double-size sprites for the scroller, the ouroboros from the brand SVG's coordinates with anti-aliased signed distances, then phosphor bloom, scanlines and vignette in one fixed-point pass.
- **Determinism.** Every "random" star is a hash of its index; the timeline is pure; the soundtrack is rendered once at startup. Two `--render` runs give the same frames and the same WAV.

```
src/
├── main.rs       window, --render, --stills
├── timeline.rs   125 BPM, 56 bars, the sections, the key presses, the flights
├── data.rs       the palette, the week, the inbox, the proposal, the credits
├── tui.rs        Planner123's chrome: header, sidebar, focused block, status bar
├── views/        week, month, inbox, agenda, help, shell
├── scenes/       which view and effect is on screen when; the flights; the close
├── canvas.rs     grid constants and colour helpers
├── font.rs       glyph masks
├── post.rs       Buffer -> pixels, sprites, logo, bloom
├── logo.rs       the SolverForge ouroboros
├── noise.rs      deterministic hashes
└── audio/        the soundtrack and its playback
```

The font, logo, noise, post-processing and audio engine started as copies of [`hammer_05`](../hammer_05/)'s; the crate shares no code with its siblings.

## Audio

The Planner123 theme, rebuilt: 125 BPM, the Cm - Ab - Bb - G loop, the C5/Bb4 lead riff, the 16th square bass, the arp and the first two phrases of the solo, all from `planner123_01`. Every key the demo presses clicks in the mix, and each task the planner lands rings a bell climbing the C minor chord. The arrangement follows the sections: a pad for the boot, the groove from the month, a breakdown with a muffled riff and a snare roll in the inbox, the driven bass while the planner works, the solo over the agenda.

## Running

```bash
make run          # window + live audio
make render       # planner123_06.mp4 (needs ffmpeg)
make render-gif   # planner123_06.gif
make check        # cargo check --release
make clippy       # cargo clippy --release -- -W clippy::all
```

| Key | Action |
|---|---|
| `ESC` | Quit |
| `SPACE` | Next section |

`planner123-06-demo --stills 10,40,60` writes PPM frames to `target/stills/` for quick review.
