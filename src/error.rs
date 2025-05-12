#[derive(Debug)]
pub enum TetronError {
    Other(String),
    IdentifierNotFound,
    JsError(String),
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
            TetronError::JsError(s) => write!(f, "JavaScript runtime error: {s}"),
        }
    }
}

impl std::error::Error for TetronError {}
