// The schedule: when every piece of the capture appears on the demo clock.
// Picture and soundtrack both read it, so a typed word clicks, a code
// excerpt lands on the anvil and feasibility hits the crash on the frame it
// is drawn. Section windows come from timeline.rs; the capture decides
// everything inside them.
use crate::board::Board;
use crate::capture::{Capture, Point, Solve};
use crate::gfx::{clip, size, wrap, Size};
use crate::story::{self, Cap};
use crate::timeline::{snap_beat, span, Section, BAR, BEAT, STEP};

/// Where a caption sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Centred under the boot banner.
    Banner,
    /// A section opener: centred, under the section's large title.
    Intro,
    /// A lower third over the running scene.
    Lower,
    /// The close: one large sentence at a time.
    Close,
}

#[derive(Clone, Debug)]
pub struct Caption {
    pub cap: Cap,
    pub text: String,
    pub from: f32,
    pub to: f32,
    pub place: Place,
}

/// Cordelia's performance on the demo clock; picture and sound share it.
#[derive(Clone, Copy, Debug)]
pub struct Performance {
    /// She glitches in, eyes narrowed on the problem.
    pub enter: f32,
    /// The glint becomes a glow.
    pub charge: f32,
    /// The glow builds.
    pub build: f32,
    /// White-hot, a beat or two before the fire.
    pub ignite: f32,
    /// The downbeat the repair enters the running solve: beams.
    pub fire: f32,
    /// The plan has healed: beams cut, a smirk.
    pub cut: f32,
    /// She tears out.
    pub exit: f32,
}

/// The first beat at or after `t`.
fn beat_up(t: f32) -> f32 {
    (t / BEAT - 1e-3).ceil() * BEAT
}

/// The first bar line at or after `t`.
fn bar_up(t: f32) -> f32 {
    (t / BAR - 1e-3).ceil() * BAR
}

/// The prompt's terminal: text area in pixels.
pub const PROMPT_BOX: (i32, i32, i32, i32) = (200, 336, 1720, 860);

pub struct Typing {
    /// (time, first display char, end display char) per word.
    pub words: Vec<(f32, usize, usize)>,
    pub dur: f32,
}

impl Typing {
    /// Display characters visible at t.
    pub fn chars_at(&self, t: f32) -> usize {
        let mut n = 0;
        for &(at, a, b) in &self.words {
            if t < at {
                break;
            }
            let k = ((t - at) / self.dur).clamp(0.0, 1.0);
            n = a + ((b - a) as f32 * k).ceil() as usize;
        }
        n
    }
}

/// Maps solver wall time onto a window of the demo clock. Solvers improve
/// fastest at the start, so the replay runs on a log clock; the on-screen
/// timer always shows the real milliseconds, counted from `origin` (the
/// solve's start, or the first point after a change was pushed in).
#[derive(Clone, Copy, Debug)]
pub struct Replay {
    pub from: f32,
    pub len: f32,
    pub t_max: f64,
    origin: f64,
    t0: f64,
    /// The replay pauses here for `hold_len` seconds (a verdict is read
    /// while the numbers stand still), then carries on.
    pub hold_at: f32,
    pub hold_len: f32,
}

impl Replay {
    fn new(solve: &Solve, from: f32, len: f32, origin: f64) -> Self {
        let last = solve.trajectory.iter().map(|p| p.t_ms).fold(0.0, f64::max) - origin;
        let t_max = solve.wall_ms.max(last).max(1.0);
        Self {
            from,
            len,
            t_max,
            origin,
            t0: (t_max / 200.0).max(0.5),
            hold_at: f32::INFINITY,
            hold_len: 0.0,
        }
    }

    pub fn tau(&self, t_ms: f64) -> f32 {
        let rel = (t_ms - self.origin).max(0.0);
        ((1.0 + rel / self.t0).ln() / (1.0 + self.t_max / self.t0).ln()) as f32
    }

    pub fn at(&self, t_ms: f64) -> f32 {
        let raw = self.from + self.tau(t_ms) * self.len;
        if raw > self.hold_at + 1e-4 {
            raw + self.hold_len
        } else {
            raw
        }
    }

    /// Demo time with the hold taken out.
    fn raw(&self, t: f32) -> f32 {
        if t <= self.hold_at {
            t
        } else if t < self.hold_at + self.hold_len {
            self.hold_at
        } else {
            t - self.hold_len
        }
    }

    /// 0..1 through the run at demo time t.
    pub fn head(&self, t: f32) -> f32 {
        ((self.raw(t) - self.from) / self.len).clamp(0.0, 1.0)
    }

    /// When the replay has shown its last point.
    pub fn end(&self) -> f32 {
        let e = self.from + self.len;
        if e > self.hold_at {
            e + self.hold_len
        } else {
            e
        }
    }

