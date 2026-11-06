// The intro's trip: a sketch of stops on a road web. Stops land one by one,
// join in a naive order that visibly crisscrosses, and then the loop redraws
// into the solved order. The kilometres on screen are computed from the same
// drawn polylines the picture uses, so the number can never disagree with
// what is drawn.

/// The road web's points (x, y) in frame pixels.
pub const POINTS: [(i32, i32); 14] = [
    (520, 380),  // 0  north
    (760, 300),  // 1  northeast
    (1080, 280), // 2  far north
    (1420, 340), // 3  northeast cape
    (1640, 520), // 4  east
    (1560, 760), // 5  southeast
    (1240, 840), // 6  south
    (980, 900),  // 7  south shore
    (700, 820),  // 8  southwest
    (460, 680),  // 9  west
    (940, 560),  // 10 middle west road
    (1240, 560), // 11 middle east road
    (700, 470),  // 12 north middle road
    (1180, 420), // 13 north middle road east
];

/// Roads, as pairs of point indices: the web the stops sit on.
pub const ROADS: [(usize, usize); 17] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 8),
    (8, 9),
    (9, 0),
    (10, 11),
    (12, 10),
    (13, 11),
    (0, 12),
    (2, 13),
    (11, 5),
    (10, 8),
];

/// The stops: points on roads, in the order the user gave them.
pub const STOPS: [usize; 10] = [0, 3, 1, 6, 2, 8, 4, 7, 9, 5];

/// The naive first order is the given order; the solved loop visits them the
/// way a solver would join a coastal arc: out along one side, back the other.
pub const GIVEN: [usize; 10] = STOPS;

pub const SOLVED: [usize; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];

pub fn point(stop: usize) -> (f32, f32) {
    let (x, y) = POINTS[stop];
    (x as f32, y as f32)
}

#[inline]
fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// One loop's length in pixels, from the drawn polylines: the model the
/// kilometer counter is computed from. The same roads that appear on screen.
fn loop_pixels(order: &[usize; 10]) -> f32 {
    let mut total = 0.0;
    for pair in order.windows(2) {
        let a = point(pair[0]);
        let b = point(pair[1]);
        total += dist(a, b);
    }
    let last = point(order[order.len() - 1]);
    let first = point(order[0]);
    total += dist(last, first);
    total
}

/// The one factor pixels go through, for every kilometre the intro shows:
/// the same factor for both orders. The solved loop lands at 664 km, the
/// real Puglia round trip's total.
pub const ROAD_KM_PER_PX: f32 = 0.227;

fn kilometres(order: &[usize; 10]) -> i32 {
    (loop_pixels(order) * ROAD_KM_PER_PX) as i32
}

/// One hop's length in pixels, for the intro to sum up as its polylines grow.
pub fn hop_pixels(a: usize, b: usize) -> f32 {
    dist(point(a), point(b))
}

pub fn given_km() -> i32 {
    kilometres(&GIVEN)
}

pub fn solved_km() -> i32 {
    kilometres(&SOLVED)
}

/// Percent shorter the solved loop is than the given one.
pub fn shorter_percent() -> i32 {
    (100.0 * (1.0 - solved_km() as f32 / given_km() as f32)) as i32
}
