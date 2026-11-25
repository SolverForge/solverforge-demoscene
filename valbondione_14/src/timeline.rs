// One clock for picture and sound: 72 BPM, one bar is three and a third
// seconds, and every scene starts on a bar line. The song (audio/song.rs)
// reads the same table, so every cut lands on a downbeat.

pub const BPM: f32 = 72.0;
pub const BEAT: f32 = 60.0 / BPM;
pub const BAR: f32 = BEAT * 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scene {
    /// An owl on a fir branch, the moon behind the fog.
    Owl,
    /// Valbondione at nightfall: lit windows, the bell tower, the Serio.
    /// Cordelia walks out of the village, worn out.
    Village,
    /// The crossroads: one traffic light swinging on its wire.
    Crossing,
    /// The firs and the fog: a knight at the forest's edge joins her.
    Forest,
    /// The river gorge: fireflies find the shortest way between them.
    Fireflies,
    /// The dam opens and the Cascate del Serio fall under floodlights.
    Falls,
    /// Up behind the falls through the old iron mine: the veins light up
    /// along the shortest way to the geode.
    Mine,
    /// Lake Barbellino: the stars in the water, a ring of standing stones
    /// whose runes wake, the aurora.
    Lake,
    /// A clearing: the fire, marshmallows, laughter.
    Campfire,
    /// First light: she sleeps on his shoulder; the fireflies make the mark.
    Dawn,
}

/// (scene, first bar, end bar)
pub const SCENES: [(Scene, f32, f32); 10] = [
    (Scene::Owl, 0.0, 4.0),
    (Scene::Village, 4.0, 9.0),
    (Scene::Crossing, 9.0, 11.0),
    (Scene::Forest, 11.0, 17.0),
    (Scene::Fireflies, 17.0, 22.0),
    (Scene::Falls, 22.0, 29.0),
    (Scene::Mine, 29.0, 34.0),
    (Scene::Lake, 34.0, 38.0),
    (Scene::Campfire, 38.0, 50.0),
    (Scene::Dawn, 50.0, 56.0),
];

pub const TOTAL_BARS: f32 = 56.0;
pub const DURATION: f32 = TOTAL_BARS * BAR;

pub const fn bars(b: f32) -> f32 {
    b * BAR
}

pub fn span(scene: Scene) -> (f32, f32) {
    SCENES
        .iter()
        .find(|(s, _, _)| *s == scene)
        .map(|(_, b0, b1)| (bars(*b0), bars(*b1)))
        .unwrap_or((0.0, 0.0))
}

#[derive(Clone, Copy, Debug)]
pub struct Now {
    pub scene: Scene,
    /// Seconds since the scene started.
    pub local: f32,
}

pub fn now(t: f32) -> Now {
    let bar = t / BAR;
    let &(scene, b0, _) = SCENES
        .iter()
        .find(|(_, _, end)| bar < *end)
        .unwrap_or(&SCENES[SCENES.len() - 1]);
    Now {
        scene,
        local: t - bars(b0),
    }
}

/// Next scene start after t (for SPACE), wrapping to 0.
pub fn next_boundary(t: f32) -> f32 {
    SCENES
        .iter()
        .map(|(_, b0, _)| bars(*b0))
        .find(|start| *start > t + 0.25)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenes_tile_the_song() {
        let mut end = 0.0;
        for (_, b0, b1) in SCENES {
            assert_eq!(b0, end);
            assert!(b1 > b0);
            end = b1;
        }
        assert_eq!(end, TOTAL_BARS);
    }
}