    /// Real solver milliseconds since the origin at demo time t (clamped
    /// to the run).
    pub fn ms_at(&self, t: f32) -> f64 {
        let tau = self.head(t) as f64;
        self.t0 * ((1.0 + self.t_max / self.t0).powf(tau) - 1.0)
    }

    /// Index of the latest trajectory point shown at t (None before the run).
    pub fn index_at(&self, traj: &[Point], t: f32) -> Option<usize> {
        if t < self.from {
            return None;
        }
        let ms = self.ms_at(t);
        traj.iter()
            // A point that landed exactly on this frame counts as shown
            // (f32 demo time round-trips the log clock only approximately).
            .rposition(|p| p.t_ms - self.origin <= ms + 1e-3 * self.t_max.max(1.0))
            .or(Some(0))
    }

    /// Stretch the window so that `t_ms` lands on a beat, within limits.
    fn snap(mut self, t_ms: f64, max_len: f32) -> Self {
        let tau = self.tau(t_ms);
        if tau < 0.05 {
            return self;
        }
        let mut target = snap_beat(self.from + tau * self.len);
        if (target - self.from) / tau > max_len {
            target = ((self.from + tau * max_len) / BEAT).floor() * BEAT;
        }
        if target > self.from + 0.25 {
            self.len = (target - self.from) / tau;
        }
        self
    }
}

/// What kind of text a reading event is; also where it sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    /// The SOLVERFORGE banner and the four verbs under it.
    Banner,
    Verbs,
    /// A section's large title, then the line under it.
    Title,
    Caption,
    /// Panel headers (prompt terminal, agent transcript).
    Header,
    /// The prompt as it types: watched, not read word by word.
    Prompt,
    /// A run of tool calls scrolling in the transcript: a glance.
    Log,
    /// Something the agent said, held in the transcript.
    Say,
    /// A code excerpt's title (with its path and count).
    Excerpt,
    /// The run command typed in SOLVE.
    Command,
    /// FEASIBLE / PLAN HEALED over the board.
    Stamp,
    /// LIVE PLAN, then the breakdown's own sentence.
    Live,
    Event,
    /// One changed job replayed old slot to new, with its label.
    Recap,
    /// A close sentence's supporting line from the capture.
    Sub,
    Url,
}

/// One piece of text the viewer has to read: when it appears, how long it
/// owns the viewer's eyes, and how long it needs. Nothing else new appears
/// while it is read.
#[derive(Clone, Debug)]
pub struct Read {
    pub spot: Spot,
    pub text: String,
    pub from: f32,
    pub len: f32,
    /// The reading time it needs.
    pub need: f32,
}

impl Read {
    pub fn to(&self) -> f32 {
        self.from + self.len
    }
}

/// Reading time for a text: 15 characters a second plus a second.
pub fn need(text: &str) -> f32 {
    text.chars().count() as f32 / 15.0 + 1.0
}

/// One changed job replayed after the heal.
#[derive(Clone, Copy, Debug)]
pub struct Recap {
    /// The card lifts off its new slot here.
    pub at: f32,
    /// How long it owns the screen.
    pub len: f32,
    /// How long it travels from the old slot to the new one.
    pub travel: f32,
}

impl Recap {
    /// Lift off the new slot, then reappear at the old one.
    pub const LIFT: f32 = 0.4;
    /// Sitting at the old slot, label up, before it travels.
    pub const WAIT: f32 = 0.6;
}

/// The recap label of a change, from the capture.
pub fn recap_label(ch: &crate::capture::Change) -> String {
    format!(
        "{}: {} day {} -> {} day {}",
        ch.job, ch.from.technician, ch.from.day, ch.to.technician, ch.to.day
    )
}

/// The close sentences' supporting lines, all from the capture.
pub fn close_subs(c: &Capture) -> [String; 3] {
    let base = c
        .workdir
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string();
    let dir = c
        .code
        .first()
        .and_then(|x| x.path.rsplit_once('/'))
        .map(|(d, _)| format!("{d}/"))
        .unwrap_or_default();
    let model = if base.is_empty() {
        format!("ordinary Rust: {dir}")
    } else {
        format!("ordinary Rust: {base}/{dir}")
    };
    let machine = format!(
        "first plan in {} ms, the repair in {} ms",
        crate::capture::num(c.solve.wall_ms.round()),
        crate::capture::num(c.repair.solve.wall_ms.round())
    );
    let mut ver = Vec::new();
    if let Some(v) = c.version("solverforge") {
        ver.push(format!("solverforge {v}"));
    }
    if let Some(v) = c.version("solverforge_cli") {
        ver.push(format!("solverforge-cli {v}"));
    }
    let cols = 100;
    [
        clip(&model, cols),
        clip(&machine, cols),
        clip(&ver.join("  /  "), cols),
    ]
}

