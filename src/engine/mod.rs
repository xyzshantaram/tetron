use crate::{
    engine::physics::vec2::Vec2,
    error::TetronError,
    fs::{SimpleFs, overlay_fs::OverlayFs, to_vfs_layer},
    scripting::{self, TetronScripting},
    sdl::TetronSdlHandle,
    utils::{parse_hex_color, resolve_physical_fs_path, typed_value::TypedValue},
};
use input::KeyState;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};
use std::{
    collections::HashSet,
    process,
    rc::Rc,
    sync::{Arc, RwLock},
    time::Instant,
};
use stupid_simple_kv::{IntoKey, Kv, KvBackend, KvValue, MemoryBackend, SqliteBackend};
use systems::Ctx;
use world::WorldRef;

mod args;
pub mod behaviours;
pub mod drawable;
pub mod entity;
pub mod input;
pub mod physics;
pub mod scene;
pub mod shape;
pub mod systems;
pub mod transform;
pub mod world;
pub use args::TetronArgs;

pub struct Game {
    fs: Rc<dyn SimpleFs>,
    pub(crate) config: Arc<Kv>,
    sdl: TetronSdlHandle,
    pub identifier: String,
    scripting: TetronScripting,
    world: Option<WorldRef>,
    input: Arc<RwLock<KeyState>>,
}

fn parse_fonts_from_config(config: &Arc<Kv>) -> Vec<(String, String)> {
    let mut fonts = Vec::new();
    if let Ok(Some(KvValue::Array(list))) = config.get(&("fonts",)) {
        for font in list {
            if let KvValue::Object(cfg) = font {
                if let (Some(KvValue::String(name)), Some(KvValue::String(path))) =
                    (cfg.get("name"), cfg.get("path"))
                {
                    fonts.push((name.clone(), path.clone()));
                }
            }
        }
    }
    fonts
}

impl Game {
    fn new<F>(fs: Rc<dyn SimpleFs>, backend_factory: F) -> Result<Self, anyhow::Error>
    where
        F: FnOnce(&str) -> Result<Box<dyn KvBackend>, anyhow::Error>,
    {
        let json = fs.read_text_file("game.json")?;
        let config = Arc::new(Kv::from_json_string(Box::new(MemoryBackend::new()), json)?);

        let identifier: String = config
            .get(&("identifier",))?
            .ok_or(TetronError::RequiredConfigNotFound("identifier".into()))?
            .try_into()?;

        let flags = Arc::new(RwLock::new(Kv::new(backend_factory(&identifier)?)));

        let width: i64 = config
            .get(&("sdl", "width").to_key())?
            .unwrap_or(800i64.into())
            .try_into()?;
        let height: i64 = config
            .get(&("sdl", "height").to_key())?
            .unwrap_or(600i64.into())
            .try_into()?;
        let title: String = config
            .get(&("sdl", "title").to_key())?
            .unwrap_or(identifier.clone().into())
            .try_into()?;

        let fonts_to_load = parse_fonts_from_config(&config);
        let mut sdl = TetronSdlHandle::new(&title, width.try_into()?, height.try_into()?)?;
        sdl.load_fonts(&fonts_to_load, fs.clone())?;
        let input = Arc::new(RwLock::new(KeyState::new()));
        let scripting =
            TetronScripting::new(fs.clone(), flags, config.clone(), Arc::clone(&input))?;
        Ok(Self {
            fs,
            config,
            sdl,
            identifier,
            scripting,
            world: None,
            input,
        })
    }
}

impl TryFrom<TetronArgs> for Game {
    type Error = anyhow::Error;

    fn try_from(args: TetronArgs) -> Result<Self, Self::Error> {
        let game_path = match args.game {
            Some(p) => resolve_physical_fs_path(&p)?,
            None => {
                eprintln!("tetron: error: No game supplied");
                process::exit(1);
            }
        };

        let mut layers: Vec<Box<dyn SimpleFs>> = vec![to_vfs_layer(&game_path)?];

        for layer in args.layers.iter().rev() {
            layers.push(to_vfs_layer(layer)?);
        }

        let fs = OverlayFs::from_layers(layers);

        let backend_factory = |identifier: &str| -> Result<Box<dyn KvBackend>, anyhow::Error> {
            let data =
                dirs::data_dir().ok_or(TetronError::Other("Error getting user data dir".into()))?;
            let db_path = data.join("tetron").join(identifier);
            std::fs::create_dir_all(&db_path)?;
            Ok(Box::new(SqliteBackend::file(&db_path.join("flags.db"))?))
        };

        Self::new(Rc::new(fs), backend_factory)
    }
}

impl Game {
    fn update(&mut self, delta: f64) -> Result<(), TetronError> {
        if let Some(world) = &mut self.world {
            world.game_loop(delta)?;
        }

        Ok(())
    }

