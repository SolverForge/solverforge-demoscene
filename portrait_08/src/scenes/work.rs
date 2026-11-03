// WORK: the six strings, ringing with the riff, come loose and become six
// routes out of one depot; then a plain local search untangles them (move
// a stop, swap two, reverse a stretch -- kept only if the routes get
// shorter). Every frame of the untangling is a kept move.
use crate::gfx::{hash, mix, Frame};
use crate::score::{strings_at, Note};
use crate::timeline::{smooth, Now};

const VANS: usize = 6;
const STOPS: usize = 42;
const CAP: usize = 8;
const BOARD: (f32, f32, f32, f32) = (110.0, 480.0, 970.0, 1560.0);
const DEPOT: (f32, f32) = (540.0, 1020.0);
const MORPH_FROM: f32 = 1.6;
const MORPH_TO: f32 = 4.0;
const SEARCH_TO: f32 = 12.4;
const SILVER: u32 = 0xd6d6d0;
const MINT: u32 = 0x6ee7b7;

pub struct Routes {
    stops: Vec<(f32, f32)>,
    steps: Vec<Vec<Vec<usize>>>,
}

impl Routes {
    pub fn solve() -> Self {
        let (x0, y0, x1, y1) = BOARD;
        let stops: Vec<(f32, f32)> = (0..STOPS)
            .map(|i| {
                let a = hash(i as i32, 3) * std::f32::consts::TAU;
                let r = 0.25 + 0.75 * hash(i as i32, 4).sqrt();
                let u = 0.5 + 0.5 * r * a.cos();
                let v = 0.5 + 0.5 * r * a.sin();
                (x0 + (x1 - x0) * u, y0 + (y1 - y0) * v)
            })
            .collect();
        let mut rng = Rng(0x0808_5eed);
        let mut order: Vec<usize> = (0..STOPS).collect();
        for i in (1..STOPS).rev() {
            order.swap(i, rng.below(i + 1));
        }
        let mut routes = vec![Vec::new(); VANS];
        for (k, s) in order.into_iter().enumerate() {
            routes[k % VANS].push(s);
        }
        let mut me = Self {
            stops,
            steps: vec![routes.clone()],
        };
        let mut best = me.length(&routes);
        for _ in 0..300_000 {
            let mut cand = routes.clone();
            let ok = match rng.below(3) {
                0 => relocate(&mut cand, &mut rng),
                1 => swap(&mut cand, &mut rng),
                _ => two_opt(&mut cand, &mut rng),
            };
            if !ok || cand.iter().any(|r| r.len() > CAP) {
                continue;
            }
            let len = me.length(&cand);
            if len < best - 0.01 {
                best = len;
                routes = cand;
                me.steps.push(routes.clone());
            }
        }
        me
    }

    fn length(&self, routes: &[Vec<usize>]) -> f32 {
        routes
            .iter()
            .map(|r| {
                let mut prev = DEPOT;
                let mut sum = 0.0;
                for &s in r {
                    sum += dist(prev, self.stops[s]);
                    prev = self.stops[s];
                }
                sum + dist(prev, DEPOT)
            })
            .sum()
    }

    fn polyline(&self, route: &[usize]) -> Vec<(f32, f32)> {
        let mut pts = vec![DEPOT];
        pts.extend(route.iter().map(|&s| self.stops[s]));
        pts.push(DEPOT);
        pts
    }
}

pub fn draw(f: &mut Frame, notes: &[Note], routes: &Routes, n: Now, t: f32) -> f32 {
    let fade = n.envelope(0.8, 0.8);
    f.gradient(&[0x050807, 0x07100c, 0x050907, 0x030403]);
    let l = n.local;
    let morph = smooth((l - MORPH_FROM) / (MORPH_TO - MORPH_FROM));
    let k_search = ((l - MORPH_TO) / (SEARCH_TO - MORPH_TO)).clamp(0.0, 1.0);
    let step = ((routes.steps.len() - 1) as f32 * k_search.powf(0.6)) as usize;
    let snap = &routes.steps[step];
    let ring = strings_at(notes, t);
    let green = smooth((l - MORPH_TO) / 3.0);

    for (i, route) in snap.iter().enumerate() {
        let target = resample(&routes.polyline(route), 80);
        // String i as a vertical line, swinging with its note.
        let sx = 290.0 + i as f32 * 100.0;
        let amp = ring[i].0 * 10.0 * (1.0 - morph);
        let string: Vec<(f32, f32)> = (0..80)
            .map(|k| {
                let u = k as f32 / 79.0;
                (
                    sx + amp * (std::f32::consts::PI * u).sin() * (t * 40.0 + i as f32).sin(),
                    480.0 + u * 1080.0,
                )
            })
            .collect();
        let c = mix(SILVER, MINT, green);
        for k in 0..79 {
            let p = |j: usize| {
                let (a, b) = (string[j], target[j]);
                (a.0 + (b.0 - a.0) * morph, a.1 + (b.1 - a.1) * morph)
            };
            let (p0, p1) = (p(k), p(k + 1));
            f.line(p0.0, p0.1, p1.0, p1.1, 2.4, c, fade, 1.0);
        }
    }
    let dots = smooth((l - MORPH_TO + 0.6) / 0.8) * fade;
    for &s in &routes.stops {
        f.disc(s.0, s.1, 7.0, MINT, dots);
        f.disc(s.0, s.1, 3.0, 0x050807, dots);
    }
    f.glow(DEPOT.0, DEPOT.1, 50.0, 0x10b981, 0.4 * dots);
    f.rect(
        DEPOT.0 as i32 - 10,
        DEPOT.1 as i32 - 10,
        DEPOT.0 as i32 + 10,
        DEPOT.1 as i32 + 10,
        0xecfdf5,
        dots,
    );
    1.0
}

/// Points evenly spaced along a polyline.
fn resample(pts: &[(f32, f32)], n: usize) -> Vec<(f32, f32)> {
    let total: f32 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
    (0..n)
        .map(|k| {
            let mut d = total * k as f32 / (n - 1) as f32;
            for w in pts.windows(2) {
                let len = dist(w[0], w[1]);
                if d <= len {
                    let u = d / len.max(1e-6);
                    return (
                        w[0].0 + (w[1].0 - w[0].0) * u,
                        w[0].1 + (w[1].1 - w[0].1) * u,
                    );
                }
                d -= len;
            }
            *pts.last().unwrap()
        })
        .collect()
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn relocate(r: &mut [Vec<usize>], rng: &mut Rng) -> bool {
    let from = rng.below(VANS);
    if r[from].is_empty() {
        return false;
    }
    let i = rng.below(r[from].len());
    let s = r[from].remove(i);
    let to = rng.below(VANS);
    let j = rng.below(r[to].len() + 1);
    r[to].insert(j, s);
    !(from == to && i == j)
}

fn swap(r: &mut [Vec<usize>], rng: &mut Rng) -> bool {
    let (a, b) = (rng.below(VANS), rng.below(VANS));
    if r[a].is_empty() || r[b].is_empty() {
        return false;
    }
    let (i, j) = (rng.below(r[a].len()), rng.below(r[b].len()));
    if a == b && i == j {
        return false;
    }
    let tmp = r[a][i];
    r[a][i] = r[b][j];
    r[b][j] = tmp;
    true
}

fn two_opt(r: &mut [Vec<usize>], rng: &mut Rng) -> bool {
    let v = rng.below(VANS);
    let n = r[v].len();
    if n < 3 {
        return false;
    }
    let i = rng.below(n - 1);
    let j = i + 1 + rng.below(n - i - 1);
    r[v][i..=j].reverse();
    true
}

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}
