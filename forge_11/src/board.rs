// The plan as a textmode Gantt board: one row per technician, one column
// per day, every job a block labelled with its bus. One layout serves both
// the solved plan and the repaired one: each cell keeps its solved jobs in
// place and appends the jobs that move in, so an untouched block never
// shifts while the changed ones travel.
use std::collections::HashMap;

use crate::capture::{Assignment, Capture};
use crate::gfx::{clip, mix, scale, size, Frame, Size, S1};

pub const AREA: (i32, i32, i32, i32) = (760, 262, 1860, 900);
const LABEL_W: i32 = 190;
const HEADER_H: i32 = 34;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Rect {
    pub fn lerp(&self, o: &Rect, k: f32) -> Rect {
        let l = |a: f32, b: f32| a + (b - a) * k;
        Rect {
            x0: l(self.x0, o.x0),
            y0: l(self.y0, o.y0),
            x1: l(self.x1, o.x1),
            y1: l(self.y1, o.y1),
        }
    }
    pub fn centre(&self) -> (f32, f32) {
        ((self.x0 + self.x1) * 0.5, (self.y0 + self.y1) * 0.5)
    }
}

pub struct Block {
    pub job: String,
    pub bus: String,
    pub solved: Option<Rect>,
    pub repaired: Option<Rect>,
    /// Index into the repair's `changed` list.
    pub change: Option<usize>,
}

pub struct Board {
    pub techs: Vec<String>,
    pub days: Vec<i64>,
    /// Solved-plan blocks in drop order (day, technician, job), then any
    /// block that only exists in the repaired plan.
    pub blocks: Vec<Block>,
    /// Number of blocks in the solved plan.
    pub solved_count: usize,
    /// The technician row the disruption takes out, if it names one.
    pub struck_tech: Option<usize>,
    /// The bus the disruption takes out, if it names one.
    pub struck_bus: Option<String>,
    row_h: f32,
    col_w: f32,
}