    fn draw(&mut self, dt: f64) -> Result<(), TetronError> {
        if let Some(world) = self.world.clone() {
            let ctx = Ctx::new(world, dt);
            let behaviours: HashSet<String> = HashSet::from_iter([
                "tetron:drawable".to_string(),
                "tetron:transform".to_string(),
            ]);
            let tags = HashSet::new();
            let queried = ctx.query_with_sets(tags, behaviours)?;
            // Drawing logic starts here
            for entity in queried {
                let drawable = match entity.behaviour("tetron:drawable") {
                    Some(d) => d,
                    None => continue,
                };
                let transform = match entity.behaviour("tetron:transform") {
                    Some(t) => t,
                    None => continue,
                };
                // Get color from drawable (fallback white)
                let color = parse_hex_color(
                    &drawable
                        .get_typed("color")?
                        .and_then(|v| match v {
                            TypedValue::String(s) => Some(s),
                            _ => None,
                        })
                        .unwrap_or_default(),
                    Color::WHITE,
                );
                // Parse position from transform
                let pos: Option<Vec2> =
                    transform
                        .get_typed("pos")
                        .ok()
                        .flatten()
                        .and_then(|v| match v {
                            TypedValue::Vector(v2) => Some(v2),
                            _ => None,
                        });
                let pos = pos.unwrap_or(Vec2::ZERO);

                // Draw text if present
                if let Some(TypedValue::String(txt)) = drawable.get_typed("text").ok().flatten() {
                    // font config (optional)
                    let font_conf = drawable.get_typed("font").ok().flatten();
                    let (font_name, font_size) = if let Some(TypedValue::Object(map)) = &font_conf {
                        (
                            map.get("face").and_then(|v| {
                                if let TypedValue::String(s) = v {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            }),
                            map.get("size").and_then(|v| {
                                if let TypedValue::Number(sz) = v {
                                    Some(*sz)
                                } else {
                                    None
                                }
                            }),
                        )
                    } else {
                        (None, None)
                    };
                    self.sdl.draw_text(&txt, pos, font_name, font_size, color)?;
                    continue;
                }
                // TODO: Sprites and animations not implemented
                if drawable.get_typed("sprite").ok().flatten().is_some() {
                    todo!("Sprite rendering not implemented!");
                }
                if drawable.get_typed("anim").ok().flatten().is_some() {
                    todo!("Anim rendering not implemented!");
                }
                // Otherwise, try shape
                if let Some(shape) = entity.behaviour("tetron:shape") {
                    if let Some(TypedValue::String(sh_type)) =
                        shape.get_typed("type").ok().flatten()
                    {
                        match sh_type.as_str() {
                            "rect" => {
                                let w = shape
                                    .get_typed("w")?
                                    .and_then(|v| match v {
                                        TypedValue::Number(f) => Some(f),
                                        _ => None,
                                    })
                                    .unwrap_or(1.0);
                                let h = shape
                                    .get_typed("h")?
                                    .and_then(|v| match v {
                                        TypedValue::Number(f) => Some(f),
                                        _ => None,
                                    })
                                    .unwrap_or(1.0);
                                self.sdl.draw_rect(pos, w, h, color, true)?;
                            }
                            "circle" => {
                                let r = shape
                                    .get_typed("r")
                                    .ok()
                                    .flatten()
                                    .and_then(|v| match v {
                                        TypedValue::Number(f) => Some(f),
                                        _ => None,
                                    })
                                    .unwrap_or(1.0);
                                self.sdl.draw_circle(pos, r, color, true)?;
                            }
                            "poly" => {
                                if let Some(TypedValue::Array(points)) =
                                    shape.get_typed("points").ok().flatten()
                                {
                                    let points: Vec<Vec2> = points
                                        .into_iter()
                                        .filter_map(|val| match val {
                                            TypedValue::Vector(v) => Some(v),
                                            _ => None,
                                        })
                                        .collect();
                                    if points.len() >= 3 {
                                        self.sdl.draw_polygon(&points, color, true)?;
                                    }
                                }
                            }
                            "line" => {
                                if let Some(TypedValue::Array(points)) =
                                    shape.get_typed("points").ok().flatten()
                                {
                                    let vv: Vec<Vec2> = points
                                        .into_iter()
                                        .filter_map(|val| match val {
                                            TypedValue::Vector(v) => Some(v),
                                            _ => None,
                                        })
                                        .collect();
                                    if vv.len() == 2 {
                                        self.sdl.draw_line(vv[0], vv[1], color)?;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // If no text and no shape, nothing is rendered
            }
            // Drawing logic ends here
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), TetronError> {
        let mut last_frame = Instant::now();

        let entrypoint: String = self
            .config
            .get(&("entrypoint",).to_key())?
            .ok_or(TetronError::RequiredConfigNotFound("entrypoint".into()))?
            .try_into()?;

        let world = WorldRef::new();

        println!("tetron: running {}", self.identifier);
        let level: String = self
            .config
            .get(&("log", "level").to_key())?
            .unwrap_or("info".into())
            .try_into()?;

        scripting::log::level(&level);

        self.scripting
            .execute(&entrypoint, ["begin"], (world.clone(),))?;
        self.world = Some(world);

        'running: loop {
            let now = Instant::now();
            let delta = now.duration_since(last_frame).as_secs_f64();
            last_frame = now;
            self.input.write()?.next_frame();
            for event in self.sdl.events.poll_iter() {
                self.input.write()?.update(&event);
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            self.update(delta)?;
            self.sdl
                .canvas
                .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
            self.sdl.canvas.clear();
            self.draw(delta)?;
            self.sdl.canvas.present();
        }

        Ok(())
    }
}
