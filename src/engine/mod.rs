use crate::TetronError;
use crate::fs::{SimpleFS, overlay_fs::OverlayFS, to_vfs_layer};
use crate::scripting::TetronScripting;
use crate::sdl::TetronSdlHandle;
use crate::utils::resolve_physical_fs_path;
use sdl2::{event::Event, keyboard::Keycode};
use std::cell::RefCell;
use std::rc::Rc;
use std::{path::PathBuf, process, time::Instant};
use stupid_simple_kv::{IntoKey, Kv, MemoryBackend, SqliteBackend};

mod args;
pub use args::TetronArgs;

pub struct Game {
    path: PathBuf,
    fs: Rc<OverlayFS>,
    pub(crate) config: Rc<Kv>,
    pub(crate) flags: Rc<RefCell<Kv>>,
    sdl: TetronSdlHandle,
    pub(crate) identifier: String,
    scripting: TetronScripting,
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

        let mut layers: Vec<Box<dyn SimpleFS>> = vec![to_vfs_layer(&game_path)?];

        for layer in args.layers.iter().rev() {
            layers.push(to_vfs_layer(layer)?);
        }

        let fs = OverlayFS::from_layers(layers);

        let json = fs.read_text_file("game.json")?;
        let config = Rc::new(Kv::from_json_string(Box::new(MemoryBackend::new()), json)?);

        let identifier: String = config
            .get(&("identifier",))?
            .ok_or(TetronError::IdentifierNotFound)?
            .try_into()?;

        // TODO: implement wasm backend (probably use IndexedDB or localstorage)
        #[cfg(not(target_arch = "wasm32"))]
        let backend = {
            let data = dirs::data_dir().ok_or(TetronError::Other(String::from(
                "Error getting user data dir",
            )))?;

            let db_path = data.join("tetron").join(&identifier);
            std::fs::create_dir_all(&db_path)?;
            SqliteBackend::file(&db_path.join("flags.db"))?
        };

        let flags = Rc::new(RefCell::new(Kv::new(Box::new(backend))));
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
        let sdl = TetronSdlHandle::new(&title, width.try_into()?, height.try_into()?)?;

        let fs_rc = Rc::new(fs);
        let scripting = TetronScripting::new(fs_rc.clone(), flags.clone(), config.clone())?;
        Ok(Self {
            path: game_path,
            fs: fs_rc,
            config,
            sdl,
            identifier,
            flags,
            scripting,
        })
    }
}

impl Game {
    fn read_text_file(&self, path: &str) -> Result<String, anyhow::Error> {
        Ok(self.fs.read_text_file(path)?)
    }

    fn update(&mut self, delta: &f32) {
        todo!()
    }

    fn draw(&mut self) {}

    pub fn run(&mut self) {
        let mut last_frame = Instant::now();
        'running: loop {
            let now = Instant::now();
            let delta = now.duration_since(last_frame);
            let delta_secs = delta.as_secs_f32();
            last_frame = now;
            for event in self.sdl.events.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            self.update(&delta_secs);
            self.sdl
                .canvas
                .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
            self.sdl.canvas.clear();
            self.draw();
            self.sdl.canvas.present();
        }
    }
}
