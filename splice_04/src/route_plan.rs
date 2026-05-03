use crate::math::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StationRole {
    Start,
    Visit,
    Required,
    Junction,
    Checkpoint,
    Detour,
    Goal,
}

#[derive(Clone, Copy, Debug)]
pub struct Station {
    pub x: f32,
    pub y: f32,
    pub role: StationRole,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct FloorEdge {
    pub a: usize,
    pub b: usize,
}

pub struct RoutePlan {
    pub stations: Vec<Station>,
    pub floor_edges: Vec<FloorEdge>,
    pub guide_axis: Vec<Vec2>,
    pub required_station: usize,
    pub blocked_from: usize,
    pub blocked_to: usize,
}

impl RoutePlan {
    pub fn new() -> Self {
        let mut builder = RoutePlanBuilder::default();
        builder.build()
    }

    pub fn station_position(&self, idx: usize) -> Vec2 {
        let station = self.stations[idx];
        Vec2::new(station.x, station.y)
    }
}

#[derive(Default)]
struct RoutePlanBuilder {
    stations: Vec<Station>,
    floor_edges: Vec<FloorEdge>,
}

impl RoutePlanBuilder {
    fn build(&mut self) -> RoutePlan {
        let start = self.station(170.0, 602.0, StationRole::Start, "START");
        let visit_1 = self.station(292.0, 584.0, StationRole::Visit, "VISIT 1");
        let visit_2 = self.station(414.0, 552.0, StationRole::Visit, "VISIT 2");
        let junction = self.station(536.0, 506.0, StationRole::Junction, "JOIN");
        let checkpoint = self.station(662.0, 456.0, StationRole::Checkpoint, "CHECK");
        let goal = self.station(790.0, 420.0, StationRole::Goal, "GOAL");
        let loop_n = self.station(926.0, 458.0, StationRole::Detour, "LOOP N");
        let visit_3 = self.station(1052.0, 514.0, StationRole::Visit, "VISIT 3");
        let loop_s = self.station(968.0, 604.0, StationRole::Detour, "LOOP S");
        let bypass = self.station(810.0, 626.0, StationRole::Detour, "BYPASS");
        let visit_4 = self.station(556.0, 642.0, StationRole::Visit, "VISIT 4");
        let required = self.station(452.0, 404.0, StationRole::Required, "REQ");
        let detour = self.station(690.0, 558.0, StationRole::Detour, "PATCH");

        for pair in [
            (start, visit_1),
            (visit_1, visit_2),
            (visit_2, junction),
            (junction, checkpoint),
            (checkpoint, goal),
            (goal, loop_n),
            (loop_n, visit_3),
            (visit_3, loop_s),
            (loop_s, bypass),
            (bypass, visit_4),
            (visit_4, junction),
            (junction, required),
            (required, checkpoint),
            (checkpoint, detour),
            (detour, bypass),
            (visit_2, required),
            (goal, visit_3),
        ] {
            self.edge(pair.0, pair.1);
        }

        RoutePlan {
            stations: std::mem::take(&mut self.stations),
            floor_edges: std::mem::take(&mut self.floor_edges),
            guide_axis: vec![
                Vec2::new(170.0, 602.0),
                Vec2::new(292.0, 584.0),
                Vec2::new(414.0, 552.0),
                Vec2::new(536.0, 506.0),
                Vec2::new(662.0, 456.0),
                Vec2::new(790.0, 420.0),
                Vec2::new(926.0, 458.0),
                Vec2::new(1052.0, 514.0),
                Vec2::new(968.0, 604.0),
                Vec2::new(810.0, 626.0),
                Vec2::new(556.0, 642.0),
                Vec2::new(170.0, 602.0),
            ],
            required_station: required,
            blocked_from: checkpoint,
            blocked_to: goal,
        }
    }

    fn station(&mut self, x: f32, y: f32, role: StationRole, label: &'static str) -> usize {
        let idx = self.stations.len();
        self.stations.push(Station { x, y, role, label });
        idx
    }

    fn edge(&mut self, a: usize, b: usize) {
        self.floor_edges.push(FloorEdge { a, b });
    }
}
