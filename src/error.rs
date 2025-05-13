use rhai::{EvalAltResult, Position};
use stupid_simple_kv::KvError;

use crate::fs::FsError;

#[derive(Debug)]
pub enum TetronError {
    Other(String),
    RequiredConfigNotFound(String),
    ModuleNotFound(String, Position),
    RhaiRuntime(String, Option<Position>),
    KvError(String),
    FsError(String),
}

impl From<String> for TetronError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<TetronError> for Box<EvalAltResult> {
    fn from(value: TetronError) -> Self {
        match value {
            TetronError::Other(e) => Box::new(EvalAltResult::ErrorRuntime(
                format!("Unknown error: {e}").into(),
                rhai::Position::NONE,
            )),
            TetronError::RequiredConfigNotFound(_) => panic!("This should never happen"),
            TetronError::ModuleNotFound(e, pos) => Box::new(EvalAltResult::ErrorModuleNotFound(
                format!("Module not found: {e}"),
                pos,
            )),
            TetronError::RhaiRuntime(e, pos) => Box::new(EvalAltResult::ErrorRuntime(
                format!("Runtime error: {e}").into(),
                pos.unwrap_or(Position::NONE),
            )),
            TetronError::KvError(s) => Box::new(EvalAltResult::ErrorRuntime(
                format!("Key-value storage error: {s}").into(),
                Position::NONE,
            )),
            TetronError::FsError(s) => Box::new(EvalAltResult::ErrorRuntime(
                format!("Overlay filesystem error: {s}").into(),
                Position::NONE,
            )),
        }
    }
}

impl std::fmt::Display for TetronError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TetronError::Other(s) => write!(f, "tetron: error: ${s}"),
            TetronError::RequiredConfigNotFound(var) => write!(
                f,
                "tetron: error: The required config item \"{var}\" was expected in game.json but not found."
            ),
            TetronError::ModuleNotFound(e, position) => {
                write!(
                    f,
                    "tetron: module not found: error {e} at position {position}"
                )
            }
            TetronError::RhaiRuntime(e, pos) => write!(
                f,
                "tetron: runtime error: '{e}' at position {}",
                pos.unwrap_or(Position::NONE)
            ),
            TetronError::KvError(s) => write!(f, "Key-value storage error: {s}"),
            TetronError::FsError(s) => write!(f, "Overlay filesystem error: {s}"),
        }
    }
}

impl From<KvError> for TetronError {
    fn from(value: KvError) -> Self {
        TetronError::KvError(value.to_string())
    }
}

impl From<FsError> for TetronError {
    fn from(value: FsError) -> Self {
        Self::FsError(value.to_string())
    }
}

impl std::error::Error for TetronError {}
