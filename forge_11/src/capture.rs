// The capture bundle (contract v1, see README): one real agent session,
// replayed. Every word, number and assignment on screen comes from here;
// nothing in the picture is invented.
//
// A placeholder bundle (schema forge11.placeholder/1) lets the demo be
// developed before the real run exists. Release renders refuse it.
use std::fmt::Write as _;

use crate::json::{self, Json};

pub const RELEASE_SCHEMA: &str = "forge11.capture/1";
pub const PLACEHOLDER_SCHEMA: &str = "forge11.placeholder/1";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Score {
    pub hard: Option<f64>,
    pub medium: Option<f64>,
    pub soft: Option<f64>,
}

impl Score {
    pub fn feasible(&self) -> bool {
        match self.hard {
            Some(h) => h >= 0.0,
            None => true,
        }
    }

    /// "-3hard/0medium/-120soft", omitting the levels the app's score lacks.
    pub fn label(&self) -> String {
        let mut out = String::new();
        for (v, name) in [
            (self.hard, "hard"),
            (self.medium, "medium"),
            (self.soft, "soft"),
        ] {
            if let Some(v) = v {
                if !out.is_empty() {
                    out.push('/');
                }
                let _ = write!(out, "{}{name}", num(v));
            }
        }
        out
    }
}

/// A score level as text: integers without a decimal point.
pub fn num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v:.2}")
    }
}

#[derive(Clone, Debug)]
pub struct Point {
    pub t_ms: f64,
    pub score: Score,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    pub job: String,
    pub bus: String,
    pub technician: String,
    pub day: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub technician: String,
    pub day: i64,
}

#[derive(Clone, Debug)]
pub struct Change {
    pub job: String,
    pub from: Slot,
    pub to: Slot,
}

