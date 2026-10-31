//! The text layer shared by the demoscene entries: the vendored glyphfx
//! engine (one copy, `vendor/glyphfx`), text cells, the series' 8x8 font and
//! timed story cards for the pixel demos, and the SolverForge logo.
pub mod cards;
pub mod cell;
pub mod clip;
pub mod font;
pub mod logo;

pub use cards::{card, reading_window, CardSpec, Cards, Pixels, Place, RgbBuffer};
pub use cell::{Cell, CellSink, CELL_H, CELL_W};
pub use clip::{Clip, ClipSpec};
