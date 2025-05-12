use clap::Parser;
use std::path::PathBuf;

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
pub struct TetronArgs {
    /// Base game path (zip or directory)
    #[arg(long, value_name = "PATH")]
    pub game: Option<PathBuf>,

    /// Additional mods to layer. Multiple can be specified and the mods
    /// are layered in the reverse of the order they are specified.
    /// For example `tetron --game foo --layer mod1 --layer mod2` will first
    /// try to find assets from `mod2`, then `mod1`, then `foo`.
    #[arg(long = "layer", value_name = "PATH")]
    pub layers: Vec<PathBuf>,
}
