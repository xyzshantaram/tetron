#![feature(try_find)]

use clap::Parser;
use fs::{SimpleFS, overlay_fs::OverlayFS};
use std::{
    env,
    path::{Path, PathBuf},
    process,
};

mod fs;

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

struct Game {
    path: PathBuf,
    fs: OverlayFS,
}

impl TryFrom<TetronArgs> for Game {
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

        Ok(Self {
            path: game_path,
            fs: OverlayFS::from_layers(layers),
        })
    }
}

impl Game {
    fn read_text_file(&self, path: &str) -> Result<String, anyhow::Error> {
        let buf = self.fs.open_file(path)?;
        Ok(String::from_utf8(buf)?)
    }
}

pub fn main() -> Result<(), anyhow::Error> {
    let args = TetronArgs::parse();
    let game = Game::try_from(args)?;
    println!("{:#?}", game.path);
    let value = game.read_text_file("game.json")?;
    println!("{value}");
    let listing = game.fs.read_dir("")?;
    println!("{listing:#?}");
    Ok(())
}
