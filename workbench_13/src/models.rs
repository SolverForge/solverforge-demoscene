// The plugin's models as its README states them: each stock model's
// constraint counts by level and the large demo it ships, plus the slot for a
// model the user builds. The intro draws its panels from this table, so the frame never
// says more than the plugin.
pub struct Model {
    /// The panel's title, two lines.
    pub name: [&'static str; 2],
    pub demo: &'static str,
    pub detail: [&'static str; 2],
    /// Constraints: hard, medium, soft. Zero for the user's own model.
    pub rules: [u32; 3],
}

pub const MODELS: [Model; 5] = [
    Model {
        name: ["EMPLOYEE", "SCHEDULING"],
        demo: "HOSPITAL MONTH",
        detail: ["50 EMPLOYEES", "536 SHIFTS"],
        rules: [56, 3, 68],
    },
    Model {
        name: ["TASK", "SCHEDULING"],
        demo: "A FACTORY WEEK",
        detail: ["24 RESOURCES", "409 OPERATIONS"],
        rules: [10, 1, 7],
    },
    Model {
        name: ["PICK-UP AND", "DELIVERY"],
        demo: "MILAN, TWO DAYS",
        detail: ["15 DRIVERS", "155 JOBS"],
        rules: [27, 1, 11],
    },
    Model {
        name: ["FIELD", "SERVICE"],
        demo: "TURIN, TWO DAYS",
        detail: ["30 TECHNICIANS", "223 VISITS"],
        rules: [34, 3, 29],
    },
    Model {
        name: ["YOUR", "MODEL"],
        demo: "BUILT BY HERMES",
        detail: ["WITH", "SOLVERFORGE"],
        rules: [0, 0, 0],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_counts_are_the_readmes() {
        let totals: Vec<u32> = MODELS.iter().map(|m| m.rules.iter().sum()).collect();
        assert_eq!(totals, [127, 18, 39, 66, 0]);
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
