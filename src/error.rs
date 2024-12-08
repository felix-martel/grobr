use std::path::PathBuf;
use thiserror::Error;
use crate::parser::Rule;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Tag '{0}' is not recognized")]
    InvalidTag(String),

    #[error("Missing key parts '{missing}' for file '{path}'")]
    MissingKeyPart {
        path: PathBuf,
        missing: String,
    },

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(err: pest::error::Error<Rule>) -> Self {
        Error::InvalidPattern(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
