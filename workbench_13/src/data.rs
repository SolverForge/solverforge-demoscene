// The recorded runs, as the production extracted them (data/*.txt): the
// factory's lanes and operations, the improving plans of the solve and of
// the repair, and the powder stock. Every card, time and score the native
// scenes draw comes from here. Times are minutes from Monday 06:00.
const LANES: &str = include_str!("../data/lanes.txt");
const SOLVE: &str = include_str!("../data/solve.txt");
const REPLAN: &str = include_str!("../data/replan.txt");
const STOCK: &str = include_str!("../data/stock.txt");

pub const DAY: i32 = 24 * 60;
/// Monday 06:00 is minute 0; midnight before Tuesday is minute 1080.
pub const FIRST_MIDNIGHT: i32 = 18 * 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Cut,
    Bend,
    Mill,
    Finish,
    Paint,
    Weld,
    Assemble,
    Inspect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Powder {
    Black,
    Grey,
    Blue,
    Red,
}

impl Powder {
    pub fn rgb(self) -> u32 {
        match self {
            Powder::Black => 0x1b2220,
            Powder::Grey => 0xbfc6c1,
            Powder::Blue => 0x2a5fa8,
            Powder::Red => 0xc2281e,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Powder::Black => "BLACK",
            Powder::Grey => "GREY",
            Powder::Blue => "BLUE",
            Powder::Red => "RED",
        }
    }
}

pub struct Lane {
    pub label: String,
}

pub struct Task {
    pub stage: Stage,
    pub product: String,
    pub paint: Option<Powder>,
}

/// Where an operation runs: lane, start, end.
pub type Slot = (u8, i32, i32);

pub struct Snap {
    /// Seconds into the search.
    pub t: f32,
    pub hard: i32,
    pub medium: i32,
    pub soft: i32,
    pub placed: usize,
    pub pos: Vec<Option<Slot>>,
}

pub struct Use {
    pub task: usize,
    pub qty: i32,
    pub issued: i32,
    pub repaired: i32,
}

pub struct Part {
    pub powder: Powder,
    pub label: String,
    pub init: i32,
    pub late: i32,
    pub uses: Vec<Use>,
}

pub struct Stock {
    pub delivery_from: i32,
    pub delivery_to: i32,
    pub keep: i32,
    pub parts: Vec<Part>,
    pub broken: String,
    pub broken_hard: i32,
    pub moved: usize,
}

pub struct Data {
    pub lanes: Vec<Lane>,
    pub tasks: Vec<Task>,
    pub solve: Vec<Snap>,
    pub replan: Vec<Snap>,
    pub stock: Stock,
}

fn powder(s: &str) -> Option<Powder> {
    match s {
        "black" => Some(Powder::Black),
        "grey" => Some(Powder::Grey),
        "blue" => Some(Powder::Blue),
        "red" => Some(Powder::Red),
        _ => None,
    }
}

fn stage(s: &str) -> Stage {
    match s {
        "cut" => Stage::Cut,
        "bend" => Stage::Bend,
        "mill" => Stage::Mill,
        "finish" => Stage::Finish,
        "paint" => Stage::Paint,
        "weld" => Stage::Weld,
        "assemble" => Stage::Assemble,
        "inspect" => Stage::Inspect,
        other => panic!("unknown stage {other}"),
    }
}

fn snaps(text: &str, tasks: usize) -> Vec<Snap> {
    let mut out: Vec<Snap> = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split(' ').collect();
        if f[0] == "snap" {
            out.push(Snap {
                t: f[1].parse().unwrap(),
                hard: f[2].parse().unwrap(),
                medium: f[3].parse().unwrap(),
                soft: f[4].parse().unwrap(),
                placed: f[5].parse().unwrap(),
                pos: vec![None; tasks],
            });
        } else {
            let s = out.last_mut().expect("a snap header first");
            let task: usize = f[0].parse().unwrap();
            s.pos[task] = Some((
                f[1].parse().unwrap(),
                f[2].parse().unwrap(),
                f[3].parse().unwrap(),
            ));
        }
    }
    out
}

