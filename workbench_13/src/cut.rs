// The film's cut, written by the production's build_cut.py: every section
// on a bar line with its band title, and every narration line with the
// seconds it is spoken. The band types these words as they are said, and
// the music follows the sections.
use crate::timeline::Kind;

pub const DURATION: f32 = 242.000;

pub const SECTIONS: [(Kind, &str, f32, f32); 11] = [
    (Kind::Intro, "", 0.000, 14.000),
    (Kind::Footage, "THE ORDERS", 14.000, 46.000),
    (Kind::Week, "THE WEEK", 46.000, 66.000),
    (Kind::Footage, "THE ISSUED WEEK", 66.000, 74.000),
    (Kind::Footage, "THE DELAY", 74.000, 124.000),
    (Kind::Powder, "THE POWDER", 124.000, 142.000),
    (Kind::Call, "THE CALL", 142.000, 152.000),
    (Kind::Footage, "REPLAN", 152.000, 172.000),
    (Kind::Repair, "THE REPAIR", 172.000, 194.000),
    (Kind::Footage, "THE REPAIRED WEEK", 194.000, 230.000),
    (Kind::Outro, "", 230.000, 242.000),
];

pub const SPEECH: [(&str, f32, f32); 16] = [
    ("SolverForge Workbench puts planning models inside Hermes. This one schedules a factory.", 14.500, 21.067),
    ("Its demo is one week: four hundred and nine operations on sixteen machines and eight people.", 28.400, 34.632),
    ("Powder for the paint lines is short. Four deliveries come on Tuesday at six in the morning.", 35.200, 41.559),
    ("The solver places every operation, then keeps improving the week. Two minutes of search, shown in twenty seconds.", 46.600, 54.525),
    ("Ten operations have no slot after the first pass. In the end, all four hundred and nine do.", 56.000, 63.042),
    ("No rule is broken. This is the week the shop works from.", 66.600, 71.044),
    ("Then the supplier calls: the powder will come on Thursday, at six in the evening.", 74.500, 79.302),
    ("The four deliveries change, and the change is saved as a new dataset. The issued week stays as it was.", 97.600, 104.705),
    ("Checked as it is, the week now breaks a hard rule: parts out of stock.", 112.900, 117.309),
    ("Red, blue and grey powder run out on Thursday. Five coating jobs would start with nothing to spray.", 126.000, 133.384),
    ("Monday to Wednesday is already on the shop floor. Replan keeps everything that starts before Thursday at six, and plans the rest again.", 152.400, 162.121),
    ("The solver searches for three minutes. The coating jobs that lacked powder move to when there is powder for them.", 172.800, 179.291),
    ("Ninety-six operations move. The hard score goes from minus three hundred to zero.", 181.500, 187.825),
    ("All four hundred and nine operations are placed, and no rule is broken.", 194.600, 199.108),
    ("Side by side: the issued week, the broken check, and the repair.", 212.600, 216.916),
    ("Four planning models ship with the plugin, and Hermes can build your own.", 223.300, 227.399),
];
