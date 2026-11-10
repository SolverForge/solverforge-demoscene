// Drawing the captions: every line comes from story.rs, every time from
// script.rs. Each caption arrives through a glyphfx entrance, settles into
// plain readable text and fades out before the next one; never two at once
// in the same place. Section openers sit centred under the section's
// large title; lower thirds sit on a scrim over the running scene.
use super::{Timed, EMERALD, LIME_STOPS, MINT};
use crate::gfx::{size, smooth, wrap, Frame, Size, H, W};
use crate::script::{Caption, Place, Script};
use crate::story::Cap;
use crate::timeline::{Section, BEAT};

const CAPTION_STOPS: &[&str] = &["--final-gradient-stops", "d1fae5", "ffffff"];
pub const INTRO_TITLE: Size = size(44, 80);
const INTRO_TEXT: Size = size(26, 46);
const LOWER: Size = size(22, 40);
const CLOSE: Size = size(28, 52);
/// The close sentences sit in the middle of the frame.
const CLOSE_Y: i32 = 440;

struct Item {
    cap: Caption,
    /// One glyphfx clip per line, so every line is centred on its own.
    clips: Vec<Timed>,
    size: Size,
    lines: Vec<String>,
}

impl Item {
    fn draw_lines(&self, f: &mut Frame, t: f32, y: i32, gain: f32) {
        for (j, (line, clip)) in self.lines.iter().zip(&self.clips).enumerate() {
            let w = line.chars().count() as i32 * self.size.w;
            clip.draw(
                f,
                t,
                (W as i32 - w) / 2,
                y + j as i32 * self.size.h,
                self.size,
                gain,
            );
        }
    }
}

pub struct Captions {
    items: Vec<Item>,
    titles: Vec<(Section, Timed)>,
}

fn layout(c: &Caption) -> (Size, Vec<String>) {
    match c.place {
        Place::Intro => (INTRO_TEXT, wrap(&c.text, 62)),
        Place::Lower => (LOWER, wrap(&c.text, 78)),
        Place::Banner => (LOWER, wrap(&c.text, 78)),
        Place::Close => (CLOSE, wrap(&c.text, 62)),
    }
}

impl Captions {
    pub fn new(s: &Script, titles: &[(Section, &str, &str)]) -> Self {
        let items = s
            .captions
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let (size, lines) = layout(c);
                let effect = match c.place {
                    Place::Intro => "decrypt",
                    Place::Lower => "decrypt",
                    Place::Banner => "slide",
                    Place::Close => ["decrypt", "beams", "laseretch"][i % 3],
                };
                let window = match c.place {
                    Place::Lower => 0.5,
                    _ => 0.9,
                };
                // Openers wait for their title to land.
                let from = if c.place == Place::Intro {
                    c.from + 2.0 * BEAT
                } else {
                    c.from
                };
                Item {
                    cap: c.clone(),
                    clips: lines
                        .iter()
                        .enumerate()
                        .map(|(j, l)| {
                            Timed::new(
                                effect,
                                l,
                                3000 + 10 * i as u64 + j as u64,
                                CAPTION_STOPS,
                                from + 0.15 * j as f32,
                                window,
                            )
                        })
                        .collect(),
                    size,
                    lines,
                }
            })
            .collect();
        let titles = titles
            .iter()
            .enumerate()
            .filter_map(|(i, &(sec, text, effect))| {
                let c = s
                    .captions
                    .iter()
                    .find(|c| c.place == Place::Intro && intro_of(c.cap) == Some(sec))?;
                Some((
                    sec,
                    Timed::new(effect, text, 900 + i as u64, LIME_STOPS, c.from, 0.9),
                ))
            })
            .collect();
        Self { items, titles }
    }

    /// Is a section opener on screen at t (the scene waits behind it)?
    pub fn intro_at(&self, t: f32) -> bool {
        self.items
            .iter()
            .any(|i| i.cap.place == Place::Intro && (i.cap.from..i.cap.to).contains(&t))
    }

    pub fn draw(&self, f: &mut Frame, t: f32) {
        for it in &self.items {
            let c = &it.cap;
            if !(c.from..c.to).contains(&t) {
                continue;
            }
            let out = 1.0 - smooth((t - (c.to - 0.4)) / 0.4);
            let cols = it
                .lines
                .iter()
                .map(|l| l.chars().count())
                .max()
                .unwrap_or(0) as i32;
            let w = cols * it.size.w;
            let h = it.lines.len() as i32 * it.size.h;
            let x = (W as i32 - w) / 2;
            match c.place {
                Place::Intro => {
                    // The section's title, large, then the line under it.
                    let sec = intro_of(c.cap);
                    let ty = 330;
                    f.dim(
                        0,
                        ty - 60,
                        W as i32,
                        ty + 120 + 40 + h + 60,
                        1.0 - 0.55 * out,
                    );
                    if let Some((_, title)) = self.titles.iter().find(|(s, _)| Some(*s) == sec) {
                        let tw = title
                            .clip
                            .final_cells()
                            .iter()
                            .map(|c| c.0)
                            .max()
                            .unwrap_or(0) as i32
                            + 1;
                        let tx = (W as i32 - tw * INTRO_TITLE.w) / 2;
                        title.draw(f, t, tx, ty, INTRO_TITLE, out);
                    }
                    let ly = ty + INTRO_TITLE.h + 50;
                    let rule = smooth((t - c.from - 0.3) / 0.6);
                    let half = (W as f32 * 0.32 * rule) as i32;
                    f.rect(
                        W as i32 / 2 - half,
                        ly - 24,
                        W as i32 / 2 + half,
                        ly - 22,
                        EMERALD,
                        0.8 * out,
                    );
                    it.draw_lines(f, t, ly, out);
                }
                Place::Lower => {
                    let pad = 14;
                    let y0 = H as i32 - 26 - h - 2 * pad;
                    let k = smooth((t - c.from) / 0.25) * out;
                    f.dim(0, y0, W as i32, y0 + h + 2 * pad, 1.0 - 0.72 * k);
                    f.rect(0, y0, W as i32, y0 + 2, EMERALD, 0.7 * k);
                    f.rect(x - 30, y0 + pad, x - 24, y0 + pad + h, MINT, k);
                    it.draw_lines(f, t, y0 + pad, out);
                }
                Place::Banner => {
                    it.draw_lines(f, t, 720, out);
                }
                Place::Close => {
                    let y = CLOSE_Y;
                    it.draw_lines(f, t, y, out);
                }
            }
        }
    }
}

