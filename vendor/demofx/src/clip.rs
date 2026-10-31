// glyphfx bridge: run one vendored glyphfx effect to completion at startup and
// keep its frames as sparse cell lists. Deterministic: same spec -> same frames.
use std::ffi::{c_char, c_int, CStr, CString};

use crate::cell::{Cell, CellSink};

#[repr(C)]
struct RawClip {
    _private: [u8; 0],
}

extern "C" {
    fn gfx_open(argc: c_int, argv: *mut *mut c_char, text: *const c_char) -> *mut RawClip;
    fn gfx_next(clip: *mut RawClip) -> *const c_char;
    fn gfx_close(clip: *mut RawClip);
}

pub struct ClipSpec<'a> {
    pub effect: &'a str,
    pub text: &'a str,
    pub cols: usize,
    pub rows: usize,
    pub seed: u64,
    pub opts: &'a [&'a str],
}

/// One frame: the non-blank cells only.
type SparseFrame = Vec<(u16, u16, Cell)>;

pub struct Clip {
    frames: Vec<SparseFrame>,
}

impl Clip {
    pub fn render(spec: &ClipSpec) -> Clip {
        let mut args: Vec<String> = vec![
            "glyphfx".into(),
            "--seed".into(),
            spec.seed.to_string(),
            "--canvas-width".into(),
            spec.cols.to_string(),
            "--canvas-height".into(),
            spec.rows.to_string(),
            "--anchor-text".into(),
            "c".into(),
            "--ignore-terminal-dimensions".into(),
            spec.effect.into(),
        ];
        args.extend(spec.opts.iter().map(|s| s.to_string()));

        let owned: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.as_str()).expect("argv has no NUL"))
            .collect();
        let mut argv: Vec<*mut c_char> = owned.iter().map(|a| a.as_ptr() as *mut c_char).collect();
        let text = CString::new(spec.text).expect("text has no NUL");

        let mut frames = Vec::new();
        // SAFETY: argv/text outlive the clip; frames are copied before the next call.
        unsafe {
            let clip = gfx_open(argv.len() as c_int, argv.as_mut_ptr(), text.as_ptr());
            assert!(!clip.is_null(), "glyphfx rejected clip {:?}", args);
            loop {
                let frame = gfx_next(clip);
                if frame.is_null() {
                    break;
                }
                frames.push(parse_frame(&CStr::from_ptr(frame).to_string_lossy()));
            }
            gfx_close(clip);
        }
        Clip { frames }
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// The settled last frame (e.g. a banner in its final colours).
    pub fn final_cells(&self) -> Vec<(u16, u16, Cell)> {
        self.frames.last().cloned().unwrap_or_default()
    }

    /// Native duration at 60fps.
    pub fn seconds(&self) -> f32 {
        self.frames.len() as f32 / 60.0
    }

    /// Draw frame `idx` (clamped to the final frame) at a cell offset.
    pub fn draw(&self, canvas: &mut impl CellSink, idx: usize, col0: i32, row0: i32, gain: f32) {
        let Some(frame) = self
            .frames
            .get(idx.min(self.frames.len().saturating_sub(1)))
        else {
            return;
        };
        for &(col, row, cell) in frame {
            canvas.set(col0 + col as i32, row0 + row as i32, cell.scaled(gain));
        }
    }

    /// Visit the cells of frame `idx` (clamped to the final frame).
    pub fn for_each_cell(&self, idx: usize, mut f: impl FnMut(u16, u16, Cell)) {
        if let Some(frame) = self
            .frames
            .get(idx.min(self.frames.len().saturating_sub(1)))
        {
            for &(col, row, cell) in frame {
                f(col, row, cell);
            }
        }
    }

    /// Draw by time since the clip started, at a playback speed multiplier.
    pub fn draw_at(
        &self,
        canvas: &mut impl CellSink,
        t: f32,
        speed: f32,
        col0: i32,
        row0: i32,
        gain: f32,
    ) {
        let idx = (t.max(0.0) * 60.0 * speed) as usize;
        self.draw(canvas, idx, col0, row0, gain);
    }
}

/// Parse one glyphfx frame: rows split by '\n', 24-bit SGR colours per cell.
fn parse_frame(frame: &str) -> SparseFrame {
    let mut out = Vec::new();
    for (row, line) in frame.split('\n').enumerate() {
        let mut col = 0u16;
        let mut rgb = 0xC8C8C8u32;
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                let mut seq = String::new();
                for c in chars.by_ref() {
                    if c == 'm' {
                        break;
                    }
                    seq.push(c);
                }
                let params: Vec<u32> = seq
                    .trim_start_matches('[')
                    .split(';')
                    .filter_map(|p| p.parse().ok())
                    .collect();
                rgb = match params.as_slice() {
                    [38, 2, r, g, b, ..] => (r << 16) | (g << 8) | b,
                    _ => 0xC8C8C8,
                };
                continue;
            }
            if ch != ' ' {
                out.push((col, row as u16, Cell::new(ch, rgb)));
            }
            col += 1;
        }
    }
    out
}
