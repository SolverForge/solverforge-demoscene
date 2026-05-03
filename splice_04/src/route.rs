use crate::route_plan::RoutePlan;
use crate::scene::SceneId;

#[derive(Clone, Copy, Debug)]
pub struct InsertionCandidate {
    pub position: usize,
    pub score: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ConstraintBadge {
    pub label: &'static str,
    pub station: usize,
    pub delay: f32,
}

pub struct RouteState {
    initial_route: Vec<usize>,
    inserted_route: Vec<usize>,
    rerouted_route: Vec<usize>,
    pub candidates: Vec<InsertionCandidate>,
    pub selected_position: usize,
    pub affected_start: usize,
    pub constraint_badges: Vec<ConstraintBadge>,
}

impl RouteState {
    pub fn new(plan: &RoutePlan) -> Self {
        let initial_route = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let selected_position = 4;
        let mut inserted_route = initial_route.clone();
        inserted_route.insert(selected_position, plan.required_station);

        let mut rerouted_route = inserted_route.clone();
        if let Some(goal_pos) = rerouted_route.iter().position(|station| *station == 5) {
            rerouted_route.insert(goal_pos, 12);
        }

        Self {
            initial_route,
            inserted_route,
            rerouted_route,
            candidates: vec![
                InsertionCandidate {
                    position: 2,
                    score: 0.72,
                },
                InsertionCandidate {
                    position: 3,
                    score: 0.46,
                },
                InsertionCandidate {
                    position: 4,
                    score: 0.18,
                },
                InsertionCandidate {
                    position: 5,
                    score: 0.64,
                },
                InsertionCandidate {
                    position: 7,
                    score: 0.88,
                },
            ],
            selected_position,
            affected_start: selected_position,
            constraint_badges: vec![
                ConstraintBadge {
                    label: "TIME",
                    station: plan.required_station,
                    delay: 0.0,
                },
                ConstraintBadge {
                    label: "CAP",
                    station: 4,
                    delay: 0.22,
                },
                ConstraintBadge {
                    label: "ORDER",
                    station: 5,
                    delay: 0.44,
                },
                ConstraintBadge {
                    label: "DIST",
                    station: 12,
                    delay: 0.66,
                },
            ],
        }
    }

    pub fn visible_route(&self, scene: SceneId) -> &[usize] {
        match scene {
            SceneId::Invocation
            | SceneId::Stations
            | SceneId::Successors
            | SceneId::RequiredStation
            | SceneId::InsertionSearch => &self.initial_route,
            SceneId::Splice | SceneId::ConstraintRepair => &self.inserted_route,
            SceneId::Reroute | SceneId::FinalRoute | SceneId::Scrolltext => &self.rerouted_route,
        }
    }

    pub fn initial_route(&self) -> &[usize] {
        &self.initial_route
    }

    pub fn inserted_route(&self) -> &[usize] {
        &self.inserted_route
    }

    pub fn rerouted_route(&self) -> &[usize] {
        &self.rerouted_route
    }

    pub fn selected_link(&self) -> (usize, usize) {
        (
            self.initial_route[self.selected_position - 1],
            self.initial_route[self.selected_position],
        )
    }

    pub fn blocked_link(&self, plan: &RoutePlan) -> (usize, usize) {
        (plan.blocked_from, plan.blocked_to)
    }
}
