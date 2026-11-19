// The narrated cut the live window plays (production/cut/cues.json): its
// length, its chapters and when each narration line is spoken, in seconds
// from the start of the take. The frame titles these chapters, and the song
// follows them and steps back under these lines.
pub const TAKE_SECONDS: f32 = 168.802;

pub const CHAPTERS: [(&str, f32, f32); 13] = [
    ("home", 0.000, 8.619),
    ("fsr", 8.619, 19.259),
    ("data", 19.259, 30.696),
    ("solve", 30.696, 46.613),
    ("focus", 46.613, 53.280),
    ("explain", 53.280, 61.280),
    ("fill", 61.280, 69.851),
    ("replan", 69.851, 90.981),
    ("profiles", 90.981, 101.289),
    ("compare", 101.289, 122.966),
    ("trends", 122.966, 130.466),
    ("chat", 130.466, 153.525),
    ("custom", 153.525, 168.802),
];

pub const SPEECH: [(&str, f32, f32); 17] = [
    ("home", 0.400, 7.264),
    ("fsr", 9.019, 17.995),
    ("data", 19.659, 25.563),
    ("solve", 31.096, 33.016),
    ("solve2", 40.296, 45.936),
    ("focus", 47.213, 51.005),
    ("explain", 53.680, 59.152),
    ("fill", 61.680, 67.392),
    ("replan", 70.251, 75.555),
    ("replan2", 85.251, 90.027),
    ("profiles", 91.381, 97.933),
    ("compare1", 101.689, 104.929),
    ("compare", 114.289, 121.969),
    ("trends", 123.366, 128.382),
    ("chat", 130.866, 137.442),
    ("chat2", 146.966, 152.654),
    ("custom", 154.525, 161.965),
];