impl Data {
    pub fn load() -> Self {
        let mut lanes = Vec::new();
        let mut tasks = Vec::new();
        for line in LANES.lines() {
            let f: Vec<&str> = line.split('|').collect();
            if f[0] == "task" {
                tasks.push(Task {
                    stage: stage(f[2]),
                    product: f[3].to_uppercase(),
                    paint: powder(f[4]),
                });
            } else {
                lanes.push(Lane {
                    label: f[1].to_string(),
                });
            }
        }
        let n = tasks.len();
        let mut stock = Stock {
            delivery_from: 0,
            delivery_to: 0,
            keep: 0,
            parts: Vec::new(),
            broken: String::new(),
            broken_hard: 0,
            moved: 0,
        };
        for line in STOCK.lines() {
            let f: Vec<&str> = line.split(['|', ' ']).collect();
            match f[0] {
                "delivery" => {
                    stock.delivery_from = f[1].parse().unwrap();
                    stock.delivery_to = f[2].parse().unwrap();
                }
                "keep" => stock.keep = f[1].parse().unwrap(),
                "moved" => stock.moved = f[1].parse().unwrap(),
                _ => {}
            }
        }
        for line in STOCK.lines() {
            let f: Vec<&str> = line.split('|').collect();
            match f[0] {
                "part" => {
                    let colour = f[1].split(' ').nth(2).unwrap_or("");
                    stock.parts.push(Part {
                        powder: powder(colour).expect("powder colour"),
                        label: f[1].trim_end_matches(" powder").to_uppercase(),
                        init: f[2].parse().unwrap(),
                        late: f[3].parse().unwrap(),
                        uses: Vec::new(),
                    })
                }
                "use" => stock.parts.last_mut().unwrap().uses.push(Use {
                    task: f[1].parse().unwrap(),
                    qty: f[2].parse().unwrap(),
                    issued: f[3].parse().unwrap(),
                    repaired: f[4].parse().unwrap(),
                }),
                "broken" => {
                    stock.broken = f[1].to_uppercase();
                    stock.broken_hard = f[2].split("hard").next().unwrap().parse().unwrap();
                }
                _ => {}
            }
        }
        Data {
            lanes,
            solve: snaps(SOLVE, n),
            replan: snaps(REPLAN, n),
            tasks,
            stock,
        }
    }

    /// The week the shop works from: the solve's last plan.
    pub fn issued(&self) -> &Snap {
        self.solve.last().unwrap()
    }

    pub fn repaired(&self) -> &Snap {
        self.replan.last().unwrap()
    }
}

impl Part {
    /// Powder left after every use up to minute `at`, with the late
    /// delivery arriving at `delivery`.
    pub fn level(&self, at: i32, delivery: f32, repaired: bool) -> i32 {
        let mut n = self.init;
        if (at as f32) >= delivery {
            n += self.late;
        }
        for u in &self.uses {
            let when = if repaired { u.repaired } else { u.issued };
            if when <= at {
                n -= u.qty;
            }
        }
        n
    }

    /// Uses that start with nothing to spray: the level after them is below
    /// zero before the delivery has come.
    pub fn short(&self, delivery: f32, repaired: bool) -> Vec<&Use> {
        self.uses
            .iter()
            .filter(|u| {
                let when = if repaired { u.repaired } else { u.issued };
                self.level(when, delivery, repaired) < 0
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_runs_are_the_recorded_ones() {
        let d = Data::load();
        assert_eq!(d.lanes.len(), 24);
        assert_eq!(d.tasks.len(), 409);
        assert_eq!(d.issued().placed, 409);
        assert_eq!((d.issued().hard, d.issued().medium), (0, 0));
        assert_eq!(d.solve[0].placed, 399);
        assert_eq!(d.repaired().placed, 409);
        assert_eq!(d.repaired().hard, 0);
        assert_eq!(d.replan[0].hard, -300);
        assert_eq!(d.stock.broken_hard, -300);
        assert_eq!(d.stock.moved, 96);
    }

    #[test]
    fn the_late_delivery_leaves_five_jobs_without_powder() {
        let d = Data::load();
        let s = &d.stock;
        let on_time: usize = s
            .parts
            .iter()
            .map(|p| p.short(s.delivery_from as f32, false).len())
            .sum();
        let late: usize = s
            .parts
            .iter()
            .map(|p| p.short(s.delivery_to as f32, false).len())
            .sum();
        let repaired: usize = s
            .parts
            .iter()
            .map(|p| p.short(s.delivery_to as f32, true).len())
            .sum();
        assert_eq!((on_time, late, repaired), (0, 5, 0));
    }

    #[test]
    fn nothing_that_started_before_the_cut_moves() {
        let d = Data::load();
        let keep = d.stock.keep;
        let (a, b) = (d.issued(), d.repaired());
        let mut moved = 0;
        for (x, y) in a.pos.iter().zip(&b.pos) {
            if x != y {
                moved += 1;
                assert!(x.unwrap().1 >= keep);
            }
        }
        assert_eq!(moved, 96);
    }
}
