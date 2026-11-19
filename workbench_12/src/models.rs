// The plugin's models as its README states them: each stock model's
// constraint counts by level and the large demo it ships, plus the slot for a
// model the user builds. The intro draws its panels from this table and the
// live frame's rack names them, so the frame never says more than the plugin.
pub struct Model {
    /// The panel's title, two lines.
    pub name: [&'static str; 2],
    /// The rack's short name under the take.
    pub short: &'static str,
    pub demo: &'static str,
    pub detail: [&'static str; 2],
    /// Constraints: hard, medium, soft. Zero for the user's own model.
    pub rules: [u32; 3],
}

pub const MODELS: [Model; 5] = [
    Model {
        name: ["EMPLOYEE", "SCHEDULING"],
        short: "EMPLOYEES",
        demo: "HOSPITAL MONTH",
        detail: ["50 EMPLOYEES", "536 SHIFTS"],
        rules: [56, 3, 68],
    },
    Model {
        name: ["TASK", "SCHEDULING"],
        short: "TASKS",
        demo: "A FACTORY WEEK",
        detail: ["24 RESOURCES", "409 OPERATIONS"],
        rules: [9, 1, 7],
    },
    Model {
        name: ["PICK-UP AND", "DELIVERY"],
        short: "DELIVERIES",
        demo: "MILAN, TWO DAYS",
        detail: ["15 DRIVERS", "155 JOBS"],
        rules: [26, 1, 9],
    },
    Model {
        name: ["FIELD", "SERVICE"],
        short: "FIELD SERVICE",
        demo: "TURIN, TWO DAYS",
        detail: ["30 TECHNICIANS", "223 VISITS"],
        rules: [32, 3, 29],
    },
    Model {
        name: ["YOUR", "MODEL"],
        short: "YOUR MODEL",
        demo: "BUILT BY HERMES",
        detail: ["WITH", "SOLVERFORGE"],
        rules: [0, 0, 0],
    },
];

/// The model on screen in each chapter of the cut; None when the chapter
/// shows them all.
pub fn on_screen(chapter: &str) -> Option<usize> {
    match chapter {
        "home" => None,
        "chat" => Some(0),
        "custom" => Some(4),
        _ => Some(3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_counts_are_the_readmes() {
        let totals: Vec<u32> = MODELS.iter().map(|m| m.rules.iter().sum()).collect();
        assert_eq!(totals, [127, 17, 36, 64, 0]);
    }

    #[test]
    fn every_line_fits_its_panel() {
        for m in &MODELS {
            for line in m.name.iter().chain(&m.detail).chain([&m.demo]) {
                assert!(line.len() <= 15, "{line} is wider than a panel");
            }
        }
    }
}
