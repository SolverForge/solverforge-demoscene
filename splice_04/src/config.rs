pub const WIDTH: usize = 1280;
pub const HEIGHT: usize = 720;
pub const FPS: u32 = 60;
/// The demo's length: the end of the last scene.
pub const DEFAULT_DURATION_SECONDS: f32 =
    crate::scene::SCENE_BOUNDARIES[crate::scene::SCENE_BOUNDARIES.len() - 1];
pub const AUDIO_WAV_OUT: &str = "target/demo_audio.wav";

pub struct DemoConfig {
    pub width: usize,
    pub height: usize,
    pub fps: u32,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            width: WIDTH,
            height: HEIGHT,
            fps: FPS,
        }
    }
}
