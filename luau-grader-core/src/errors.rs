use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraderError {
    #[error("failed to read source file: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("analysis error: {0}")]
    Analysis(String),

    #[error("config error: {0}")]
    Config(String),
}
