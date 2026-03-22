# Repository Guidelines

## Project Structure & Module Organization
This repository contains standalone Rust demo crates, not a shared workspace. Top-level directories map to products or releases: `planner123_01/`, `serio_01/`, `serio_02/`, and `screensaver_03/`. Each crate keeps runtime code in `src/` with modules grouped by concern such as `effects/`, `audio/`, and `orrery/`. Embedded assets stay next to the code that uses them, for example `planner123_01/src/effects/screenshots/*.png`. Build outputs land in the root `target/` directory.

## Build, Test, and Development Commands
Run commands from the specific demo directory you are changing.

- `make help`: list supported targets and controls.
- `make run`: build and launch the demo window.
- `make render`: render the headless capture to MP4 through `ffmpeg`.
- `make render-gif`: produce a shareable GIF render.
- `make check`: run `cargo check --release` for fast compile validation.
- `make clippy`: run `cargo clippy --release -- -W clippy::all`.
- `cargo install --path .` in `screensaver_03/`: install the `solverforge-screensaver` binary locally.

Example: `cd serio_02 && make check && make clippy`.

## Coding Style & Naming Conventions
Follow idiomatic Rust 2021 style: 4-space indentation, `snake_case` for functions/modules, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep modules narrowly scoped by effect or subsystem. Prefer descriptive filenames such as `starfield.rs`, `score_display.rs`, and `orrery_render.rs`. There is no repo-level formatter config yet, so use standard `cargo fmt` defaults before submitting changes even though it is not wrapped by the Makefiles.

## Testing Guidelines
There are currently no committed unit-test modules in this repository. Minimum validation is:

- `make check`
- `make clippy`
- `make run` for interactive changes
- `make render` when changing timing, visuals, or audio/export paths

If you add tests, keep them close to the code under test with Rust’s usual `#[cfg(test)]` conventions.

## Commit & Pull Request Guidelines
Recent history uses short imperative subjects with prefixes like `feat:` and `refactor:`. Match that pattern, for example `feat: add commentary effect to serio_02`. PRs should state which demo changed, summarize visible or audio behavior changes, list validation commands run, and include screenshots or rendered clips for UI/visual changes. Link the relevant issue when one exists.
