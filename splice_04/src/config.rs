pub const WIDTH: usize = 1280;
pub const HEIGHT: usize = 720;
pub const FPS: u32 = 60;
pub const DEFAULT_DURATION_SECONDS: f32 = 124.0;
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