/// The describe terminal's header: who talks to whom, with what, when.
pub fn prompt_header(c: &Capture) -> String {
    let model = c.version("agent_model").unwrap_or("coding agent");
    let when = c.captured_at.split('T').next().unwrap_or("");
    clip(
        &format!("YOU -> CODING AGENT ({model}, captured {when})"),
        90,
    )
}

/// The transcript's header.
pub const LOG_HEADER: &str = "AGENT TRANSCRIPT + solverforge-modeling skill";

/// What the transcript shows of something the agent said: its first
/// sentence (the rest is in the capture; the screen keeps one line short
/// enough to read in its slot).
pub fn say_line(text: &str) -> String {
    match text.find(". ") {
        Some(i) => text[..=i].to_string(),
        None => text.to_string(),
    }
}

/// How long a section's large title is read before its line appears.
pub const TITLE_READ: f32 = 4.0 * BEAT;

pub struct Script {
    pub prompt_lines: Vec<String>,
    pub prompt_size: Size,
    pub typing: Typing,
    /// DESCRIBE: the terminal's header appears.
    pub header: f32,
    /// GENERATE: the transcript's header appears.
    pub log_head: f32,
    /// One time per agent step.
    pub steps: Vec<f32>,
    /// (start, end) per code excerpt; the excerpt lands on `start`.
    pub code: Vec<(f32, f32)>,
    /// SOLVE: the run command types in.
    pub cmd: f32,
    /// SOLVE: the score panel opens (its line is read with it).
    pub panel_at: f32,
    pub solve: Replay,
    pub solve_feasible: Option<f32>,
    /// Drop time per block of the solved plan, in `board order`.
    pub drops: Vec<f32>,
    pub event: f32,
    pub repair: Replay,
    pub repair_feasible: Option<f32>,
    /// Start of each changed job's move during the heal.
    pub moves: Vec<f32>,
    /// After the heal, each change replayed in turn.
    pub recap: Vec<Recap>,
    /// Close sentences' landings, then the logo.
    pub close: [f32; 3],
    /// Each close sentence's supporting line and when it appears.
    pub subs: [(String, f32); 3],
    pub logo: f32,
    pub url: f32,
    /// BOOT: the four verbs start landing.
    pub verbs: f32,
    /// When each section's opener ends and its scene takes over.
    pub intros: Vec<(Section, f32)>,
    /// Every caption, in order.
    pub captions: Vec<Caption>,
    /// SOLVE: when the board's axes are introduced.
    pub axes: f32,
    pub cordelia: Performance,
    /// (time, 0..1 progress) for every score improvement, both runs.
    pub improvements: Vec<(f32, f32)>,
    /// Every text the viewer reads, one at a time.
    pub reads: Vec<Read>,
}

/// The prompt's cell size: the largest that fits the terminal, else a
/// still-readable size that scrolls.
pub fn prompt_layout(prompt: &str) -> (Size, Vec<String>) {
    let (x0, y0, x1, y1) = PROMPT_BOX;
    let (w, h) = (x1 - x0, y1 - y0);
    for s in [size(28, 50), size(24, 44), size(20, 36), size(16, 30)] {
        let cols = (w / s.w) as usize - 2;
        let lines = wrap(prompt, cols);
        if lines.len() as i32 * (s.h + 4) <= h {
            return (s, lines);
        }
    }
    // Longer prompts stay readable: the terminal scrolls as they type.
    let s = size(16, 30);
    (s, wrap(prompt, (w / s.w) as usize - 2))
}

/// Builds the reading timeline: every text is placed after the previous
/// one has had its time.
#[derive(Default)]
struct Reads(Vec<Read>);

impl Reads {
    fn end(&self) -> f32 {
        self.0.last().map_or(0.0, |r| r.to())
    }
    /// Place a text at `at` (never before the last one ends), up for at
    /// least `len` and at least what it needs; returns its end.
    fn add(&mut self, spot: Spot, text: &str, at: f32, need: f32, len: f32) -> f32 {
        let from = at.max(self.end());
        self.0.push(Read {
            spot,
            text: text.to_string(),
            from,
            len: len.max(need),
            need,
        });
        from + len.max(need)
    }
}

