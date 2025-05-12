use sdl2::{AudioSubsystem, EventPump, Sdl, VideoSubsystem, render::Canvas, video::Window};

use crate::TetronError;

pub struct TetronSdlHandle {
    pub(crate) context: Sdl,
    pub(crate) video: VideoSubsystem,
    pub(crate) audio: AudioSubsystem,
    pub(crate) canvas: Canvas<Window>,
    pub(crate) events: EventPump,
}

impl TetronSdlHandle {
    pub fn new(title: &str, w: u32, h: u32) -> Result<Self, TetronError> {
        let context = sdl2::init()?;
        let video = context.video()?;
        let audio = context.audio()?;
        let window = video
            .window(title, w, h)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        let events = context.event_pump()?;

        Ok(Self {
            context,
            video,
            audio,
            canvas,
            events,
        })
    }
}
