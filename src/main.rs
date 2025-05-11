#![feature(try_find)]

use clap::Parser;
use fs::{SimpleFS, overlay_fs::OverlayFS};
use std::{
    env,
    path::{Path, PathBuf},
    process,
};
use stupid_simple_kv::{Kv, MemoryBackend, SqliteBackend};

mod fs;

#[derive(Debug)]
enum TetronError {
    Other(String),
    IdentifierNotFound,
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

struct Game<'a> {
    path: PathBuf,
    fs: OverlayFS,
    pub(crate) config: Kv<'a>,
    pub(crate) flags: Kv<'a>,
}

impl<'a> TryFrom<TetronArgs> for Game<'a> {
    type Error = anyhow::Error;

    fn try_from(args: TetronArgs) -> Result<Self, Self::Error> {
        let game_path = match args.game {
            Some(p) => normalize_path(&p)?,
            None => {
                eprintln!("Error: No game supplied");
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

            let db_path = data.join("tetron").join(identifier);
            std::fs::create_dir_all(&db_path)?;
            SqliteBackend::file(&db_path.join("flags.db"))?
        };

        let flags = Kv::new(Box::new(backend));

        Ok(Self {
            path: game_path,
            fs,
            config,
            flags,
        })
    }
}

impl<'a> Game<'a> {
    fn read_text_file(&self, path: &str) -> Result<String, anyhow::Error> {
        Ok(self.fs.read_text_file(path)?)
    }
}

pub fn main() -> Result<(), anyhow::Error> {
    let args = TetronArgs::parse();
    let game = Game::try_from(args)?;
    println!("{:#?}", game.path);

    println!(
        "{:#?} {:#?}",
        game.config.get(&("flag1"))?,
        game.config.get(&("entrypoint"))?
    );

    Ok(())
}