#[derive(Clone, Debug)]
pub struct Step {
    pub t_ms: f64,
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Code {
    pub path: String,
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Solve {
    pub wall_ms: f64,
    pub trajectory: Vec<Point>,
    pub plan: Vec<Assignment>,
}

#[derive(Clone, Debug)]
pub struct Repair {
    pub kind: String,
    pub label: String,
    pub detail: Vec<(String, String)>,
    pub solve: Solve,
    pub changed: Vec<Change>,
    pub unchanged_count: usize,
}

#[derive(Clone, Debug)]
pub struct Capture {
    pub schema: String,
    pub captured_at: String,
    pub versions: Vec<(String, String)>,
    pub prompt: String,
    pub steps: Vec<Step>,
    pub code: Vec<Code>,
    pub solve: Solve,
    pub repair: Repair,
    pub workdir: String,
    pub commands: Vec<String>,
}

impl Capture {
    pub fn load(path: &str) -> Result<Capture, String> {
        let src = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        Self::parse(&src).map_err(|e| format!("{path}: {e}"))
    }

    pub fn parse(src: &str) -> Result<Capture, String> {
        let root = json::parse(src).map_err(|e| e.to_string())?;
        let schema = string(&root, "schema")?;
        if schema != RELEASE_SCHEMA && schema != PLACEHOLDER_SCHEMA {
            return Err(format!("unknown schema {schema:?}"));
        }
        let versions = root
            .get("versions")
            .and_then(Json::as_obj)
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| Some((k.clone(), fold(&v.text()?))))
                    .collect()
            })
            .unwrap_or_default();
        let steps = array(&root, "agent_steps")?
            .iter()
            .map(|s| {
                Ok(Step {
                    t_ms: number(s, "t_ms")?,
                    kind: string(s, "kind")?,
                    text: fold(&string(s, "text")?),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let code = array(&root, "code")?
            .iter()
            .map(|c| {
                Ok(Code {
                    path: fold(&string(c, "path")?),
                    title: fold(&string(c, "title")?),
                    lines: fold(&string(c, "excerpt")?)
                        .trim_end_matches('\n')
                        .lines()
                        .map(str::to_owned)
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let solve = parse_solve(field(&root, "solve")?)?;
        let rep = field(&root, "repair")?;
        let event = field(rep, "event")?;
        let detail = event
            .get("detail")
            .and_then(Json::as_obj)
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| Some((k.clone(), fold(&v.text()?))))
                    .collect()
            })
            .unwrap_or_default();
        let changed = array(rep, "changed")?
            .iter()
            .map(|c| {
                Ok(Change {
                    job: label(c, "job")?,
                    from: slot(field(c, "from")?)?,
                    to: slot(field(c, "to")?)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        // The mechanism stays required in the bundle (it is the evidence of
        // how the change went in) but is not drawn: see the README.
        string(rep, "mechanism")?;
        let repair = Repair {
            kind: string(event, "kind")?,
            label: fold(&string(event, "label")?),
            detail,
            solve: parse_solve(rep)?,
            changed,
            unchanged_count: number(rep, "unchanged_count")? as usize,
        };
        let prov = field(&root, "provenance")?;
        let capture = Capture {
            schema,
            captured_at: root
                .get("captured_at")
                .and_then(Json::text)
                .unwrap_or_default(),
            versions,
            prompt: fold(&string(&root, "prompt")?),
            steps,
            code,
            solve,
            repair,
            workdir: prov.get("workdir").and_then(Json::text).unwrap_or_default(),
            commands: prov
                .get("commands")
                .and_then(Json::as_arr)
                .map(|a| a.iter().filter_map(Json::text).map(|s| fold(&s)).collect())
                .unwrap_or_default(),
        };
        capture.validate()?;
        Ok(capture)
    }

    fn validate(&self) -> Result<(), String> {
        let need = |ok: bool, what: &str| if ok { Ok(()) } else { Err(what.to_string()) };
        need(!self.prompt.trim().is_empty(), "prompt is empty")?;
        need(!self.steps.is_empty(), "agent_steps is empty")?;
        need(!self.code.is_empty(), "code is empty")?;
        need(
            self.code.iter().all(|c| !c.lines.is_empty()),
            "a code excerpt is empty",
        )?;
        for s in [&self.solve, &self.repair.solve] {
            need(!s.trajectory.is_empty(), "a trajectory is empty")?;
            need(!s.plan.is_empty(), "a plan is empty")?;
        }
        Ok(())
    }

    pub fn is_release(&self) -> bool {
        self.schema == RELEASE_SCHEMA
    }

    /// The contract's guard: a release render takes the real capture only.
    pub fn guard(&self, allow_placeholder: bool) -> Result<(), String> {
        if self.is_release() || allow_placeholder {
            Ok(())
        } else {
            Err(format!(
                "refusing to render: schema is {:?}, a release render needs {RELEASE_SCHEMA:?} \
                 (use `make preview` for the placeholder)",
                self.schema
            ))
        }
    }

    pub fn version(&self, key: &str) -> Option<&str> {
        self.versions
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

fn field<'a>(v: &'a Json, key: &str) -> Result<&'a Json, String> {
    v.get(key).ok_or_else(|| format!("missing {key:?}"))
}

fn string(v: &Json, key: &str) -> Result<String, String> {
    field(v, key)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{key:?} is not a string"))
}

/// A label that may be a string or a number in the capture.
fn label(v: &Json, key: &str) -> Result<String, String> {
    field(v, key)?
        .text()
        .map(|s| fold(&s))
        .ok_or_else(|| format!("{key:?} is not a label"))
}

fn number(v: &Json, key: &str) -> Result<f64, String> {
    field(v, key)?
        .as_f64()
        .ok_or_else(|| format!("{key:?} is not a number"))
}

fn array<'a>(v: &'a Json, key: &str) -> Result<&'a [Json], String> {
    field(v, key)?
        .as_arr()
        .ok_or_else(|| format!("{key:?} is not an array"))
}

fn day(v: &Json) -> Result<i64, String> {
    let d = field(v, "day")?;
    d.as_f64()
        .map(|n| n as i64)
        .or_else(|| d.as_str().and_then(|s| s.trim().parse().ok()))
        .ok_or_else(|| "\"day\" is not a number".to_string())
}

fn slot(v: &Json) -> Result<Slot, String> {
    Ok(Slot {
        technician: label(v, "technician")?,
        day: day(v)?,
    })
}

fn parse_solve(v: &Json) -> Result<Solve, String> {
    let trajectory = array(v, "trajectory")?
        .iter()
        .map(|p| {
            Ok(Point {
                t_ms: number(p, "t_ms")?,
                score: Score {
                    hard: p.get("hard").and_then(Json::as_f64),
                    medium: p.get("medium").and_then(Json::as_f64),
                    soft: p.get("soft").and_then(Json::as_f64),
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let plan = array(v, "plan")?
        .iter()
        .map(|a| {
            Ok(Assignment {
                job: label(a, "job")?,
                bus: label(a, "bus")?,
                technician: label(a, "technician")?,
                day: day(a)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Solve {
        wall_ms: number(v, "wall_ms")?,
        trajectory,
        plan,
    })
}

/// The 8x8 font is ASCII: fold typography to its nearest ASCII spelling and
/// tabs to four spaces; anything else unknown becomes '?'. Home directories
/// display as `~`, the shell's own spelling, so no user name reaches a frame.
/// The session's scratch directory (`~/.hermes/cache/scratch/<run>`) shows
/// as `$SCRATCH`, so the host's scratch layout and run name stay off screen.
pub fn fold(s: &str) -> String {
    collapse_scratch(&collapse_home(&ascii(s)))
}

const SCRATCH_ROOT: &str = "~/.hermes/cache/scratch/";

fn collapse_scratch(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find(SCRATCH_ROOT) {
        out.push_str(&rest[..i]);
        let after = &rest[i + SCRATCH_ROOT.len()..];
        let end = after
            .find(|c: char| c == '/' || c.is_whitespace() || c == '"' || c == '\'')
            .unwrap_or(after.len());
        out.push_str("$SCRATCH");
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

fn collapse_home(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("/home/") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 6..];
        let end = after
            .find(|c: char| c == '/' || c.is_whitespace() || c == '"' || c == '\'')
            .unwrap_or(after.len());
        if end == 0 {
            out.push_str("/home/");
        } else {
            out.push('~');
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

fn ascii(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\t' => out.push_str("    "),
            '\n' => out.push('\n'),
            '\r' => {}
            ' '..='~' => out.push(c),
            '\u{2018}' | '\u{2019}' | '\u{2032}' => out.push('\''),
            '\u{201C}' | '\u{201D}' | '\u{2033}' => out.push('"'),
            '\u{2013}' | '\u{2014}' | '\u{2212}' | '\u{2500}' => out.push('-'),
            '\u{2192}' | '\u{27F6}' => out.push_str("->"),
            '\u{2190}' => out.push_str("<-"),
            '\u{21D2}' => out.push_str("=>"),
            '\u{2026}' => out.push_str("..."),
            '\u{00D7}' => out.push('x'),
            '\u{2264}' => out.push_str("<="),
            '\u{2265}' => out.push_str(">="),
            '\u{2260}' => out.push_str("!="),
            '\u{00A0}' | '\u{2009}' | '\u{202F}' => out.push(' '),
            '\u{2022}' | '\u{00B7}' => out.push('*'),
            '\u{2713}' | '\u{2714}' => out.push('v'),
            'é' | 'è' | 'ê' => out.push('e'),
            'à' | 'á' => out.push('a'),
            'ò' | 'ó' => out.push('o'),
            'ù' | 'ú' => out.push('u'),
            'ì' | 'í' => out.push('i'),
            _ => out.push('?'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLACEHOLDER: &str = include_str!("../assets/capture.placeholder.json");

    #[test]
    fn placeholder_parses_and_is_refused_for_release() {
        let c = Capture::parse(PLACEHOLDER).expect("placeholder parses");
        assert_eq!(c.schema, PLACEHOLDER_SCHEMA);
        assert!(!c.is_release());
        assert!(c.guard(false).is_err());
        assert!(c.guard(true).is_ok());
    }

    #[test]
    fn release_schema_passes_the_guard() {
        let real = PLACEHOLDER.replace(PLACEHOLDER_SCHEMA, RELEASE_SCHEMA);
        let c = Capture::parse(&real).unwrap();
        assert!(c.is_release());
        assert!(c.guard(false).is_ok());
    }

    #[test]
    fn unknown_schema_is_rejected() {
        let bad = PLACEHOLDER.replace(PLACEHOLDER_SCHEMA, "forge11.capture/2");
        assert!(Capture::parse(&bad).is_err());
    }

    #[test]
    fn absent_score_levels_are_omitted() {
        let s = Score {
            hard: Some(-2.0),
            medium: None,
            soft: Some(-130.0),
        };
        assert_eq!(s.label(), "-2hard/-130soft");
        assert!(!s.feasible());
        assert!(Score {
            hard: Some(0.0),
            ..s
        }
        .feasible());
    }

    #[test]
    fn home_directories_become_tilde() {
        assert_eq!(fold("cd /home/someone/x && ls /home/b"), "cd ~/x && ls ~");
        assert_eq!(fold("/home/"), "/home/");
        assert_eq!(
            fold("cd /home/u/.hermes/cache/scratch/run1/app && ls"),
            "cd $SCRATCH/app && ls"
        );
    }

    #[test]
    fn folds_typography_to_ascii() {
        assert_eq!(
            fold("a\u{2192}b \u{2014} \u{201C}x\u{201D}\t!"),
            "a->b - \"x\"    !"
        );
    }

    /// The real capture, when it has been copied in, must parse too.
    #[test]
    fn real_capture_parses_when_present() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/capture.json");
        if std::path::Path::new(path).exists() {
            let c = Capture::load(path).expect("assets/capture.json parses");
            assert!(
                c.is_release(),
                "assets/capture.json must be the real capture"
            );
        }
    }
}
