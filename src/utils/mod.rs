use std::{
    env,
    path::{Path, PathBuf},
};

use ::rune::{ContextError, Module};

pub fn resolve_physical_fs_path(path: &Path) -> Result<PathBuf, anyhow::Error> {
    let cwd = env::current_dir()?;
    let full_path = cwd.join(path);
    Ok(full_path.canonicalize()?)
}

pub type RuneString = ::rune::alloc::String;
pub type RuneVec = ::rune::runtime::Vec;

pub mod typed_value;

pub trait Registrable {
    fn register(module: &mut Module) -> Result<(), ContextError>;
}

#[macro_export]
macro_rules! system_log {
    ($($arg:tt)*) => {
        println!("tetron::log \x1b[36m[SYSTEM]\x1b[0m {}", format!($($arg)*))
    };
}

pub fn parse_hex_color(hex: &str, fallback: sdl2::pixels::Color) -> sdl2::pixels::Color {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => u32::from_str_radix(hex, 16)
            .ok()
            .map(|rgb| {
                sdl2::pixels::Color::RGB(
                    ((rgb >> 16) & 0xFF) as u8,
                    ((rgb >> 8) & 0xFF) as u8,
                    (rgb & 0xFF) as u8,
                )
            })
            .unwrap_or(fallback),
        3 => u16::from_str_radix(hex, 16)
            .ok()
            .map(|rgb| {
                sdl2::pixels::Color::RGB(
                    (((rgb >> 8) & 0xF) * 17) as u8,
                    (((rgb >> 4) & 0xF) * 17) as u8,
                    ((rgb & 0xF) * 17) as u8,
                )
            })
            .unwrap_or(fallback),
        _ => fallback,
    }
}
