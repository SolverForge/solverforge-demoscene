// Written by the production's build.py from its cut: do not edit by hand.
// The film's segments (Desktop takes and the demo's own scenes), the chapter
// titles over the takes, the sped-up searches, the rings that point at what
// the narration names, and when each narration line is spoken. Seconds on
// the film clock; boxes in frame pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Take,
    Intro,
    Call,
    Outro,
}

pub const TOTAL: f32 = 126.000;

pub const SEGMENTS: [(f32, f32, Kind); 15] = [
    (0.000, 8.000, Kind::Take),
    (8.000, 18.000, Kind::Intro),
    (18.000, 26.000, Kind::Take),
    (26.000, 38.000, Kind::Take),
    (38.000, 48.000, Kind::Take),
    (48.000, 56.000, Kind::Take),
    (56.000, 62.000, Kind::Call),
    (62.000, 74.000, Kind::Take),
    (74.000, 88.000, Kind::Take),
    (88.000, 92.000, Kind::Take),
    (92.000, 98.000, Kind::Take),
    (98.000, 106.000, Kind::Take),
    (106.000, 112.000, Kind::Take),
    (112.000, 118.000, Kind::Take),
    (118.000, 126.000, Kind::Outro),
];

pub const CHAPTERS: [(f32, f32, &str); 8] = [
    (0.000, 8.000, "THU 4 MAR  07:00"),
    (18.000, 48.000, "01  SOLVE"),
    (48.000, 56.000, "02  EXPLAIN"),
    (62.000, 92.000, "04  REPLAN"),
    (92.000, 98.000, "05  BEFORE"),
    (98.000, 106.000, "05  AFTER"),
    (106.000, 112.000, "06  COMPARE"),
    (112.000, 118.000, "07  EVERY MODEL"),
];

/// Where one take cuts straight to another.
pub const CUTS: [f32; 5] = [48.000, 92.000, 98.000, 106.000, 112.000];

/// (from, to, speed): a search played faster than it ran.
pub const SEARCHES: [(f32, f32, f32); 2] = [(26.000, 38.000, 4.13), (74.000, 88.000, 4.20)];

/// (from, to, (x0, y0, x1, y1), tone): 0 jade, 1 emerald, 2 blue, 3 amber.
pub type Ring = (f32, f32, (i32, i32, i32, i32), usize);

pub const RINGS: [Ring; 18] = [
    (2.000, 7.600, (896, 642, 1236, 824), 0),
    (2.000, 7.600, (364, 640, 470, 680), 0),
    (20.400, 22.400, (1794, 313, 1894, 361), 0),
    (23.000, 26.000, (352, 306, 1880, 592), 2),
    (26.000, 38.000, (352, 306, 1880, 592), 2),
    (45.000, 48.000, (352, 306, 1880, 592), 1),
    (51.000, 56.000, (1290, 515, 1880, 1060), 0),
    (62.800, 74.000, (560, 272, 1358, 810), 3),
    (74.000, 88.000, (352, 306, 1880, 592), 2),
    (89.400, 92.000, (352, 306, 1880, 592), 1),
    (93.900, 98.000, (896, 642, 1236, 824), 3),
    (93.900, 98.000, (364, 640, 470, 680), 3),
    (101.400, 104.000, (902, 676, 1230, 830), 1),
    (101.400, 104.000, (362, 676, 462, 712), 1),
    (104.200, 106.000, (348, 28, 566, 100), 0),
    (106.600, 109.200, (352, 298, 1880, 334), 0),
    (109.200, 112.000, (352, 956, 1880, 990), 1),
    (113.000, 118.000, (352, 216, 1880, 522), 0),
];

pub const SPEECH: [(&str, f32, f32); 14] = [
    ("cold", 0.700, 6.344),
    ("intro", 9.300, 17.084),
    ("solve", 18.300, 26.409),
    ("search", 27.200, 37.101),
    ("verdict", 45.000, 48.876),
    ("rule", 50.600, 55.415),
    ("call", 56.700, 61.460),
    ("dialog", 62.400, 71.519),
    ("replan", 75.000, 79.164),
    ("before", 93.600, 97.825),
    ("after", 100.200, 103.761),
    ("published", 104.200, 111.976),
    ("catalogue", 112.400, 117.682),
    ("outro", 119.900, 125.023),
];
