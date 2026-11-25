// The fireflies' night, which is the day's work told by nature: a swarm
// over the gorge finds the shortest round between its lights. A random
// round first, then 2-opt local search (reverse the stretch between two
// edges whenever that makes the loop shorter), each improving move kept,
// so the film can replay the search: the lines untangle one move at a
// time until no move helps.

use crate::fx::h2;

pub struct Tour {
    /// Firefly positions in pixels.
    pub pts: Vec<(f32, f32)>,
    /// Every tour the search passed through, first (random) to last.
    pub steps: Vec<Vec<usize>>,
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

pub fn length(pts: &[(f32, f32)], order: &[usize]) -> f32 {
    (0..order.len())
        .map(|i| dist(pts[order[i]], pts[order[(i + 1) % order.len()]]))
        .sum()
}

impl Tour {
    /// `n` fireflies scattered over the rectangle, the search run to its end.
    pub fn solve(n: usize, (x0, y0, x1, y1): (f32, f32, f32, f32), seed: i32) -> Self {
        let pts: Vec<(f32, f32)> = (0..n as i32)
            .map(|i| {
                (
                    x0 + (x1 - x0) * h2(i, seed),
                    y0 + (y1 - y0) * h2(i, seed + 101),
                )
            })
            .collect();
        // A deliberately tangled start: the lights in hashed order.
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| h2(a as i32, seed + 7).total_cmp(&h2(b as i32, seed + 7)));
        let mut steps = vec![order.clone()];
        loop {
            let mut best = (0.0f32, 0usize, 0usize);
            for i in 0..n - 1 {
                for j in i + 2..n {
                    if i == 0 && j == n - 1 {
                        continue;
                    }
                    let (a, b) = (pts[order[i]], pts[order[i + 1]]);
                    let (c, d) = (pts[order[j]], pts[order[(j + 1) % n]]);
                    let gain = dist(a, b) + dist(c, d) - dist(a, c) - dist(b, d);
                    if gain > best.0 + 1e-3 {
                        best = (gain, i, j);
                    }
                }
            }
            if best.0 <= 0.0 {
                break;
            }
            order[best.1 + 1..=best.2].reverse();
            steps.push(order.clone());
        }
        Self { pts, steps }
    }

    pub fn last(&self) -> &[usize] {
        self.steps.last().expect("a tour has its start")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_search_only_ever_shortens_the_round_and_ends_untangled() {
        let t = Tour::solve(26, (100.0, 100.0, 1800.0, 600.0), 9);
        assert!(
            t.steps.len() > 8,
            "{} moves: too little to watch",
            t.steps.len()
        );
        let lens: Vec<f32> = t.steps.iter().map(|o| length(&t.pts, o)).collect();
        for w in lens.windows(2) {
            assert!(w[1] < w[0]);
        }
        // Every tour visits every light once.
        for o in &t.steps {
            let mut s = o.clone();
            s.sort();
            assert_eq!(s, (0..26).collect::<Vec<_>>());
        }
    }
}
