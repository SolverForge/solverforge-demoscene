// The route plan of the first offer: 36 stops, 4 vans from one depot, at
// most 10 stops a van. It starts from a careless plan and is improved by a
// plain local search: pick a move (move one stop elsewhere, or reverse a
// stretch of one route, or swap two stops), keep it only if the total length goes down and no
// van is over capacity. Every kept move is a snapshot the picture steps
// through; `length()` is both the search's yardstick and the panel's number.

use crate::gfx::hash;

pub const STOPS: usize = 36;
pub const VANS: usize = 4;
pub const CAPACITY: usize = 10;

/// The board the stops sit on, in screen pixels.
pub const BOARD: (f32, f32, f32, f32) = (150.0, 150.0, 900.0, 640.0);

#[derive(Clone)]
pub struct Snapshot {
    pub routes: Vec<Vec<usize>>,
    /// The stop the move touched, for the flash.
    pub moved: Option<usize>,
}

pub struct Plan {
    pub depot: (f32, f32),
    pub stops: Vec<(f32, f32)>,
    pub steps: Vec<Snapshot>,
}

impl Plan {
    pub fn solve() -> Self {
        let (x0, y0, x1, y1) = BOARD;
        let depot = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        // Stops in four loose neighbourhoods plus a scatter, so a good plan
        // is visibly better than a bad one.
        let hubs = [(0.22, 0.25), (0.78, 0.22), (0.25, 0.8), (0.8, 0.75)];
        let stops: Vec<(f32, f32)> = (0..STOPS)
            .map(|i| {
                let (hx, hy) = hubs[i % 4];
                let spread = if i % 9 == 8 { 0.45 } else { 0.16 };
                let a = hash(i as i32, 11) * std::f32::consts::TAU;
                let r = hash(i as i32, 23).sqrt() * spread;
                let u = (hx + a.cos() * r).clamp(0.04, 0.96);
                let v = (hy + a.sin() * r * 1.1).clamp(0.04, 0.96);
                (x0 + (x1 - x0) * u, y0 + (y1 - y0) * v)
            })
            .collect();

        // The careless plan: stops dealt to vans in a shuffled order.
        let mut order: Vec<usize> = (0..STOPS).collect();
        let mut rng = Rng(0x5eed_0007);
        for i in (1..STOPS).rev() {
            order.swap(i, rng.below(i + 1));
        }
        let mut routes = vec![Vec::new(); VANS];
        for (k, s) in order.into_iter().enumerate() {
            routes[k % VANS].push(s);
        }

        let mut plan = Plan {
            depot,
            stops,
            steps: Vec::new(),
        };
        plan.steps.push(Snapshot {
            routes: routes.clone(),
            moved: None,
        });

        let mut best = plan.length(&routes);
        for _ in 0..400_000 {
            let mut cand = routes.clone();
            let moved = match rng.below(3) {
                0 => relocate(&mut cand, &mut rng),
                1 => swap(&mut cand, &mut rng),
                _ => two_opt(&mut cand, &mut rng),
            };
            let Some(moved) = moved else { continue };
            if cand.iter().any(|r| r.len() > CAPACITY) {
                continue;
            }
            let len = plan.length(&cand);
            if len < best - 0.01 {
                best = len;
                routes = cand;
                plan.steps.push(Snapshot {
                    routes: routes.clone(),
                    moved: Some(moved),
                });
            }
        }
        plan
    }

    pub fn pos(&self, stop: usize) -> (f32, f32) {
        self.stops[stop]
    }

    /// Total driven length: depot, every stop in order, back to the depot.
    pub fn length(&self, routes: &[Vec<usize>]) -> f32 {
        routes.iter().map(|r| self.route_length(r)).sum()
    }

    pub fn route_length(&self, route: &[usize]) -> f32 {
        if route.is_empty() {
            return 0.0;
        }
        let mut prev = self.depot;
        let mut sum = 0.0;
        for &s in route {
            sum += dist(prev, self.stops[s]);
            prev = self.stops[s];
        }
        sum + dist(prev, self.depot)
    }
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// Take one stop out of a route and put it anywhere in any route.
fn relocate(routes: &mut [Vec<usize>], rng: &mut Rng) -> Option<usize> {
    let from = rng.below(VANS);
    if routes[from].is_empty() {
        return None;
    }
    let i = rng.below(routes[from].len());
    let stop = routes[from].remove(i);
    let to = rng.below(VANS);
    let j = rng.below(routes[to].len() + 1);
    if from == to && i == j {
        return None;
    }
    routes[to].insert(j, stop);
    Some(stop)
}

/// Exchange two stops between (or within) routes.
fn swap(routes: &mut [Vec<usize>], rng: &mut Rng) -> Option<usize> {
    let (a, b) = (rng.below(VANS), rng.below(VANS));
    if routes[a].is_empty() || routes[b].is_empty() {
        return None;
    }
    let (i, j) = (rng.below(routes[a].len()), rng.below(routes[b].len()));
    if a == b && i == j {
        return None;
    }
    let (sa, sb) = (routes[a][i], routes[b][j]);
    routes[a][i] = sb;
    routes[b][j] = sa;
    Some(sa)
}

/// Reverse a stretch of one route (undoes a crossing).
fn two_opt(routes: &mut [Vec<usize>], rng: &mut Rng) -> Option<usize> {
    let r = rng.below(VANS);
    let n = routes[r].len();
    if n < 3 {
        return None;
    }
    let i = rng.below(n - 1);
    let j = i + 1 + rng.below(n - i - 1);
    routes[r][i..=j].reverse();
    Some(routes[r][i])
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