impl Board {
    pub fn new(c: &Capture) -> Self {
        let solved = &c.solve.plan;
        let repaired = &c.repair.solve.plan;
        let mut techs: Vec<String> = Vec::new();
        let mut days: Vec<i64> = Vec::new();
        for a in solved.iter().chain(repaired) {
            if !techs.contains(&a.technician) {
                techs.push(a.technician.clone());
            }
            if !days.contains(&a.day) {
                days.push(a.day);
            }
        }
        for ch in &c.repair.changed {
            for s in [&ch.from, &ch.to] {
                if !techs.contains(&s.technician) {
                    techs.push(s.technician.clone());
                }
                if !days.contains(&s.day) {
                    days.push(s.day);
                }
            }
        }
        techs.sort();
        days.sort();
        let ti = |a: &Assignment| techs.iter().position(|t| *t == a.technician).unwrap();
        let di = |a: &Assignment| days.iter().position(|d| *d == a.day).unwrap();

        // Cell lists: solved jobs first, then arrivals.
        let mut order: Vec<&Assignment> = solved.iter().collect();
        order.sort_by(|a, b| (a.day, ti(a), &a.job).cmp(&(b.day, ti(b), &b.job)));
        let mut cells: HashMap<(usize, usize), Vec<String>> = HashMap::new();
        for a in &order {
            cells.entry((ti(a), di(a))).or_default().push(a.job.clone());
        }
        let solved_slot: HashMap<&str, (usize, usize)> = order
            .iter()
            .map(|a| (a.job.as_str(), (ti(a), di(a))))
            .collect();
        let mut repaired_cell: HashMap<&str, (usize, usize)> = HashMap::new();
        for a in repaired {
            let cell = (ti(a), di(a));
            repaired_cell.insert(a.job.as_str(), cell);
            if solved_slot.get(a.job.as_str()) != Some(&cell) {
                cells.entry(cell).or_default().push(a.job.clone());
            }
        }

        let (x0, y0, x1, y1) = AREA;
        let row_h = ((y1 - y0 - HEADER_H) as f32 / techs.len().max(1) as f32).min(150.0);
        let col_w = (x1 - x0 - LABEL_W) as f32 / days.len().max(1) as f32;
        let rect_of = |cell: (usize, usize), job: &str| -> Rect {
            // A cell's jobs stack as rows while each keeps a label's
            // height, then spread into columns.
            let list = &cells[&cell];
            let k = list.iter().position(|j| j == job).unwrap_or(0);
            let n = list.len().max(1);
            let (iw, ih) = (col_w - 8.0, row_h - 10.0);
            let fit = ((ih / 24.0).floor() as usize).max(1);
            let rows = n.min(fit);
            let cols = n.div_ceil(rows);
            let (w, h) = (iw / cols as f32, ih / rows as f32);
            let (r, c) = (k % rows, k / rows);
            let cx0 = (x0 + LABEL_W) as f32 + cell.1 as f32 * col_w + 4.0;
            let cy0 = (y0 + HEADER_H) as f32 + cell.0 as f32 * row_h + 5.0;
            Rect {
                x0: cx0 + c as f32 * w + 1.0,
                y0: cy0 + r as f32 * h + 1.0,
                x1: cx0 + (c + 1) as f32 * w - 2.0,
                y1: cy0 + (r + 1) as f32 * h - 2.0,
            }
        };

        let change_of = |job: &str| c.repair.changed.iter().position(|ch| ch.job == job);
        let mut blocks: Vec<Block> = order
            .iter()
            .map(|a| Block {
                job: a.job.clone(),
                bus: a.bus.clone(),
                solved: Some(rect_of((ti(a), di(a)), &a.job)),
                repaired: repaired_cell
                    .get(a.job.as_str())
                    .map(|&cell| rect_of(cell, &a.job)),
                change: change_of(&a.job),
            })
            .collect();
        let solved_count = blocks.len();
        for a in repaired {
            if !solved_slot.contains_key(a.job.as_str()) {
                blocks.push(Block {
                    job: a.job.clone(),
                    bus: a.bus.clone(),
                    solved: None,
                    repaired: Some(rect_of((ti(a), di(a)), &a.job)),
                    change: change_of(&a.job),
                });
            }
        }

        let detail = |k: &str| {
            c.repair
                .detail
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.clone())
        };
        let struck_tech = detail("technician")
            .and_then(|name| techs.iter().position(|t| *t == name))
            .or_else(|| {
                // A technician with solved work and none after the repair.
                techs.iter().position(|t| {
                    solved.iter().any(|a| a.technician == *t)
                        && !repaired.iter().any(|a| a.technician == *t)
                })
            });
        // A bus event names its bus, or brings in that bus's new job.
        let struck_bus = detail("bus").or_else(|| {
            c.repair
                .kind
                .contains("bus")
                .then(|| {
                    blocks
                        .iter()
                        .find(|b| b.solved.is_none())
                        .map(|b| b.bus.clone())
                })
                .flatten()
        });
        Self {
            techs,
            days,
            blocks,
            solved_count,
            struck_tech,
            struck_bus,
            row_h,
            col_w,
        }
    }

    /// Jobs that exist only in the repaired plan (the disruption's own).
    pub fn added(&self) -> usize {
        self.blocks.iter().filter(|b| b.solved.is_none()).count()
    }

    /// The board's lower edge in pixels.
    pub fn bottom(&self) -> i32 {
        AREA.1 + HEADER_H + (self.row_h * self.techs.len() as f32) as i32
    }

    pub fn row_rect(&self, i: usize) -> Rect {
        let (x0, y0, x1, _) = AREA;
        let y = (y0 + HEADER_H) as f32 + i as f32 * self.row_h;
        Rect {
            x0: x0 as f32,
            y0: y,
            x1: x1 as f32,
            y1: y + self.row_h,
        }
    }

    /// The grid: day headers, technician labels, cell rules. `struck` paints
    /// the disrupted row out at strength 0..1.
    pub fn draw_grid(&self, f: &mut Frame, a: f32, accent: u32, struck: f32, t: f32) {
        if a <= 0.0 {
            return;
        }
        let (x0, y0, x1, _) = AREA;
        let bottom = y0 + HEADER_H + (self.row_h * self.techs.len() as f32) as i32;
        f.dim(x0, y0, x1, bottom, 1.0 - 0.75 * a);
        let lab = label_size(LABEL_W - 16, 18);
        for (i, d) in self.days.iter().enumerate() {
            let cx = x0 + LABEL_W + (i as f32 * self.col_w) as i32;
            let txt = clip(
                &format!("DAY {d}"),
                ((self.col_w as i32 - 8) / S1.w) as usize,
            );
            f.text(cx + 6, y0 + 8, &txt, S1, accent, a * 0.9);
            f.rect(cx, y0, cx + 1, bottom, accent, a * 0.25);
        }
        for (i, name) in self.techs.iter().enumerate() {
            let r = self.row_rect(i);
            let ry = r.y0 as i32;
            f.rect(x0, ry, x1, ry + 1, accent, a * 0.2);
            if Some(i) == self.struck_tech && struck > 0.0 {
                let k = struck * a;
                f.rect(x0, ry + 1, x1, r.y1 as i32, 0x400008, k * 0.6);
                // Hatched out, textmode.
                let cols = (x1 - x0) / S1.w;
                let rows = ((r.y1 - r.y0) as i32 / S1.h).max(1);
                for rr in 0..rows {
                    for cc in 0..cols {
                        if (cc + rr * 2 + (t * 8.0) as i32) % 6 == 0 {
                            f.char_at(
                                x0 + cc * S1.w,
                                ry + 2 + rr * S1.h,
                                '/',
                                S1,
                                0xff3b4a,
                                k * 0.5,
                            );
                        }
                    }
                }
            }
            let txt = clip(name, ((LABEL_W - 16) / lab.w) as usize);
            let ty = ry + ((self.row_h as i32 - lab.h) / 2).max(0);
            let struck_here = Some(i) == self.struck_tech && struck > 0.0;
            let col = if struck_here {
                mix(0xd8e2ea, 0xff3b4a, struck)
            } else {
                0xd8e2ea
            };
            f.text(x0 + 10, ty, &txt, lab, col, a);
            if struck_here {
                let w = txt.chars().count() as i32 * lab.w;
                f.rect(
                    x0 + 6,
                    ty + lab.h / 2,
                    x0 + 14 + (w as f32 * struck) as i32,
                    ty + lab.h / 2 + 3,
                    0xff3b4a,
                    a,
                );
            }
        }
        f.rect(x0, bottom, x1, bottom + 1, accent, a * 0.2);
    }

    /// One job block: a filled bar with its bus label (and job when tall).
    pub fn draw_block(f: &mut Frame, r: &Rect, b: &Block, rgb: u32, a: f32) {
        if a <= 0.0 {
            return;
        }
        let (x0, y0, x1, y1) = (r.x0 as i32, r.y0 as i32, r.x1 as i32, r.y1 as i32);
        if x1 - x0 < 3 || y1 - y0 < 3 {
            return;
        }
        f.rect(x0, y0, x1, y1, scale(rgb, 0.32), a * 0.9);
        f.rect(x0, y0, x1, y0 + 2, rgb, a);
        f.rect(x0, y0, x0 + 2, y1, rgb, a * 0.8);
        let s = label_size(x1 - x0 - 6, 18);
        let cols = ((x1 - x0 - 6) / s.w) as usize;
        if cols < 2 || y1 - y0 < s.h + 4 {
            return;
        }
        let bus = short(&b.bus, cols);
        let ty = y0 + ((y1 - y0 - s.h) / 2).min(5);
        f.text(x0 + 5, ty, &bus, s, 0xf4fff0, a);
        let job_col = mix(rgb, 0xffffff, 0.45);
        if y1 - y0 >= 2 * s.h + 10 {
            f.text(
                x0 + 5,
                ty + 3 + s.h,
                &clip(&b.job, cols),
                s,
                job_col,
                a * 0.8,
            );
        } else if cols >= bus.len() + 9 {
            // One line: the job follows the bus.
            let x = x0 + 5 + (bus.len() as i32 + 1) * s.w;
            f.text(
                x,
                ty,
                &clip(&b.job, cols - bus.len() - 1),
                s,
                job_col,
                a * 0.8,
            );
        }
    }
}

