use sdl2::{
    AudioSubsystem, EventPump, Sdl, VideoSubsystem,
    gfx::primitives::DrawRenderer,
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    ttf::Sdl2TtfContext,
    video::Window,
};
use std::{collections::HashMap, rc::Rc};

use crate::{engine::physics::vec2::Vec2, error::TetronError, fs::SimpleFs};

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

    pub fn draw_rect(
        &mut self,
        pos: Vec2,
        width: f64,
        height: f64,
        color: Color,
        filled: bool,
    ) -> Result<(), TetronError> {
        let rect = Rect::new(pos.x as i32, pos.y as i32, width as u32, height as u32);

        if filled {
            self.canvas.set_draw_color(color);
            self.canvas.fill_rect(rect)?;
        } else {
            self.canvas.set_draw_color(color);
            self.canvas.draw_rect(rect)?;
        }
        Ok(())
    }

    pub fn draw_circle(
        &mut self,
        pos: Vec2,
        radius: f64,
        color: Color,
        filled: bool,
    ) -> Result<(), TetronError> {
        let x = pos.x as i16;
        let y = pos.y as i16;
        let r = radius as i16;

        if filled {
            self.canvas.filled_circle(x, y, r, color)?;
        } else {
            self.canvas.circle(x, y, r, color)?;
        }
        Ok(())
    }

    pub fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color) -> Result<(), TetronError> {
        self.canvas.set_draw_color(color);
        self.canvas.draw_line(
            Point::new(start.x as i32, start.y as i32),
            Point::new(end.x as i32, end.y as i32),
        )?;
        Ok(())
    }

    pub fn draw_polygon(
        &mut self,
        points: &[Vec2],
        color: Color,
        filled: bool,
    ) -> Result<(), TetronError> {
        if points.len() < 3 {
            return Err(TetronError::Runtime(
                "Polygon must have at least 3 points".into(),
            ));
        }

        let xs: Vec<i16> = points.iter().map(|p| p.x as i16).collect();
        let ys: Vec<i16> = points.iter().map(|p| p.y as i16).collect();

        if filled {
            self.canvas.filled_polygon(&xs, &ys, color)?;
        } else {
            self.canvas.polygon(&xs, &ys, color)?;
        }
        Ok(())
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        pos: Vec2,
        font_name: Option<String>,
        font_size: Option<f64>,
        color: Color,
    ) -> Result<(), TetronError> {
        use sdl2::rwops::RWops;

        let font_key = font_name
            .clone()
            .or_else(|| self.font_data.keys().next().cloned())
            .ok_or_else(|| {
                TetronError::Runtime("No font available for text rendering".to_string())
            })?;
        let font_bytes = self
            .font_data
            .get(&font_key)
            .ok_or_else(|| TetronError::Runtime(format!("Font '{}' not loaded", font_key)))?;
        let font_size = font_size.map(|fs| fs as u16).unwrap_or(16);
        let rw = RWops::from_bytes(font_bytes)
            .map_err(|e| TetronError::Runtime(format!("RWops error: {e}")))?;
        let font = self
            .ttf_context
            .load_font_from_rwops(rw, font_size)
            .map_err(|e| {
                TetronError::Runtime(format!("ttf_context.load_font_from_rwops error: {e}"))
            })?;
        let sdl_color = color;
        let surface = font
            .render(text)
            .blended(sdl_color)
            .map_err(|e| TetronError::Runtime(format!("Font render error: {e}")))?;
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| TetronError::Runtime(format!("texture creation error: {e}")))?;
        let target = sdl2::rect::Rect::new(
            pos.x as i32,
            pos.y as i32,
            surface.width(),
            surface.height(),
        );
        self.canvas
            .copy(&texture, None, Some(target))
            .map_err(|e| TetronError::Runtime(format!("canvas.copy error: {e}")))?;
        Ok(())
    }
}
