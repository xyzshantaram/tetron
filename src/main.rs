#![feature(try_find)]

use clap::Parser;
use fs::{SimpleFS, overlay_fs::OverlayFS};
use sdl2::{Sdl, event::Event, keyboard::Keycode};
use std::{
    env,
    path::{Path, PathBuf},
    process,
    time::Instant,
};
use stupid_simple_kv::{IntoKey, Kv, MemoryBackend, SqliteBackend};

mod fs;

#[derive(Debug)]
enum TetronError {
    Other(String),
    IdentifierNotFound,
}

impl From<String> for TetronError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl std::fmt::Display for TetronError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TetronError::Other(s) => write!(f, "tetron: error: ${s}"),
            TetronError::IdentifierNotFound => write!(
                f,
                "tetron: error: An identifier was expected in game.json but not found."
            ),
        }
    }
}

impl std::error::Error for TetronError {}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(author = "shantaram <me@shantaram.xyz>")]
#[command(bin_name = "tetron")]
#[command(help_template = "\
{name} {version}
by {author-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
")]
struct TetronArgs {
    /// Base game path (zip or directory)
    #[arg(long, value_name = "PATH")]
    game: Option<PathBuf>,

    /// Additional mods to layer. Multiple can be specified and the mods
    /// are layered in the reverse of the order they are specified.
    /// For example `tetron --game foo --layer mod1 --layer mod2` will first
    /// try to find assets from `mod2`, then `mod1`, then `foo`.
    #[arg(long = "layer", value_name = "PATH")]
    layers: Vec<PathBuf>,
}

fn normalize_path(path: &Path) -> Result<PathBuf, anyhow::Error> {
    let cwd = env::current_dir()?;
    let full_path = cwd.join(path);
    Ok(full_path.canonicalize()?)
}

struct TetronSdlHandle {
    pub(crate) context: Sdl,
    pub(crate) video: sdl2::VideoSubsystem,
    pub(crate) audio: sdl2::AudioSubsystem,
    pub(crate) canvas: sdl2::render::Canvas<sdl2::video::Window>,
    pub(crate) events: sdl2::EventPump,
}

impl TetronSdlHandle {
    fn new(title: &str, w: u32, h: u32) -> Result<Self, TetronError> {
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

struct Game<'a> {
    path: PathBuf,
    fs: OverlayFS,
    pub(crate) config: Kv<'a>,
    pub(crate) flags: Kv<'a>,
    sdl: TetronSdlHandle,
    identifier: String,
}

impl<'a> TryFrom<TetronArgs> for Game<'a> {
    type Error = anyhow::Error;

    fn try_from(args: TetronArgs) -> Result<Self, Self::Error> {
        let game_path = match args.game {
            Some(p) => normalize_path(&p)?,
            None => {
                eprintln!("tetron: error: No game supplied");
                process::exit(1);
            }
        };

        let mut layers: Vec<Box<dyn SimpleFS>> = vec![fs::to_vfs_layer(&game_path)?];

        for layer in args.layers.iter().rev() {
            layers.push(fs::to_vfs_layer(layer)?);
        }

        let fs = OverlayFS::from_layers(layers);

        let json = fs.read_text_file("game.json")?;
        let config = Kv::from_json_string(Box::new(MemoryBackend::new()), json)?;

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

        let flags = Kv::new(Box::new(backend));
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

        Ok(Self {
            path: game_path,
            fs,
            config,
            flags,
            sdl,
            identifier,
        })
    }
}

impl<'a> Game<'a> {
    fn read_text_file(&self, path: &str) -> Result<String, anyhow::Error> {
        Ok(self.fs.read_text_file(path)?)
    }

    fn update(&mut self, delta: &f32) {}

    fn draw(&mut self) {}

    fn run(&mut self) {
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

pub fn main() -> Result<(), anyhow::Error> {
    let args = TetronArgs::parse();
    let mut game = Game::try_from(args)?;
    println!("tetron: running {}", game.identifier);
    game.run();
    Ok(())
}
