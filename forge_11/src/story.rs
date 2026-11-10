// Every caption the viewer reads, in one table: a narrator telling what
// happens at the depot and to its people, third person, one line leading
// into the next. Captions say what the viewer is about to see and what it
// means for the depot; the numbers in them come from the capture
// (`{bus}`, `{changed}`, `{unchanged}`, `{speed}` are filled in by
// `Script`). Timing lives in script.rs; drawing in scenes/captions.rs.

/// Which moment a caption belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cap {
    /// Under the banner, before the story starts.
    Boot,
    /// Section openers: centred, under the section's title.
    Describe,
    Generate,
    Solve,
    Repair,
    /// Lower thirds inside the sections.
    Board,
    Score,
    Feasible,
    Cost,
    Live,
    Tell,
    Moved,
    Done,
    /// The close, one sentence at a time.
    Close1,
    Close2,
    Close3,
}

pub const CAPTIONS: &[(Cap, &str)] = &[
    (Cap::Boot, "This is a real session, replayed as it ran."),
    (Cap::Describe, "A bus depot has a week of repairs to plan."),
    (Cap::Generate, "A coding agent turns those rules into code."),
    (Cap::Solve, "The code runs on the depot's own laptop."),
    (
        Cap::Board,
        "Each row is a mechanic, each column a working day.",
    ),
    (
        Cap::Score,
        "Red counts broken rules, and it has to reach zero.",
    ),
    (Cap::Feasible, "Zero: every job has a mechanic and a day."),
    (
        Cap::Cost,
        "Amber is what's left to trim, like late days and uneven loads.",
    ),
    (
        Cap::Repair,
        "Then, with the plan already running, a bus breaks down.",
    ),
    (Cap::Live, "This is the week as it runs."),
    (Cap::Tell, "The depot sends it to the running app."),
    (
        Cap::Moved,
        "{changed} jobs move to make room for {bus}'s repair; the other {unchanged} stay put.",
    ),
    (
        Cap::Done,
        "Nobody at the depot has to rebuild the week by hand.",
    ),
    (
        Cap::Close1,
        "The code stays in the depot's own repo, readable line by line.",
    ),
    (Cap::Close2, "It runs on their laptop and answers {speed}."),
    (
        Cap::Close3,
        "They choose when to upgrade, so nothing shifts behind their back.",
    ),
];

/// The breakdown's second line: what the broken bus needs, and by when
/// (`{skill}` and `{day}` come from the capture's event label).
pub const EVENT_NEED: &str = "It needs urgent {skill} repair by day {day}.";

/// The breakdown as short lines read one after the other, from the
/// capture's event label ("Bus 05 breaks down in service: it needs an
/// urgent electrical repair, parts on hand from day 1, back by day 2"):
/// what happened, then what it needs and by when. A label without that
/// shape is read as one line.
pub fn event_lines(label: &str) -> Vec<String> {
    let (what, rest) = label
        .split_once(':')
        .map_or((label.trim(), ""), |(a, b)| (a.trim(), b.trim()));
    let skill = rest
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[1].trim_end_matches(',') == "repair")
        .map(|w| w[0].to_string());
    match (skill, due_day(label)) {
        (Some(skill), Some(day)) => vec![
            format!("{}.", what.trim_end_matches('.')),
            fill(EVENT_NEED, &[("skill", skill), ("day", day.to_string())]),
        ],
        _ => vec![format!("{}.", label.trim().trim_end_matches('.'))],
    }
}

/// The day a breakdown's repair is due, from "by day N" in its label.
pub fn due_day(label: &str) -> Option<i64> {
    let i = label.find("by day ")?;
    label[i + 7..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

/// The caption template for a moment.
pub fn template(cap: Cap) -> &'static str {
    CAPTIONS
        .iter()
        .find(|(c, _)| *c == cap)
        .map(|(_, s)| *s)
        .unwrap_or("")
}

/// Fill `{name}` placeholders.
pub fn fill(template: &str, vars: &[(&str, String)]) -> String {
    let mut s = template.to_string();
    for (k, v) in vars {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

/// How long a caption must stay up: the series' card reading window, and
/// at least 15 characters a second plus a second, plus its entrance and
/// exit.
pub fn window(text: &str) -> f32 {
    let chars = text.chars().count() as f32;
    demofx::reading_window(text).max(chars / 15.0 + 1.0 + 1.0)
}

/// Spoken words for a solve time: what a person would say about it.
pub fn speed(max_ms: f64) -> String {
    if max_ms < 300.0 {
        "faster than a blink".to_string()
    } else if max_ms < 1500.0 {
        "in about a second".to_string()
    } else {
        format!("in {} seconds", (max_ms / 1000.0).ceil() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_moment_has_copy() {
        for cap in [
            Cap::Boot,
            Cap::Describe,
            Cap::Generate,
            Cap::Solve,
            Cap::Repair,
            Cap::Board,
            Cap::Score,
            Cap::Feasible,
            Cap::Cost,
            Cap::Live,
            Cap::Tell,
            Cap::Moved,
            Cap::Done,
            Cap::Close1,
            Cap::Close2,
            Cap::Close3,
        ] {
            assert!(!template(cap).is_empty(), "{cap:?}");
        }
    }

    /// The copy rules the owner set: no em dashes, no marketing words, no
    /// "No X." fragments, ASCII the 8x8 font can draw.
    #[test]
    fn copy_follows_the_house_rules() {
        let banned = [
            "seamless",
            "effortless",
            "powerful",
            "unlock",
            "leverage",
            "game-changer",
            "solver",
            "sign-in",
            "your ",
        ];
        // A narrator, not a persona: no first person.
        let extra = [(Cap::Live, EVENT_NEED)];
        for (cap, s) in CAPTIONS.iter().chain(extra.iter()) {
            assert!(s.is_ascii(), "{cap:?}");
            assert!(!s.contains('\u{2014}') && !s.contains(" - "), "{cap:?}");
            assert!(!s.contains('?'), "{cap:?} asks a rhetorical question");
            let low = s.to_lowercase();
            for b in banned {
                assert!(!low.contains(b), "{cap:?} uses {b:?}");
            }
            assert!(!s.starts_with("No "), "{cap:?}");
            for w in s.split(|c: char| !c.is_ascii_alphabetic() && c != '\'') {
                assert!(
                    !matches!(
                        w,
                        "I" | "my" | "My" | "me" | "I'm" | "I'd" | "we" | "We" | "our" | "Our"
                    ),
                    "{cap:?} speaks in the first person"
                );
            }
        }
    }

    #[test]
    fn the_breakdown_reads_as_two_short_lines() {
        let l = event_lines(
            "Bus 05 breaks down in service: it needs an urgent electrical repair, \
             parts on hand from day 1, back by day 2",
        );
        assert_eq!(
            l,
            [
                "Bus 05 breaks down in service.",
                "It needs urgent electrical repair by day 2."
            ]
        );
        assert_eq!(
            event_lines("PH-TECH-C calls in sick"),
            ["PH-TECH-C calls in sick."]
        );
        assert_eq!(due_day("x, back by day 2"), Some(2));
    }

    #[test]
    fn placeholders_fill() {
        let s = fill(
            template(Cap::Moved),
            &[
                ("changed", "5".into()),
                ("bus", "Bus 05".into()),
                ("unchanged", "15".into()),
            ],
        );
        assert!(!s.contains('{'), "{s}");
        assert!(window(&s) > 5.0);
    }
}