/// Text size for a label in `w` pixels: 1x, or smaller cells when a label
/// would not fit at all.
fn label_size(w: i32, min_cols: i32) -> Size {
    if w / S1.w >= min_cols.min(6) {
        S1
    } else {
        size(8, 15)
    }
}

/// Fit a label to `cols`: keep the tail when cutting (ids differ at the end).
pub fn short(s: &str, cols: usize) -> String {
    let n = s.chars().count();
    if n <= cols {
        s.to_string()
    } else {
        s.chars().skip(n - cols).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> (Capture, Board) {
        let c = Capture::parse(include_str!("../assets/capture.placeholder.json")).unwrap();
        let b = Board::new(&c);
        (c, b)
    }

    /// The placeholder, and the real capture once it has been copied in.
    fn boards() -> Vec<(Capture, Board)> {
        let mut v = vec![board()];
        let real = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");
        if std::path::Path::new(real).exists() {
            let c = Capture::load(real).unwrap();
            let b = Board::new(&c);
            v.push((c, b));
        }
        v
    }

    #[test]
    fn untouched_blocks_stay_put_and_changed_ones_move() {
        for (c, b) in boards() {
            assert_eq!(b.solved_count, c.solve.plan.len());
            let mut moved = 0;
            let mut stayed = 0;
            for blk in &b.blocks {
                match (blk.change, blk.solved) {
                    (Some(_), _) => {
                        moved += 1;
                        assert_ne!(blk.solved, blk.repaired, "{} should move", blk.job);
                    }
                    (None, Some(_)) => {
                        stayed += 1;
                        assert_eq!(blk.solved, blk.repaired, "{} should stay", blk.job);
                    }
                    (None, None) => assert!(blk.repaired.is_some()),
                }
            }
            assert_eq!(moved, c.repair.changed.len());
            assert_eq!(c.repair.unchanged_count, stayed);
            assert_eq!(
                b.blocks.len(),
                c.repair.solve.plan.len().max(c.solve.plan.len())
            );
        }
    }

    #[test]
    fn blocks_stay_inside_the_board() {
        for (_, b) in boards() {
            let (x0, y0, x1, y1) = AREA;
            for blk in &b.blocks {
                for r in [blk.solved, blk.repaired].into_iter().flatten() {
                    assert!(r.x0 >= x0 as f32 && r.x1 <= x1 as f32);
                    assert!(r.y0 >= y0 as f32 && r.y1 <= y1 as f32);
                    assert!(r.x1 > r.x0 && r.y1 > r.y0);
                }
            }
        }
    }

    #[test]
    fn the_sick_technician_is_struck() {
        let (_, b) = board();
        assert_eq!(b.techs[b.struck_tech.unwrap()], "PH-TECH-C");
    }
}
