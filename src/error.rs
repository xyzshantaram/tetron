use std::{
    cell::{BorrowError, BorrowMutError},
    sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard},
};

use rune::{
    ContextError,
    diagnostics::EmitError,
    runtime::{RuntimeError, VmError},
};
use stupid_simple_kv::KvError;

use crate::fs::FsError;

#[derive(Debug, rune::Any)]
pub enum TetronError {
    Other(String),
    RequiredConfigNotFound(String),
    ModuleNotFound(String),
    Runtime(String),
    KvError(String),
    FsError(String),
    ContextError(String),
    Conversion(String),
    Quit,
}

impl From<String> for TetronError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<ContextError> for TetronError {
    fn from(value: ContextError) -> Self {
        Self::ContextError(value.to_string())
    }
}

impl From<RuntimeError> for TetronError {
    fn from(value: RuntimeError) -> Self {
        Self::Runtime(value.to_string())
    }
}

impl From<rune::alloc::Error> for TetronError {
    fn from(value: rune::alloc::Error) -> Self {
        TetronError::Other(format!("Allocation error: {value}"))
    }
}

impl From<rune::BuildError> for TetronError {
    fn from(value: rune::BuildError) -> Self {
        TetronError::Runtime(format!("error building sources: {value}"))
    }
}

impl From<EmitError> for TetronError {
    fn from(value: EmitError) -> Self {
        TetronError::Runtime(format!("error emitting diagnostics: {value}"))
    }
}

impl From<VmError> for TetronError {
    fn from(value: VmError) -> Self {
        TetronError::Runtime(format!("fatal scripting runtime error: {value}"))
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
            TetronError::ModuleNotFound(e) => {
                write!(f, "tetron: module not found: {e}")
            }
            TetronError::Runtime(e) => write!(f, "tetron: runtime error: {e}"),
            TetronError::KvError(s) => write!(f, "Key-value storage error: {s}"),
            TetronError::FsError(s) => write!(f, "Overlay filesystem error: {s}"),
            TetronError::ContextError(s) => write!(f, "Error building Rune context: {s}"),
            TetronError::Conversion(s) => write!(f, "Error converting types: {s}"),
            TetronError::Quit => write!(f, "Player initiated quit"),
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

impl From<BorrowError> for TetronError {
    fn from(value: BorrowError) -> Self {
        Self::Runtime(value.to_string())
    }
}

impl From<BorrowMutError> for TetronError {
    fn from(value: BorrowMutError) -> Self {
        Self::Runtime(value.to_string())
    }
}

impl<'a> From<PoisonError<RwLockReadGuard<'a, crate::engine::input::KeyState>>> for TetronError {
    fn from(err: PoisonError<RwLockReadGuard<'a, crate::engine::input::KeyState>>) -> Self {
        TetronError::Runtime(format!("KeyState RwLock read guard poisoned: {}", err))
    }
}

impl<'a> From<PoisonError<RwLockWriteGuard<'a, crate::engine::input::KeyState>>> for TetronError {
    fn from(err: PoisonError<RwLockWriteGuard<'a, crate::engine::input::KeyState>>) -> Self {
        TetronError::Runtime(format!("KeyState RwLock write guard poisoned: {}", err))
    }
}

impl std::error::Error for TetronError {}
