use rhai::{EvalAltResult, Position};

#[derive(Debug)]
pub enum TetronError {
    Other(String),
    IdentifierNotFound,
    ModuleNotFound(String, Position),
    RhaiRuntime(String, Option<Position>),
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
            TetronError::IdentifierNotFound => panic!("This should never happen"),
            TetronError::ModuleNotFound(e, pos) => Box::new(EvalAltResult::ErrorModuleNotFound(
                format!("Module not found: {e}"),
                pos,
            )),
            TetronError::RhaiRuntime(e, pos) => Box::new(EvalAltResult::ErrorRuntime(
                format!("Runtime error: {e}").into(),
                pos.unwrap_or_default(),
            )),
        }
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
            TetronError::ModuleNotFound(e, position) => {
                write!(
                    f,
                    "tetron: module not found: error {e} at position {position}"
                )
            }
            TetronError::RhaiRuntime(e, pos) => write!(
                f,
                "tetron: runtime error: {e} at position {}",
                pos.unwrap_or_default()
            ),
        }
    }
}

impl std::error::Error for TetronError {}
