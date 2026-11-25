// The veins of the old iron mine: ore threads through the gallery wall as
// a graph (each knot joined to its nearest neighbours). Light enters at
// one knot and spreads along the veins as Dijkstra's wavefront, every
// knot lit at its shortest distance; when the front reaches the geode, the
// shortest way to it burns gold. The same search a router runs, done by
// the mountain.
use crate::fx::h2;

pub struct Veins {
    /// Knots, in world cells (col, row).
    pub knots: Vec<(f32, f32)>,
    /// Edges as knot pairs, each once.
    pub edges: Vec<(usize, usize)>,
    /// Shortest distance from the source to each knot.
    pub dist: Vec<f32>,
    pub source: usize,
    pub target: usize,
    /// The shortest path from source to target, knot by knot.
    pub path: Vec<usize>,
}

fn d(a: (f32, f32), b: (f32, f32)) -> f32 {
    // Rows are 1.8 times as tall as columns are wide.
    ((a.0 - b.0).powi(2) + ((a.1 - b.1) * 1.8).powi(2)).sqrt()
}

impl Veins {
    /// `n` knots over world columns 0..width, rows r0..r1; the source is
    /// the knot nearest `from`, the target the one nearest `to`.
    pub fn grow(
        n: usize,
        width: f32,
        (r0, r1): (f32, f32),
        from: (f32, f32),
        to: (f32, f32),
        seed: i32,
    ) -> Self {
        let knots: Vec<(f32, f32)> = (0..n as i32)
            .map(|i| (width * h2(i, seed), r0 + (r1 - r0) * h2(i, seed + 1)))
            .collect();
        // A spanning tree (Prim) keeps every knot reachable; each knot's
        // two nearest neighbours add the loops that give the search a
        // choice.
        let mut edges: Vec<(usize, usize)> = Vec::new();
        let mut in_tree = vec![false; n];
        let mut best = vec![(f32::INFINITY, 0usize); n];
        in_tree[0] = true;
        for j in 1..n {
            best[j] = (d(knots[0], knots[j]), 0);
        }
        for _ in 1..n {
            let u = (0..n)
                .filter(|&k| !in_tree[k])
                .min_by(|&a, &b| best[a].0.total_cmp(&best[b].0))
                .expect("a knot left");
            in_tree[u] = true;
            let p = best[u].1;
            edges.push((u.min(p), u.max(p)));
            for j in 0..n {
                let dj = d(knots[u], knots[j]);
                if !in_tree[j] && dj < best[j].0 {
                    best[j] = (dj, u);
                }
            }
        }
        for i in 0..n {
            let mut near: Vec<usize> = (0..n).filter(|&j| j != i).collect();
            near.sort_by(|&a, &b| d(knots[i], knots[a]).total_cmp(&d(knots[i], knots[b])));
            for &j in near.iter().take(2) {
                let e = (i.min(j), i.max(j));
                if !edges.contains(&e) {
                    edges.push(e);
                }
            }
        }
        let nearest = |p: (f32, f32)| {
            (0..n)
                .min_by(|&a, &b| d(knots[a], p).total_cmp(&d(knots[b], p)))
                .expect("knots")
        };
        let (source, target) = (nearest(from), nearest(to));
        // Dijkstra, O(n^2): a few hundred knots.
        let mut dist = vec![f32::INFINITY; n];
        let mut prev = vec![usize::MAX; n];
        let mut done = vec![false; n];
        dist[source] = 0.0;
        for _ in 0..n {
            let Some(u) = (0..n)
                .filter(|&k| !done[k] && dist[k].is_finite())
                .min_by(|&a, &b| dist[a].total_cmp(&dist[b]))
            else {
                break;
            };
            done[u] = true;
            for &(a, b) in &edges {
                let v = if a == u {
                    b
                } else if b == u {
                    a
                } else {
                    continue;
                };
                let alt = dist[u] + d(knots[u], knots[v]);
                if alt < dist[v] {
                    dist[v] = alt;
                    prev[v] = u;
                }
            }
        }
        let mut path = vec![target];
        while let Some(&p) = path.last() {
            if p == source || prev[p] == usize::MAX {
                break;
            }
            path.push(prev[p]);
        }
        path.reverse();
        Self {
            knots,
            edges,
            dist,
            source,
            target,
            path,
        }
    }

    pub fn on_path(&self, a: usize, b: usize) -> bool {
        self.path
            .windows(2)
            .any(|w| (w[0] == a && w[1] == b) || (w[0] == b && w[1] == a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_light_reaches_the_geode_along_a_shortest_path() {
        let v = Veins::grow(220, 360.0, (3.0, 48.0), (8.0, 30.0), (232.0, 22.0), 5);
        assert_eq!(v.path.first(), Some(&v.source));
        assert_eq!(v.path.last(), Some(&v.target));
        assert!(v.dist[v.target].is_finite(), "the geode is cut off");
        // The path's length is the target's distance.
        let len: f32 = v
            .path
            .windows(2)
            .map(|w| d(v.knots[w[0]], v.knots[w[1]]))
            .sum();
        assert!((len - v.dist[v.target]).abs() < 1e-2);
        // Distances never decrease along the path.
        for w in v.path.windows(2) {
            assert!(v.dist[w[1]] > v.dist[w[0]]);
        }
    }
}
