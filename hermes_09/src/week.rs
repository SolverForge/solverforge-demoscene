// The intro's week: fixed meetings, and task blocks that first land on top
// of them and then move to free time. The score on screen is computed from
// the blocks' current positions with the same overlap test the picture uses,
// so the number can never disagree with what is drawn.

/// Working hours shown: 09:00 to 18:00.
pub const DAY_FROM: f32 = 9.0;
pub const DAY_TO: f32 = 18.0;
pub const DAYS: [&str; 5] = ["MON", "TUE", "WED", "THU", "FRI"];

#[derive(Clone, Copy)]
pub struct Slot {
    pub day: f32,
    pub start: f32,
    pub hours: f32,
}

impl Slot {
    const fn at(day: f32, start: f32, hours: f32) -> Self {
        Self { day, start, hours }
    }

    /// Same day (to the nearest column) and the hours intersect.
    pub fn overlaps(&self, other: &Slot) -> bool {
        self.day.round() == other.day.round()
            && self.start < other.start + other.hours
            && other.start < self.start + self.hours
    }
}

pub struct Meeting {
    pub title: &'static str,
    pub slot: Slot,
}

pub struct Task {
    pub title: &'static str,
    /// Where a naive first guess drops it: on top of a meeting.
    pub first: Slot,
    /// Where the solver moves it: free time, inside working hours.
    pub solved: Slot,
}

pub const MEETINGS: [Meeting; 8] = [
    Meeting {
        title: "STANDUP",
        slot: Slot::at(0.0, 9.5, 0.5),
    },
    Meeting {
        title: "STANDUP",
        slot: Slot::at(1.0, 9.5, 0.5),
    },
    Meeting {
        title: "STANDUP",
        slot: Slot::at(2.0, 9.5, 0.5),
    },
    Meeting {
        title: "STANDUP",
        slot: Slot::at(4.0, 9.5, 0.5),
    },
    Meeting {
        title: "CLIENT CALL",
        slot: Slot::at(1.0, 14.0, 2.0),
    },
    Meeting {
        title: "BOARD",
        slot: Slot::at(2.0, 11.0, 1.5),
    },
    Meeting {
        title: "OFFSITE",
        slot: Slot::at(3.0, 9.0, 3.0),
    },
    Meeting {
        title: "1:1",
        slot: Slot::at(4.0, 15.0, 1.0),
    },
];

pub const TASKS: [Task; 5] = [
    Task {
        title: "Q4 DECK",
        first: Slot::at(3.0, 9.5, 3.0),
        solved: Slot::at(0.0, 10.0, 3.0),
    },
    Task {
        title: "REVIEW PR",
        first: Slot::at(1.0, 14.5, 1.0),
        solved: Slot::at(1.0, 10.0, 1.0),
    },
    Task {
        title: "ACCOUNTANT",
        first: Slot::at(2.0, 11.5, 0.5),
        solved: Slot::at(2.0, 16.0, 0.5),
    },
    Task {
        title: "DEMO SCRIPT",
        first: Slot::at(4.0, 14.5, 2.0),
        solved: Slot::at(4.0, 10.0, 2.0),
    },
    Task {
        title: "INBOX ZERO",
        first: Slot::at(0.0, 9.25, 1.0),
        solved: Slot::at(3.0, 13.0, 1.0),
    },
];

/// Hard penalty of a set of block positions: one per block-meeting overlap,
/// one per block-block overlap.
pub fn hard(blocks: &[Slot]) -> usize {
    let meetings = blocks
        .iter()
        .map(|b| MEETINGS.iter().filter(|m| b.overlaps(&m.slot)).count())
        .sum::<usize>();
    let mut pairs = 0;
    for (i, a) in blocks.iter().enumerate() {
        pairs += blocks[i + 1..].iter().filter(|b| a.overlaps(b)).count();
    }
    meetings + pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_solved_week_keeps_every_hard_rule_and_the_first_guess_does_not() {
        let solved: Vec<Slot> = TASKS.iter().map(|t| t.solved).collect();
        assert_eq!(hard(&solved), 0);
        for s in &solved {
            assert!(s.start >= DAY_FROM && s.start + s.hours <= DAY_TO);
        }
        let first: Vec<Slot> = TASKS.iter().map(|t| t.first).collect();
        assert!(TASKS
            .iter()
            .all(|t| MEETINGS.iter().any(|m| t.first.overlaps(&m.slot))));
        assert!(hard(&first) >= TASKS.len());
    }
}
