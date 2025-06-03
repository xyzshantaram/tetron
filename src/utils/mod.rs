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

pub mod rune {
    use super::RuneString;
    use crate::error::TetronError;
    use rune::{
        alloc::{clone::TryClone, string::TryToString},
        runtime::Object,
    };

    pub(crate) fn clone_obj(obj: &Object) -> Result<Object, TetronError> {
        let mut copy = Object::new();
        for item in obj.iter() {
            copy.insert(item.0.try_to_string()?, item.1.try_clone()?)?;
        }

        Ok(copy)
    }

    pub(crate) fn obj_key(s: &str) -> Result<RuneString, rune::alloc::Error> {
        RuneString::try_from(s)
    }
}

pub trait Registrable {
    fn register(module: &mut Module) -> Result<(), ContextError>;
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