impl Script {
    pub fn new(c: &Capture) -> Self {
        let board = Board::new(c);
        let mut captions: Vec<Caption> = Vec::new();
        let mut intros = Vec::new();
        let mut reads = Reads::default();
        // A caption: its own reading event, up for its window and ending on
        // a beat.
        let caption = |reads: &mut Reads,
                       captions: &mut Vec<Caption>,
                       cap: Cap,
                       text: String,
                       place: Place,
                       at: f32|
         -> f32 {
            let from = at.max(reads.end());
            let to = beat_up(from + story::window(&text));
            reads.add(Spot::Caption, &text, from, story::window(&text), to - from);
            captions.push(Caption {
                cap,
                text,
                from,
                to,
                place,
            });
            to
        };
        // Each data section opens on its title, read alone, then one line
        // saying what comes next; the scene takes over on the beat after.
        let intro = |reads: &mut Reads,
                     captions: &mut Vec<Caption>,
                     intros: &mut Vec<(Section, f32)>,
                     sec: Section,
                     cap: Cap,
                     title: &str| {
            let (a, _) = span(sec);
            reads.add(Spot::Title, title, a, need(title), TITLE_READ);
            let text = story::template(cap).to_string();
            let end = caption(reads, captions, cap, text, Place::Intro, a + TITLE_READ);
            // The caption's record spans the title too, for drawing.
            if let Some(last) = captions.last_mut() {
                last.from = a;
            }
            intros.push((sec, end));
            end
        };

        // BOOT: the banner beams in and shines once; the line; the verbs.
        let banner_at = 2.0 * BAR;
        reads.add(
            Spot::Banner,
            "SOLVERFORGE",
            banner_at,
            need("SOLVERFORGE"),
            6.8,
        );
        let boot_text = story::template(Cap::Boot).to_string();
        let (_, b1) = span(Section::Boot);
        let from = 5.5 * BAR;
        let to = beat_up(from + story::window(&boot_text));
        reads.add(
            Spot::Caption,
            &boot_text,
            from,
            story::window(&boot_text),
            to - from,
        );
        captions.push(Caption {
            cap: Cap::Boot,
            from,
            to,
            text: boot_text,
            place: Place::Banner,
        });
        let verbs_text = "DESCRIBE / GENERATE / SOLVE / REPAIR";
        let verbs = to;
        reads.add(Spot::Verbs, verbs_text, verbs, need(verbs_text), b1 - verbs);

        // DESCRIBE: the opener, the terminal's header, then the prompt
        // types word by word on the sixteenth-note grid; its last screen
        // holds for three seconds.
        let d_in = intro(
            &mut reads,
            &mut captions,
            &mut intros,
            Section::Describe,
            Cap::Describe,
            "01 DESCRIBE",
        );
        let head_text = prompt_header(c);
        let header = d_in;
        let ty0 = beat_up(reads.add(Spot::Header, &head_text, header, need(&head_text), 0.0));
        let (_, d1) = span(Section::Describe);
        let (prompt_size, prompt_lines) = prompt_layout(&c.prompt);
        let mut tokens = Vec::new();
        let mut idx = 0;
        for line in &prompt_lines {
            let mut start = None;
            for (i, ch) in line.chars().enumerate() {
                match (ch == ' ', start) {
                    (false, None) => start = Some(i),
                    (true, Some(s)) => {
                        tokens.push((idx + s, idx + i + 1));
                        start = None;
                    }
                    _ => {}
                }
            }
            let n = line.chars().count();
            if let Some(s) = start {
                tokens.push((idx + s, idx + n));
            }
            idx += n;
        }
        let ty1 = d1 - 3.0;
        reads.add(Spot::Prompt, &c.prompt, ty0, 8.0, d1 - ty0);
        let slots = ((ty1 - ty0) / STEP) as usize;
        let per = slots as f32 / tokens.len().max(1) as f32;
        let words = tokens
            .iter()
            .enumerate()
            .map(|(i, &(a, b))| {
                let at = ty0 + (i as f32 * per).floor() * STEP;
                // Each word starts where the previous one ended.
                let a = if i == 0 { 0 } else { a.min(tokens[i - 1].1) };
                (at, a, b)
            })
            .collect();
        let typing = Typing {
            words,
            dur: (per * STEP).clamp(0.03, 0.2),
        };

        // GENERATE: the transcript's header; tool calls scroll in as a
        // glance; what the agent says is held for reading; after "the
        // constraint bodies", the excerpts land one at a time on the beat
        // while the transcript stands still.
        let g_in = intro(
            &mut reads,
            &mut captions,
            &mut intros,
            Section::Generate,
            Cap::Generate,
            "02 GENERATE",
        );
        let log_head = g_in;
        let mut at = reads.add(Spot::Header, LOG_HEADER, log_head, need(LOG_HEADER), 0.0);
        let mut steps = vec![0.0; c.steps.len()];
        let mut code = Vec::new();
        // The excerpts follow the first thing the agent says after it has
        // started writing (or the end of the transcript).
        let code_after = c
            .steps
            .iter()
            .position(|s| s.kind == "say" && s.text.to_lowercase().contains("constraint"))
            .or_else(|| c.steps.iter().position(|s| s.kind == "file"))
            .unwrap_or(c.steps.len().saturating_sub(1));
        let place_code = |reads: &mut Reads, code: &mut Vec<(f32, f32)>, mut at: f32| -> f32 {
            for x in &c.code {
                let start = beat_up(at.max(reads.end()));
                let n = need(&x.title) + 0.6;
                let end = beat_up(start + n.max(2.0 + 0.15 * x.lines.len() as f32));
                reads.add(Spot::Excerpt, &x.title, start, n, end - start);
                code.push((start, end));
                at = end;
            }
            at
        };
        let mut i = 0;
        while i < c.steps.len() {
            if c.steps[i].kind == "say" {
                let line = say_line(&c.steps[i].text);
                steps[i] = at.max(reads.end());
                at = reads.add(Spot::Say, &line, steps[i], need(&line), 0.0);
                if i == code_after {
                    at = place_code(&mut reads, &mut code, at);
                }
                i += 1;
                continue;
            }
            // A run of tool calls and files: a quarter second each, then a
            // second to take the burst in.
            let j = (i..c.steps.len())
                .find(|&j| c.steps[j].kind == "say")
                .unwrap_or(c.steps.len());
            let from = at.max(reads.end());
            for (k, s) in steps.iter_mut().enumerate().take(j).skip(i) {
                *s = from + (k - i) as f32 * 0.25;
            }
            let len = (j - i) as f32 * 0.25 + 1.0;
            at = reads.add(Spot::Log, &c.steps[j - 1].text, from, len, len);
            if code_after >= i && code_after < j {
                at = place_code(&mut reads, &mut code, at);
            }
            i = j;
        }
        if code.is_empty() {
            place_code(&mut reads, &mut code, at);
        }

        // SOLVE: the command; the board and its axes; the score panel and
        // what red means; then the run alone; FEASIBLE, read while the
        // numbers stand still; what it means; the rest of the run; amber.
        let s_in = intro(
            &mut reads,
            &mut captions,
            &mut intros,
            Section::Solve,
            Cap::Solve,
            "03 SOLVE",
        );
        let cmd = s_in;
        let line = crate::scenes::command_line(c).unwrap_or_default();
        let cmd_end = reads.add(Spot::Command, &line, cmd, need(&line) + 1.2, 0.0);
        let axes = beat_up(cmd_end);
        let board_to = caption(
            &mut reads,
            &mut captions,
            Cap::Board,
            story::template(Cap::Board).into(),
            Place::Lower,
            axes,
        );
        let panel_at = board_to;
        let score_to = caption(
            &mut reads,
            &mut captions,
            Cap::Score,
            story::template(Cap::Score).into(),
            Place::Lower,
            panel_at,
        );
        let run_from = score_to + BEAT;
        let first_ok = |s: &Solve| {
            s.trajectory
                .iter()
                .find(|p| p.score.feasible())
                .map(|p| p.t_ms)
        };
        let mut solve = Replay::new(&c.solve, run_from, 8.0, 0.0);
        if let Some(ms) = first_ok(&c.solve) {
            // Feasible about three seconds into the run, on a beat.
            let tau = solve.tau(ms).max(0.05);
            solve.len = (3.0 / tau).clamp(3.0, 10.0);
            solve = solve.snap(ms, 10.0);
        }
        let solve_feasible = first_ok(&c.solve).map(|ms| solve.at(ms));
        let mut cursor = solve.end();
        if let Some(at) = solve_feasible {
            let stamp_end = reads.add(Spot::Stamp, "FEASIBLE", at, need("FEASIBLE"), STAMP_HOLD);
            let ok_to = caption(
                &mut reads,
                &mut captions,
                Cap::Feasible,
                story::template(Cap::Feasible).into(),
                Place::Lower,
                stamp_end,
            );
            solve.hold_at = at;
            solve.hold_len = beat_up(ok_to) - at;
            cursor = solve.end();
        }
        caption(
            &mut reads,
            &mut captions,
            Cap::Cost,
            story::template(Cap::Cost).into(),
            Place::Lower,
            cursor + BEAT,
        );
        let drop_end = solve_feasible.unwrap_or(solve.from + solve.len * 0.85);
        let drops = spread(c.solve.plan.len(), solve.from + 0.2, drop_end - 0.15, STEP);

        // REPAIR: the opener; a bar of the live week; the breakdown strikes
        // on a bar line and its sentence is read; then what the depot does,
        // while Cordelia charges; she fires on the downbeat after; the plan
        // heals; PLAN HEALED is read while the numbers stand still.
        let r_in = intro(
            &mut reads,
            &mut captions,
            &mut intros,
            Section::Repair,
            Cap::Repair,
            "04 REPAIR",
        );
        let event = bar_up(r_in + 2.0);
        reads.add(
            Spot::Live,
            "LIVE PLAN",
            r_in,
            need("LIVE PLAN"),
            event - r_in,
        );
        let label = &c.repair.label;
        let ev_end = reads.add(Spot::Event, label, event, need(label) + 0.7, 0.0);
        let tell = story::template(Cap::Tell).to_string();
        let tell_to = caption(
            &mut reads,
            &mut captions,
            Cap::Tell,
            tell,
            Place::Lower,
            beat_up(ev_end),
        );
        // The fire lands on a bar line after the line is read, at least
        // two bars after the event (Cordelia charges for two bars).
        let fire = bar_up(tell_to.max(event + 3.0 * BAR));
        let enter = fire - 2.0 * BAR;
        let origin = c
            .repair
            .solve
            .trajectory
            .iter()
            .map(|p| p.t_ms)
            .fold(f64::MAX, f64::min);
        let mut repair = Replay::new(&c.repair.solve, fire, 8.0, origin);
        if let Some(ms) = first_ok(&c.repair.solve) {
            let tau = repair.tau(ms).max(0.05);
            repair.len = ((3.0 / tau).min(14.0) / BEAT).round().max(1.0) * BEAT;
            repair = repair.snap(ms, 14.0);
        }
        let repair_feasible = first_ok(&c.repair.solve).map(|ms| repair.at(ms));
        let move_end = repair_feasible.unwrap_or(repair.from + repair.len * 0.85);
        let moves = spread(
            c.repair.changed.len(),
            repair.from,
            move_end - 0.5,
            BEAT / 2.0,
        );
        let cut = beat_up(move_end.max(fire + BAR) + BEAT);
        let exit = cut + BAR;
        let cordelia = Performance {
            enter,
            charge: enter + 2.0 * BEAT,
            build: enter + BAR,
            ignite: enter + BAR + 2.0 * BEAT,
            fire,
            cut,
            exit,
        };
        if let Some(at) = repair_feasible {
            let stamp_end = reads.add(
                Spot::Stamp,
                "PLAN HEALED",
                at,
                need("PLAN HEALED"),
                STAMP_HOLD,
            );
            repair.hold_at = at;
            repair.hold_len = stamp_end - at;
        }
        // Once she is gone: what moved; then each change replayed, the
        // card back at its old slot travelling to its new one; then the
        // line that closes the section.
        let bus = board
            .struck_bus
            .clone()
            .unwrap_or_else(|| "the broken bus".to_string());
        let moved = story::fill(
            story::template(Cap::Moved),
            &[
                ("changed", c.repair.changed.len().to_string()),
                ("bus", bus),
                ("unchanged", c.repair.unchanged_count.to_string()),
            ],
        );
        let moved_to = caption(
            &mut reads,
            &mut captions,
            Cap::Moved,
            moved,
            Place::Lower,
            beat_up(repair.end().max(exit + super_out())),
        );
        let mut at = moved_to;
        let mut recap = Vec::new();
        for (k, ch) in c.repair.changed.iter().enumerate() {
            // The first three travel slowly; the rest a little faster, but
            // every one moves and every label gets its reading time.
            let travel = if k < 3 { 1.4 } else { 1.0 };
            let label = recap_label(ch);
            let n = need(&label);
            let len = beat_up(Recap::LIFT + (Recap::WAIT + travel + 0.6).max(n) + 0.3);
            let from = at.max(reads.end());
            reads.add(
                Spot::Recap,
                &label,
                from + Recap::LIFT,
                n,
                len - Recap::LIFT,
            );
            recap.push(Recap {
                at: from,
                len,
                travel,
            });
            at = from + len;
        }
        let done = story::template(Cap::Done).to_string();
        caption(&mut reads, &mut captions, Cap::Done, done, Place::Lower, at);

        // CLOSE: three sentences, one at a time, each read alone, then its
        // line from the capture under it; then the logo and the address.
        let (c0, _) = span(Section::Close);
        let speed = story::speed(c.solve.wall_ms.max(c.repair.solve.wall_ms));
        let sub_text = close_subs(c);
        let mut close = [0.0; 3];
        let mut subs: [(String, f32); 3] = Default::default();
        let mut at = c0;
        for (i, cap) in [Cap::Close1, Cap::Close2, Cap::Close3]
            .into_iter()
            .enumerate()
        {
            let text = story::fill(story::template(cap), &[("speed", speed.clone())]);
            let w = story::window(&text);
            let read_to = beat_up(at + w);
            reads.add(Spot::Caption, &text, at, w, read_to - at);
            let sub = sub_text[i].clone();
            let to = beat_up(reads.add(Spot::Sub, &sub, read_to, need(&sub) + 0.4, 0.0) + 0.4);
            close[i] = at;
            subs[i] = (sub, read_to);
            captions.push(Caption {
                cap,
                text,
                from: at,
                to,
                place: Place::Close,
            });
            at = to;
        }
        let logo = at;
        let url = logo + BAR;
        reads.add(
            Spot::Url,
            "solverforge.org",
            url,
            need("solverforge.org"),
            0.0,
        );

        let mut improvements = progress(&c.solve.trajectory, &solve);
        improvements.extend(progress(&c.repair.solve.trajectory, &repair));

        Self {
            prompt_lines,
            prompt_size,
            typing,
            header,
            log_head,
            steps,
            code,
            cmd,
            panel_at,
            solve,
            solve_feasible,
            drops,
            event,
            repair,
            repair_feasible,
            moves,
            recap,
            close,
            subs,
            logo,
            url,
            verbs,
            intros,
            captions,
            axes,
            cordelia,
            improvements,
            reads: reads.0,
        }
    }
}

