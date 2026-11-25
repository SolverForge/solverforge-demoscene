// Cordelia, the assistant, in three stills from the SolverForge art
// direction: she hears the call, she decides, she raises her hand. Stored as
// 1920x1080 JPEGs and decoded once at start.
use crate::gfx::{H, W};

const STILLS: [&[u8]; 3] = [
    include_bytes!("../assets/cordelia-1-hears.jpg"),
    include_bytes!("../assets/cordelia-2-resolve.jpg"),
    include_bytes!("../assets/cordelia-3-hand.jpg"),
];

pub struct Cordelia {
    pub stills: Vec<Vec<u32>>,
}

impl Cordelia {
    pub fn load() -> Self {
        let stills = STILLS
            .iter()
            .map(|bytes| {
                let mut d = jpeg_decoder::Decoder::new(*bytes);
                let px = d.decode().expect("a Cordelia still decodes");
                let info = d.info().expect("JPEG info");
                assert_eq!((info.width as usize, info.height as usize), (W, H));
                assert_eq!(info.pixel_format, jpeg_decoder::PixelFormat::RGB24);
                px.as_chunks::<3>()
                    .0
                    .iter()
                    .map(|c| (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32)
                    .collect()
            })
            .collect();
        Self { stills }
    }
}
