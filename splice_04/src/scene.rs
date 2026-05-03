#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneId {
    Invocation,
    Stations,
    Successors,
    RequiredStation,
    InsertionSearch,
    Splice,
    ConstraintRepair,
    Reroute,
    FinalRoute,
    Scrolltext,
}

#[derive(Clone, Copy, Debug)]
pub struct SceneState {
    pub id: SceneId,
    pub local_t: f32,
    pub progress: f32,
}

pub const SCENE_BOUNDARIES: &[f32] = &[
    0.0, 10.0, 22.0, 34.0, 46.0, 60.0, 74.0, 90.0, 100.0, 108.0, 124.0,
];

pub fn scene_state(t: f32) -> SceneState {
    let (id, start, end) = match t {
        t if t < 10.0 => (SceneId::Invocation, 0.0, 10.0),
        t if t < 22.0 => (SceneId::Stations, 10.0, 22.0),
        t if t < 34.0 => (SceneId::Successors, 22.0, 34.0),
        t if t < 46.0 => (SceneId::RequiredStation, 34.0, 46.0),
        t if t < 60.0 => (SceneId::InsertionSearch, 46.0, 60.0),
        t if t < 74.0 => (SceneId::Splice, 60.0, 74.0),
        t if t < 90.0 => (SceneId::ConstraintRepair, 74.0, 90.0),
        t if t < 100.0 => (SceneId::Reroute, 90.0, 100.0),
        t if t < 108.0 => (SceneId::FinalRoute, 100.0, 108.0),
        _ => (SceneId::Scrolltext, 108.0, 124.0),
    };

    let duration = end - start;
    let duration = if duration > 0.0 { duration } else { 0.0 };
    let local_t = (t - start).max(0.0);
    let progress = if duration > 0.0 {
        (local_t / duration).clamp(0.0, 1.0)
    } else {
        0.0
    };

    SceneState {
        id,
        local_t,
        progress,
    }
}

pub fn next_scene_boundary(t: f32) -> Option<f32> {
    SCENE_BOUNDARIES
        .iter()
        .copied()
        .find(|boundary| *boundary > t + 0.5)
}