/// The section a section-opener caption belongs to.
pub fn intro_of(cap: Cap) -> Option<Section> {
    match cap {
        Cap::Describe => Some(Section::Describe),
        Cap::Generate => Some(Section::Generate),
        Cap::Solve => Some(Section::Solve),
        Cap::Repair => Some(Section::Repair),
        _ => None,
    }
}

/// Where a close sentence's lower edge sits, for its data line.
pub fn close_bottom(s: &Script, i: usize) -> i32 {
    let c = s.captions.iter().filter(|c| c.place == Place::Close).nth(i);
    c.map_or(CLOSE_Y + CLOSE.h, |c| {
        CLOSE_Y + layout(c).1.len() as i32 * CLOSE.h
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::Capture;

    #[test]
    fn captions_fit_and_never_overlap() {
        let mut caps =
            vec![Capture::parse(include_str!("../../assets/capture.placeholder.json")).unwrap()];
        let real = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");
        if std::path::Path::new(real).exists() {
            caps.push(Capture::load(real).unwrap());
        }
        for c in caps {
            let s = Script::new(&c);
            for cap in &s.captions {
                let (size, lines) = layout(cap);
                for l in &lines {
                    assert!(l.chars().count() as i32 * size.w <= W as i32 - 120, "{l}");
                }
                assert!(
                    lines.len() <= 2,
                    "{:?} wraps to {} lines",
                    cap.cap,
                    lines.len()
                );
                assert!(
                    cap.to - cap.from >= crate::story::window(&cap.text) - 0.01,
                    "{:?} up {:.2}s, needs {:.2}s",
                    cap.cap,
                    cap.to - cap.from,
                    crate::story::window(&cap.text)
                );
                assert!(cap.to <= crate::timeline::DURATION);
            }
            for place in [Place::Intro, Place::Lower, Place::Close] {
                let mut v: Vec<&Caption> = s.captions.iter().filter(|c| c.place == place).collect();
                v.sort_by(|a, b| a.from.total_cmp(&b.from));
                for w in v.windows(2) {
                    assert!(
                        w[0].to <= w[1].from + 1e-3,
                        "{:?} overlaps {:?}",
                        w[0].cap,
                        w[1].cap
                    );
                }
            }
        }
    }
}
