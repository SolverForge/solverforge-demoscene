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

/// Scene starts in order, then the end of the demo. The lengths fit the
/// story cards (story.rs), each held long enough to read; the end sits on the
/// song's 77th bar line (125 BPM) so the music finishes on the beat.
pub const SCENE_BOUNDARIES: &[f32] = &[
    0.0, 10.0, 26.5, 44.0, 60.0, 75.5, 92.0, 108.0, 123.5, 131.5, 147.84,
];

const SCENES: [SceneId; 10] = [
    SceneId::Invocation,
    SceneId::Stations,
    SceneId::Successors,
    SceneId::RequiredStation,
    SceneId::InsertionSearch,
    SceneId::Splice,
    SceneId::ConstraintRepair,
    SceneId::Reroute,
    SceneId::FinalRoute,
    SceneId::Scrolltext,
];

/// When a scene starts, in seconds.
pub fn scene_start(id: SceneId) -> f32 {
    let idx = SCENES.iter().position(|s| *s == id).unwrap_or(0);
    SCENE_BOUNDARIES[idx]
}

pub fn scene_state(t: f32) -> SceneState {
    let idx = SCENE_BOUNDARIES[1..SCENES.len()]
        .iter()
        .position(|start| t < *start)
        .unwrap_or(SCENES.len() - 1);
    let (id, start, end) = (
        SCENES[idx],
        SCENE_BOUNDARIES[idx],
        SCENE_BOUNDARIES[idx + 1],
    );

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
