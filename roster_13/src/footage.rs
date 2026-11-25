// The cut Desktop takes (the production's footage.mp4: 1920x1080, 60 fps,
// black where the demo's own scenes play), decoded by ffmpeg frame by frame
// as the demo asks for them. Without footage the takes' place shows a dark
// grid, so the demo still runs on its own.
use std::io::Read;
use std::process::{Child, ChildStdout, Command, Stdio};

use crate::gfx::{Frame, H, W};

pub struct Footage {
    child: Option<(Child, ChildStdout)>,
    path: Option<String>,
    /// Index of the frame in `px`, if any.
    at: Option<usize>,
    buf: Vec<u8>,
    px: Vec<u32>,
}

impl Footage {
    pub fn open(path: Option<String>) -> Self {
        Self {
            child: None,
            path,
            at: None,
            buf: vec![0; W * H * 4],
            px: vec![0; W * H],
        }
    }

    pub fn present(&self) -> bool {
        self.path.is_some()
    }

    fn start(&mut self, from: usize) {
        let Some(path) = &self.path else { return };
        if let Some((mut c, _)) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        let mut child = Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-ss",
                &format!("{:.4}", from as f32 / 60.0),
                "-i",
                path,
            ])
            .args([
                "-f",
                "rawvideo",
                "-pix_fmt",
                "bgra",
                "-s",
                &format!("{W}x{H}"),
                "-r",
                "60",
                "-",
            ])
            .stdout(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()
            .expect("ffmpeg must be on PATH to read the footage");
        let out = child.stdout.take().expect("ffmpeg stdout");
        self.child = Some((child, out));
        self.at = from.checked_sub(1);
    }

    /// Frame `idx` (60 fps) into the cache. Sequential reads stream; a jump
    /// backwards or far ahead restarts the decoder there.
    fn seek(&mut self, idx: usize) {
        if self.at == Some(idx) {
            return;
        }
        let ahead = self.at.map(|a| idx > a && idx - a < 90).unwrap_or(false);
        if self.child.is_none() || !ahead {
            self.start(idx);
        }
        while self.at.is_none_or(|a| a < idx) {
            let Some((_, out)) = &mut self.child else {
                return;
            };
            if out.read_exact(&mut self.buf).is_err() {
                // Past the end: keep the last frame.
                self.at = Some(idx);
                return;
            }
            self.at = Some(self.at.map_or(0, |a| a + 1));
        }
        for (p, c) in self.px.iter_mut().zip(self.buf.as_chunks::<4>().0) {
            *p = (c[2] as u32) << 16 | (c[1] as u32) << 8 | c[0] as u32;
        }
    }

    /// Draw the footage at film time `t` into the frame.
    pub fn draw(&mut self, f: &mut Frame, t: f32) {
        if !self.present() {
            f.clear(0x030806);
            crate::scenes::dot_grid(f, 0x10b981, 0.25);
            return;
        }
        self.seek((t * 60.0).round() as usize);
        f.px.copy_from_slice(&self.px);
    }
}

impl Drop for Footage {
    fn drop(&mut self) {
        if let Some((mut c, _)) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}