/// How long Cordelia takes to tear out.
fn super_out() -> f32 {
    crate::scenes::cordelia::OUT
}

/// How long a stamp holds over the board.
pub const STAMP_HOLD: f32 = 3.5;

impl Script {
    /// When a section's opener hands over to its scene (its start if it
    /// has none).
    pub fn intro_end(&self, sec: Section) -> f32 {
        self.intros
            .iter()
            .find(|(s, _)| *s == sec)
            .map_or(span(sec).0, |(_, t)| *t)
    }

    /// 0..1: how much a text of this kind holds the viewer's eyes at t,
    /// easing in and out over a quarter second (for dimming the rest).
    pub fn spot_gain(&self, t: f32, spot: Spot) -> f32 {
        self.reads
            .iter()
            .filter(|r| r.spot == spot)
            .map(|r| {
                crate::gfx::smooth((t - r.from + 0.25) / 0.25)
                    * (1.0 - crate::gfx::smooth((t - r.to()) / 0.25))
            })
            .fold(0.0, f32::max)
    }
}

/// `n` times from `a` to `b`, each on a `grid` step after `a`.
fn spread(n: usize, a: f32, b: f32, grid: f32) -> Vec<f32> {
    let slots = (((b - a) / grid).floor() as usize).max(1);
    (0..n)
        .map(|i| {
            let k = if n > 1 {
                (i as f32 * slots as f32 / (n - 1) as f32).round()
            } else {
                0.0
            };
            a + k.min(slots as f32) * grid
        })
        .collect()
}

