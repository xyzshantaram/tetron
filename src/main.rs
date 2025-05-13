#![feature(try_find)]

mod engine;
mod error;
mod fs;
mod scripting;
mod sdl;
mod utils;

use clap::Parser;
use engine::{Game, TetronArgs};
pub use error::TetronError;

pub fn main() -> Result<(), anyhow::Error> {
    let args = TetronArgs::parse();
    let mut game = Game::try_from(args)?;
    println!("tetron: running {}", game.identifier);
    game.run()?;
    Ok(())
}
