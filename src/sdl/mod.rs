use sdl2::{
    AudioSubsystem, EventPump, Sdl, VideoSubsystem, render::Canvas, ttf::Sdl2TtfContext,
    video::Window,
};
use std::{collections::HashMap, rc::Rc};

use crate::{error::TetronError, fs::SimpleFs};

pub struct TetronSdlHandle {
    pub(crate) context: Sdl,
    pub(crate) video: VideoSubsystem,
    pub(crate) audio: AudioSubsystem,
    pub(crate) canvas: Canvas<Window>,
    pub(crate) events: EventPump,
    pub(crate) ttf_context: Sdl2TtfContext,
    pub(crate) font_data: HashMap<String, Vec<u8>>,
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
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let font_data = HashMap::new();

        Ok(Self {
            context,
            video,
            audio,
            canvas,
            events,
            ttf_context,
            font_data,
        })
    }

    pub fn load_fonts(
        &mut self,
        font_list: &[(String, String)],
        fs: Rc<dyn SimpleFs>,
    ) -> Result<(), TetronError> {
        for (name, path) in font_list {
            let font_data = fs.open_file(path)?;
            self.font_data.insert(name.clone(), font_data);
        }
        Ok(())
    }
}