/// Score-improvement events and their progress towards the best score.
fn progress(traj: &[Point], replay: &Replay) -> Vec<(f32, f32)> {
    let hard_min = traj.iter().filter_map(|p| p.score.hard).fold(0.0, f64::min);
    let soft: Vec<f64> = traj.iter().filter_map(|p| p.score.soft).collect();
    let (s_lo, s_hi) = soft
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let mut out = Vec::new();
    for (i, p) in traj.iter().enumerate() {
        if i > 0 && p.score == traj[i - 1].score {
            continue;
        }
        let ph = match p.score.hard {
            Some(h) if hard_min < 0.0 => 1.0 - h / hard_min,
            _ => 1.0,
        };
        let ps = match p.score.soft {
            Some(s) if s_hi > s_lo => (s - s_lo) / (s_hi - s_lo),
            _ => 1.0,
        };
        out.push((
            replay.at(p.t_ms),
            (0.7 * ph + 0.3 * ps).clamp(0.0, 1.0) as f32,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture() -> Capture {
        Capture::parse(include_str!("../assets/capture.placeholder.json")).unwrap()
    }

    /// The placeholder, and the real capture once it has been copied in.
    fn all() -> Vec<Capture> {
        let mut v = vec![capture()];
        let real = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");
        if std::path::Path::new(real).exists() {
            v.push(Capture::load(real).unwrap());
        }
        v
    }

    #[test]
    fn typing_finishes_inside_describe() {
        for c in all() {
            let s = Script::new(&c);
            let total: usize = s.prompt_lines.iter().map(|l| l.chars().count()).sum();
            let (_, d1) = span(Section::Describe);
            assert_eq!(s.typing.chars_at(d1 - 2.5), total);
            assert_eq!(s.typing.chars_at(0.0), 0);
        }
    }

    #[test]
    fn events_stay_in_their_sections() {
        for c in all() {
            let s = Script::new(&c);
            let inside = |t: f32, sec: Section| {
                let (a, b) = span(sec);
                assert!(t >= a && t < b, "{t} outside {sec:?}");
            };
            for &t in &s.steps {
                inside(t, Section::Generate);
            }
            for &(a, b) in &s.code {
                inside(a, Section::Generate);
                assert!(b > a + 0.9, "an excerpt gets at least a bar-ish slot");
            }
            for &t in &s.drops {
                inside(t, Section::Solve);
            }
            inside(s.solve_feasible.unwrap(), Section::Solve);
            inside(s.event, Section::Repair);
            for &t in s.moves.iter().chain(s.recap.iter().map(|r| &r.at)) {
                inside(t, Section::Repair);
            }
            inside(s.repair_feasible.unwrap(), Section::Repair);
            let (_, s1) = span(Section::Solve);
            assert!(s.solve.end() <= s1);
            let (_, r1) = span(Section::Repair);
            assert!(s.repair.end() <= r1 - 4.0);
            for r in &s.recap {
                assert!(r.at + r.len <= r1);
            }
        }
    }

    /// One text at a time: no two reading windows overlap anywhere in the
    /// video, each text is up for at least the time it needs (15 chars a
    /// second plus a second for anything meant to be read), and every
    /// caption on screen is on the reading timeline.
    #[test]
    fn one_text_is_read_at_a_time() {
        for c in all() {
            let s = Script::new(&c);
            let mut v = s.reads.clone();
            v.sort_by(|a, b| a.from.total_cmp(&b.from));
            for w in v.windows(2) {
                assert!(
                    w[1].from >= w[0].to() - 1e-3,
                    "{:?} {:?} ({:.2}..{:.2}) overlaps {:?} {:?} ({:.2}..)",
                    w[0].spot,
                    w[0].text,
                    w[0].from,
                    w[0].to(),
                    w[1].spot,
                    w[1].text,
                    w[1].from
                );
            }
            for r in &v {
                assert!(r.len >= r.need - 1e-3, "{:?} {:?} rushed", r.spot, r.text);
                // Glances (a scrolling log, the typing prompt) declare
                // their own time; everything else must meet the rule.
                if !matches!(r.spot, Spot::Log | Spot::Prompt) {
                    assert!(
                        r.need >= need(&r.text) - 1e-3,
                        "{:?} {:?} needs more",
                        r.spot,
                        r.text
                    );
                }
                assert!(r.to() <= crate::timeline::DURATION);
            }
            for cap in &s.captions {
                assert!(
                    v.iter().any(|r| r.text == cap.text
                        && r.from >= cap.from - 1e-3
                        && r.to() <= cap.to + 1e-3),
                    "{:?} is not on the reading timeline",
                    cap.cap
                );
            }
            // Brisk: every section ends within two bars of its last text.
            for &(sec, _, _) in crate::timeline::SECTIONS.iter() {
                let (a, b) = span(sec);
                let last = v
                    .iter()
                    .filter(|r| r.from >= a && r.from < b)
                    .map(|r| r.to())
                    .fold(a, f32::max);
                // The placeholder's synthetic transcript is wordier than the
                // real one; it only previews layouts, so only the release
                // capture must fit the sections exactly.
                if c.is_release() {
                    assert!(last <= b + 1e-3, "{sec:?} overruns");
                    assert!(b - last <= 2.0 * BAR + 1.6, "{sec:?} pads {:.1}s", b - last);
                }
            }
        }
    }

    /// Each recapped job visibly travels and its label is read.
    #[test]
    fn every_recap_moves_and_is_read() {
        for c in all() {
            let s = Script::new(&c);
            assert_eq!(s.recap.len(), c.repair.changed.len());
            for (r, ch) in s.recap.iter().zip(&c.repair.changed) {
                assert!(r.travel >= 1.0);
                let label = recap_label(ch);
                assert!(r.len - Recap::LIFT >= need(&label) - 1e-3);
                assert!(r.len >= Recap::LIFT + Recap::WAIT + r.travel + 0.5);
            }
            for w in s.recap.windows(2) {
                assert!(w[1].at >= w[0].at + w[0].len - 1e-3);
            }
        }
    }

    #[test]
    fn feasibility_lands_on_a_beat() {
        for c in all() {
            let s = Script::new(&c);
            for t in [s.solve_feasible.unwrap(), s.repair_feasible.unwrap()] {
                let off = (t / BEAT - (t / BEAT).round()).abs();
                assert!(off < 1e-3, "{t} is off the beat");
            }
        }
    }

    #[test]
    fn replay_clock_round_trips() {
        let c = capture();
        let s = Script::new(&c);
        for ms in [0.0, 10.0, 1234.0, s.solve.t_max] {
            let t = s.solve.at(ms);
            assert!((s.solve.ms_at(t) - ms).abs() < 1e-3 * ms.max(1.0));
        }
    }

    #[test]
    fn prompt_fits_its_terminal() {
        let c = capture();
        let (sz, lines) = prompt_layout(&c.prompt);
        let (x0, y0, x1, y1) = PROMPT_BOX;
        assert!(lines.len() as i32 * (sz.h + 4) <= y1 - y0);
        for l in &lines {
            assert!(l.chars().count() as i32 * sz.w <= x1 - x0);
        }
        // A prompt four times longer scrolls at a readable size, inside
        // the terminal's width.
        let long = c.prompt.repeat(4);
        let (sz, lines) = prompt_layout(&long);
        assert!(sz.w >= 16);
        for l in &lines {
            assert!(l.chars().count() as i32 * sz.w <= x1 - x0);
        }
    }
}
